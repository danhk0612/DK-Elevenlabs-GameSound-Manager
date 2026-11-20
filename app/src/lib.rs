pub mod audio;
mod audio_process;
mod audio_queue;
pub mod catalog;
#[cfg(any(windows, target_os = "linux"))]
mod engine;
pub mod generation;
#[cfg(target_os = "linux")]
mod linux;
pub mod mcp;
pub mod protocol;
pub mod provider;
mod review;
pub mod storage;
#[cfg(test)]
mod storage_tests;
#[cfg(windows)]
mod windows;

use serde_json::Value;

pub fn runtime() -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
}

/// All entrypoints share validation, transport, autostart and response envelopes.
pub async fn request(command: Value) -> Value {
    if let Err(code) = protocol::validate(&command) {
        return protocol::error(code);
    }
    #[cfg(windows)]
    {
        match windows::request(command).await {
            Ok(response) => response,
            Err(code) => protocol::error(code),
        }
    }
    #[cfg(target_os = "linux")]
    {
        match linux::request(command).await {
            Ok(response) => response,
            Err(code) => protocol::error(code),
        }
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    protocol::error("unsupported_platform")
}

pub async fn run_engine() -> Result<(), &'static str> {
    #[cfg(windows)]
    {
        windows::run_engine().await
    }
    #[cfg(target_os = "linux")]
    {
        linux::run_engine().await
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        Err("unsupported_platform")
    }
}
