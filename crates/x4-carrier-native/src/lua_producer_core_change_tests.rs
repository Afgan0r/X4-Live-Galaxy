#![expect(
    clippy::expect_used,
    reason = "ABI regression fixtures fail immediately when setup violates the contract"
)]

use std::num::NonZeroUsize;

use observation_domain::{SourceBoundary, SourceEpochStatus, TransportEpoch};
use observation_ingest::{
    CarrierControl, CarrierIdentity, CollectionIntentBody, ControlBody, HandshakeBody,
    encode_carrier_control,
};

use super::super::fail_section;
use crate::{
    CarrierLimits, HandleRegistry, OpenConfig, Producer, ProducerLimits, ProducerOutcome,
    ProducerSource,
    abi::{API, PRODUCER, REGISTRY},
    lua_producer_test_support::{ABI_TEST_LOCK, FakeLuaState, fake_api},
};

#[test]
fn core_change_failure_allows_a_fresh_core_intent() {
    let _test_guard = ABI_TEST_LOCK.lock().expect("ABI test lock");
    let _ = API.set(fake_api());
    let token = {
        let mut registry = REGISTRY.lock().expect("registry lock");
        *registry = HandleRegistry::new();
        crate::lua_generation::initialize(&mut registry).expect("generation initializes");
        registry
            .open(OpenConfig::current(CarrierLimits::new(
                NonZeroUsize::new(2_048).expect("data limit"),
                NonZeroUsize::new(512).expect("control limit"),
            )))
            .expect("handle opens")
    };
    let source = ProducerSource {
        session_id: "session:core-change".into(),
        producer_incarnation: "producer:core-change".into(),
        transport_epoch: u64::from(token.generation),
        source_scope: "x4:carrier_b_acceptance".into(),
        source_epoch_status: SourceEpochStatus::Unknown,
        source_boundary: SourceBoundary::RuntimeStart,
    };
    let mut producer =
        Producer::new(ProducerLimits::bring_up(), source.clone(), 0).expect("producer starts");
    producer.mark_local_handoff(0).expect("bootstrap handoff");
    producer
        .apply_control(&control(&source, ControlBody::Handshake(handshake())), 0)
        .expect("handshake accepted");
    *PRODUCER.lock().expect("producer lock") = Some(producer);
    let mut state = FakeLuaState {
        token: format!("{}:{}", token.generation, token.slot).into_bytes(),
        reason: Some(b"core_changed".to_vec()),
        pushed: Vec::new(),
        pushed_strings: Vec::new(),
    };
    assert_eq!(unsafe { fail_section((&raw mut state).cast()) }, 1);
    assert_eq!(state.pushed.pop(), Some(0));
    let intent = ControlBody::CollectionIntent(CollectionIntentBody {
        section_key: "ship_core".into(),
        next_revision: 1,
        max_records: 1,
        max_raw_bytes: 96,
        max_work: 1,
    });
    assert_eq!(
        PRODUCER
            .lock()
            .expect("producer lock")
            .as_mut()
            .expect("producer retained")
            .apply_control(&control(&source, intent), 1),
        Ok(ProducerOutcome::Accepted)
    );
    *PRODUCER.lock().expect("producer lock") = None;
    let _ = REGISTRY.lock().map(|mut registry| registry.close(token));
}

fn control(source: &ProducerSource, body: ControlBody) -> Vec<u8> {
    encode_carrier_control(
        &CarrierControl {
            identity: CarrierIdentity {
                session_id: source.session_id.clone(),
                producer_incarnation: source.producer_incarnation.clone(),
                epoch: TransportEpoch::new(source.transport_epoch).expect("epoch"),
            },
            body,
        },
        512,
    )
    .expect("control encodes")
}

const fn handshake() -> HandshakeBody {
    HandshakeBody {
        native_abi: 2,
        envelope_contract: 2,
        schema_version: 1,
        policy_version: 2,
        canonicalization_version: 3,
        digest_version: 1,
    }
}
