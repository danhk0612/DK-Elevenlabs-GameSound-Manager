use serde_json::{json, Value};
use std::io;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const PROTOCOL: u64 = 1;
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(300);
pub const MAX_FRAME: usize = 64 * 1024;

pub fn validate(command: &Value) -> Result<(), &'static str> {
    crate::catalog::validate(command)
}

pub fn error(code: &str) -> Value {
    json!({"protocol_version":PROTOCOL,"ok":false,"error":{"code":code}})
}

pub fn success(data: Value) -> Value {
    json!({"protocol_version":PROTOCOL,"ok":true,"data":data})
}

pub async fn read_frame<R: AsyncRead + Unpin>(stream: &mut R) -> io::Result<Value> {
    let size = stream.read_u32_le().await? as usize;
    if size == 0 || size > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid frame size",
        ));
    }
    let mut bytes = vec![0; size];
    stream.read_exact(&mut bytes).await?;
    serde_json::from_slice(&bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid JSON"))
}

pub async fn write_frame<W: AsyncWrite + Unpin>(stream: &mut W, value: &Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    stream.write_u32_le(bytes.len() as u32).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await
}

pub struct State {
    pub clients: usize,
    // Local commands and persistent remote jobs both prevent idle shutdown.
    pub queued: usize,
    pub running: usize,
    pub generation_queued: usize,
    pub generation_running: usize,
    pub stopping: bool,
    pub shutdown_flushed: bool,
    pub last_empty: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            clients: 0,
            queued: 0,
            running: 0,
            generation_queued: 0,
            generation_running: 0,
            stopping: false,
            shutdown_flushed: false,
            last_empty: Instant::now(),
        }
    }
}

impl State {
    pub fn connect(&mut self, now: Instant) {
        self.clients += 1;
        self.last_empty = now;
    }
    pub fn disconnect(&mut self, now: Instant) {
        self.clients = self.clients.saturating_sub(1);
        if self.clients == 0 {
            self.last_empty = now;
        }
    }
    pub fn busy(&self) -> bool {
        self.queued != 0
            || self.running != 0
            || self.generation_queued != 0
            || self.generation_running != 0
    }
    pub fn idle_expired(&self, now: Instant) -> bool {
        self.clients == 0 && !self.busy() && now.duration_since(self.last_empty) >= IDLE_TIMEOUT
    }
    pub fn execute(&mut self, command: &Value) -> Value {
        if let Err(code) = validate(command) {
            return error(code);
        }
        if self.stopping {
            return error("engine_stopping");
        }
        match command["op"].as_str().unwrap() {
            "shutdown" if self.busy() => error("engine_busy"),
            "shutdown" => {
                self.stopping = true;
                success(json!({"shutdown":"accepted"}))
            }
            op @ ("version" | "status") => {
                let mut data = json!({"name":"DK-Elevenlabs-GameSound-Manager",
                    "version":env!("CARGO_PKG_VERSION"),"engine_pid":std::process::id(),
                    "capabilities":crate::catalog::COMMANDS.iter().map(|c|c.op).collect::<Vec<_>>(),
                    "stage":"T03","sound_features_available":true,"remote_generation_available":true,"remote_generation_requires":"local_credentials_and_explicit_limits"});
                if op == "status" {
                    data["state"] = json!("ready");
                    data["connected_clients"] = json!(self.clients);
                    data["jobs"] = json!({"available":true,"scope":"serialized_local_and_generation","generation_queued":self.generation_queued,"generation_running":self.generation_running,"queued":self.queued+self.generation_queued,"running":self.running+self.generation_running});
                    data["idle_timeout_ms"] = json!(IDLE_TIMEOUT.as_millis());
                    data["transport"] = json!("windows_named_pipe_owner_sid_local_only");
                }
                success(data)
            }
            _ => error("requires_storage_dispatch"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_hidden_commands_or_extra_fields() {
        for v in [
            json!({"protocol_version":1,"op":"apply"}),
            json!({"protocol_version":1,"op":"shutdown","force":true}),
            json!({"protocol_version":2,"op":"status"}),
            json!([]),
        ] {
            assert!(validate(&v).is_err());
        }
    }
    #[test]
    fn busy_shutdown_and_continuous_idle() {
        let mut state = State::default();
        let start = state.last_empty;
        state.queued = 1;
        assert!(!state.idle_expired(start + IDLE_TIMEOUT));
        assert_eq!(
            state.execute(&json!({"protocol_version":1,"op":"shutdown"}))["error"]["code"],
            "engine_busy"
        );
        state.queued = 0;
        state.connect(start + IDLE_TIMEOUT);
        assert!(!state.idle_expired(start + IDLE_TIMEOUT * 2));
        state.disconnect(start + IDLE_TIMEOUT * 2);
        assert!(!state.idle_expired(start + IDLE_TIMEOUT * 2 + Duration::from_secs(299)));
        assert!(state.idle_expired(start + IDLE_TIMEOUT * 3));
    }
    #[test]
    fn shutdown_needs_response_and_rejects_further_commands() {
        let mut state = State::default();
        assert_eq!(
            state.execute(&json!({"protocol_version":1,"op":"shutdown"}))["ok"],
            true
        );
        assert!(!state.shutdown_flushed);
        assert_eq!(
            state.execute(&json!({"protocol_version":1,"op":"status"}))["error"]["code"],
            "engine_stopping"
        );
    }
}
