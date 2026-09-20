use std::{num::NonZeroUsize, time::Duration};

use observation_domain::{SourceBoundary, SourceEpochStatus};

use observation_ingest::{
    CarrierControl, CarrierIdentity, ControlBody, HandshakeBody, encode_carrier_control,
};

use super::poll_control;
use crate::{
    BridgePeer, CarrierLimits, HandleRegistry, NativeTransport, OpenConfig, Producer,
    ProducerLimits, ProducerSource, TransportConfig,
    abi::{API, PRODUCER, REGISTRY, TRANSPORT},
    lua_producer_test_support::{ABI_TEST_LOCK, FakeLuaState, fake_api},
};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "ABI recovery scenario keeps the ordered real transport assertions together"
)]
fn abi_poll_control_retains_control_and_state_while_clock_is_unavailable() {
    let _test_guard = ABI_TEST_LOCK.lock().expect("ABI test lock");
    let _ = API.set(fake_api());
    let limits = CarrierLimits::new(
        NonZeroUsize::new(2_048).expect("data limit"),
        NonZeroUsize::new(512).expect("control limit"),
    );
    let token = {
        let mut registry = REGISTRY.lock().expect("registry lock");
        *registry = HandleRegistry::new();
        crate::lua_generation::initialize(&mut registry).expect("generation initializes");
        registry
            .open(OpenConfig::current(limits))
            .expect("handle opens")
    };
    let config = TransportConfig {
        pipe_name: format!(r"\\.\pipe\live-galaxy-abi-clock-{}", std::process::id()),
        max_data_message_bytes: 2_048,
        max_control_message_bytes: 512,
    };
    let transport = NativeTransport::start(config.clone(), token).expect("transport starts");
    let mut peer = BridgePeer::connect(&config, Duration::from_secs(2)).expect("peer connects");
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while !transport.snapshot().connected && std::time::Instant::now() < deadline {
        std::thread::yield_now();
    }
    let snapshot = transport.snapshot();
    assert!(snapshot.connected);
    let source = ProducerSource {
        session_id: "session:abi-clock".to_owned(),
        producer_incarnation: "producer:abi-clock".to_owned(),
        transport_epoch: u64::from(token.generation),
        source_scope: "x4:carrier_b_acceptance".to_owned(),
        source_epoch_status: SourceEpochStatus::Unknown,
        source_boundary: SourceBoundary::RuntimeStart,
    };
    let mut producer =
        Producer::new(ProducerLimits::bring_up(), source.clone(), 100).expect("producer starts");
    producer
        .observe_connection(snapshot.connection_generation, 100)
        .expect("connection observed");
    producer.mark_local_handoff(100).expect("bootstrap handoff");
    let pending = producer.pending_bytes().map(<[u8]>::to_vec);
    let state = producer.state();
    let control = encode_carrier_control(
        &CarrierControl {
            identity: CarrierIdentity {
                session_id: source.session_id,
                producer_incarnation: source.producer_incarnation,
                epoch: observation_domain::TransportEpoch::new(source.transport_epoch)
                    .expect("epoch"),
            },
            body: ControlBody::Handshake(HandshakeBody {
                native_abi: 2,
                envelope_contract: 2,
                schema_version: 1,
                policy_version: 2,
                canonicalization_version: 3,
                digest_version: 1,
            }),
        },
        512,
    )
    .expect("control encodes");
    peer.send_control(&control).expect("control queued");
    assert!(transport.wait_for_control_for_test(Duration::from_secs(2)));
    assert!(transport.set_clock_for_test(None));
    *TRANSPORT.lock().expect("transport lock") = Some(transport);
    *PRODUCER.lock().expect("producer lock") = Some(producer);
    let mut state_value = FakeLuaState {
        token: format!("{}:{}", token.generation, token.slot).into_bytes(),
        reason: None,
        pushed: Vec::new(),
        pushed_strings: Vec::new(),
    };
    let state_pointer = (&raw mut state_value).cast();

    let status_count = {
        let producer = PRODUCER.lock().expect("producer lock");
        let transport = TRANSPORT.lock().expect("transport lock");
        unsafe {
            crate::lua_progress::push(
                fake_api(),
                state_pointer,
                0,
                producer.as_ref().expect("producer retained"),
                transport.as_ref().expect("transport retained"),
                100,
            )
        }
    };
    assert_eq!(
        status_count, 9,
        "status includes profile, remaining capacity and exact revision"
    );
    assert_eq!(
        &state_value.pushed_strings[5..],
        ["none", "1", "1"],
        "status exposes a bounded inactive selection"
    );

    assert_eq!(unsafe { poll_control(state_pointer) }, 1);
    assert_eq!(state_value.pushed.pop(), Some(-22));
    {
        let producer = PRODUCER.lock().expect("producer lock");
        let producer = producer.as_ref().expect("producer retained");
        assert_eq!(producer.state(), state);
        assert_eq!(producer.pending_bytes(), pending.as_deref());
    }
    let transport_guard = TRANSPORT.lock().expect("transport lock");
    let transport = transport_guard.as_ref().expect("transport retained");
    assert_eq!(
        transport.snapshot().connection_generation,
        snapshot.connection_generation
    );
    assert!(transport.set_clock_for_test(Some(101)));
    drop(transport_guard);

    assert_eq!(unsafe { poll_control(state_pointer) }, 1);
    assert_eq!(state_value.pushed.pop(), Some(0));
    assert_eq!(unsafe { poll_control(state_pointer) }, 1);
    assert_eq!(state_value.pushed.pop(), Some(3));

    let transport = TRANSPORT.lock().expect("transport lock").take();
    if let Some(transport) = transport {
        let _ = transport.request_close(token);
    }
    *PRODUCER.lock().expect("producer lock") = None;
    let _ = REGISTRY.lock().map(|mut registry| registry.close(token));
}

#[path = "lua_teardown_tests.rs"]
mod teardown;

#[path = "lua_producer_core_change_tests.rs"]
mod core_change;
