use crate::process_schema::schema_path;
use mind_orchestration::ProviderFailure;
use std::{
    io::Read,
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

pub const MAX_OUTPUT_BYTES: usize = 65_536;
pub const TIMEOUT: Duration = Duration::from_secs(30);
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(1);

pub trait BenchmarkProcess {
    fn invoke(&mut self, request_identity: &str) -> Result<Vec<u8>, ProviderFailure>;
}

#[derive(Default, Debug)]
pub struct CodexProcess;

impl BenchmarkProcess for CodexProcess {
    fn invoke(&mut self, request_identity: &str) -> Result<Vec<u8>, ProviderFailure> {
        let schema = schema_path()?;
        let mut child = Command::new("codex")
            .args([
                "exec",
                "--json",
                "--output-schema",
                schema.as_str(),
                request_identity,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| ProviderFailure::Unavailable)?;
        let stdout = child.stdout.take().ok_or(ProviderFailure::Transport)?;
        let stderr = child.stderr.take().ok_or(ProviderFailure::Transport)?;
        let out_rx = start_drain(stdout);
        let err_rx = start_drain(stderr);
        let deadline = Instant::now() + TIMEOUT;
        match wait_for_exit(&mut child, deadline)? {
            Some(success) => complete(success, out_rx, err_rx, deadline),
            None => timed_out(&mut child, out_rx, err_rx),
        }
    }
}

trait ProcessControl {
    fn poll(&mut self) -> Result<Option<bool>, ProviderFailure>;
}

impl ProcessControl for Child {
    fn poll(&mut self) -> Result<Option<bool>, ProviderFailure> {
        self.try_wait()
            .map(|status| status.map(|value| value.success()))
            .map_err(|_| ProviderFailure::Transport)
    }
}

fn wait_for_exit<P: ProcessControl>(
    process: &mut P,
    deadline: Instant,
) -> Result<Option<bool>, ProviderFailure> {
    loop {
        match process.poll()? {
            Some(success) => return Ok(Some(success)),
            None if Instant::now() >= deadline => return Ok(None),
            None => thread::sleep(Duration::from_millis(10)),
        }
    }
}

fn timed_out(
    child: &mut Child,
    stdout: DrainWorker,
    stderr: DrainWorker,
) -> Result<Vec<u8>, ProviderFailure> {
    terminate_tree(child)?;
    child.wait().map_err(|_| ProviderFailure::Transport)?;
    let cleanup = Instant::now() + CLEANUP_TIMEOUT;
    cleanup_both(
        || receive_worker(stdout, cleanup),
        || receive_worker(stderr, cleanup),
    )?;
    Err(ProviderFailure::Timeout)
}

fn cleanup_both<F, G>(stdout: F, stderr: G) -> Result<(), ProviderFailure>
where
    F: FnOnce() -> Result<Drained, ProviderFailure>,
    G: FnOnce() -> Result<Drained, ProviderFailure>,
{
    let first = stdout().err();
    let second = stderr().err();
    first.or(second).map_or(Ok(()), Err)
}

#[cfg(target_os = "windows")]
fn terminate_tree(child: &mut Child) -> Result<(), ProviderFailure> {
    let status = Command::new("taskkill")
        .args(["/PID", &child.id().to_string(), "/T", "/F"])
        .status()
        .map_err(|_| ProviderFailure::Transport)?;
    status
        .success()
        .then_some(())
        .ok_or(ProviderFailure::Transport)
}

#[cfg(not(target_os = "windows"))]
fn terminate_tree(child: &mut Child) -> Result<(), ProviderFailure> {
    child.kill().map_err(|_| ProviderFailure::Transport)
}

fn complete(
    success: bool,
    out_rx: DrainWorker,
    err_rx: DrainWorker,
    deadline: Instant,
) -> Result<Vec<u8>, ProviderFailure> {
    let stdout = receive_worker(out_rx, deadline)?;
    let stderr = receive_worker(err_rx, deadline)?;
    if stdout.failed || stderr.failed {
        return Err(ProviderFailure::Stream);
    }
    if stdout.oversized || stderr.oversized {
        return Err(ProviderFailure::Oversized);
    }
    if success {
        Ok(stdout.bytes)
    } else {
        Err(ProviderFailure::Transport)
    }
}

struct Drained {
    bytes: Vec<u8>,
    oversized: bool,
    failed: bool,
}

struct DrainWorker {
    receiver: mpsc::Receiver<Drained>,
    handle: thread::JoinHandle<()>,
}
fn start_drain<R: Read + Send + 'static>(reader: R) -> DrainWorker {
    let (sender, receiver) = mpsc::channel();
    DrainWorker {
        receiver,
        handle: thread::spawn(move || drain(reader, sender)),
    }
}

fn drain<R: Read>(mut reader: R, sender: mpsc::Sender<Drained>) {
    let mut bytes = Vec::new();
    let mut oversized = false;
    let mut buffer = [0; 4096];
    let mut failed = false;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Err(_) => {
                failed = true;
                break;
            }
            Ok(count) => {
                let room = MAX_OUTPUT_BYTES.saturating_sub(bytes.len());
                bytes.extend_from_slice(&buffer[..count.min(room)]);
                oversized |= count > room;
            }
        }
    }
    let _ = sender.send(Drained {
        bytes,
        oversized,
        failed,
    });
}

fn receive(
    receiver: mpsc::Receiver<Drained>,
    deadline: Instant,
) -> Result<Drained, ProviderFailure> {
    receiver
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .map_err(|_| ProviderFailure::DrainIncomplete)
}

fn receive_worker(worker: DrainWorker, deadline: Instant) -> Result<Drained, ProviderFailure> {
    let drained = receive(worker.receiver, deadline)?;
    worker.handle.join().map_err(|_| ProviderFailure::Stream)?;
    Ok(drained)
}

#[cfg(test)]
#[path = "process_tests.rs"]
mod process_tests;
