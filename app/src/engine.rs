use crate::protocol::{self, State};
use serde_json::Value;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time;

pub(crate) type Shared = Arc<Mutex<State>>;
type Work = (Value, tokio::sync::oneshot::Sender<Value>);
pub(crate) type Worker = std::sync::mpsc::SyncSender<Work>;

pub(crate) fn storage_worker(shared: Shared) -> (Worker, tokio::sync::oneshot::Receiver<()>) {
    let (sender, receiver) = std::sync::mpsc::sync_channel::<Work>(32);
    let (ready_sender, ready) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let mut store = crate::storage::Store::default_root()
            .and_then(|p| crate::storage::Store::open(&p))
            .ok();
        let _ = ready_sender.send(());
        let mut audio_turn = true;
        let mut job_deadline = Instant::now() + Duration::from_millis(50);
        loop {
            let incoming = if Instant::now() >= job_deadline {
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            } else {
                receiver.recv_timeout(job_deadline.saturating_duration_since(Instant::now()))
            };
            match incoming {
                Ok((command, reply)) => {
                    {
                        let mut state = shared.lock().unwrap();
                        state.queued -= 1;
                        state.running += 1;
                    }
                    if store.is_none() {
                        store = crate::storage::Store::default_root()
                            .and_then(|p| crate::storage::Store::open(&p))
                            .ok();
                    }
                    let result = store
                        .as_mut()
                        .map(|store| store.execute(&command))
                        .unwrap_or_else(|| protocol::error("storage_unavailable"));
                    {
                        let mut state = shared.lock().unwrap();
                        state.running -= 1;
                        state.generation_queued =
                            store.as_ref().map(crate::generation::pending).unwrap_or(0);
                        state.audio_queued =
                            store.as_ref().map(crate::audio_queue::pending).unwrap_or(0);
                        if state.clients == 0 && !state.busy() {
                            state.last_empty = Instant::now();
                        }
                    }
                    let _ = reply.send(result);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    job_deadline = Instant::now() + Duration::from_millis(50);
                    if let Some(store) = store.as_mut() {
                        let audio;
                        {
                            let mut state = shared.lock().unwrap();
                            let remote = crate::generation::pending(store);
                            let local = crate::audio_queue::pending(store);
                            state.generation_queued = remote;
                            state.audio_queued = local;
                            if remote + local == 0 {
                                continue;
                            }
                            audio = local > 0 && (remote == 0 || audio_turn);
                            audio_turn = !audio;
                            if state.stopping {
                                continue;
                            }
                            state.generation_queued = remote;
                            state.audio_queued = local;
                            if audio {
                                state.audio_queued = local - 1;
                                state.audio_running = 1;
                            } else {
                                state.generation_queued = remote - 1;
                                state.generation_running = 1;
                            }
                        }
                        if audio {
                            let _ = crate::audio_queue::tick(store);
                        } else {
                            let _ = crate::generation::tick(store);
                        }
                        job_deadline = Instant::now() + Duration::from_millis(50);
                        {
                            let mut state = shared.lock().unwrap();
                            state.generation_running = 0;
                            state.audio_running = 0;
                            state.generation_queued = crate::generation::pending(store);
                            state.audio_queued = crate::audio_queue::pending(store);
                            if state.clients == 0 && !state.busy() {
                                state.last_empty = Instant::now();
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
    (sender, ready)
}

async fn dispatch(command: Value, shared: &Shared, worker: &Worker) -> Value {
    if let Err(code) = protocol::validate(&command) {
        return protocol::error(code);
    }
    if !command["op"].as_str().unwrap().contains('.') {
        return shared.lock().unwrap().execute(&command);
    }
    if matches!(
        command["op"].as_str(),
        Some("audio.submit" | "audio.get" | "audio.list" | "audio.cancel")
    ) {
        {
            let mut state = shared.lock().unwrap();
            if state.stopping {
                return protocol::error("engine_stopping");
            }
            state.running += 1;
        }
        let result = tokio::task::spawn_blocking(move || {
            crate::storage::Store::default_root()
                .and_then(|root| crate::audio_queue::fast(&root, &command))
        })
        .await;
        let mut state = shared.lock().unwrap();
        state.running -= 1;
        match crate::storage::Store::default_root()
            .and_then(|root| crate::audio_queue::pending_at(&root))
        {
            Ok(pending) => state.audio_queued = pending,
            Err(_) => state.audio_queued = state.audio_queued.max(1),
        }
        let response = match result {
            Ok(Ok((value, _))) => protocol::success(value),
            Ok(Err(error)) => protocol::error(error.0),
            Err(_) => protocol::error("local_worker_lost"),
        };
        if state.clients == 0 && !state.busy() {
            state.last_empty = Instant::now();
        }
        return response;
    }
    if matches!(
        command["op"].as_str(),
        Some("review.get" | "review.list" | "asset.history")
    ) {
        if shared.lock().unwrap().stopping {
            return protocol::error("engine_stopping");
        }
        return tokio::task::spawn_blocking(move || {
            match crate::storage::Store::default_root()
                .and_then(|root| crate::review::read(&root, &command))
            {
                Ok(value) => protocol::success(value),
                Err(error) => protocol::error(error.0),
            }
        })
        .await
        .unwrap_or_else(|_| protocol::error("local_worker_lost"));
    }
    if matches!(command["op"].as_str(), Some("job.get" | "job.list")) {
        if shared.lock().unwrap().stopping {
            return protocol::error("engine_stopping");
        }
        return tokio::task::spawn_blocking(move || {
            match crate::storage::Store::default_root()
                .and_then(|root| crate::generation::read_query(&root, &command))
            {
                Ok(data) => protocol::success(data),
                Err(error) => protocol::error(error.0),
            }
        })
        .await
        .unwrap_or_else(|_| protocol::error("local_worker_lost"));
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

pub(crate) async fn serve<T>(mut pipe: T, shared: Shared, worker: Worker)
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
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
