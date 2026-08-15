//! Bounded Git plumbing for explicitly requested repository-state comparisons.

use std::{
    ffi::OsString,
    io::{self, Read},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::app::error::CliError;

const MAX_GIT_ERROR_BYTES: usize = 64 * 1024;
const GIT_TIMEOUT: Duration = Duration::from_mins(1);

pub(super) fn output(
    root: &Path,
    arguments: &[OsString],
    limit: usize,
    operation: &str,
) -> Result<Vec<u8>, CliError> {
    let mut child = Command::new("git")
        .arg("--no-replace-objects")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| CliError::new(format!("start Git {operation}: {error}")))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CliError::new(format!("capture Git {operation} output")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| CliError::new(format!("capture Git {operation} errors")))?;
    let output = match thread::Builder::new()
        .name("zrail-git-stdout".into())
        .spawn(move || read_bounded(stdout, limit))
    {
        Ok(output) => output,
        Err(error) => {
            stop(&mut child);
            return Err(CliError::new(format!(
                "capture Git {operation} output: start reader thread: {error}"
            )));
        }
    };
    let errors = match thread::Builder::new()
        .name("zrail-git-stderr".into())
        .spawn(move || drain_bounded(stderr, MAX_GIT_ERROR_BYTES))
    {
        Ok(errors) => errors,
        Err(error) => {
            stop(&mut child);
            let _joined = output.join();
            return Err(CliError::new(format!(
                "capture Git {operation} errors: start reader thread: {error}"
            )));
        }
    };
    let status = wait_bounded(&mut child, operation);
    if status.is_err() {
        stop(&mut child);
    }
    let captured = output
        .join()
        .map_err(|_| CliError::new(format!("capture Git {operation} output")))?
        .map_err(|error| CliError::new(format!("read Git {operation} output: {error}")))?;
    let errors = errors
        .join()
        .map_err(|_| CliError::new(format!("capture Git {operation} errors")))?
        .map_err(|error| CliError::new(format!("read Git {operation} errors: {error}")))?;
    let status = status?;
    if captured.overflowed {
        return Err(CliError::new(format!(
            "Git {operation} output exceeds the {limit}-byte safety limit"
        )));
    }
    if !status.success() {
        let detail = String::from_utf8_lossy(&errors.bytes);
        let detail = detail.trim();
        let message = if detail.is_empty() {
            format!("Git {operation} failed with {status}")
        } else {
            format!("Git {operation} failed: {detail}")
        };
        return Err(CliError::new(message));
    }
    Ok(captured.bytes)
}

fn wait_bounded(
    child: &mut std::process::Child,
    operation: &str,
) -> Result<std::process::ExitStatus, CliError> {
    let deadline = Instant::now() + GIT_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            Ok(None) => {
                return Err(CliError::new(format!(
                    "Git {operation} exceeded the {}-second time limit",
                    GIT_TIMEOUT.as_secs()
                )));
            }
            Err(error) => {
                return Err(CliError::new(format!("wait for Git {operation}: {error}")));
            }
        }
    }
}

fn stop(child: &mut std::process::Child) {
    let _killed = child.kill();
    let _waited = child.wait();
}

struct Captured {
    bytes: Vec<u8>,
    overflowed: bool,
}

fn read_bounded(mut reader: impl Read, limit: usize) -> io::Result<Captured> {
    let mut bytes = Vec::with_capacity(limit.min(64 * 1024));
    let mut buffer = [0_u8; 8192];
    while bytes.len() <= limit {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Ok(Captured {
                bytes,
                overflowed: false,
            });
        }
        let remaining = limit.saturating_add(1).saturating_sub(bytes.len());
        bytes.extend_from_slice(&buffer[..read.min(remaining)]);
        if bytes.len() > limit {
            return Ok(Captured {
                bytes,
                overflowed: true,
            });
        }
    }
    Ok(Captured {
        bytes,
        overflowed: true,
    })
}

fn drain_bounded(mut reader: impl Read, limit: usize) -> io::Result<Captured> {
    let mut bytes = Vec::with_capacity(limit.min(8192));
    let mut overflowed = false;
    let mut buffer = [0_u8; 8192];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Ok(Captured { bytes, overflowed });
        }
        let remaining = limit.saturating_sub(bytes.len());
        bytes.extend_from_slice(&buffer[..read.min(remaining)]);
        overflowed |= read > remaining;
    }
}

#[cfg(test)]
#[path = "git_process_test.rs"]
mod git_process_test;
