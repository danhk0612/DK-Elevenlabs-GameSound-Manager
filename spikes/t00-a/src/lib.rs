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
fn connect() -> Result<(TcpStream, Value), String> {
    let endpoint: Value = serde_json::from_slice(
        &fs::read(root()?.join("endpoint.json")).map_err(|_| "engine unavailable")?,
    ).map_err(|_| "invalid endpoint")?;
    let port = endpoint["port"].as_u64().ok_or("invalid endpoint")?;
    let socket = TcpStream::connect(format!("127.0.0.1:{port}")).map_err(|_| "engine unavailable")?;
    Ok((socket, endpoint))
}
fn connect_or_start() -> Result<(TcpStream, Value), String> {
    if let Ok(c) = connect() { return Ok(c); }
    if std::env::var("T00_SPIKE_AUTOSTART").as_deref() != Ok("1") {
        return Err("engine unavailable".into());
    }
    // Opt-in fixture only. Serialize first-start attempts, never replay a sent command.
    use fs2::FileExt;
    use std::{fs::OpenOptions, process::{Command, Stdio}, time::Instant};
    fs::create_dir_all(root()?).map_err(|e| e.to_string())?;
    let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false)
        .open(root()?.join("starter.lock")).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if lock.try_lock_exclusive().is_ok() { break; }
        if Instant::now() >= deadline { return Err("starter timeout".into()); }
        std::thread::sleep(Duration::from_millis(20));
    }
    if let Ok(c) = connect() { return Ok(c); }
    let exe = std::env::current_exe().map_err(|e|e.to_string())?
        .with_file_name(if cfg!(windows) { "spike-engine.exe" } else { "spike-engine" });
    let mut cmd = Command::new(exe);
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW; independent of GUI lifetime.
    }
    let mut child = cmd.spawn().map_err(|e|e.to_string())?;
    loop {
        if let Ok(c) = connect() { return Ok(c); }
        if child.try_wait().map_err(|e|e.to_string())?.is_some() {
            // Another explicitly started engine may own the workspace; wait for publication.
            if let Ok(c) = connect() { return Ok(c); }
            return Err("fixture engine exited before readiness".into());
        }
        if Instant::now() >= deadline { return Err("engine readiness timeout".into()); }
        std::thread::sleep(Duration::from_millis(20));
    }
}
pub fn request(command: Value) -> Result<Value, String> {
    let (mut s, endpoint) = connect_or_start()?;
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
