use crate::{AcceptAttempt, AcceptDisposition, PIPE_ENDPOINT, PipeServer};

#[cfg(windows)]
const MAX_PIPE_MESSAGE_BYTES: usize = 512;

#[cfg(windows)]
pub fn run_windows_listener() -> std::io::Result<()> {
    use {
        interprocess::os::windows::named_pipe::{PipeListenerOptions, pipe_mode},
        std::path::Path,
    };

    let listener = PipeListenerOptions::new()
        .path(Path::new(PIPE_ENDPOINT))
        .create_duplex::<pipe_mode::Messages>()?;
    let mut server = PipeServer::new();
    for connection in listener.incoming() {
        match connection {
            Ok(connection) => serve(connection, &mut server),
            Err(_) => delay(server.record_accept(AcceptAttempt::TransientFailure)),
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
) {
    use recvmsg::{MsgBuf, prelude::*};

    let _ = server.record_accept(AcceptAttempt::ClientAccepted);
    loop {
        let mut buffer = MsgBuf::from(Vec::with_capacity(MAX_PIPE_MESSAGE_BYTES));
        buffer.quota = Some(MAX_PIPE_MESSAGE_BYTES);
        let Ok(_) = connection.recv_msg(&mut buffer, None) else {
            break;
        };
        if let Ok(payload) = std::str::from_utf8(buffer.filled_part()) {
            let _ = server.admit_message(payload);
        }
    }
    server.discard_pending();
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
