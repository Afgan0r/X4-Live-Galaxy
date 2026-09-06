#![expect(clippy::expect_used, reason = "fixtures must fail on invalid setup")]

use std::time::Duration;

use observation_ingest::{ControlEnvelope, TransportEpoch};
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
        facade.poll_control(1),
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
            b"v=1;kind=mutation;session=session-a;incarnation=incarnation-a;epoch=7",
            &expected,
            512
        ),
        Err(CarrierCodecError::MutationForbidden)
    );
    assert_eq!(
        decode_carrier_control(
            b"v=1;kind=health;session=session-a;incarnation=incarnation-a;epoch=7;extra=x",
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
