#![expect(
    clippy::panic,
    reason = "bounded integration watchdogs fail immediately"
)]

use observation_domain::{
    CanonicalizationVersion, CompleteMessage, CompletionCoverage, DigestAlgorithmVersion,
    ObservationPolicyVersion, ObservationSchemaVersion, ProducerIncarnationId,
    SectionCompletionEnvelope, SectionKey, SectionRevisionId, SourceScopeId, TransportEpoch,
};
use observation_ingest::{
    CarrierControl, CarrierIdentity, ContractVersions, ControlBody, bind_completion_certificate,
    complete_message_digest, decode_carrier_bootstrap, decode_carrier_control,
    decode_complete_message, encode_carrier_control, encode_complete_message,
};
use x4_bridge::{CarrierCodecError, PIPE_ENDPOINT};
use x4_carrier_native::{HandleToken, NativeTransport, TransportConfig};

use super::carrier_b_support;
use super::startup_support::{
    TempDirectory, await_control, fresh_readback, send_data, spawn_bridge, valid_limits,
    wait_connected, wait_for_history,
};

#[test]
fn bootstrap_is_handshake_only_and_message_digest_is_byte_exact() {
    let expected = identity("bootstrap");
    let bootstrap = control(&expected, ControlBody::Handshake(exact_handshake()));
    assert_eq!(decode_carrier_bootstrap(&bootstrap, 2_048), Ok(expected));
    assert_eq!(
        decode_carrier_bootstrap(
            br#"{"control_version":2,"kind":"health","session":"s","incarnation":"i","epoch":1,"body":{"native_abi":2,"envelope_contract":2,"schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1}}"#,
            2_048,
        ),
        Err(CarrierCodecError::UnknownKind)
    );
    assert_eq!(
        complete_message_digest(b"abc"),
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ]
    );
    assert_ne!(
        complete_message_digest(b"abc"),
        complete_message_digest(b"abd")
    );
}

#[test]
fn both_startup_orders_publish_actual_three_message_sample() {
    run_actual_sample("native-first", true);
    run_actual_sample("bridge-first", false);
}

fn run_actual_sample(label: &str, native_first: bool) {
    let directory = TempDirectory::new(label);
    let limits = directory.path().join("limits.json");
    std::fs::write(&limits, valid_limits()).expect("limits written");
    let config = TransportConfig {
        pipe_name: PIPE_ENDPOINT.to_owned(),
        max_data_message_bytes: 8_192,
        max_control_message_bytes: 2_048,
    };
    let token = HandleToken {
        generation: 1,
        slot: 1,
    };
    let (transport, mut child) = if native_first {
        let transport = NativeTransport::start(config, token).expect("native server");
        (transport, spawn_bridge(directory.path(), &limits))
    } else {
        let child = spawn_bridge(directory.path(), &limits);
        wait_for_history(directory.path(), "peer-absent");
        (
            NativeTransport::start(config, token).expect("native server"),
            child,
        )
    };
    wait_connected(&transport);
    let identity = identity(label);
    send_data(
        &transport,
        token,
        &control(&identity, ControlBody::Handshake(exact_handshake())),
    );
    for expected in ["handshake", "collection_intent", "demand"] {
        let value = decode_carrier_control(&await_control(&transport, token), &identity, 2_048)
            .expect("bound control");
        assert!(matches!(
            (expected, value.body),
            ("handshake", ControlBody::Handshake(_))
                | ("collection_intent", ControlBody::CollectionIntent(_))
                | ("demand", ControlBody::Demand(_))
        ));
    }
    let mut dispositions = Vec::new();
    for message in sample_messages() {
        send_data(&transport, token, &message);
        let value = decode_carrier_control(&await_control(&transport, token), &identity, 2_048)
            .expect("disposition");
        let ControlBody::Disposition(value) = value.body else {
            panic!("disposition expected");
        };
        dispositions.push(value.disposition);
    }
    child.kill().expect("runner-owned child terminates");
    child.wait().expect("runner-owned child reaped");
    assert_eq!(dispositions, ["received", "received", "committed"]);
    wait_for_history(directory.path(), "\"state\":\"committed\"");
    fresh_readback(directory.path());
    let _ = transport.request_close(token);
    assert!(transport.wait_closed_for_test(std::time::Duration::from_secs(2)));
}

fn identity(label: &str) -> CarrierIdentity {
    CarrierIdentity {
        session_id: format!("session-{label}"),
        producer_incarnation: "producer:1".to_owned(),
        epoch: TransportEpoch::new(1).expect("epoch"),
    }
}

const fn exact_handshake() -> observation_ingest::HandshakeBody {
    observation_ingest::HandshakeBody {
        native_abi: 2,
        envelope_contract: 2,
        schema_version: 1,
        policy_version: 2,
        canonicalization_version: 3,
        digest_version: 1,
    }
}

fn control(identity: &CarrierIdentity, body: ControlBody) -> Vec<u8> {
    encode_carrier_control(
        &CarrierControl {
            identity: identity.clone(),
            body,
        },
        2_048,
    )
    .expect("control encodes")
}

fn sample_messages() -> Vec<Vec<u8>> {
    let section = "carrier_b_realtime_sample";
    let start = String::from_utf8(carrier_b_support::start_bytes(section))
        .expect("UTF-8")
        .replacen("\"expected_records\":0", "\"expected_records\":1", 1)
        .replacen(
            "\"coverage\":\"complete\"",
            "\"coverage\":\"point_measurement\"",
            1,
        )
        .into_bytes();
    let batch = format!("{{\"type\":\"immutable_batch\",\"contract_version\":2,\"source_scope\":\"scope:x4\",\"producer_incarnation\":\"producer:1\",\"transport_epoch\":1,\"section_key\":\"{section}\",\"section_revision\":1,\"batch_id\":\"inner:sample:1\",\"section_ordinal\":1,\"records\":[{{\"record_id\":\"record:sample:1\",\"entity_id\":\"x4:runtime:realtime_clock\",\"observation_version\":1,\"content\":\"123.0\"}}],\"optional_detail\":null}}").into_bytes();
    let CompleteMessage::ImmutableBatch(decoded) =
        decode_complete_message(&batch, 8_192).expect("batch decodes")
    else {
        panic!("batch expected");
    };
    let mut evidence = observation_domain::SenderEvidence::legacy_default();
    evidence.section_state = observation_domain::SectionState::new(
        observation_domain::SectionFreshness::Fresh,
        observation_domain::SectionCoverage::PointMeasurement,
    );
    let completion = SectionCompletionEnvelope {
        source_scope: SourceScopeId::new("scope:x4").expect("scope"),
        producer_incarnation: ProducerIncarnationId::new("producer:1").expect("producer"),
        transport_epoch: TransportEpoch::new(1).expect("epoch"),
        section_key: SectionKey::new(section).expect("section"),
        section_revision: SectionRevisionId::new(1).expect("revision"),
        batch_count: 0,
        record_count: 0,
        raw_bytes: 0,
        ordered_batch_manifest_digest: [0; 32],
        canonical_content_digest: [0; 32],
        schema_version: ObservationSchemaVersion::new(1).expect("schema"),
        policy_version: ObservationPolicyVersion::new(2).expect("policy"),
        canonicalization_version: CanonicalizationVersion::new(3).expect("canonicalization"),
        digest_version: DigestAlgorithmVersion::new(1).expect("digest"),
        coverage: CompletionCoverage::PointMeasurement,
        sender_evidence: evidence,
    };
    let versions = ContractVersions::new(
        ObservationSchemaVersion::new(1).expect("schema"),
        ObservationPolicyVersion::new(2).expect("policy"),
        CanonicalizationVersion::new(3).expect("canonicalization"),
        DigestAlgorithmVersion::new(1).expect("digest"),
    );
    let completion = bind_completion_certificate(completion, &[decoded], versions).expect("binds");
    let completion =
        encode_complete_message(&CompleteMessage::SectionCompletion(completion), 8_192)
            .expect("completion encodes");
    vec![start, batch, completion]
}
