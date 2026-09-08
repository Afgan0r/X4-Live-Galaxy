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
fn disconnected_peer_reconnects_and_receives_retained_write() {
    let config = config();
    let transport = NativeTransport::start(config.clone(), token()).expect("transport");
    let peer = BridgePeer::connect(&config, Duration::from_secs(2)).expect("first peer");
    drop(peer);
    assert_eq!(
        transport.try_send(token(), b"retained-across-reconnect"),
        TransportSendOutcome::LocalHandoff
    );
    let mut replacement = connect_until(&config, &transport);
    assert_eq!(
        replacement.receive(2_048),
        Ok(b"retained-across-reconnect".to_vec())
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
