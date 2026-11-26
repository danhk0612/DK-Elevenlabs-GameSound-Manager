#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde_json::Value;
#[tauri::command]
fn spike_command(command: Value) -> Value {
    t00_spike::response(command)
}
#[tauri::command]
fn fixture_audio() -> Result<String, String> {
    use base64::Engine;
    let bytes =
        std::fs::read(t00_spike::root()?.join("합성 음원.wav")).map_err(|e| e.to_string())?;
    Ok(format!(
        "data:audio/wav;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![spike_command, fixture_audio])
        .run(tauri::generate_context!())
        .expect("spike GUI failed");
}
