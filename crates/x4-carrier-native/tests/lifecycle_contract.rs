use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use x4_carrier_native::{
    BridgePeer, CloseProgress, HandleToken, NativeTransport, TransportConfig, TransportError,
    TransportPoll, TransportSendOutcome,
};

static PIPE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn config() -> TransportConfig {
    let sequence = PIPE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    TransportConfig {
        pipe_name: format!(
            r"\\.\pipe\live-galaxy-plan-05-4-01-{}-{sequence}",
            std::process::id()
        ),
        max_data_message_bytes: 2_048,
        max_control_message_bytes: 512,
    }
}

const fn token(generation: u32) -> HandleToken {
    HandleToken {
        generation,
        slot: 1,
    }
}

fn started(config: TransportConfig, token: HandleToken) -> NativeTransport {
    let result = NativeTransport::start(config, token);
    assert!(result.is_ok(), "transport must start: {result:?}");
    let Ok(transport) = result else {
        unreachable!()
    };
    transport
}

#[test]
fn actual_pipe_is_same_logon_only_and_rejects_remote_clients() {
    let transport = started(config(), token(1));
    let evidence = transport.security_evidence();

    assert!(evidence.explicit_descriptor);
    assert!(evidence.current_logon_allowed);
    assert!(evidence.outside_logon_denied);
    assert!(evidence.remote_clients_rejected);
    assert!(transport.snapshot().monotonic_millis.is_some());
    let _ = transport.request_close(token(1));
    assert!(transport.wait_closed_for_test(Duration::from_secs(2)));
}

#[test]
fn real_peer_exchanges_exact_complete_messages_and_bounded_control() {
    let config = config();
    let transport = started(config.clone(), token(7));
    let mut peer_result = BridgePeer::connect(&config, Duration::from_secs(2));
    assert!(peer_result.is_ok(), "peer must connect: {peer_result:?}");
    let Ok(ref mut peer) = peer_result else {
        unreachable!()
    };

    let before = Instant::now();
    assert_eq!(
        transport.try_send(token(7), b"complete-message"),
        TransportSendOutcome::LocalHandoff
    );
    assert!(before.elapsed() < Duration::from_millis(50));
    assert_eq!(peer.receive(2_048), Ok(b"complete-message".to_vec()));
    assert_eq!(peer.send_control(b"bounded-control"), Ok(()));
    assert_eq!(
        poll_message(&transport, token(7)),
        TransportPoll::Message(b"bounded-control".to_vec())
    );
    let _ = transport.request_close(token(7));
    assert!(transport.wait_closed_for_test(Duration::from_secs(2)));
}

#[test]
fn pending_cancellation_retains_owners_and_stale_generation_is_isolated() {
    let config = config();
    let transport = started(config, token(9));
    assert_eq!(
        transport.try_send(token(8), b"stale"),
        TransportSendOutcome::Rejected(TransportError::StaleGeneration)
    );
    assert_eq!(
        transport.try_send(token(9), &vec![b'x'; 2_048]),
        TransportSendOutcome::LocalHandoff
    );
    assert!(transport.snapshot().pending_operation_owners <= 2);
    assert_eq!(transport.request_close(token(9)), CloseProgress::Requested);
    assert!(transport.wait_closed_for_test(Duration::from_secs(2)));
    assert_eq!(transport.snapshot().pending_operation_owners, 0);
    assert!(transport.snapshot().closed);
}

fn poll_message(transport: &NativeTransport, token: HandleToken) -> TransportPoll {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let result = transport.poll_control(token);
        if result != TransportPoll::NoMessage || Instant::now() >= deadline {
            return result;
        }
        std::thread::yield_now();
    }
}
