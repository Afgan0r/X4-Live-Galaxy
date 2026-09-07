use observation_ingest::{
    CompleteMessage, EnvelopeDecodeError, decode_complete_message, encode_complete_message,
};

const START_V2: &[u8] = br#"{"type":"section_start","contract_version":2,"source_scope":"scope:x4","producer_incarnation":"producer:1","transport_epoch":1,"section_key":"ship_point","section_revision":1,"expected_records":1,"sender_evidence":{"capture_clock":"game_time_millis","capture_start_millis":10,"capture_end_millis":20,"freshness":"fresh","quality":"fresh","availability":"available","coverage":"point_measurement","source_epoch":null,"source_epoch_status":"unknown","source_boundary":"runtime_start","source_consistency":"observed_count_fill_only","stable_identity":false,"schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1}}"#;

const COMPLETION_V2_WITH_DECODED_BYTES: &[u8] = br#"{"type":"section_completion","contract_version":2,"source_scope":"scope:x4","producer_incarnation":"producer:1","transport_epoch":1,"section_key":"ship_point","section_revision":1,"batch_count":0,"record_count":0,"raw_bytes":0,"decoded_bytes":0,"ordered_batch_manifest_digest":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","canonical_content_digest":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1,"coverage":"point_measurement","sender_evidence":{"capture_clock":"game_time_millis","capture_start_millis":10,"capture_end_millis":20,"freshness":"fresh","quality":"fresh","availability":"available","coverage":"point_measurement","source_epoch":null,"source_epoch_status":"unknown","source_boundary":"runtime_start","source_consistency":"observed_count_fill_only","stable_identity":false,"schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1}}"#;

#[test]
fn exact_v2_start_decodes_with_sender_evidence() {
    let decoded = decode_complete_message(START_V2, 4_096);
    assert!(
        matches!(decoded, Ok(CompleteMessage::SectionStart(_))),
        "exact envelope contract v2 must decode: {decoded:?}"
    );
}

#[test]
fn decoded_bytes_is_rejected_instead_of_ignored() {
    assert_eq!(
        decode_complete_message(COMPLETION_V2_WITH_DECODED_BYTES, 4_096),
        Err(EnvelopeDecodeError::InvalidShape)
    );
}

#[test]
fn envelope_v1_and_unknown_versions_fail_closed() {
    for version in [1, 3] {
        let fixture = String::from_utf8(START_V2.to_vec())
            .expect("fixture is UTF-8")
            .replacen(
                "\"contract_version\":2",
                &format!("\"contract_version\":{version}"),
                1,
            );
        assert_eq!(
            decode_complete_message(fixture.as_bytes(), 4_096),
            Err(EnvelopeDecodeError::UnsupportedVersion)
        );
    }
}

#[test]
fn exact_v2_round_trips_with_a_bounded_pure_encoder() {
    let decoded = decode_complete_message(START_V2, 4_096).expect("exact v2 fixture decodes");
    let encoded = encode_complete_message(&decoded, 4_096).expect("exact v2 message encodes");
    assert_eq!(decode_complete_message(&encoded, 4_096), Ok(decoded));
    assert_eq!(
        encode_complete_message(
            &decode_complete_message(START_V2, 4_096).expect("fixture decodes"),
            1
        ),
        Err(EnvelopeDecodeError::MessageTooLarge)
    );
}

#[test]
fn receiver_owned_context_fields_are_rejected_on_the_wire() {
    let forged = String::from_utf8(START_V2.to_vec())
        .expect("fixture is UTF-8")
        .replacen(
            "\"sender_evidence\":{",
            "\"expected_current\":7,\"sender_evidence\":{",
            1,
        );
    assert_eq!(
        decode_complete_message(forged.as_bytes(), 4_096),
        Err(EnvelopeDecodeError::InvalidShape)
    );
}
