use crate::{
    provider,
    storage::{Error, Result, Store},
};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs::OpenOptions, io::Write};

pub fn migrate(db: &mut Connection) -> Result<()> {
    let version: i64 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version == 1 {
        let tx = db.transaction()?;
        tx.execute_batch("CREATE TABLE accounts(id TEXT PRIMARY KEY,provider TEXT NOT NULL,identity TEXT);
CREATE TABLE budgets(identity TEXT PRIMARY KEY,request_limit INTEGER NOT NULL,credit_limit INTEGER NOT NULL);
CREATE TABLE generation_jobs(id TEXT PRIMARY KEY,project_id TEXT NOT NULL,account_id TEXT NOT NULL REFERENCES accounts(id),identity TEXT NOT NULL,
asset_id TEXT NOT NULL,version_id TEXT NOT NULL,snapshot TEXT NOT NULL,state TEXT NOT NULL,reservation INTEGER NOT NULL,
attempted INTEGER NOT NULL DEFAULT 0,confirmed INTEGER,result TEXT,error TEXT,extension TEXT,output_hash TEXT,
UNIQUE(project_id,asset_id,version_id));")?;
        tx.pragma_update(None, "user_version", 2)?;
        tx.commit()?;
    }
    // Never replay requests after engine restart. A queued/pre-dispatch job can be cancelled explicitly.
    db.execute("UPDATE generation_jobs SET state=CASE WHEN attempted=1 THEN 'outcome_unknown' ELSE 'interrupted' END,error='engine_restarted' WHERE state IN ('queued','running')",[])?;
    Ok(())
}

fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn profile(db: &Connection, id: &str) -> Result<(String, Option<String>)> {
    db.query_row(
        "SELECT provider,identity FROM accounts WHERE id=?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()?
    .ok_or(Error("account_not_found"))
}
fn identity(workspace: &str, provider: &str) -> String {
    format!("{provider}-{:x}", Sha256::digest(workspace.as_bytes()))
}
fn policy(db: &Connection, key: &str, calls: i64, credits: i64, tighten: bool) -> Result<()> {
    if calls <= 0 || credits <= 0 {
        return Err(Error("budget_not_configured"));
    }
    db.execute(if tighten {"INSERT INTO budgets VALUES(?1,?2,?3) ON CONFLICT(identity) DO UPDATE SET request_limit=min(request_limit,excluded.request_limit),credit_limit=min(credit_limit,excluded.credit_limit)"}else{"INSERT INTO budgets VALUES(?1,?2,?3) ON CONFLICT(identity) DO UPDATE SET request_limit=excluded.request_limit,credit_limit=excluded.credit_limit"},params![key,calls,credits])?;
    Ok(())
}
fn sync_account(store: &mut Store, id: &str) -> Result<Value> {
    let (provider, _) = profile(&store.catalog, id)?;
    let user = provider::json_response(provider::call(
        &provider,
        id,
        "GET",
        "/v1/user",
        &Value::Null,
    )?)?;
    let workspace = user["workspace_id"]
        .as_str()
        .filter(|w| !w.is_empty())
        .ok_or(Error("provider_workspace_identity_missing"))?;
    let key = identity(workspace, &provider);
    let old: Option<String> =
        store
            .catalog
            .query_row("SELECT identity FROM accounts WHERE id=?1", [id], |r| {
                r.get(0)
            })?;
    if old.as_deref().is_some_and(|old| old != key) {
        return Err(Error("account_identity_changed"));
    }
    store.catalog.execute(
        "UPDATE accounts SET identity=?2 WHERE id=?1",
        params![id, key],
    )?;
    if provider == "elevenlabs" {
        let connection = provider::credentials(id)?;
        if connection.enabled {
            policy(
                &store.catalog,
                &key,
                connection.request_limit,
                connection.credit_limit,
                true,
            )?;
        }
    }
    Ok(
        json!({"account_id":id,"provider":provider,"identity":key,"tier":bounded(&user["subscription"]["tier"],64),
        "service_usage":{"character_count":user["subscription"]["character_count"].as_u64(),"character_limit":user["subscription"]["character_limit"].as_u64()},
        "mock":provider=="mock"}),
    )
}
fn bounded(v: &Value, max: usize) -> Value {
    v.as_str()
        .map(|s| json!(s.chars().take(max).collect::<String>()))
        .unwrap_or(Value::Null)
}
fn voice(v: &Value) -> Value {
    json!({"voice_id":bounded(&v["voice_id"],128),"name":bounded(&v["name"],200),"category":bounded(&v["category"],64),"preview_url":v["preview_url"].as_str().filter(|url|url.starts_with("https://")&&url.len()<=2048)})
}
fn settings(v: &Value) -> Value {
    let mut out = json!({});
    for key in [
        "stability",
        "similarity_boost",
        "style",
        "speed",
        "use_speaker_boost",
    ] {
        if v[key].is_boolean() || v[key].is_number() {
            out[key] = v[key].clone();
        }
    }
    out
}
fn job(db: &Connection, project: &str, id: &str) -> Result<Value> {
    db.query_row("SELECT id,account_id,state,reservation,attempted,confirmed,result,error,asset_id,version_id FROM generation_jobs WHERE id=?1 AND project_id=?2",params![id,project],|r|{
        let result:Option<String>=r.get(6)?;
        Ok(json!({"job_id":r.get::<_,String>(0)?,"project_id":project,"account_id":r.get::<_,String>(1)?,"state":r.get::<_,String>(2)?,"reserved_credits":r.get::<_,i64>(3)?,"request_attempted":r.get::<_,i64>(4)?==1,"confirmed_credits":r.get::<_,Option<i64>>(5)?,"result":result.and_then(|s|serde_json::from_str::<Value>(&s).ok()),"error":r.get::<_,Option<String>>(7)?,"asset_id":r.get::<_,String>(8)?,"version_id":r.get::<_,String>(9)?}))
    }).optional()?.ok_or(Error("job_not_found"))
}

pub fn command(store: &mut Store, c: &Value) -> Result<Value> {
    let id = s(c, "account_id");
    let op = s(c, "op");
    match op {
        "account.create" => {
            let provider = s(c, "provider");
            if let Ok((old, _)) = profile(&store.catalog, id) {
                if old != provider {
                    return Err(Error("account_id_conflict"));
                }
            } else {
                store.catalog.execute(
                    "INSERT INTO accounts(id,provider) VALUES(?1,?2)",
                    params![id, provider],
                )?;
            }
            Ok(
                json!({"account_id":id,"provider":provider,"configured":provider=="mock"||provider::credentials(id).is_ok()}),
            )
        }
        "account.get" => {
            let (p, identity) = profile(&store.catalog, id)?;
            Ok(
                json!({"account_id":id,"provider":p,"identity":identity,"credential_configured":p=="mock"||provider::credentials(id).is_ok()}),
            )
        }
        "account.refresh" => sync_account(store, id),
        "budget.set" => {
            let (p, _) = profile(&store.catalog, id)?;
            if p != "mock" {
                return Err(Error("live_policy_requires_gui"));
            }
            let identity = sync_account(store, id)?["identity"]
                .as_str()
                .unwrap()
                .to_string();
            policy(
                &store.catalog,
                &identity,
                c["request_limit"].as_i64().unwrap(),
                c["credit_limit"].as_i64().unwrap(),
                false,
            )?;
            command(store, &json!({"op":"budget.get","account_id":id}))
        }
        "budget.get" => {
            let (_, identity) = profile(&store.catalog, id)?;
            let identity = identity.ok_or(Error("account_not_refreshed"))?;
            let cap: Option<(i64, i64)> = store
                .catalog
                .query_row(
                    "SELECT request_limit,credit_limit FROM budgets WHERE identity=?1",
                    [&identity],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            let (calls,held):(i64,i64)=store.catalog.query_row("SELECT count(*),coalesce(sum(max(reservation,coalesce(confirmed,0))),0) FROM generation_jobs WHERE identity=?1 AND state!='cancelled'",[&identity],|r|Ok((r.get(0)?,r.get(1)?)))?;
            Ok(
                json!({"identity":identity,"configured":cap.is_some(),"request_limit":cap.map(|v|v.0),"reserved_credit_limit":cap.map(|v|v.1),"requests_held":calls,"credits_held":held,"billing_cap_guaranteed":false}),
            )
        }
        "voice.list" | "voice.get" | "voice.settings" | "model.list" => {
            let (p, _) = profile(&store.catalog, id)?;
            let path = match op {
                "voice.list" => format!(
                    "/v2/voices?page_size=10&next_page_token={}",
                    provider::encoded(s(c, "page_token"))
                ),
                "voice.get" => format!("/v1/voices/{}", provider::encoded(s(c, "voice_id"))),
                "voice.settings" => format!(
                    "/v1/voices/{}/settings",
                    provider::encoded(s(c, "voice_id"))
                ),
                _ => "/v1/models".to_string(),
            };
            let v = provider::json_response(provider::call(&p, id, "GET", &path, &Value::Null)?)?;
            match op {
                "voice.list" => Ok(
                    json!({"items":v["voices"].as_array().ok_or(Error("invalid_provider_response"))?.iter().take(10).map(voice).collect::<Vec<_>>(),"next_page_token":bounded(&v["next_page_token"],200),"mock":p=="mock"}),
                ),
                "voice.get" => Ok(voice(&v)),
                "voice.settings" => Ok(settings(&v)),
                _ => Ok(
                    json!({"items":v.as_array().ok_or(Error("invalid_provider_response"))?.iter().take(30).map(|v|json!({"model_id":bounded(&v["model_id"],128),"name":bounded(&v["name"],200),"can_do_text_to_speech":v["can_do_text_to_speech"]==true})).collect::<Vec<_>>()}),
                ),
            }
        }
        "job.submit" => submit(store, c),
        "job.get" | "job.list" => read_query(&store.root, c),
        "job.cancel" => {
            store.project(s(c, "project_id"))?;
            let changed=store.catalog.execute("UPDATE generation_jobs SET state='cancelled',error=NULL WHERE id=?1 AND project_id=?2 AND attempted=0 AND state IN ('queued','interrupted','failed_preflight')",params![s(c,"job_id"),s(c,"project_id")])?;
            if changed == 0 {
                return Err(Error("job_not_cancellable"));
            }
            job(&store.catalog, s(c, "project_id"), s(c, "job_id"))
        }
        "job.recover" => {
            let state = job(&store.catalog, s(c, "project_id"), s(c, "job_id"))?;
            if state["state"] != "output_ready" {
                return Err(Error("job_output_not_recoverable"));
            }
            attach(store, s(c, "job_id"))?;
            job(&store.catalog, s(c, "project_id"), s(c, "job_id"))
        }
        _ => Err(Error("unsupported_operation")),
    }
}

fn target(store: &Store, c: &Value) -> Result<()> {
    let (_, db) = store.project(s(c, "project_id"))?;
    if db.query_row(
        "SELECT EXISTS(SELECT 1 FROM versions WHERE asset_id=?1 AND id=?2)",
        params![s(c, "asset_id"), s(c, "version_id")],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(Error("generation_target_exists"));
    }
    let asset: Option<(String, String)> = db
        .query_row(
            "SELECT name,kind FROM assets WHERE id=?1",
            [s(c, "asset_id")],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let kind = if s(c, "generation_kind") == "tts" {
        "voice"
    } else {
        "sfx"
    };
    if let Some((name, old_kind)) = asset {
        if name != s(c, "name") || old_kind != kind {
            return Err(Error("asset_id_conflict"));
        }
        if !db.query_row(
            "SELECT EXISTS(SELECT 1 FROM versions WHERE asset_id=?1 AND id=?2 AND state='ready')",
            params![s(c, "asset_id"), s(c, "parent_version_id")],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(Error("candidate_parent_required"));
        }
    } else if !s(c, "parent_version_id").is_empty() {
        return Err(Error("parent_version_not_found"));
    }
    if !s(c, "dialogue_id").is_empty() {
        let saved:Option<(String,String,String)>=db.query_row("SELECT d.text,d.language,c.voice_id FROM dialogues d JOIN characters c ON c.id=d.character_id WHERE d.id=?1",[s(c,"dialogue_id")],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
        if !saved.is_some_and(|(text, language, voice)| {
            text == s(c, "text") && language == s(c, "language") && voice == s(c, "voice_id")
        }) {
            return Err(Error("dialogue_snapshot_mismatch"));
        }
    }
    Ok(())
}

fn submit(store: &mut Store, c: &Value) -> Result<Value> {
    let id = s(c, "job_id");
    if let Some((project, snapshot)) = store
        .catalog
        .query_row(
            "SELECT project_id,snapshot FROM generation_jobs WHERE id=?1",
            [id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        if project != s(c, "project_id") || snapshot != c.to_string() {
            return Err(Error("job_id_conflict"));
        }
        return job(&store.catalog, &project, id);
    }
    if store.catalog.query_row("SELECT EXISTS(SELECT 1 FROM audio_operations WHERE project_id=?1 AND asset_id=?2 AND version_id=?3 AND kind='audio.edit')",params![s(c,"project_id"),s(c,"asset_id"),s(c,"version_id")],|r|r.get::<_,bool>(0))? {return Err(Error("audio_target_reserved"));}
    target(store, c)?;
    let account = sync_account(store, s(c, "account_id"))?;
    let identity = account["identity"].as_str().unwrap();
    if account["provider"] == "elevenlabs" && !provider::credentials(s(c, "account_id"))?.enabled {
        return Err(Error("live_generation_disabled"));
    }
    let tx = store
        .catalog
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let (request_limit, credit_limit): (i64, i64) = tx
        .query_row(
            "SELECT request_limit,credit_limit FROM budgets WHERE identity=?1",
            [identity],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or(Error("budget_not_configured"))?;
    let (count,held):(i64,i64)=tx.query_row("SELECT count(*),coalesce(sum(max(reservation,coalesce(confirmed,0))),0) FROM generation_jobs WHERE identity=?1 AND state!='cancelled'",[identity],|r|Ok((r.get(0)?,r.get(1)?)))?;
    let reservation = c["reserve_credits"].as_i64().unwrap();
    if count >= request_limit || held.saturating_add(reservation) > credit_limit {
        return Err(Error("budget_exceeded"));
    }
    if tx.query_row("SELECT EXISTS(SELECT 1 FROM generation_jobs WHERE project_id=?1 AND asset_id=?2 AND version_id=?3)",params![s(c,"project_id"),s(c,"asset_id"),s(c,"version_id")],|r|r.get::<_,bool>(0))? {return Err(Error("generation_target_reserved"));}
    tx.execute("INSERT INTO generation_jobs(id,project_id,account_id,identity,asset_id,version_id,snapshot,state,reservation) VALUES(?1,?2,?3,?4,?5,?6,?7,'queued',?8)",params![id,s(c,"project_id"),s(c,"account_id"),identity,s(c,"asset_id"),s(c,"version_id"),c.to_string(),reservation])?;
    tx.commit()?;
    job(&store.catalog, s(c, "project_id"), id)
}

pub fn pending(store: &Store) -> usize {
    store
        .catalog
        .query_row(
            "SELECT count(*) FROM generation_jobs WHERE state='queued'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
        .max(0) as usize
}
pub fn tick(store: &mut Store) -> Result<bool> {
    let next: Option<(String, String)> = store
        .catalog
        .query_row(
            "SELECT id,snapshot FROM generation_jobs WHERE state='queued' ORDER BY rowid LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((id, snapshot)) = next else {
        return Ok(false);
    };
    let c: Value = serde_json::from_str(&snapshot).map_err(|_| Error("invalid_job_snapshot"))?;
    let result = run(store, &id, &c);
    if let Err(error) = result {
        store.catalog.execute("UPDATE generation_jobs SET state=CASE WHEN attempted=1 THEN 'outcome_unknown' ELSE 'failed_preflight' END,error=?2 WHERE id=?1 AND state IN ('queued','running')",params![id,error.0])?;
    }
    Ok(true)
}

fn run(store: &mut Store, id: &str, c: &Value) -> Result<()> {
    target(store, c)?;
    let account = sync_account(store, s(c, "account_id"))?;
    let p = account["provider"].as_str().unwrap();
    if p == "elevenlabs" && !provider::credentials(s(c, "account_id"))?.enabled {
        return Err(Error("live_generation_disabled"));
    }
    let budget = command(
        store,
        &json!({"op":"budget.get","account_id":c["account_id"]}),
    )?;
    if budget["requests_held"].as_i64().unwrap_or(i64::MAX)
        > budget["request_limit"].as_i64().unwrap_or(0)
        || budget["credits_held"].as_i64().unwrap_or(i64::MAX)
            > budget["reserved_credit_limit"].as_i64().unwrap_or(0)
    {
        return Err(Error("budget_exceeded"));
    }
    let mut body = json!({"text":c["text"],"model_id":c["model_id"]});
    let path = if s(c, "generation_kind") == "tts" {
        let models = provider::json_response(provider::call(
            p,
            s(c, "account_id"),
            "GET",
            "/v1/models",
            &Value::Null,
        )?)?;
        if !models.as_array().is_some_and(|m| {
            m.iter()
                .any(|m| m["model_id"] == c["model_id"] && m["can_do_text_to_speech"] == true)
        }) {
            return Err(Error("tts_model_not_available"));
        }
        if let Some(language) = c.get("language") {
            body["language_code"] = language.clone();
        }
        if let Some(settings) = c.get("voice_settings") {
            body["voice_settings"] = settings.clone();
        }
        format!(
            "/v1/text-to-speech/{}?output_format=mp3_44100_128",
            provider::encoded(s(c, "voice_id"))
        )
    } else {
        if p == "elevenlabs"
            && !["eleven_text_to_sound_v2", "eleven_text_to_sound_v3"].contains(&s(c, "model_id"))
        {
            return Err(Error("unsupported_sfx_model"));
        }
        body["duration_seconds"] = json!(c["duration_ms"].as_f64().unwrap() / 1000.0);
        if let Some(looped) = c.get("loop") {
            body["loop"] = looped.clone();
        }
        if let Some(influence) = c.get("prompt_influence") {
            body["prompt_influence"] = influence.clone();
        }
        "/v1/sound-generation?output_format=mp3_44100_128".to_string()
    };
    store.catalog.execute(
        "UPDATE generation_jobs SET state='running',attempted=1 WHERE id=?1",
        [id],
    )?;
    // Exactly one POST. Network/HTTP/uncertain results are never retried here.
    let response = provider::call(p, s(c, "account_id"), "POST", &path, &body)?;
    store.catalog.execute(
        "UPDATE generation_jobs SET confirmed=?2 WHERE id=?1",
        params![id, response.credits],
    )?;
    if response.status != 200 {
        store.catalog.execute(
            "UPDATE generation_jobs SET state='failed',error=?2 WHERE id=?1",
            params![
                id,
                match response.status {
                    401 | 403 => "provider_access_denied",
                    429 => "provider_rate_limited",
                    _ => "provider_http_error",
                }
            ],
        )?;
        return Ok(());
    }
    let extension = if p == "mock" { "wav" } else { "mp3" };
    if response.body.len() < 4
        || p == "elevenlabs"
            && !(response.body.starts_with(b"ID3")
                || response.body[0] == 0xff && response.body[1] & 0xe0 == 0xe0)
    {
        return Err(Error("invalid_audio_response"));
    }
    let folder = store.root.join("generated");
    crate::storage::directory(&folder)?;
    let file = folder.join(format!("{id}.{extension}"));
    crate::storage::no_link(&file)?;
    let mut out = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&file)?;
    out.write_all(&response.body)?;
    out.sync_all()?;
    drop(out);
    let hash = format!("{:x}", Sha256::digest(&response.body));
    store.catalog.execute(
        "UPDATE generation_jobs SET state='output_ready',extension=?2,output_hash=?3 WHERE id=?1",
        params![id, extension, hash],
    )?;
    attach(store, id)
}

fn attach(store: &mut Store, id: &str) -> Result<()> {
    let (snapshot,ext,hash):(String,String,String)=store.catalog.query_row("SELECT snapshot,extension,output_hash FROM generation_jobs WHERE id=?1 AND state='output_ready'",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    let c: Value = serde_json::from_str(&snapshot).map_err(|_| Error("invalid_job_snapshot"))?;
    if !["mp3", "wav"].contains(&ext.as_str()) {
        return Err(Error("invalid_job_output"));
    }
    let file = store.root.join("generated").join(format!("{id}.{ext}"));
    crate::storage::no_link(&file)?;
    if crate::storage::hash_file(&file)?.0 != hash {
        return Err(Error("job_output_integrity_mismatch"));
    }
    let mut command = json!({"op":"asset.import","protocol_version":1,"project_id":c["project_id"],"asset_id":c["asset_id"],"version_id":c["version_id"],"name":c["name"],"kind":if c["generation_kind"]=="tts"{"voice"}else{"sfx"},"source_path":file,"source_note":format!("{} job {}; candidate only",if ext=="wav"{"mock synthetic"}else{"ElevenLabs"},id)});
    if let Some(parent) = c.get("parent_version_id") {
        command["parent_version_id"] = parent.clone();
    }
    let (root, mut db) = store.project(s(&c, "project_id"))?;
    let result = crate::storage::import_generated(&root, &mut db, &command)?;
    store.catalog.execute(
        "UPDATE generation_jobs SET state='succeeded',result=?2,error=NULL WHERE id=?1",
        params![id, result.to_string()],
    )?;
    // Keep spool output as recovery evidence; no automatic deletion of sound material.
    Ok(())
}

pub fn read_query(root: &std::path::Path, c: &Value) -> Result<Value> {
    crate::catalog::validate(c).map_err(Error)?;
    crate::storage::no_link(root)?;
    let path = root.join("catalog.sqlite3");
    crate::storage::no_link(&path)?;
    for suffix in ["catalog.sqlite3-wal", "catalog.sqlite3-shm"] {
        let path = root.join(suffix);
        if path.exists() {
            crate::storage::no_link(&path)?;
        }
    }
    let db = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let version: i64 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
    let app: i64 = db.pragma_query_value(None, "application_id", |r| r.get(0))?;
    if !(2..=3).contains(&version) || app != 1145784135 {
        return Err(Error("unsupported_database_schema"));
    }
    let exists: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id=?1)",
        [s(c, "project_id")],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(Error("project_not_found"));
    }
    match s(c, "op") {
        "job.get" => job(&db, s(c, "project_id"), s(c, "job_id")),
        "job.list" => {
            let limit = c["limit"].as_i64().unwrap_or(10);
            let mut statement = db.prepare(
                "SELECT id FROM generation_jobs WHERE project_id=?1 AND id>?2 ORDER BY id LIMIT ?3",
            )?;
            let ids = statement
                .query_map(params![s(c, "project_id"), s(c, "after"), limit], |r| {
                    r.get::<_, String>(0)
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let items = ids
                .iter()
                .map(|id| job(&db, s(c, "project_id"), id))
                .collect::<Result<Vec<_>>>()?;
            Ok(
                json!({"next_after":if ids.len()==limit as usize{ids.last()}else{None},"items":items}),
            )
        }
        _ => Err(Error("unsupported_operation")),
    }
}
