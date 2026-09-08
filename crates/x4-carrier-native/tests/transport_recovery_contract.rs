use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use x4_carrier_native::{
    BridgePeer, HandleToken, NativeTransport, TransportConfig, TransportPoll, TransportSendOutcome,
};

static PIPE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn config() -> TransportConfig {
    let sequence = PIPE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    TransportConfig {
        pipe_name: format!(
            r"\\.\pipe\live-galaxy-recovery-{}-{sequence}",
            std::process::id()
        ),
        max_data_message_bytes: 2_048,
        max_control_message_bytes: 512,
    }
}

const fn token() -> HandleToken {
    HandleToken {
        generation: 1,
        slot: 1,
    }
}

#[test]
fn control_burst_is_retained_until_lua_drains_each_message() {
    let config = config();
    let transport = NativeTransport::start(config.clone(), token()).expect("transport");
    let mut peer = BridgePeer::connect(&config, Duration::from_secs(2)).expect("peer");
    for control in [b"handshake".as_slice(), b"intent", b"demand"] {
        peer.send_control(control).expect("control admitted");
    }
    std::thread::sleep(Duration::from_millis(50));
    for expected in [b"handshake".as_slice(), b"intent", b"demand"] {
        assert_eq!(poll(&transport), TransportPoll::Message(expected.to_vec()));
    }
    let _ = transport.request_close(token());
    assert!(transport.wait_closed_for_test(Duration::from_secs(2)));
}

#[test]
fn generation_reports_uncertain_write_for_exact_caller_reconciliation() {
    let config = config();
    let transport = NativeTransport::start(config.clone(), token()).expect("transport");
    let peer = BridgePeer::connect(&config, Duration::from_secs(2)).expect("first peer");
    let old_generation = transport.snapshot().connection_generation;
    drop(peer);
    assert_eq!(
        transport.try_send(token(), b"retained-across-reconnect"),
        TransportSendOutcome::LocalHandoff
    );
    transport
        .request_reconnect(token())
        .expect("reconnect request");
    let mut replacement = connect_until(&config, &transport);
    let deadline = Instant::now() + Duration::from_secs(2);
    while transport.snapshot().connection_generation <= old_generation {
        assert!(Instant::now() < deadline, "generation did not advance");
        std::thread::yield_now();
    }
    let uncertain = replacement
        .receive_timeout(2_048, Duration::from_millis(50))
        .expect("bounded read");
    if let Some(bytes) = uncertain {
        assert_eq!(bytes, b"retained-across-reconnect");
        drop(replacement);
        replacement = connect_until(&config, &transport);
    }
    assert_eq!(
        transport.try_send(token(), b"retained-across-reconnect"),
        TransportSendOutcome::LocalHandoff
    );
    assert_eq!(
        replacement.receive(2_048),
        Ok(b"retained-across-reconnect".to_vec())
    );
    let _ = transport.request_close(token());
    assert!(transport.wait_closed_for_test(Duration::from_secs(2)));
}

#[test]
fn explicit_reconnect_advances_generation_without_peer_failure() {
    let config = config();
    let transport = NativeTransport::start(config.clone(), token()).expect("transport");
    let _peer = BridgePeer::connect(&config, Duration::from_secs(2)).expect("first peer");
    let old_generation = transport.snapshot().connection_generation;
    transport
        .request_reconnect(token())
        .expect("reconnect request");
    let mut replacement = connect_until(&config, &transport);
    let deadline = Instant::now() + Duration::from_secs(2);
    while transport.snapshot().connection_generation <= old_generation {
        assert!(Instant::now() < deadline, "generation did not advance");
        std::thread::yield_now();
    }
    assert_eq!(
        transport.try_send(token(), b"fresh-application"),
        TransportSendOutcome::LocalHandoff
    );
    assert_eq!(
        replacement.receive(2_048),
        Ok(b"fresh-application".to_vec())
    );
    let _ = transport.request_close(token());
    assert!(transport.wait_closed_for_test(Duration::from_secs(2)));
}

#[test]
fn stale_control_is_fenced_from_replacement_connection() {
    let config = config();
    let transport = NativeTransport::start(config.clone(), token()).expect("transport");
    let mut peer = BridgePeer::connect(&config, Duration::from_secs(2)).expect("first peer");
    peer.send_control(b"stale-control").expect("old control");
    std::thread::sleep(Duration::from_millis(20));
    let old_generation = transport.snapshot().connection_generation;
    drop(peer);
    let mut replacement = connect_until(&config, &transport);
    while transport.snapshot().connection_generation <= old_generation {
        std::thread::yield_now();
    }
    replacement
        .send_control(b"current-control")
        .expect("new control");
    assert_eq!(
        poll(&transport),
        TransportPoll::Message(b"current-control".to_vec())
    );
    let _ = transport.request_close(token());
    assert!(transport.wait_closed_for_test(Duration::from_secs(2)));
}

fn poll(transport: &NativeTransport) -> TransportPoll {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let result = transport.poll_control(token(), 512);
        if result != TransportPoll::NoMessage || Instant::now() >= deadline {
            return result;
        }
        std::thread::yield_now();
    }
}

fn connect_until(config: &TransportConfig, transport: &NativeTransport) -> BridgePeer {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(peer) = BridgePeer::connect(config, Duration::from_millis(10)) {
            return peer;
        }
        assert!(
            Instant::now() < deadline,
            "server did not reconnect: {:?}",
            transport.snapshot()
        );
        std::thread::yield_now();
    }
}
