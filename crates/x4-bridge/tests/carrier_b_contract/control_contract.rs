use observation_ingest::{
    CarrierControl, CollectionIntentBody, ControlBody, HandshakeBody,
    decode_carrier_control as decode_typed_control, encode_carrier_control as encode_typed_control,
};
use x4_bridge::CarrierCodecError;

use super::identity;

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
    for version in [1, 2] {
        let frame = format!(
            r#"{{"control_version":{version},"kind":"handshake","session":"session-a","incarnation":"incarnation-a","epoch":7,"body":{{"native_abi":2,"envelope_contract":2,"schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1}}}}"#
        );
        assert_eq!(
            decode_typed_control(frame.as_bytes(), &expected, 512),
            Err(CarrierCodecError::RestartRequired)
        );
    }
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
