use observation_domain::{
    CanonicalizationVersion, CompleteMessage, CompletionCoverage, DigestAlgorithmVersion,
    ObservationPolicyVersion, ObservationSchemaVersion, ProducerIncarnationId,
    SectionCompletionEnvelope, SectionKey, SectionRevisionId, SourceScopeId, TransportEpoch,
};
use observation_ingest::{
    ContractVersions, bind_completion_certificate, decode_complete_message, encode_complete_message,
};

pub fn messages() -> Vec<Vec<u8>> {
    let section = "carrier_b_realtime_sample";
    let start = String::from_utf8(super::super::carrier_b_support::start_bytes(section))
        .expect("UTF-8")
        .replacen("\"expected_records\":0", "\"expected_records\":1", 1)
        .replacen(
            "\"coverage\":\"complete\"",
            "\"coverage\":\"point_measurement\"",
            1,
        )
        .into_bytes();
    let batch = format!(
        "{{\"type\":\"immutable_batch\",\"contract_version\":2,\"source_scope\":\"scope:x4\",\"producer_incarnation\":\"producer:1\",\"transport_epoch\":1,\"section_key\":\"{section}\",\"section_revision\":1,\"batch_id\":\"inner:sample:1\",\"section_ordinal\":1,\"records\":[{{\"record_id\":\"record:sample:1\",\"entity_id\":\"x4:runtime:realtime_clock\",\"observation_version\":1,\"content\":\"123.0\"}}],\"optional_detail\":null}}"
    )
    .into_bytes();
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
