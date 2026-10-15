use serde_json::{json, Value};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    path::PathBuf,
    time::Duration,
};
pub fn root() -> Result<PathBuf, String> {
    let p = PathBuf::from(std::env::var_os("T00_SPIKE_ROOT").ok_or("T00_SPIKE_ROOT required")?);
    if !p.is_absolute() {
        return Err("absolute temporary root required".into());
    }
    Ok(p)
}
pub fn request(command: Value) -> Result<Value, String> {
    let endpoint: Value = serde_json::from_slice(
        &fs::read(root()?.join("endpoint.json")).map_err(|_| "engine unavailable")?,
    )
    .map_err(|_| "invalid endpoint")?;
    let port = endpoint["port"].as_u64().ok_or("invalid endpoint")?;
    let mut s =
        TcpStream::connect(format!("127.0.0.1:{port}")).map_err(|_| "engine unavailable")?;
    s.set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    writeln!(
        s,
        "{}",
        json!({"token": endpoint["token"], "command": command})
    )
    .map_err(|e| e.to_string())?;
    let mut line = String::new();
    BufReader::new(s)
        .read_line(&mut line)
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&line).map_err(|_| "invalid engine response".into())
}
pub fn response(command: Value) -> Value {
    request(command).unwrap_or_else(|e| json!({"ok":false,"error":e}))
}
