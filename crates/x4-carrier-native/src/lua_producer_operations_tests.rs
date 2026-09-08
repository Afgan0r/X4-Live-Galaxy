use core::ffi::{c_char, c_void};
use std::{num::NonZeroUsize, sync::Mutex, time::Duration};

use observation_domain::{SourceBoundary, SourceEpochStatus};

use observation_ingest::{
    CarrierControl, CarrierIdentity, ControlBody, HandshakeBody, encode_carrier_control,
};

use super::poll_control;
use crate::{
    BridgePeer, CarrierLimits, HandleRegistry, NativeTransport, OpenConfig, Producer,
    ProducerLimits, ProducerSource, TransportConfig,
    abi::{API, PRODUCER, REGISTRY, TRANSPORT},
    abi_windows::LuaApi,
};

struct FakeLuaState {
    token: Vec<u8>,
    reason: Option<Vec<u8>>,
    pushed: Vec<isize>,
}
static ABI_TEST_LOCK: Mutex<()> = Mutex::new(());

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
    };
    let state_pointer = (&raw mut state_value).cast();

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

fn fake_api() -> LuaApi {
    LuaApi {
        create_table: noop_table,
        push_closure: noop_closure,
        push_integer,
        push_string: noop_string,
        set_field: noop_field,
        to_integer: noop_integer,
        to_string,
        get_top: noop_int,
        get_metatable: noop_index_int,
        lua_type,
        next: noop_index_int,
        push_nil: noop_state,
        raw_get: noop_index,
        set_top: noop_index,
        to_boolean: noop_index_int,
        to_number: noop_number,
    }
}

unsafe extern "C" fn push_integer(state: *mut c_void, value: isize) {
    unsafe { &mut *state.cast::<FakeLuaState>() }
        .pushed
        .push(value);
}
unsafe extern "C" fn to_string(
    state: *mut c_void,
    index: i32,
    length: *mut usize,
) -> *const c_char {
    let state = unsafe { &mut *state.cast::<FakeLuaState>() };
    let bytes = if index == 1 {
        &state.token
    } else {
        state.reason.as_ref().unwrap_or(&state.token)
    };
    unsafe { *length = bytes.len() };
    bytes.as_ptr().cast()
}
unsafe extern "C" fn lua_type(state: *mut c_void, index: i32) -> i32 {
    let state = unsafe { &*state.cast::<FakeLuaState>() };
    i32::from(index == 1 || (index == 2 && state.reason.is_some())) * 4
}
unsafe extern "C" fn noop_table(_: *mut c_void, _: i32, _: i32) {}
unsafe extern "C" fn noop_closure(_: *mut c_void, _: Option<crate::abi_windows::LuaFn>, _: i32) {}
unsafe extern "C" fn noop_string(_: *mut c_void, _: *const c_char, _: usize) {}
unsafe extern "C" fn noop_field(_: *mut c_void, _: i32, _: *const c_char) {}
unsafe extern "C" fn noop_integer(_: *mut c_void, _: i32) -> isize {
    0
}
unsafe extern "C" fn noop_int(_: *mut c_void) -> i32 {
    0
}
unsafe extern "C" fn noop_index_int(_: *mut c_void, _: i32) -> i32 {
    0
}
unsafe extern "C" fn noop_state(_: *mut c_void) {}
unsafe extern "C" fn noop_index(_: *mut c_void, _: i32) {}
unsafe extern "C" fn noop_number(_: *mut c_void, _: i32) -> f64 {
    0.0
}

#[path = "lua_teardown_tests.rs"]
mod teardown;
