#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde_json::{json, Value};

#[tauri::command]
async fn sound_status() -> Value {
    dk_gamesound::request(json!({"protocol_version":1,"op":"status"})).await
}

#[tauri::command]
async fn local_query(command: Value) -> Value {
    let Some(def) = command["op"].as_str().and_then(dk_gamesound::catalog::find) else {
        return dk_gamesound::protocol::error("unsupported_operation");
    };
    if !def.read_only {
        return dk_gamesound::protocol::error("gui_query_only");
    }
    dk_gamesound::request(command).await
}

#[tauri::command]
async fn cancel_audio(project_id: String, operation_id: String) -> Value {
    dk_gamesound::request(json!({"protocol_version":1,"op":"audio.cancel","project_id":project_id,"operation_id":operation_id})).await
}

#[tauri::command]
async fn audio_source(
    project_id: String,
    asset_id: String,
    version_id: String,
    expected_sha256: Option<String>,
) -> Result<String, String> {
    let response=dk_gamesound::request(json!({"protocol_version":1,"op":"asset.file","project_id":project_id,"asset_id":asset_id,"version_id":version_id})).await;
    if response["ok"] != true {
        return Err(response["error"]["code"]
            .as_str()
            .unwrap_or("audio_unavailable")
            .to_string());
    }
    let data = response["data"].clone();
    if expected_sha256
        .as_ref()
        .is_some_and(|hash| data["sha256"].as_str() != Some(hash.as_str()))
    {
        return Err("review_candidate_hash_mismatch".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        use base64::Engine;
        use sha2::{Digest, Sha256};
        use std::io::Read;
        const LIMIT: u64 = 16 * 1024 * 1024;
        if data["bytes"].as_u64().unwrap_or(u64::MAX) > LIMIT {
            return Err("preview_limit_16_mib".to_string());
        }
        let path = data["path"].as_str().ok_or("audio_unavailable")?;
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|_| "audio_read_failed")?
            .take(LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "audio_read_failed")?;
        if bytes.len() as u64 > LIMIT
            || format!("{:x}", Sha256::digest(&bytes)) != data["sha256"].as_str().unwrap_or("")
        {
            return Err("audio_integrity_mismatch".to_string());
        }
        let mime = match data["extension"].as_str().unwrap_or("") {
            "wav" => "audio/wav",
            "mp3" => "audio/mpeg",
            "ogg" | "opus" => "audio/ogg",
            "m4a" => "audio/mp4",
            "aac" => "audio/aac",
            "flac" => "audio/flac",
            _ => return Err("preview_format_not_supported".to_string()),
        };
        Ok(format!(
            "data:{mime};base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        ))
    })
    .await
    .map_err(|_| "audio_read_failed".to_string())?
}

#[tauri::command]
async fn save_connection(
    account_id: String,
    key: String,
    enabled: bool,
    request_limit: i64,
    credit_limit: i64,
) -> Value {
    let created=dk_gamesound::request(json!({"protocol_version":1,"op":"account.create","account_id":account_id,"provider":"elevenlabs"})).await;
    if created["ok"] != true {
        return created;
    }
    tauri::async_runtime::spawn_blocking(move || {
        match dk_gamesound::provider::save(&account_id, key, enabled, request_limit, credit_limit) {
            Ok(()) => dk_gamesound::protocol::success(json!({"saved":true})),
            Err(error) => dk_gamesound::protocol::error(error.0),
        }
    })
    .await
    .unwrap_or_else(|_| dk_gamesound::protocol::error("credential_write_failed"))
}

#[tauri::command]
async fn disconnect_connection(account_id: String) -> Value {
    match dk_gamesound::provider::disconnect(&account_id) {
        Ok(()) => dk_gamesound::protocol::success(json!({"disconnected":true})),
        Err(error) => dk_gamesound::protocol::error(error.0),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            sound_status,
            local_query,
            audio_source,
            cancel_audio,
            save_connection,
            disconnect_connection
        ])
        .run(tauri::generate_context!())
        .expect("minimal GUI failed");
}
