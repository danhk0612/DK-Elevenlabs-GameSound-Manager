//! Persistent audio submission and cooperative cancellation. Only the engine executes media work.
use crate::{
    audio,
    storage::{self, Error, Result, Store},
};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::path::Path;
fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v[k].as_str().unwrap_or("")
}
pub(crate) fn migrate(db: &mut Connection) -> Result<()> {
    let version: i64 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version == 3 {
        let tx = db.transaction()?;
        tx.execute_batch(
            "ALTER TABLE audio_operations ADD COLUMN cancel_requested INTEGER NOT NULL DEFAULT 0;
ALTER TABLE audio_operations ADD COLUMN phase TEXT NOT NULL DEFAULT 'legacy';
ALTER TABLE audio_operations ADD COLUMN tools_snapshot TEXT;",
        )?;
        tx.pragma_update(None, "user_version", 4)?;
        tx.commit()?;
    }
    db.execute("UPDATE audio_operations SET state=CASE WHEN cancel_requested=1 THEN 'cancelled' ELSE 'interrupted' END,phase='engine_restarted',error='engine_restarted' WHERE state IN ('queued','processing')",[])?;
    Ok(())
}
pub(crate) struct Control {
    db: Connection,
    id: String,
}
impl Control {
    pub(crate) fn new(root: &Path, id: &str) -> Result<Self> {
        Ok(Self {
            db: storage::catalog_connection(root, true)?,
            id: id.to_string(),
        })
    }
    pub(crate) fn check(&self) -> Result<()> {
        let (cancel, state): (bool, String) = self.db.query_row(
            "SELECT cancel_requested,state FROM audio_operations WHERE id=?1",
            [&self.id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if cancel || state != "processing" {
            return Err(Error("audio_cancelled"));
        }
        Ok(())
    }
    pub(crate) fn phase(&self, phase: &str) -> Result<()> {
        self.check()?;
        self.db.execute(
            "UPDATE audio_operations SET phase=?2 WHERE id=?1 AND state='processing'",
            params![self.id, phase],
        )?;
        Ok(())
    }
}
pub(crate) fn canonical(c: &Value) -> Result<Value> {
    let op = match s(c, "action") {
        "analyze" => "audio.analyze",
        "edit" => "audio.edit",
        _ => return Err(Error("invalid_audio_action")),
    };
    let mut request = c.clone();
    request
        .as_object_mut()
        .ok_or(Error("invalid_request"))?
        .remove("action");
    request["op"] = json!(op);
    crate::catalog::validate(&request).map_err(Error)?;
    Ok(request)
}
fn input_hash(store: &Store, c: &Value) -> Result<String> {
    let (_, db) = store.project(s(c, "project_id"))?;
    let version = if c["op"] == "audio.edit" {
        s(c, "parent_version_id")
    } else {
        s(c, "version_id")
    };
    db.query_row(
        "SELECT sha256 FROM versions WHERE asset_id=?1 AND id=?2 AND state='ready'",
        params![s(c, "asset_id"), version],
        |r| r.get(0),
    )
    .optional()?
    .ok_or(Error("audio_source_unavailable"))
}
pub(crate) fn submit(store: &mut Store, c: &Value) -> Result<Value> {
    let c = canonical(c)?;
    let project = s(&c, "project_id");
    let id = s(&c, "operation_id");
    if let Some((old_project, snapshot)) = store
        .catalog
        .query_row(
            "SELECT project_id,snapshot FROM audio_operations WHERE id=?1",
            [id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        if old_project != project || snapshot != c.to_string() {
            return Err(Error("audio_operation_id_conflict"));
        }
        return audio::operation(store, project, id);
    }
    let hash = input_hash(store, &c)?;
    let (_, project_db) = store.project(project)?;
    // Pin configured metadata without starting executables or hashing large media on submission.
    let tools: String = store
        .catalog
        .query_row("SELECT settings FROM audio_tools WHERE id=1", [], |r| {
            r.get(0)
        })
        .optional()?
        .ok_or(Error("audio_tools_not_configured"))?;
    let tx = store
        .catalog
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    if let Some((old_project, snapshot)) = tx
        .query_row(
            "SELECT project_id,snapshot FROM audio_operations WHERE id=?1",
            [id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        if old_project != project || snapshot != c.to_string() {
            return Err(Error("audio_operation_id_conflict"));
        }
        drop(tx);
        return audio::operation(store, project, id);
    }
    let queued: i64 = tx.query_row(
        "SELECT count(*) FROM audio_operations WHERE state='queued'",
        [],
        |r| r.get(0),
    )?;
    if queued >= 32 {
        return Err(Error("audio_queue_full"));
    }
    if c["op"] == "audio.edit" {
        if project_db.query_row("SELECT EXISTS(SELECT 1 FROM versions WHERE asset_id=?1 AND id=?2)",params![s(&c,"asset_id"),s(&c,"version_id")],|r|r.get::<_,bool>(0))?
        ||tx.query_row("SELECT EXISTS(SELECT 1 FROM generation_jobs WHERE project_id=?1 AND asset_id=?2 AND version_id=?3) OR EXISTS(SELECT 1 FROM audio_operations WHERE project_id=?1 AND asset_id=?2 AND version_id=?3 AND kind='audio.edit')",params![project,s(&c,"asset_id"),s(&c,"version_id")],|r|r.get::<_,bool>(0))?{return Err(Error("audio_target_reserved"));}
    }
    tx.execute("INSERT INTO audio_operations(id,project_id,kind,snapshot,input_hash,state,asset_id,version_id,phase,tools_snapshot) VALUES(?1,?2,?3,?4,?5,'queued',?6,?7,'queued',?8)",params![id,project,s(&c,"op"),c.to_string(),hash,s(&c,"asset_id"),s(&c,"version_id"),tools])?;
    tx.commit()?;
    audio::operation(store, project, id)
}
pub(crate) fn cancel(store: &mut Store, c: &Value) -> Result<Value> {
    let project = s(c, "project_id");
    let id = s(c, "operation_id");
    store.project(project)?;
    let tx = store
        .catalog
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let state: String = tx
        .query_row(
            "SELECT state FROM audio_operations WHERE project_id=?1 AND id=?2",
            params![project, id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or(Error("audio_operation_not_found"))?;
    match state.as_str() {
        "cancelled" => (),
        "queued" | "interrupted" => {
            tx.execute("UPDATE audio_operations SET state='cancelled',cancel_requested=1,phase='cancelled',error='audio_cancelled' WHERE id=?1",[id])?;
        }
        "processing" => {
            tx.execute(
                "UPDATE audio_operations SET cancel_requested=1 WHERE id=?1",
                [id],
            )?;
        }
        _ => return Err(Error("audio_not_cancellable")),
    }
    tx.commit()?;
    audio::operation(store, project, id)
}
#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub(crate) fn pending(store: &Store) -> usize {
    store
        .catalog
        .query_row(
            "SELECT count(*) FROM audio_operations WHERE state='queued'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
        .max(0) as usize
}
pub(crate) fn fail(store: &Store, id: &str, error: &Error) -> Result<()> {
    store.catalog.execute("UPDATE audio_operations SET state=CASE WHEN state='output_ready' THEN state WHEN cancel_requested=1 OR ?2='audio_cancelled' THEN 'cancelled' ELSE 'failed' END,phase=CASE WHEN state='output_ready' THEN phase WHEN cancel_requested=1 OR ?2='audio_cancelled' THEN 'cancelled' ELSE 'failed' END,error=?2 WHERE id=?1 AND state IN ('processing','output_ready')",params![id,error.0])?;
    Ok(())
}
#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub(crate) fn tick(store: &mut Store) -> Result<bool> {
    let row:Option<(String,String,String,Option<String>)>=store.catalog.query_row("SELECT id,snapshot,input_hash,tools_snapshot FROM audio_operations WHERE state='queued' ORDER BY rowid LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
    let Some((id, snapshot, hash, tools)) = row else {
        return Ok(false);
    };
    if store.catalog.execute("UPDATE audio_operations SET state='processing',phase='starting' WHERE id=?1 AND state='queued' AND cancel_requested=0",[&id])?==0{return Ok(true);}
    let result = (|| {
        let c: Value =
            serde_json::from_str(&snapshot).map_err(|_| Error("invalid_audio_snapshot"))?;
        crate::catalog::validate(&c).map_err(Error)?;
        let tools: Value =
            serde_json::from_str(tools.as_deref().ok_or(Error("audio_tools_invalid"))?)
                .map_err(|_| Error("audio_tools_invalid"))?;
        let control = Control::new(&store.root, &id)?;
        control.phase("checking_tools")?;
        audio::verify_tools(&tools)?;
        control.phase("checking_input")?;
        let input = audio::source(store, &c)?;
        if input["sha256"] != hash {
            return Err(Error("audio_source_changed"));
        }
        control.check()?;
        audio::run_registered(store, &c, &input, &tools, &control)
    })();
    if let Err(error) = result {
        fail(store, &id, &error)?;
    }
    Ok(true)
}
#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub(crate) fn fast(root: &Path, c: &Value) -> Result<(Value, usize)> {
    crate::catalog::validate(c).map_err(Error)?;
    let write = matches!(s(c, "op"), "audio.submit" | "audio.cancel");
    let mut store = Store {
        root: root.to_path_buf(),
        catalog: storage::catalog_connection(root, write)?,
    };
    let value = match s(c, "op") {
        "audio.submit" => submit(&mut store, c)?,
        "audio.cancel" => cancel(&mut store, c)?,
        "audio.get" | "audio.list" => audio::command(&mut store, c)?,
        _ => return Err(Error("unsupported_operation")),
    };
    Ok((value, pending(&store)))
}

#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub(crate) fn pending_at(root: &Path) -> Result<usize> {
    let db = storage::catalog_connection(root, false)?;
    let n: i64 = db.query_row(
        "SELECT count(*) FROM audio_operations WHERE state='queued'",
        [],
        |r| r.get(0),
    )?;
    Ok(n.max(0) as usize)
}

pub(crate) fn publish_output(
    store: &Store,
    id: &str,
    ext: &str,
    hash: &str,
    details: &Value,
) -> Result<()> {
    if store.catalog.execute("UPDATE audio_operations SET state='output_ready',phase='publishing',extension=?2,output_hash=?3,result=?4 WHERE id=?1 AND state='processing' AND cancel_requested=0",params![id,ext,hash,details.to_string()])?==0{return Err(Error("audio_cancelled"));}
    Ok(())
}
