#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde_json::{json, Value};

#[tauri::command]
async fn sound_status() -> Value {
    dk_gamesound::request(json!({"protocol_version":1,"op":"status"})).await
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![sound_status])
        .run(tauri::generate_context!())
        .expect("minimal GUI failed");
}
