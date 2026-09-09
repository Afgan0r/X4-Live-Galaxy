#![expect(clippy::expect_used, reason = "fixtures must fail on invalid setup")]

use std::time::Duration;

use observation_ingest::{
    CarrierControl, CollectionIntentBody, ControlBody, ControlEnvelope, HandshakeBody,
    TransportEpoch, decode_carrier_control as decode_typed_control,
    encode_carrier_control as encode_typed_control,
};
use x4_bridge::{
    CarrierBFacade, CarrierCodecError, CarrierIdentity, CompleteMessageSendOutcome,
    ConnectionState, ControlPollOutcome, ObservationCarrierFacade, decode_carrier_control,
    encode_carrier_control,
};
use x4_carrier_native::{BridgePeer, TransportConfig};

fn identity(epoch: u64) -> CarrierIdentity {
    CarrierIdentity {
        session_id: "session-a".to_owned(),
        producer_incarnation: "incarnation-a".to_owned(),
        epoch: TransportEpoch::new(epoch).expect("epoch is non-zero"),
    }
}

fn config(name: &str) -> TransportConfig {
    TransportConfig {
        pipe_name: format!(r"\\.\pipe\{name}-{}", std::process::id()),
        max_data_message_bytes: 2_048,
        max_control_message_bytes: 512,
    }
}

fn await_control(facade: &mut CarrierBFacade) -> ControlPollOutcome {
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    loop {
        let outcome = facade.poll_control(1);
        if outcome != ControlPollOutcome::NoMessage || std::time::Instant::now() >= deadline {
            return outcome;
        }
        std::thread::yield_now();
    }
}

#[test]
fn compatible_session_precedes_exact_local_exchange() {
    let config = config("carrier-b-contract");
    let expected = identity(7);
    let started = CarrierBFacade::start(config.clone(), expected.clone());
    assert!(started.is_ok(), "secure transport must start: {started:?}");
    let mut facade = started.expect("asserted successful start");
    let mut peer = BridgePeer::connect(&config, Duration::from_secs(1)).expect("peer connects");

    assert_eq!(facade.connection_state(), ConnectionState::Disconnected);
    assert_eq!(
        facade.try_send_complete(expected.epoch, b"section"),
        CompleteMessageSendOutcome::Rejected(x4_bridge::FacadeError::TransportUnavailable)
    );
    let hello = encode_carrier_control(ControlEnvelope::Handshake, &expected, 512)
        .expect("compatible hello encodes");
    peer.send_control(&hello).expect("hello reaches carrier");
    assert_eq!(
        await_control(&mut facade),
        ControlPollOutcome::Message(ControlEnvelope::Handshake)
    );
    assert_eq!(
        facade.connection_state(),
        ConnectionState::Connected(expected.epoch)
    );

    assert_eq!(
        facade.try_send_complete(expected.epoch, b"section"),
        CompleteMessageSendOutcome::LocalHandoff
    );
    assert_eq!(
        peer.receive(2_048).expect("exact message arrives"),
        b"section"
    );
}

#[test]
fn codec_rejects_stale_unknown_and_mutation_controls() {
    let expected = identity(7);
    let encoded = encode_carrier_control(ControlEnvelope::Health, &identity(6), 512);
    assert!(
        encoded.is_ok(),
        "stale identity must encode before admission: {encoded:?}"
    );
    let stale = encoded.expect("asserted successful encoding");
    assert_eq!(
        decode_carrier_control(&stale, &expected, 512),
        Err(CarrierCodecError::StaleEpoch)
    );
    assert_eq!(
        decode_carrier_control(
            br#"{"control_version":3,"kind":"mutation","session":"session-a","incarnation":"incarnation-a","epoch":7,"body":{}}"#,
            &expected,
            512
        ),
        Err(CarrierCodecError::MutationForbidden)
    );
    assert_eq!(
        decode_carrier_control(
            br#"{"control_version":3,"kind":"health","session":"session-a","incarnation":"incarnation-a","epoch":7,"body":{"status":"available"},"extra":"x"}"#,
            &expected,
            512
        ),
        Err(CarrierCodecError::UnknownField)
    );
}

#[test]
fn data_and_control_limits_reject_one_over() {
    let expected = identity(7);
    let encoded = encode_carrier_control(ControlEnvelope::Health, &expected, 512);
    assert!(encoded.is_ok(), "bounded control must encode: {encoded:?}");
    let frame = encoded.expect("asserted successful encoding");
    assert_eq!(
        decode_carrier_control(&frame, &expected, frame.len()),
        Ok(ControlEnvelope::Health)
    );
    assert_eq!(
        decode_carrier_control(&frame, &expected, frame.len() - 1),
        Err(CarrierCodecError::MessageTooLarge)
    );
}

#[test]
fn handshake_binds_the_exact_contract_tuple() {
    let expected = identity(7);
    let mut handshake = HandshakeBody {
        native_abi: 2,
        envelope_contract: 2,
        schema_version: 1,
        policy_version: 2,
        canonicalization_version: 3,
        digest_version: 1,
    };
    let exact = CarrierControl {
        identity: expected.clone(),
        body: ControlBody::Handshake(handshake.clone()),
    };
    let encoded = encode_typed_control(&exact, 512).expect("exact handshake encodes");
    assert_eq!(decode_typed_control(&encoded, &expected, 512), Ok(exact));

    handshake.envelope_contract = 1;
    let incompatible = CarrierControl {
        identity: expected.clone(),
        body: ControlBody::Handshake(handshake),
    };
    assert_eq!(
        encode_typed_control(&incompatible, 512),
        Err(CarrierCodecError::RestartRequired)
    );
    assert_eq!(
        decode_typed_control(
            br#"{"control_version":2,"kind":"handshake","session":"session-a","incarnation":"incarnation-a","epoch":7,"body":{"native_abi":2,"envelope_contract":2,"schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1}}"#,
            &expected,
            512,
        ),
        Err(CarrierCodecError::RestartRequired)
    );
    assert_eq!(
        decode_typed_control(
            br#"{"control_version":1,"kind":"handshake","session":"session-a","incarnation":"incarnation-a","epoch":7,"body":{"native_abi":2,"envelope_contract":2,"schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1}}"#,
            &expected,
            512,
        ),
        Err(CarrierCodecError::RestartRequired)
    );
}

#[test]
fn collection_intent_requires_a_positive_durable_revision_floor() {
    let expected = identity(7);
    let mut intent = CollectionIntentBody {
        section_key: "carrier_b_realtime_sample".to_owned(),
        next_revision: 3,
        max_records: 1,
        max_raw_bytes: 96,
        max_work: 2_048,
    };
    let exact = CarrierControl {
        identity: expected.clone(),
        body: ControlBody::CollectionIntent(intent.clone()),
    };
    let encoded = encode_typed_control(&exact, 512).expect("exact intent encodes");
    assert_eq!(decode_typed_control(&encoded, &expected, 512), Ok(exact));

    intent.next_revision = observation_ingest::MAX_DURABLE_SECTION_REVISION;
    assert!(
        encode_typed_control(
            &CarrierControl {
                identity: expected.clone(),
                body: ControlBody::CollectionIntent(intent.clone()),
            },
            512,
        )
        .is_ok()
    );
    intent.next_revision = observation_ingest::MAX_DURABLE_SECTION_REVISION + 1;
    assert_eq!(
        encode_typed_control(
            &CarrierControl {
                identity: expected.clone(),
                body: ControlBody::CollectionIntent(intent.clone()),
            },
            512,
        ),
        Err(CarrierCodecError::InvalidShape)
    );
    intent.next_revision = 0;
    assert_eq!(
        encode_typed_control(
            &CarrierControl {
                identity: expected.clone(),
                body: ControlBody::CollectionIntent(intent),
            },
            512,
        ),
        Err(CarrierCodecError::InvalidShape)
    );
    assert_eq!(
        decode_typed_control(
            br#"{"control_version":3,"kind":"collection_intent","session":"session-a","incarnation":"incarnation-a","epoch":7,"body":{"section_key":"carrier_b_realtime_sample","max_records":1,"max_raw_bytes":96,"max_work":2048}}"#,
            &expected,
            512,
        ),
        Err(CarrierCodecError::InvalidShape)
    );
}
