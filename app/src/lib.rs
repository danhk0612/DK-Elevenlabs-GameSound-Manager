pub mod catalog;
pub mod mcp;
pub mod protocol;
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
    #[cfg(not(windows))]
    protocol::error("unsupported_platform")
}

pub async fn run_engine() -> Result<(), &'static str> {
    #[cfg(windows)]
    {
        windows::run_engine().await
    }
    #[cfg(not(windows))]
    {
        Err("unsupported_platform")
    }
}
