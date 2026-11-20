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
async fn audio_source(
    project_id: String,
    asset_id: String,
    version_id: String,
) -> Result<String, String> {
    let response=dk_gamesound::request(json!({"protocol_version":1,"op":"asset.file","project_id":project_id,"asset_id":asset_id,"version_id":version_id})).await;
    if response["ok"] != true {
        return Err(response["error"]["code"]
            .as_str()
            .unwrap_or("audio_unavailable")
            .to_string());
    }
    let data = response["data"].clone();
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            sound_status,
            local_query,
            audio_source
        ])
        .run(tauri::generate_context!())
        .expect("minimal GUI failed");
}
