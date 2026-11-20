//! Review handoff records only. No approval, application, or source mutation.
use crate::storage::{self, Error, Result, Store};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;
fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v[k].as_str().unwrap_or("")
}
fn digest(v: &Value) -> String {
    format!("{:x}", Sha256::digest(v.to_string().as_bytes()))
}
pub(crate) fn migrate(db: &mut Connection) -> Result<()> {
    let version: i64 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version == 4 {
        let tx = db.transaction()?;
        tx.execute_batch("CREATE TABLE reviews(id TEXT PRIMARY KEY,project_id TEXT NOT NULL REFERENCES projects(id),asset_id TEXT NOT NULL,version_id TEXT NOT NULL,request TEXT NOT NULL,snapshot TEXT NOT NULL,state TEXT NOT NULL CHECK(state IN ('pending','withdrawn')),created_at TEXT NOT NULL DEFAULT(strftime('%Y-%m-%dT%H:%M:%fZ','now')),withdrawn_at TEXT,withdrawal_reason TEXT);
CREATE INDEX reviews_project ON reviews(project_id,id);
CREATE INDEX reviews_asset ON reviews(project_id,asset_id,version_id);")?;
        tx.pragma_update(None, "user_version", 5)?;
        tx.commit()?;
    }
    Ok(())
}
fn version(store: &Store, project: &str, asset: &str, id: &str) -> Result<Value> {
    let (_, db) = store.project(project)?;
    db.query_row("SELECT a.name,a.kind,v.parent_id,v.role,v.state,v.sha256,v.bytes,v.extension,v.source_note FROM versions v JOIN assets a ON a.id=v.asset_id WHERE a.id=?1 AND v.id=?2",params![asset,id],|r|Ok(json!({"asset_id":asset,"version_id":id,"name":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"parent_version_id":r.get::<_,Option<String>>(2)?,"role":r.get::<_,String>(3)?,"state":r.get::<_,String>(4)?,"sha256":r.get::<_,String>(5)?,"bytes":r.get::<_,u64>(6)?,"extension":r.get::<_,String>(7)?,"source_note":r.get::<_,String>(8)?}))).optional()?.ok_or(Error("version_not_found"))
}
fn requirement(store: &Store, project: &str, id: &str) -> Result<Value> {
    let (_, db) = store.project(project)?;
    db.query_row("SELECT name,kind,description,asset_id,version_id FROM requirements WHERE id=?1",[id],|r|Ok(json!({"requirement_id":id,"name":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"description":r.get::<_,String>(2)?,"asset_id":r.get::<_,Option<String>>(3)?,"version_id":r.get::<_,Option<String>>(4)?}))).optional()?.ok_or(Error("requirement_not_found"))
}
fn analysis(
    store: &Store,
    project: &str,
    id: &str,
    asset: &str,
    version: &str,
    hash: &str,
) -> Result<Value> {
    let row:Option<(String,String,String,String,String,Option<String>)>=store.catalog.query_row("SELECT kind,state,asset_id,version_id,input_hash,result FROM audio_operations WHERE project_id=?1 AND id=?2",params![project,id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional()?;
    let (kind, state, a, v, h, result) = row.ok_or(Error("review_analysis_not_found"))?;
    if kind != "audio.analyze" || state != "succeeded" || a != asset || v != version || h != hash {
        return Err(Error("review_analysis_mismatch"));
    }
    let report: Value =
        serde_json::from_str(result.as_deref().ok_or(Error("review_analysis_invalid"))?)
            .map_err(|_| Error("review_analysis_invalid"))?;
    if report["input_sha256"] != hash {
        return Err(Error("review_analysis_mismatch"));
    }
    let mut metrics = json!({});
    for field in [
        "duration_ms",
        "sample_peak",
        "sample_peak_dbfs",
        "rms_dbfs",
        "threshold_exceeding_samples",
        "quiet_ms",
        "longest_quiet_ms",
        "silence_region_count",
        "loop_seam_sample_delta",
    ] {
        metrics[field] = report["metrics"][field].clone();
    }
    Ok(
        json!({"analysis_id":id,"report_sha256":digest(&report),"metadata":report["metadata"],"conditions":report["conditions"],"tools":report["tools"],"metrics":metrics,"subjective_quality":"not_evaluated"}),
    )
}
fn get(store: &Store, project: &str, id: &str) -> Result<Value> {
    store.project(project)?;
    store.catalog.query_row("SELECT state,snapshot,created_at,withdrawn_at,withdrawal_reason FROM reviews WHERE project_id=?1 AND id=?2",params![project,id],|r|{
        let snapshot:String=r.get(1)?;
        Ok(json!({"review_id":id,"project_id":project,"state":r.get::<_,String>(0)?,"snapshot":serde_json::from_str::<Value>(&snapshot).ok(),"created_at":r.get::<_,String>(2)?,"withdrawn_at":r.get::<_,Option<String>>(3)?,"withdrawal_reason":r.get::<_,Option<String>>(4)?,"approval":"not_granted","current_integrity":"not_checked"}))
    }).optional()?.ok_or(Error("review_not_found"))
}
fn submit(store: &mut Store, c: &Value) -> Result<Value> {
    let project = s(c, "project_id");
    let id = s(c, "review_id");
    let asset = s(c, "asset_id");
    let ver = s(c, "version_id");
    if let Some((p, request)) = store
        .catalog
        .query_row(
            "SELECT project_id,request FROM reviews WHERE id=?1",
            [id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        if p != project || request != c.to_string() {
            return Err(Error("review_id_conflict"));
        }
        return get(store, project, id);
    }
    let v = version(store, project, asset, ver)?;
    if v["state"] != "ready" || v["role"] != "candidate" {
        return Err(Error("review_ready_candidate_required"));
    }
    if v["sha256"] != c["expected_sha256"] {
        return Err(Error("review_candidate_hash_mismatch"));
    }
    let file=store.execute(&json!({"protocol_version":1,"op":"asset.file","project_id":project,"asset_id":asset,"version_id":ver}));
    if file["ok"] != true {
        return Err(Error("review_candidate_unavailable"));
    }
    if file["data"]["sha256"] != c["expected_sha256"] {
        return Err(Error("review_candidate_hash_mismatch"));
    }
    let req = if s(c, "requirement_id").is_empty() {
        Value::Null
    } else {
        let req = requirement(store, project, s(c, "requirement_id"))?;
        if req["kind"] != v["kind"] {
            return Err(Error("requirement_kind_mismatch"));
        }
        req
    };
    let report = if s(c, "analysis_id").is_empty() {
        Value::Null
    } else {
        analysis(
            store,
            project,
            s(c, "analysis_id"),
            asset,
            ver,
            s(c, "expected_sha256"),
        )?
    };
    let snapshot = json!({"candidate":v,"requirement":req,"analysis":report,"summary":c["summary"],"notes":s(c,"notes"),"submitted_by":"automation_or_local_client","approval":"not_granted"});
    store.catalog.execute("INSERT INTO reviews(id,project_id,asset_id,version_id,request,snapshot,state) VALUES(?1,?2,?3,?4,?5,?6,'pending')",params![id,project,asset,ver,c.to_string(),snapshot.to_string()])?;
    get(store, project, id)
}
fn list(store: &Store, c: &Value) -> Result<Value> {
    store.project(s(c, "project_id"))?;
    let limit = c["limit"].as_i64().unwrap_or(10);
    let mut statement=store.catalog.prepare("SELECT id,asset_id,version_id,state,created_at,withdrawn_at FROM reviews WHERE project_id=?1 AND id>?2 AND (?3='' OR state=?3) AND (?4='' OR asset_id=?4) AND (?5='' OR version_id=?5) ORDER BY id LIMIT ?6")?;
    let items=statement.query_map(params![s(c,"project_id"),s(c,"after"),s(c,"state"),s(c,"asset_id"),s(c,"version_id"),limit],|r|Ok(json!({"review_id":r.get::<_,String>(0)?,"asset_id":r.get::<_,String>(1)?,"version_id":r.get::<_,String>(2)?,"state":r.get::<_,String>(3)?,"created_at":r.get::<_,String>(4)?,"withdrawn_at":r.get::<_,Option<String>>(5)?})))?.collect::<std::result::Result<Vec<_>,_>>()?;
    Ok(
        json!({"next_after":if items.len()==limit as usize{items.last().map(|r|r["review_id"].clone())}else{None},"items":items}),
    )
}
fn withdraw(store: &mut Store, c: &Value) -> Result<Value> {
    let project = s(c, "project_id");
    let id = s(c, "review_id");
    store.project(project)?;
    let tx = store
        .catalog
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let (state, reason): (String, Option<String>) = tx
        .query_row(
            "SELECT state,withdrawal_reason FROM reviews WHERE project_id=?1 AND id=?2",
            params![project, id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or(Error("review_not_found"))?;
    if state == "withdrawn" && reason.as_deref() != Some(s(c, "reason")) {
        return Err(Error("review_withdrawal_conflict"));
    }
    if state == "pending" {
        tx.execute("UPDATE reviews SET state='withdrawn',withdrawn_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),withdrawal_reason=?2 WHERE id=?1",params![id,s(c,"reason")])?;
    }
    tx.commit()?;
    get(store, project, id)
}
fn check(store: &mut Store, c: &Value) -> Result<Value> {
    let record = get(store, s(c, "project_id"), s(c, "review_id"))?;
    let snapshot = &record["snapshot"];
    let candidate = &snapshot["candidate"];
    let file=store.execute(&json!({"protocol_version":1,"op":"asset.file","project_id":c["project_id"],"asset_id":candidate["asset_id"],"version_id":candidate["version_id"]}));
    let file_match = file["ok"] == true && file["data"]["sha256"] == candidate["sha256"];
    let req_match = if snapshot["requirement"].is_null() {
        None
    } else {
        Some(
            requirement(
                store,
                s(c, "project_id"),
                s(&snapshot["requirement"], "requirement_id"),
            )
            .is_ok_and(|r| r == snapshot["requirement"]),
        )
    };
    let analysis_match = if snapshot["analysis"].is_null() {
        None
    } else {
        Some(
            analysis(
                store,
                s(c, "project_id"),
                s(&snapshot["analysis"], "analysis_id"),
                s(candidate, "asset_id"),
                s(candidate, "version_id"),
                s(candidate, "sha256"),
            )
            .is_ok_and(|r| r["report_sha256"] == snapshot["analysis"]["report_sha256"]),
        )
    };
    Ok(
        json!({"review_id":c["review_id"],"project_id":c["project_id"],"state":record["state"],"matches":file_match&&req_match!=Some(false)&&analysis_match!=Some(false),"candidate_matches":file_match,"candidate_error":if file["ok"]==true{Value::Null}else{file["error"]["code"].clone()},"requirement_matches":req_match,"analysis_matches":analysis_match,"approval":"not_granted","check_scope":"current_bytes_and_linked_metadata_only"}),
    )
}
fn history(store: &Store, c: &Value) -> Result<Value> {
    let project = s(c, "project_id");
    let asset = s(c, "asset_id");
    let ver = s(c, "version_id");
    let v = version(store, project, asset, ver)?;
    let limit = c["limit"].as_i64().unwrap_or(10);
    let mut statement=store.catalog.prepare("SELECT cursor,kind,id,state,error FROM (
SELECT 'generation:'||id AS cursor,'generation' AS kind,id,state,error FROM generation_jobs WHERE project_id=?1 AND asset_id=?2 AND version_id=?3
UNION ALL SELECT 'audio:'||id,kind,id,state,error FROM audio_operations WHERE project_id=?1 AND asset_id=?2 AND version_id=?3
UNION ALL SELECT 'review:'||id,'review',id,state,NULL FROM reviews WHERE project_id=?1 AND asset_id=?2 AND version_id=?3
) WHERE cursor>?4 ORDER BY cursor LIMIT ?5")?;
    let items=statement.query_map(params![project,asset,ver,s(c,"after"),limit],|r|Ok(json!({"cursor":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"id":r.get::<_,String>(2)?,"state":r.get::<_,String>(3)?,"error":r.get::<_,Option<String>>(4)?})))?.collect::<std::result::Result<Vec<_>,_>>()?;
    Ok(
        json!({"version":v,"file_verified":false,"order":"category_then_identifier","next_after":if items.len()==limit as usize{items.last().map(|r|r["cursor"].clone())}else{None},"items":items}),
    )
}
pub(crate) fn command(store: &mut Store, c: &Value) -> Result<Value> {
    match s(c, "op") {
        "review.submit" => submit(store, c),
        "review.get" => get(store, s(c, "project_id"), s(c, "review_id")),
        "review.list" => list(store, c),
        "review.withdraw" => withdraw(store, c),
        "review.check" => check(store, c),
        "asset.history" => history(store, c),
        _ => Err(Error("unsupported_operation")),
    }
}
#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub(crate) fn read(root: &Path, c: &Value) -> Result<Value> {
    crate::catalog::validate(c).map_err(Error)?;
    if !matches!(s(c, "op"), "review.get" | "review.list" | "asset.history") {
        return Err(Error("unsupported_operation"));
    }
    let mut store = Store {
        root: root.to_path_buf(),
        catalog: storage::catalog_connection(root, false)?,
    };
    command(&mut store, c)
}
