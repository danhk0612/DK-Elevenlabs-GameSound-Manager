//! Opt-in Linux headless host. No network listener or credential fallback.
use crate::{
    engine::{serve, storage_worker},
    protocol::{self, State},
};
use serde_json::Value;
use std::os::unix::process::CommandExt;
use std::{
    fs::{self, File, OpenOptions},
    os::{
        fd::AsRawFd,
        unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    },
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    net::{UnixListener, UnixStream},
    time,
};

pub(crate) fn root() -> Result<PathBuf, &'static str> {
    let path = PathBuf::from(std::env::var_os("DK_SOUND_ROOT").ok_or("linux_root_required")?);
    if !path.is_absolute() {
        return Err("linux_root_must_be_absolute");
    }
    // One explicit directory, never recursively create/relax parent permissions.
    match fs::DirBuilder::new().mode(0o700).create(&path) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(_) => return Err("linux_root_unavailable"),
    }
    let metadata = fs::symlink_metadata(&path).map_err(|_| "linux_root_unavailable")?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o777 != 0o700
        || path.canonicalize().map_err(|_| "linux_root_unavailable")? != path
    {
        return Err("linux_root_not_private");
    }
    if path
        .join("engine.sock")
        .as_os_str()
        .as_encoded_bytes()
        .len()
        >= 108
    {
        return Err("linux_socket_path_too_long");
    }
    Ok(path)
}

struct Ownership {
    _file: File,
}
impl Ownership {
    fn acquire(root: &std::path::Path) -> Result<Self, &'static str> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(root.join("engine.lock"))
            .map_err(|_| "engine_lock_failed")?;
        let meta = file.metadata().map_err(|_| "engine_lock_failed")?;
        if !meta.is_file()
            || meta.uid() != unsafe { libc::geteuid() }
            || meta.mode() & 0o777 != 0o600
            || meta.nlink() != 1
        {
            return Err("engine_lock_not_private");
        }
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err("engine_already_owned_or_lock_failed");
        }
        Ok(Self { _file: file })
    }
}
// Kernel releases flock on process exit. Never unlink the lock inode.
struct SocketPath(PathBuf);
impl Drop for SocketPath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn start_engine() -> Result<(), &'static str> {
    let exe = std::env::current_exe().map_err(|_| "executable_path_unavailable")?;
    let engine = exe
        .parent()
        .ok_or("executable_path_unavailable")?
        .join("dk-sound-engine");
    let mut child = Command::new(engine)
        .process_group(0) // Client terminal Ctrl+C must not kill queued engine work.
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "engine_start_failed")?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

pub async fn request(command: Value) -> Result<Value, &'static str> {
    let path = root()?.join("engine.sock");
    let mut started = false;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut client = loop {
        match UnixStream::connect(&path).await {
            Ok(client) => break client,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                ) =>
            {
                if command["op"] == "shutdown" {
                    return Err("engine_not_running");
                }
                if !started {
                    start_engine()?;
                    started = true;
                }
            }
            Err(_) => return Err("engine_connect_failed"),
        }
        if Instant::now() >= deadline {
            return Err("engine_start_timeout");
        }
        time::sleep(Duration::from_millis(40)).await;
    };
    if client
        .peer_cred()
        .map_err(|_| "identity_unavailable")?
        .uid()
        != unsafe { libc::geteuid() }
    {
        return Err("engine_peer_denied");
    }
    // Never replay a request once a write has been attempted.
    time::timeout(Duration::from_secs(300), async {
        protocol::write_frame(&mut client, &command).await?;
        protocol::read_frame(&mut client).await
    })
    .await
    .map_err(|_| "engine_response_timeout")?
    .map_err(|_| "engine_response_lost")
}

pub async fn run_engine() -> Result<(), &'static str> {
    let root = root()?;
    let _owner = Ownership::acquire(&root)?;
    let path = root.join("engine.sock");
    match fs::symlink_metadata(&path) {
        Ok(meta) if meta.file_type().is_socket() && meta.uid() == unsafe { libc::geteuid() } => {
            fs::remove_file(&path).map_err(|_| "socket_cleanup_failed")?
        }
        Ok(_) => return Err("socket_path_conflict"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(_) => return Err("socket_cleanup_failed"),
    }
    let server = UnixListener::bind(&path).map_err(|_| "socket_create_failed")?;
    let _socket = SocketPath(path.clone());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
        .map_err(|_| "socket_permissions_failed")?;
    let shared = Arc::new(Mutex::new(State::default()));
    let (worker, ready) = storage_worker(shared.clone());
    time::timeout(Duration::from_secs(10), ready)
        .await
        .map_err(|_| "storage_start_timeout")?
        .map_err(|_| "local_worker_lost")?;
    let mut tick = time::interval(Duration::from_millis(100));
    loop {
        tokio::select! {
            connected = server.accept() => {
                let (client, _) = connected.map_err(|_| "socket_accept_failed")?;
                if client.peer_cred().map(|c| c.uid() == unsafe { libc::geteuid() }).unwrap_or(false) {
                    shared.lock().unwrap().connect(Instant::now());
                    tokio::spawn(serve(client, shared.clone(), worker.clone()));
                }
            },
            _ = tick.tick() => {
                crate::linux_provider::clear_expired();
                let state = shared.lock().unwrap();
                if state.shutdown_flushed || state.idle_expired(Instant::now()) { break; }
            }
        }
    }
    Ok(())
}
