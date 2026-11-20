use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub struct Error(pub &'static str);
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self("file_io_error")
    }
}
impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self {
        Self("database_error")
    }
}

const APP_ID: i64 = 1145784135;
const SCHEMA:&str = "
CREATE TABLE assets(id TEXT PRIMARY KEY,name TEXT NOT NULL,kind TEXT NOT NULL);
CREATE TABLE versions(asset_id TEXT NOT NULL REFERENCES assets(id),id TEXT NOT NULL,parent_id TEXT,role TEXT NOT NULL,
 state TEXT NOT NULL,sha256 TEXT NOT NULL,bytes INTEGER NOT NULL,extension TEXT NOT NULL,source_path TEXT NOT NULL,source_note TEXT NOT NULL,
 PRIMARY KEY(asset_id,id),FOREIGN KEY(asset_id,parent_id) REFERENCES versions(asset_id,id));
CREATE TABLE requirements(id TEXT PRIMARY KEY,name TEXT NOT NULL,kind TEXT NOT NULL,description TEXT NOT NULL,
 asset_id TEXT,version_id TEXT,FOREIGN KEY(asset_id,version_id) REFERENCES versions(asset_id,id));
CREATE TABLE characters(id TEXT PRIMARY KEY,name TEXT NOT NULL,voice_id TEXT NOT NULL,notes TEXT NOT NULL);
CREATE TABLE dialogues(id TEXT PRIMARY KEY,character_id TEXT NOT NULL REFERENCES characters(id),text TEXT NOT NULL,
 language TEXT NOT NULL,requirement_id TEXT REFERENCES requirements(id));
CREATE TABLE events(seq INTEGER PRIMARY KEY AUTOINCREMENT,op TEXT NOT NULL,entity_id TEXT NOT NULL,created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
";

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn linked(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}
pub(crate) fn no_link(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) if linked(&meta) => Err(Error("managed_path_is_link")),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
pub(crate) fn directory(path: &Path) -> Result<()> {
    no_link(path)?;
    if !path.exists() {
        fs::create_dir(path)?;
    }
    if !path.is_dir() {
        return Err(Error("not_directory"));
    }
    Ok(())
}

fn local_directory(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(Error("workspace_must_be_existing_absolute_directory"));
    }
    let canonical = path.canonicalize()?;
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;
        let drive = match canonical.components().next() {
            Some(Component::Prefix(p)) => match p.kind() {
                Prefix::Disk(d) | Prefix::VerbatimDisk(d) => d,
                _ => return Err(Error("network_workspace_not_supported")),
            },
            _ => return Err(Error("network_workspace_not_supported")),
        };
        let root = [drive as u16, 58, 92, 0];
        if !matches!(unsafe { GetDriveTypeW(root.as_ptr()) }, 2 | 3) {
            return Err(Error("network_workspace_not_supported"));
        }
    }
    canonical.to_str().ok_or(Error("non_unicode_path"))?;
    Ok(canonical)
}

fn database(path: &Path, schema: &str, create: bool, max_version: i64) -> Result<Connection> {
    no_link(path)?;
    for suffix in ["-wal", "-shm", "-journal"] {
        no_link(&PathBuf::from(format!(
            "{}{suffix}",
            path.to_string_lossy()
        )))?;
    }
    let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
        | if create {
            rusqlite::OpenFlags::SQLITE_OPEN_CREATE
        } else {
            rusqlite::OpenFlags::empty()
        };
    let mut db = Connection::open_with_flags(path, flags)?;
    db.busy_timeout(Duration::from_secs(3))?;
    let version: i64 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
    let app: i64 = db.pragma_query_value(None, "application_id", |r| r.get(0))?;
    if version == 0 && app == 0 && create {
        let tables: i64 = db.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'",
            [],
            |r| r.get(0),
        )?;
        if tables != 0 {
            return Err(Error("unrecognized_database"));
        }
        let tx = db.transaction()?;
        tx.execute_batch(schema)?;
        tx.pragma_update(None, "application_id", APP_ID)?;
        tx.pragma_update(None, "user_version", 1)?;
        tx.commit()?;
    } else if !(1..=max_version).contains(&version) || app != APP_ID {
        return Err(Error("unsupported_schema"));
    }
    db.pragma_update(None, "foreign_keys", true)?;
    db.pragma_update(None, "journal_mode", "WAL")?;
    db.pragma_update(None, "synchronous", "FULL")?;
    Ok(db)
}

pub struct Store {
    pub(crate) root: PathBuf,
    pub(crate) catalog: Connection,
}
impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        if !root.is_absolute() {
            return Err(Error("app_root_must_be_absolute"));
        }
        local_directory(root.parent().ok_or(Error("invalid_app_root"))?)?;
        no_link(root)?;
        fs::create_dir_all(root)?;
        let root = local_directory(root)?;
        let mut catalog=database(&root.join("catalog.sqlite3"),"CREATE TABLE projects(id TEXT PRIMARY KEY,name TEXT NOT NULL,workspace TEXT NOT NULL,root TEXT NOT NULL UNIQUE);",true,3)?;
        crate::generation::migrate(&mut catalog)?;
        crate::audio::migrate(&mut catalog)?;
        Ok(Self { root, catalog })
    }
    pub fn default_root() -> Result<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var_os("LOCALAPPDATA")
                .map(|p| PathBuf::from(p).join("DK-Elevenlabs-GameSound-Manager"))
                .ok_or(Error("local_appdata_unavailable"))
        }
        #[cfg(not(windows))]
        {
            Err(Error("unsupported_platform"))
        }
    }
    pub fn execute(&mut self, command: &Value) -> Value {
        if let Err(code) = crate::protocol::validate(command) {
            return crate::protocol::error(code);
        }
        match self.dispatch(command) {
            Ok(data) => crate::protocol::success(data),
            Err(e) => crate::protocol::error(e.0),
        }
    }
    pub(crate) fn project(&self, id: &str) -> Result<(PathBuf, Connection)> {
        let root: String = self
            .catalog
            .query_row("SELECT root FROM projects WHERE id=?1", [id], |r| r.get(0))
            .optional()?
            .ok_or(Error("project_not_found"))?;
        let root = PathBuf::from(root);
        no_link(&root)?;
        no_link(&root.join("project.json"))?;
        if root.canonicalize()? != root {
            return Err(Error("project_path_changed"));
        }
        for name in ["originals", "candidates", "imports"] {
            no_link(&root.join(name))?;
        }
        let marker: Value = serde_json::from_slice(&fs::read(root.join("project.json"))?)
            .map_err(|_| Error("invalid_project_marker"))?;
        if marker["project_id"] != id {
            return Err(Error("invalid_project_marker"));
        }
        let db = database(&root.join("project.sqlite3"), SCHEMA, false, 1)?;
        Ok((root, db))
    }
    fn create(&mut self, c: &Value) -> Result<Value> {
        let id = text(c, "project_id");
        let workspace = local_directory(Path::new(text(c, "workspace")))?;
        if workspace.starts_with(&self.root)
            || workspace.starts_with(std::env::current_exe()?.parent().unwrap_or(Path::new("")))
        {
            return Err(Error("workspace_overlaps_app"));
        }
        let root = workspace.join(format!("dk-gamesound-{id}"));
        let name = text(c, "name");
        if let Some((old_name, old_root)) = self
            .catalog
            .query_row("SELECT name,root FROM projects WHERE id=?1", [id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .optional()?
        {
            if old_name != name || Path::new(&old_root) != root {
                return Err(Error("project_id_conflict"));
            }
            self.project(id)?;
            return Ok(json!({"project_id":id,"name":name,"reused":true}));
        }
        let marker = json!({"schema_version":1,"project_id":id,"name":name});
        if root.exists() {
            no_link(&root)?;
            no_link(&root.join("project.json"))?;
            let old: Value = serde_json::from_slice(&fs::read(root.join("project.json"))?)
                .map_err(|_| Error("workspace_conflict"))?;
            if old != marker {
                return Err(Error("workspace_conflict"));
            }
        } else {
            fs::create_dir(&root)?;
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join("project.json"))?;
            f.write_all(serde_json::to_string_pretty(&marker).unwrap().as_bytes())?;
            f.sync_all()?;
        }
        for name in ["originals", "candidates", "imports"] {
            directory(&root.join(name))?;
        }
        let db = database(&root.join("project.sqlite3"), SCHEMA, true, 1)?;
        db.execute(
            "INSERT INTO events(op,entity_id) VALUES('project.create',?1)",
            [id],
        )?;
        self.catalog.execute(
            "INSERT INTO projects(id,name,workspace,root) VALUES(?1,?2,?3,?4)",
            params![
                id,
                name,
                workspace.to_str().unwrap(),
                root.to_str().unwrap()
            ],
        )?;
        Ok(json!({"project_id":id,"name":name,"reused":false}))
    }
    fn dispatch(&mut self, c: &Value) -> Result<Value> {
        let op = text(c, "op");
        if op.starts_with("audio.") {
            return crate::audio::command(self, c);
        }
        if ["account.", "budget.", "voice.", "model.", "job."]
            .iter()
            .any(|prefix| op.starts_with(prefix))
        {
            return crate::generation::command(self, c);
        }
        if op == "project.create" {
            return self.create(c);
        }
        if op == "project.list" {
            return list(&self.catalog,"SELECT id,name FROM projects WHERE id>?1 AND instr(lower(name),lower(?2))>0 ORDER BY id LIMIT ?3",c,&[],&["project_id","name"]);
        }
        let (root, mut db) = self.project(text(c, "project_id"))?;
        match op {
            "project.get"=>{
                let name:String=self.catalog.query_row("SELECT name FROM projects WHERE id=?1",[text(c,"project_id")],|r|r.get(0))?;
                Ok(json!({"project_id":c["project_id"],"name":name,"schema_version":1}))
            },
            "asset.import"=>{
                if self.catalog.query_row("SELECT EXISTS(SELECT 1 FROM audio_operations WHERE project_id=?1 AND asset_id=?2 AND version_id=?3 AND kind='audio.edit')",params![text(c,"project_id"),text(c,"asset_id"),text(c,"version_id")],|r|r.get::<_,bool>(0))?{return Err(Error("audio_target_reserved"));}
                if self.catalog.query_row("SELECT EXISTS(SELECT 1 FROM generation_jobs WHERE project_id=?1 AND asset_id=?2 AND version_id=?3)",params![text(c,"project_id"),text(c,"asset_id"),text(c,"version_id")],|r|r.get::<_,bool>(0))?{return Err(Error("generation_target_reserved"));}
                import(&root,&mut db,c)
            },
            "asset.list"=>list(&db,"SELECT id,name,kind FROM assets WHERE id>?1 AND instr(lower(name),lower(?2))>0 AND (?4='' OR kind=?4) ORDER BY id LIMIT ?3",c,&[text(c,"kind")],&["asset_id","name","kind"]),
            "asset.versions"=>list(&db,"SELECT id,parent_id,role,state,sha256,bytes,extension,source_note FROM versions WHERE id>?1 AND ?2='' AND asset_id=?4 ORDER BY id LIMIT ?3",c,&[text(c,"asset_id")],&["version_id","parent_version_id","role","state","sha256","bytes","extension","source_note"]),
            "asset.file"=>file_info(&root,&db,text(c,"asset_id"),text(c,"version_id")),
            "project.recover"=>recover(&root,&mut db,c),
            "requirement.list"=>list(&db,"SELECT id,name,kind,description,asset_id,version_id FROM requirements WHERE id>?1 AND instr(lower(name||' '||description),lower(?2))>0 AND (?4='' OR kind=?4) ORDER BY id LIMIT ?3",c,&[text(c,"kind")],&["requirement_id","name","kind","description","asset_id","version_id"]),
            "character.list"=>list(&db,"SELECT id,name,voice_id,notes FROM characters WHERE id>?1 AND instr(lower(name),lower(?2))>0 ORDER BY id LIMIT ?3",c,&[],&["character_id","name","voice_id","notes"]),
            "dialogue.list"=>list(&db,"SELECT id,character_id,text,language,requirement_id FROM dialogues WHERE id>?1 AND instr(lower(text),lower(?2))>0 AND (?4='' OR character_id=?4) ORDER BY id LIMIT ?3",c,&[text(c,"character_id")],&["dialogue_id","character_id","text","language","requirement_id"]),
            "requirement.link"=>{file_info(&root,&db,text(c,"asset_id"),text(c,"version_id"))?;put(&mut db,c)},
            "requirement.put"|"character.put"|"dialogue.put"=>put(&mut db,c),
            _=>Err(Error("unsupported_operation")),
        }
    }
}

fn list(db: &Connection, sql: &str, c: &Value, extra: &[&str], names: &[&str]) -> Result<Value> {
    let limit = c["limit"].as_i64().unwrap_or(10);
    let mut statement = db.prepare(sql)?;
    let after = text(c, "after");
    let search = text(c, "search");
    let mut values: Vec<&dyn rusqlite::ToSql> = vec![&after, &search, &limit];
    for item in extra {
        values.push(item);
    }
    let mut rows = statement
        .query_map(values.as_slice(), |row| {
            let mut object = serde_json::Map::new();
            for (index, name) in names.iter().enumerate() {
                let value = match row.get_ref(index)? {
                    rusqlite::types::ValueRef::Null => Value::Null,
                    rusqlite::types::ValueRef::Integer(n) => json!(n),
                    rusqlite::types::ValueRef::Text(s) => json!(String::from_utf8_lossy(s)),
                    _ => Value::Null,
                };
                object.insert((*name).to_string(), value);
            }
            Ok(Value::Object(object))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut more = rows.len() == limit as usize;
    while serde_json::to_vec(&rows)
        .map_err(|_| Error("response_encoding_failed"))?
        .len()
        > 48 * 1024
    {
        rows.pop();
        more = true;
    }
    let next = if more {
        rows.last()
            .map(|r| r[names[0]].clone())
            .unwrap_or(Value::Null)
    } else {
        Value::Null
    };
    Ok(json!({"items":rows,"next_after":next}))
}

fn event(db: &Connection, op: &str, id: &str) -> Result<()> {
    db.execute(
        "INSERT INTO events(op,entity_id) VALUES(?1,?2)",
        params![op, id],
    )?;
    Ok(())
}

fn put(db: &mut Connection, c: &Value) -> Result<Value> {
    let tx = db.transaction()?;
    let op = text(c, "op");
    let id = match op {
        "requirement.put" => {
            let id = text(c, "requirement_id");
            let linked:Option<String>=tx.query_row("SELECT a.kind FROM requirements r JOIN assets a ON a.id=r.asset_id WHERE r.id=?1",[id],|r|r.get(0)).optional()?;
            if linked.as_deref().is_some_and(|k| k != text(c, "kind")) {
                return Err(Error("linked_kind_conflict"));
            }
            tx.execute("INSERT INTO requirements(id,name,kind,description) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name,kind=excluded.kind,description=excluded.description",params![id,text(c,"name"),text(c,"kind"),text(c,"description")])?;
            id
        }
        "requirement.link" => {
            let id = text(c, "requirement_id");
            let kind: String = tx
                .query_row("SELECT kind FROM requirements WHERE id=?1", [id], |r| {
                    r.get(0)
                })
                .optional()?
                .ok_or(Error("requirement_not_found"))?;
            let actual:Option<String>=tx.query_row("SELECT a.kind FROM assets a JOIN versions v ON a.id=v.asset_id WHERE a.id=?1 AND v.id=?2 AND v.state='ready'",params![text(c,"asset_id"),text(c,"version_id")],|r|r.get(0)).optional()?;
            if actual.as_deref() != Some(&kind) {
                return Err(Error("asset_kind_or_version_mismatch"));
            }
            tx.execute(
                "UPDATE requirements SET asset_id=?2,version_id=?3 WHERE id=?1",
                params![id, text(c, "asset_id"), text(c, "version_id")],
            )?;
            id
        }
        "character.put" => {
            let id = text(c, "character_id");
            tx.execute("INSERT INTO characters VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name,voice_id=excluded.voice_id,notes=excluded.notes",params![id,text(c,"name"),text(c,"voice_id"),text(c,"notes")])?;
            id
        }
        "dialogue.put" => {
            let id = text(c, "dialogue_id");
            if let Some(req) = c
                .get("requirement_id")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                let kind: Option<String> = tx
                    .query_row("SELECT kind FROM requirements WHERE id=?1", [req], |r| {
                        r.get(0)
                    })
                    .optional()?;
                if kind.as_deref() != Some("voice") {
                    return Err(Error("voice_requirement_not_found"));
                }
            }
            if !tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM characters WHERE id=?1)",
                [text(c, "character_id")],
                |r| r.get::<_, bool>(0),
            )? {
                return Err(Error("character_not_found"));
            }
            tx.execute("INSERT INTO dialogues VALUES(?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET character_id=excluded.character_id,text=excluded.text,language=excluded.language,requirement_id=excluded.requirement_id",params![id,text(c,"character_id"),text(c,"text"),text(c,"language"),c.get("requirement_id").and_then(Value::as_str).filter(|s|!s.is_empty())])?;
            id
        }
        _ => return Err(Error("unsupported_operation")),
    };
    event(&tx, op, id)?;
    tx.commit()?;
    Ok(json!({"id":id,"saved":true}))
}

pub(crate) fn hash_file(path: &Path) -> Result<(String, u64)> {
    let mut file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(Error("source_not_regular_file"));
    }
    let mut hasher = Sha256::new();
    let mut size = 0;
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        size += n as u64;
        if size > 512 * 1024 * 1024 {
            return Err(Error("file_too_large"));
        }
        hasher.update(&buffer[..n]);
    }
    Ok((format!("{:x}", hasher.finalize()), size))
}

fn version_paths(
    root: &Path,
    asset: &str,
    version: &str,
    role: &str,
    extension: &str,
    create: bool,
) -> Result<(PathBuf, PathBuf)> {
    if !crate::catalog::valid_id(asset)
        || !crate::catalog::valid_id(version)
        || !["original", "candidate"].contains(&role)
        || ![
            "wav", "mp3", "ogg", "flac", "m4a", "aac", "opus", "aiff", "aif", "wma",
        ]
        .contains(&extension)
    {
        return Err(Error("invalid_version_metadata"));
    }
    let directory_name = if role == "original" {
        "originals"
    } else {
        "candidates"
    };
    let base = root.join(directory_name);
    no_link(&base)?;
    let folder = base.join(asset);
    no_link(&folder)?;
    if create {
        directory(&folder)?;
    }
    let folder = folder.join(version);
    no_link(&folder)?;
    if create {
        directory(&folder)?;
    }
    let final_path = folder.join(format!("source.{extension}"));
    no_link(&final_path)?;
    let imports = root.join("imports");
    no_link(&imports)?;
    let staging = imports.join(asset);
    no_link(&staging)?;
    if create {
        directory(&staging)?;
    }
    let staging = staging.join(format!("{version}.part"));
    no_link(&staging)?;
    Ok((staging, final_path))
}

fn import(root: &Path, db: &mut Connection, c: &Value) -> Result<Value> {
    import_role(root, db, c, false)
}
pub(crate) fn import_generated(root: &Path, db: &mut Connection, c: &Value) -> Result<Value> {
    import_role(root, db, c, true)
}
fn import_role(root: &Path, db: &mut Connection, c: &Value, generated: bool) -> Result<Value> {
    let asset = text(c, "asset_id");
    let version = text(c, "version_id");
    let source = Path::new(text(c, "source_path"));
    if !source.is_absolute() {
        return Err(Error("source_must_be_absolute"));
    }
    let source = source.canonicalize()?;
    let extension = source
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ![
        "wav", "mp3", "ogg", "flac", "m4a", "aac", "opus", "aiff", "aif", "wma",
    ]
    .contains(&extension.as_str())
    {
        return Err(Error("unsupported_audio_extension"));
    }
    let (sha, bytes) = hash_file(&source)?;
    if bytes == 0 {
        return Err(Error("empty_audio_file"));
    }
    let existing: Option<(String, String, String, String)> = db
        .query_row(
            "SELECT state,sha256,role,extension FROM versions WHERE asset_id=?1 AND id=?2",
            params![asset, version],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    let parent = c
        .get("parent_version_id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());
    let role;
    if let Some((state, old_sha, old_role, old_ext)) = existing {
        let metadata:(String,String,Option<String>,String)=db.query_row("SELECT a.name,a.kind,v.parent_id,v.source_note FROM assets a JOIN versions v ON a.id=v.asset_id WHERE a.id=?1 AND v.id=?2",params![asset,version],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
        if old_sha != sha
            || old_ext != extension
            || metadata.0 != text(c, "name")
            || metadata.1 != text(c, "kind")
            || metadata.2.as_deref() != parent
            || metadata.3 != text(c, "source_note")
        {
            return Err(Error("version_id_conflict"));
        }
        if state == "ready" {
            return file_info(root, db, asset, version);
        }
        if state != "copying" {
            return Err(Error("import_needs_attention"));
        }
        role = old_role;
    } else {
        let tx = db.transaction()?;
        let found: Option<(String, String)> = tx
            .query_row("SELECT name,kind FROM assets WHERE id=?1", [asset], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .optional()?;
        if let Some((name, kind)) = found {
            if name != text(c, "name") || kind != text(c, "kind") {
                return Err(Error("asset_id_conflict"));
            }
            let parent = parent.ok_or(Error("candidate_parent_required"))?;
            let ready:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM versions WHERE asset_id=?1 AND id=?2 AND state='ready')",params![asset,parent],|r|r.get(0))?;
            if !ready {
                return Err(Error("parent_version_not_ready"));
            }
            role = "candidate".to_string();
        } else {
            if parent.is_some() {
                return Err(Error("parent_version_not_found"));
            }
            tx.execute(
                "INSERT INTO assets VALUES(?1,?2,?3)",
                params![asset, text(c, "name"), text(c, "kind")],
            )?;
            role = if generated { "candidate" } else { "original" }.to_string();
        }
        tx.execute(
            "INSERT INTO versions VALUES(?1,?2,?3,?4,'copying',?5,?6,?7,?8,?9)",
            params![
                asset,
                version,
                parent,
                role,
                sha,
                bytes,
                extension,
                source.to_string_lossy(),
                text(c, "source_note")
            ],
        )?;
        event(&tx, "asset.import.started", version)?;
        tx.commit()?;
    }
    let (staging, final_path) = version_paths(root, asset, version, &role, &extension, true)?;
    if final_path.exists() {
        finish(root, db, asset, version)?;
        return file_info(root, db, asset, version);
    }
    if staging.exists() {
        fs::remove_file(&staging)?;
    }
    let mut from = File::open(&source)?;
    let mut to = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)?;
    let mut hasher = Sha256::new();
    let mut count = 0;
    let mut buffer = [0u8; 65536];
    loop {
        let n = from.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > bytes {
            return Err(Error("source_changed_during_copy"));
        }
        to.write_all(&buffer[..n])?;
        hasher.update(&buffer[..n]);
    }
    to.sync_all()?;
    drop(to);
    if count != bytes || format!("{:x}", hasher.finalize()) != sha {
        return Err(Error("source_changed_during_copy"));
    }
    finish(root, db, asset, version)?;
    file_info(root, db, asset, version)
}

fn finish(root: &Path, db: &mut Connection, asset: &str, version: &str) -> Result<()> {
    let (role, ext, sha, bytes): (String, String, String, u64) = db.query_row(
        "SELECT role,extension,sha256,bytes FROM versions WHERE asset_id=?1 AND id=?2",
        params![asset, version],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )?;
    let (staging, final_path) = version_paths(root, asset, version, &role, &ext, true)?;
    let candidate = if final_path.exists() {
        &final_path
    } else {
        &staging
    };
    if !candidate.is_file() {
        return Err(Error("import_waiting_for_source"));
    }
    if hash_file(candidate)? != (sha, bytes) {
        return Err(Error("import_integrity_mismatch"));
    }
    // Atomic create-without-replacement. Both paths are on the same local volume.
    if !final_path.exists() {
        fs::hard_link(&staging, &final_path).map_err(|_| Error("atomic_publish_failed"))?;
    }
    let tx = db.transaction()?;
    tx.execute(
        "UPDATE versions SET state='ready' WHERE asset_id=?1 AND id=?2",
        params![asset, version],
    )?;
    event(&tx, "asset.import.ready", version)?;
    tx.commit()?;
    if staging.exists() {
        fs::remove_file(staging)?;
    }
    Ok(())
}

fn file_info(root: &Path, db: &Connection, asset: &str, version: &str) -> Result<Value> {
    let row: Option<(String, String, String, String, u64)> = db
        .query_row(
            "SELECT state,role,extension,sha256,bytes FROM versions WHERE asset_id=?1 AND id=?2",
            params![asset, version],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()?;
    let (state, role, ext, sha, bytes) = row.ok_or(Error("version_not_found"))?;
    if state != "ready" {
        return Err(Error("version_not_ready"));
    }
    let (_, file) = version_paths(root, asset, version, &role, &ext, false)?;
    if hash_file(&file)? != (sha.clone(), bytes) {
        return Err(Error("asset_integrity_mismatch"));
    }
    Ok(
        json!({"asset_id":asset,"version_id":version,"role":role,"sha256":sha,"bytes":bytes,"path":file,"extension":ext,"state":"ready"}),
    )
}

fn recover(root: &Path, db: &mut Connection, c: &Value) -> Result<Value> {
    let limit = c["limit"].as_i64().unwrap_or(10);
    // IDs contain no colon, making this recovery-only cursor unambiguous.
    let after = text(c, "after");
    let mut stmt=db.prepare("SELECT asset_id,id FROM versions WHERE state='copying' AND asset_id||':'||id>?1 ORDER BY asset_id||':'||id LIMIT ?2")?;
    let rows = stmt
        .query_map(params![after, limit], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    drop(stmt);
    let mut items = Vec::new();
    for (asset, version) in rows {
        let state = match finish(root, db, &asset, &version) {
            Ok(()) => "ready",
            Err(e) => e.0,
        };
        items.push(json!({"asset_id":asset,"version_id":version,"result":state,"cursor":format!("{asset}:{version}")}));
    }
    let next = if items.len() == limit as usize {
        items.last().unwrap()["cursor"].clone()
    } else {
        Value::Null
    };
    Ok(json!({"items":items,"next_after":next}))
}
