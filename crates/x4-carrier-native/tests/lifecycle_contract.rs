use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use x4_carrier_native::{
    BridgePeer, CloseProgress, HandleToken, NativeTransport, SecurityControl, TransportConfig,
    TransportError, TransportPoll, TransportSendOutcome,
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

    assert_eq!(evidence.explicit_descriptor, SecurityControl::Enforced);
    assert_eq!(evidence.current_logon_allowed, SecurityControl::Enforced);
    assert_eq!(evidence.outside_logon_denied, SecurityControl::Enforced);
    assert_eq!(evidence.remote_clients_rejected, SecurityControl::Enforced);
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
    assert_eq!(
        transport.poll_control(token(7), 512),
        TransportPoll::NoMessage
    );

    let before = Instant::now();
    assert_eq!(
        transport.try_send(token(7), b"complete-message"),
        TransportSendOutcome::LocalHandoff
    );
    assert_eq!(
        transport.try_send(token(7), b"occupied"),
        TransportSendOutcome::CapacityUnavailable
    );
    assert!(before.elapsed() < Duration::from_millis(50));
    assert_eq!(peer.receive(2_048), Ok(b"complete-message".to_vec()));
    assert_eq!(peer.send_control(b"bounded-control"), Ok(()));
    assert_eq!(
        poll_result(&transport, token(7), 4),
        TransportPoll::Rejected(TransportError::InvalidOutputCapacity)
    );
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
    let transport = started(config.clone(), token(9));
    assert_eq!(
        transport.try_send(token(8), b"stale"),
        TransportSendOutcome::Rejected(TransportError::StaleGeneration)
    );
    assert_eq!(
        transport.try_send(token(9), &vec![b'x'; 2_048]),
        TransportSendOutcome::LocalHandoff
    );
    assert!(wait_for_pending_owner(&transport));
    assert!(transport.snapshot().pending_operation_owners <= 2);
    assert_eq!(transport.request_close(token(9)), CloseProgress::Requested);
    assert!(transport.wait_closed_for_test(Duration::from_secs(2)));
    assert_eq!(transport.snapshot().pending_operation_owners, 0);
    assert!(transport.snapshot().closed);

    let replacement = started(config, token(10));
    assert_eq!(
        replacement.try_send(token(9), b"old-generation"),
        TransportSendOutcome::Rejected(TransportError::StaleGeneration)
    );
    let _ = replacement.request_close(token(10));
    assert!(replacement.wait_closed_for_test(Duration::from_secs(2)));
}

#[test]
fn short_peer_read_is_rejected_instead_of_publishing_partial_bytes() {
    let config = config();
    let transport = started(config.clone(), token(11));
    let peer = BridgePeer::connect(&config, Duration::from_secs(2));
    assert!(peer.is_ok(), "peer must connect: {peer:?}");
    let Ok(mut peer) = peer else { unreachable!() };
    assert_eq!(
        transport.try_send(token(11), b"complete-message"),
        TransportSendOutcome::LocalHandoff
    );
    assert_eq!(peer.receive(8), Err(TransportError::Unavailable));
    let _ = transport.request_close(token(11));
    assert!(transport.wait_closed_for_test(Duration::from_secs(2)));
}

#[test]
fn finalizer_requests_bounded_shutdown_without_waiting_for_completion() {
    let config = config();
    let transport = started(config.clone(), token(12));
    assert!(wait_for_pending_owner(&transport));
    let before = Instant::now();
    drop(transport);
    assert!(before.elapsed() < Duration::from_millis(50));

    let deadline = Instant::now() + Duration::from_secs(2);
    let replacement = loop {
        if let Ok(value) = NativeTransport::start(config.clone(), token(13)) {
            break value;
        }
        assert!(
            Instant::now() < deadline,
            "old worker did not release bounded resources"
        );
        std::thread::yield_now();
    };
    let _ = replacement.request_close(token(13));
    assert!(replacement.wait_closed_for_test(Duration::from_secs(2)));
}

fn poll_message(transport: &NativeTransport, token: HandleToken) -> TransportPoll {
    poll_result(transport, token, 512)
}

fn poll_result(transport: &NativeTransport, token: HandleToken, capacity: usize) -> TransportPoll {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let result = transport.poll_control(token, capacity);
        if result != TransportPoll::NoMessage || Instant::now() >= deadline {
            return result;
        }
        std::thread::yield_now();
    }
}

fn wait_for_pending_owner(transport: &NativeTransport) -> bool {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if transport.snapshot().pending_operation_owners > 0 {
            return true;
        }
        std::thread::yield_now();
    }
    false
}
