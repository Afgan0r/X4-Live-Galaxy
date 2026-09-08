#![expect(
    clippy::expect_used,
    reason = "ABI security fixtures must stop immediately when setup is invalid"
)]

use std::{num::NonZeroUsize, time::Duration};

use observation_domain::{SourceBoundary, SourceEpochStatus};

use super::{ABI_TEST_LOCK, FakeLuaState, fake_api};
use crate::{
    CarrierLimits, HandleRegistry, NativeTransport, OpenConfig, Producer, ProducerLimits,
    ProducerSource, TransportConfig, TransportSendOutcome,
    abi::{API, PRODUCER, REGISTRY, TRANSPORT},
};

#[test]
fn stale_and_malformed_teardown_cannot_mutate_replacement_owner() {
    let _test_guard = ABI_TEST_LOCK.lock().expect("ABI test lock");
    let (stale, active) = install_replacement("ownership");

    assert_call("close", stale, None, -16);
    assert_call("reset", stale, Some(b"operator_reset"), -16);
    assert_call_raw("close", b"malformed", None, -20);
    assert_call_raw("reset", b"malformed", Some(b"operator_reset"), -20);
    assert_call("reset", active, None, -20);
    assert_call("reset", active, Some(b""), -20);
    assert_call("reset", active, Some(&[b'x'; 65]), -20);

    assert!(REGISTRY.lock().expect("registry lock").is_active(active));
    let producer = PRODUCER.lock().expect("producer lock");
    assert_eq!(
        producer
            .as_ref()
            .expect("producer retained")
            .source()
            .producer_incarnation,
        format!("x4-producer-{}", active.generation)
    );
    drop(producer);
    let transport = TRANSPORT.lock().expect("transport lock");
    assert_eq!(
        transport
            .as_ref()
            .expect("transport retained")
            .try_send(active, b"still-owned"),
        TransportSendOutcome::LocalHandoff
    );
    drop(transport);
    cleanup(active);
}

#[test]
fn active_owner_reset_and_close_have_distinct_registry_effects() {
    let _test_guard = ABI_TEST_LOCK.lock().expect("ABI test lock");
    let (_stale, active) = install_replacement("reset");

    assert_call("reset", active, Some(b"operator_reset"), 0);
    assert!(REGISTRY.lock().expect("registry lock").is_active(active));
    assert!(PRODUCER.lock().expect("producer lock").is_none());
    assert!(TRANSPORT.lock().expect("transport lock").is_none());

    let (_stale, active) = install_replacement("close");
    assert_call("close", active, None, 0);
    assert!(!REGISTRY.lock().expect("registry lock").is_active(active));
    assert!(PRODUCER.lock().expect("producer lock").is_none());
    assert!(TRANSPORT.lock().expect("transport lock").is_none());
}

fn install_replacement(label: &str) -> (crate::HandleToken, crate::HandleToken) {
    let _ = API.set(fake_api());
    let limits = CarrierLimits::new(
        NonZeroUsize::new(2_048).expect("data limit"),
        NonZeroUsize::new(512).expect("control limit"),
    );
    let (stale, active) = {
        let mut registry = REGISTRY.lock().expect("registry lock");
        *registry = HandleRegistry::new();
        crate::lua_generation::initialize(&mut registry).expect("generation initializes");
        let stale = registry
            .open(OpenConfig::current(limits))
            .expect("first handle opens");
        let active = registry
            .open(OpenConfig::current(limits))
            .expect("replacement handle opens");
        (stale, active)
    };
    let config = TransportConfig {
        pipe_name: format!(
            r"\\.\pipe\live-galaxy-abi-teardown-{label}-{}",
            std::process::id()
        ),
        max_data_message_bytes: 2_048,
        max_control_message_bytes: 512,
    };
    let transport = NativeTransport::start(config, active).expect("transport starts");
    let source = ProducerSource {
        session_id: format!("x4-session-{}", active.generation),
        producer_incarnation: format!("x4-producer-{}", active.generation),
        transport_epoch: u64::from(active.generation),
        source_scope: "x4:carrier_b_acceptance".to_owned(),
        source_epoch_status: SourceEpochStatus::Unknown,
        source_boundary: SourceBoundary::RuntimeStart,
    };
    let producer = Producer::new(ProducerLimits::bring_up(), source, 0).expect("producer starts");
    *TRANSPORT.lock().expect("transport lock") = Some(transport);
    *PRODUCER.lock().expect("producer lock") = Some(producer);
    (stale, active)
}

fn assert_call(operation: &str, token: crate::HandleToken, reason: Option<&[u8]>, expected: isize) {
    let encoded = format!("{}:{}", token.generation, token.slot);
    assert_call_raw(operation, encoded.as_bytes(), reason, expected);
}

fn assert_call_raw(operation: &str, token: &[u8], reason: Option<&[u8]>, expected: isize) {
    let function = crate::lua_operations::REGISTRATIONS
        .iter()
        .find(|(name, _)| &name[..name.len() - 1] == operation.as_bytes())
        .map(|(_, function)| *function)
        .expect("operation registered");
    let mut state = FakeLuaState {
        token: token.to_vec(),
        reason: reason.map(<[u8]>::to_vec),
        pushed: Vec::new(),
    };
    assert_eq!(unsafe { function((&raw mut state).cast()) }, 1);
    assert_eq!(state.pushed, vec![expected]);
}

fn cleanup(token: crate::HandleToken) {
    if let Some(transport) = TRANSPORT.lock().expect("transport lock").take() {
        let _ = transport.request_close(token);
        assert!(transport.wait_closed_for_test(Duration::from_secs(2)));
    }
    *PRODUCER.lock().expect("producer lock") = None;
    let _ = REGISTRY.lock().map(|mut registry| registry.close(token));
}
