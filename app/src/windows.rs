use crate::protocol::{self, State};
use serde_json::Value;
use std::{
    ffi::c_void,
    io, mem, ptr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    net::windows::named_pipe::{ClientOptions, NamedPipeServer, ServerOptions},
    time,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, LocalFree, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0},
    Security::{
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
        },
        GetTokenInformation, TokenUser, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
    },
    System::Threading::{
        CreateMutexW, GetCurrentProcess, OpenProcessToken, ReleaseMutex, WaitForSingleObject,
    },
};

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
struct LocalAllocation(*mut c_void);
impl Drop for LocalAllocation {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn sid() -> io::Result<String> {
    unsafe {
        let mut token = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = Handle(token);
        let mut needed = 0;
        GetTokenInformation(token.0, TokenUser, ptr::null_mut(), 0, &mut needed);
        if needed == 0 {
            return Err(io::Error::last_os_error());
        }
        // usize storage preserves TOKEN_USER alignment.
        let mut buffer = vec![0usize; (needed as usize).div_ceil(mem::size_of::<usize>())];
        if GetTokenInformation(
            token.0,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        let user = &*buffer.as_ptr().cast::<TOKEN_USER>();
        let mut string = ptr::null_mut();
        if ConvertSidToStringSidW(user.User.Sid, &mut string) == 0 {
            return Err(io::Error::last_os_error());
        }
        let _string = LocalAllocation(string.cast());
        let mut length = 0;
        while *string.add(length) != 0 {
            length += 1;
        }
        String::from_utf16(std::slice::from_raw_parts(string, length))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid SID"))
    }
}

struct Security(LocalAllocation);
impl Security {
    fn owner(sid: &str) -> io::Result<Self> {
        let sddl = wide(&format!("D:P(A;;GA;;;{sid})"));
        let mut descriptor = ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(LocalAllocation(descriptor)))
    }
    fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0 .0,
            bInheritHandle: 0,
        }
    }
    fn pipe(&self, name: &str, first: bool) -> io::Result<NamedPipeServer> {
        let mut attributes = self.attributes();
        // Descriptor/attributes remain live for the synchronous OS create call.
        unsafe {
            ServerOptions::new()
                .first_pipe_instance(first)
                .reject_remote_clients(true)
                .create_with_security_attributes_raw(
                    name,
                    (&mut attributes as *mut SECURITY_ATTRIBUTES).cast(),
                )
        }
    }
}

// A named mutex is machine-wide per SID, across sessions and project workspaces.
// Ownership is acquired/released on the engine's single runtime thread.
struct Ownership(Handle);
impl Ownership {
    fn acquire(security: &Security, sid: &str) -> io::Result<Self> {
        let name = wide(&format!("Global\\DKGameSound-{sid}-engine-v1"));
        let attributes = security.attributes();
        let handle = unsafe { CreateMutexW(&attributes, 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let handle = Handle(handle);
        match unsafe { WaitForSingleObject(handle.0, 0) } {
            WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(Self(handle)),
            _ => Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "engine already owned",
            )),
        }
    }
}
impl Drop for Ownership {
    fn drop(&mut self) {
        unsafe {
            ReleaseMutex(self.0 .0);
        }
    }
}

fn pipe_name(sid: &str) -> String {
    format!(r"\\.\pipe\DKGameSound-{sid}-engine-v1")
}

pub async fn request(command: Value) -> Result<Value, &'static str> {
    let name = pipe_name(&sid().map_err(|_| "identity_unavailable")?);
    let mut started = false;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut client = loop {
        match ClientOptions::new().open(&name) {
            Ok(client) => break client,
            Err(error)
                if error.kind() == io::ErrorKind::NotFound
                    && !started
                    && command["op"] != "shutdown" =>
            {
                start_engine()?;
                started = true;
            }
            Err(error)
                if error.kind() == io::ErrorKind::NotFound && command["op"] == "shutdown" =>
            {
                return Err("engine_not_running")
            }
            Err(error)
                if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(231) => {
            }
            Err(_) => return Err("engine_connect_failed"),
        }
        if Instant::now() >= deadline {
            return Err("engine_start_timeout");
        }
        time::sleep(Duration::from_millis(40)).await;
    };
    // No automatic replay after a write/response failure, even for future commands.
    time::timeout(Duration::from_secs(300), async {
        protocol::write_frame(&mut client, &command).await?;
        protocol::read_frame(&mut client).await
    })
    .await
    .map_err(|_| "engine_response_timeout")?
    .map_err(|_| "engine_response_lost")
}

fn start_engine() -> Result<(), &'static str> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    let exe = std::env::current_exe().map_err(|_| "executable_path_unavailable")?;
    let engine = exe
        .parent()
        .ok_or("executable_path_unavailable")?
        .join("dk-sound-engine.exe");
    let mut child = Command::new(engine)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(0x00000008 | 0x08000000) // DETACHED_PROCESS | CREATE_NO_WINDOW
        .spawn()
        .map_err(|_| "engine_start_failed")?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

type Shared = Arc<Mutex<State>>;
type Work = (Value, tokio::sync::oneshot::Sender<Value>);
type Worker = std::sync::mpsc::SyncSender<Work>;

fn storage_worker(shared: Shared) -> Worker {
    let (sender, receiver) = std::sync::mpsc::sync_channel::<Work>(32);
    std::thread::spawn(move || {
        let mut store = None;
        while let Ok((command, reply)) = receiver.recv() {
            {
                let mut state = shared.lock().unwrap();
                state.queued -= 1;
                state.running += 1;
            }
            let result = if let Some(store) = store.as_mut() {
                crate::storage::Store::execute(store, &command)
            } else {
                match crate::storage::Store::default_root()
                    .and_then(|p| crate::storage::Store::open(&p))
                {
                    Ok(mut opened) => {
                        let result = opened.execute(&command);
                        store = Some(opened);
                        result
                    }
                    Err(e) => protocol::error(e.0),
                }
            };
            {
                let mut state = shared.lock().unwrap();
                state.running -= 1;
                if state.clients == 0 && !state.busy() {
                    state.last_empty = Instant::now();
                }
            }
            let _ = reply.send(result);
        }
    });
    sender
}

async fn dispatch(command: Value, shared: &Shared, worker: &Worker) -> Value {
    if let Err(code) = protocol::validate(&command) {
        return protocol::error(code);
    }
    if !command["op"].as_str().unwrap().contains('.') {
        return shared.lock().unwrap().execute(&command);
    }
    let (sender, receiver) = tokio::sync::oneshot::channel();
    {
        let mut state = shared.lock().unwrap();
        if state.stopping {
            return protocol::error("engine_stopping");
        }
        state.queued += 1;
        if worker.try_send((command, sender)).is_err() {
            state.queued -= 1;
            return protocol::error("local_queue_unavailable");
        }
    }
    receiver
        .await
        .unwrap_or_else(|_| protocol::error("local_worker_lost"))
}

struct Connection(Shared);
impl Drop for Connection {
    fn drop(&mut self) {
        self.0.lock().unwrap().disconnect(Instant::now());
    }
}

async fn serve(mut pipe: NamedPipeServer, shared: Shared, worker: Worker) {
    let _connection = Connection(shared.clone());
    loop {
        let command = match protocol::read_frame(&mut pipe).await {
            Ok(command) => command,
            Err(_) => break,
        };
        let response = dispatch(command.clone(), &shared, &worker).await;
        let shutdown = command["op"] == "shutdown" && response["ok"] == true;
        let written = time::timeout(
            Duration::from_secs(5),
            protocol::write_frame(&mut pipe, &response),
        )
        .await;
        if shutdown {
            shared.lock().unwrap().shutdown_flushed = true;
        }
        if shutdown || !matches!(written, Ok(Ok(()))) {
            break;
        }
    }
}

pub async fn run_engine() -> Result<(), &'static str> {
    let sid = sid().map_err(|_| "identity_unavailable")?;
    let security = Security::owner(&sid).map_err(|_| "security_descriptor_failed")?;
    let _ownership =
        Ownership::acquire(&security, &sid).map_err(|_| "engine_already_owned_or_lock_failed")?;
    let name = pipe_name(&sid);
    let mut server = security
        .pipe(&name, true)
        .map_err(|_| "pipe_create_failed")?;
    let shared = Arc::new(Mutex::new(State::default()));
    let worker = storage_worker(shared.clone());
    let mut tick = time::interval(Duration::from_millis(100));
    loop {
        tokio::select! {
            connected = server.connect() => {
                connected.map_err(|_| "pipe_accept_failed")?;
                let next = security.pipe(&name, false).map_err(|_| "pipe_create_failed")?;
                shared.lock().unwrap().connect(Instant::now());
                tokio::spawn(serve(server, shared.clone(),worker.clone()));
                server = next;
            },
            _ = tick.tick() => {
                let state = shared.lock().unwrap();
                if state.shutdown_flushed || state.idle_expired(Instant::now()) { break; }
            }
        }
    }
    Ok(())
}
