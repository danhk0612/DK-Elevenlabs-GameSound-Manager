//! Bounded, shell-free child execution. Tool stderr is never forwarded to IPC/logs.
use crate::storage::{Error, Result};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
pub(crate) fn run(
    exe: &Path,
    args: &[String],
    output: Option<File>,
    limit: usize,
    timeout: Duration,
) -> Result<Vec<u8>> {
    let mut command = Command::new(exe);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .spawn()
        .map_err(|_| Error("audio_tool_start_failed"))?;
    let mut stdout = child.stdout.take().ok_or(Error("audio_tool_pipe_failed"))?;
    let overflow = Arc::new(AtomicBool::new(false));
    let reader_overflow = overflow.clone();
    let reader = thread::spawn(move || -> Result<Vec<u8>> {
        let mut output = output;
        let mut captured = Vec::new();
        let mut size = 0usize;
        let mut chunk = [0u8; 16384];
        loop {
            let n = stdout.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            size = size.saturating_add(n);
            if size > limit {
                reader_overflow.store(true, Ordering::Release);
                return Err(Error("audio_output_limit"));
            }
            if let Some(file) = output.as_mut() {
                file.write_all(&chunk[..n])?;
            } else {
                captured.extend_from_slice(&chunk[..n]);
            }
        }
        if let Some(file) = output.as_mut() {
            file.sync_all()?;
        }
        Ok(captured)
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        if overflow.load(Ordering::Acquire) || Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            return Err(Error(if overflow.load(Ordering::Acquire) {
                "audio_output_limit"
            } else {
                "audio_tool_timeout"
            }));
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return Err(Error("audio_tool_wait_failed"));
            }
        }
    };
    let data = reader
        .join()
        .map_err(|_| Error("audio_tool_reader_failed"))??;
    if !status.success() {
        return Err(Error("audio_tool_failed"));
    }
    Ok(data)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn bounded_child_output_and_timeout_are_errors() {
        let command = Path::new("/bin/sh");
        let args = vec!["-c".into(), "while :; do printf 0123456789; done".into()];
        assert_eq!(
            run(command, &args, None, 1024, Duration::from_secs(2))
                .unwrap_err()
                .0,
            "audio_output_limit"
        );
        let args = vec!["-c".into(), "exec sleep 10".into()];
        let start = Instant::now();
        assert_eq!(
            run(command, &args, None, 1024, Duration::from_millis(50))
                .unwrap_err()
                .0,
            "audio_tool_timeout"
        );
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
