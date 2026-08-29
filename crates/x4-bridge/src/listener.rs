use crate::{AcceptAttempt, AcceptDisposition, PIPE_ENDPOINT, PipeServer};

#[cfg(windows)]
const MAX_PIPE_MESSAGE_BYTES: usize = 2_048;
#[cfg(windows)]
const MAX_TRACE_FRAME_EVENTS: usize = 64;

#[cfg(windows)]
pub fn run_windows_listener() -> std::io::Result<()> {
    use {
        interprocess::os::windows::named_pipe::{PipeListenerOptions, PipeMode, pipe_mode},
        std::path::Path,
    };

    let listener = PipeListenerOptions::new()
        .path(Path::new(PIPE_ENDPOINT))
        .mode(PipeMode::Messages)
        .create_duplex::<pipe_mode::Messages>()?;
    let mut debug_sink = DebugSink::from_environment()?;
    let mut server = PipeServer::new();
    write_debug_event(&mut debug_sink, "listener_ready", None, None, &server);
    for connection in listener.incoming() {
        if let Ok(connection) = connection {
            write_debug_event(&mut debug_sink, "client_connected", None, None, &server);
            serve(connection, &mut server, &mut debug_sink);
        } else {
            let disposition = server.record_accept(AcceptAttempt::TransientFailure);
            write_debug_event(
                &mut debug_sink,
                "accept_failed",
                Some(&disposition),
                None,
                &server,
            );
            delay(disposition);
        }
    }
    Ok(())
}

#[cfg(windows)]
fn serve(
    mut connection: interprocess::os::windows::named_pipe::PipeStream<
        interprocess::os::windows::named_pipe::pipe_mode::Messages,
        interprocess::os::windows::named_pipe::pipe_mode::Messages,
    >,
    server: &mut PipeServer,
    debug_sink: &mut Option<DebugSink>,
) {
    use recvmsg::{MsgBuf, prelude::*};

    let _ = server.record_accept(AcceptAttempt::ClientAccepted);
    loop {
        let mut buffer = MsgBuf::from(Vec::with_capacity(MAX_PIPE_MESSAGE_BYTES));
        buffer.quota = Some(MAX_PIPE_MESSAGE_BYTES);
        let Ok(_) = connection.recv_msg(&mut buffer, None) else {
            break;
        };
        let frame_bytes = buffer.filled_part().len();
        if frame_bytes == 0 {
            write_debug_event(debug_sink, "client_eof", None, None, server);
            break;
        }
        if let Ok(payload) = std::str::from_utf8(buffer.filled_part()) {
            let summary = frame_summary(payload);
            let disposition = server.admit_message(payload);
            write_debug_event(
                debug_sink,
                "frame_received",
                Some(&disposition),
                Some((frame_bytes, summary.as_str())),
                server,
            );
        } else {
            write_debug_event(
                debug_sink,
                "frame_not_utf8",
                None,
                Some((frame_bytes, "kind=invalid reason=not_utf8")),
                server,
            );
        }
    }
    server.discard_pending();
    write_debug_event(debug_sink, "client_disconnected", None, None, server);
}

#[cfg(windows)]
fn frame_summary(payload: &str) -> String {
    use observation_ingest::{FrameHeader, inspect_frame};

    match inspect_frame(payload) {
        Ok(FrameHeader::Hello {
            protocol_major,
            capabilities,
            generation,
            ..
        }) => format!(
            "kind=hello protocol_major={protocol_major} capability_count={} generation={generation}",
            capabilities.len()
        ),
        Ok(FrameHeader::Data {
            kind,
            version,
            generation,
            sequence,
            ..
        }) => format!("kind={kind} version={version} generation={generation} sequence={sequence}"),
        Err(error) => format!("kind=invalid reason={error:?}"),
    }
}

#[cfg(windows)]
struct DebugSink {
    file: std::fs::File,
    attempt_id: Option<String>,
    trace_frame_events: usize,
}

#[cfg(windows)]
impl DebugSink {
    fn from_environment() -> std::io::Result<Option<Self>> {
        use std::fs::OpenOptions;

        std::env::var_os("LIVE_GALAXY_DEBUG_EVIDENCE_PATH")
            .map(|path| {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map(|file| Self {
                        file,
                        attempt_id: std::env::var("LIVE_GALAXY_TRACE_ATTEMPT_ID")
                            .ok()
                            .filter(|attempt_id| valid_attempt_id(attempt_id)),
                        trace_frame_events: 0,
                    })
            })
            .transpose()
    }

    const fn should_write_frame(&mut self) -> bool {
        if self.attempt_id.is_none() {
            return false;
        }
        self.trace_frame_events += 1;
        self.trace_frame_events <= MAX_TRACE_FRAME_EVENTS
    }
}

#[cfg(windows)]
fn valid_attempt_id(attempt_id: &str) -> bool {
    !attempt_id.is_empty()
        && attempt_id.len() <= 64
        && attempt_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[cfg(windows)]
fn write_debug_event(
    debug_sink: &mut Option<DebugSink>,
    event: &str,
    disposition: Option<&dyn std::fmt::Debug>,
    frame: Option<(usize, &str)>,
    server: &PipeServer,
) {
    use std::io::Write;

    let Some(sink) = debug_sink else {
        return;
    };
    if frame.is_some() && !sink.should_write_frame() {
        return;
    }
    let snapshot = server.snapshot();
    let attempt_id = sink.attempt_id.as_deref().unwrap_or("none");
    let (frame_bytes, frame_summary) = frame.unwrap_or((0, "kind=none"));
    let _ = writeln!(
        sink.file,
        "attempt_id={attempt_id}\tevent={event}\tdisposition={disposition:?}\tframe_bytes={frame_bytes}\t{frame_summary}\tentities={}\tentity_ids={:?}",
        snapshot.entity_ids().len(),
        snapshot.entity_ids()
    );
}

#[cfg(windows)]
fn delay(disposition: AcceptDisposition) {
    if let AcceptDisposition::RetryAcceptDegraded { delay_millis } = disposition {
        std::thread::sleep(std::time::Duration::from_millis(delay_millis));
    }
}

#[cfg(not(windows))]
pub fn run_windows_listener() -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Windows named pipes require a Windows host",
    ))
}
