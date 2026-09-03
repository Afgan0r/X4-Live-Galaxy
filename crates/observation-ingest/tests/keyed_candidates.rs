#![expect(clippy::expect_used, reason = "invalid test fixtures fail immediately")]

use observation_domain::{
    BatchId, EntityId, EnvelopeRecord, ImmutableBatchEnvelope, ObservationVersion,
    ProducerIncarnationId, RecordId, SectionKey, SectionRevisionId, SectionStartEnvelope,
    SourceScopeId, TransportEpoch,
};
use observation_ingest::{
    AcceptedProjection, AggregateLimits, CandidateLimits, GenerationLimits, GenerationStager,
    ReceiverDisposition,
};

fn id<T>(value: &str, make: impl FnOnce(String) -> Option<T>) -> T {
    make(value.to_owned()).expect("fixture identity is valid")
}

fn start(section: &str, revision: u64, expected: usize) -> SectionStartEnvelope {
    SectionStartEnvelope {
        source_scope: id("scope:x4", SourceScopeId::new),
        producer_incarnation: id("producer:1", ProducerIncarnationId::new),
        transport_epoch: TransportEpoch::new(1).expect("epoch is positive"),
        section_key: id(section, SectionKey::new),
        section_revision: SectionRevisionId::new(revision).expect("revision is positive"),
        expected_records: expected,
    }
}

fn batch(section: &str, revision: u64, identity: &str, version: u64) -> ImmutableBatchEnvelope {
    ImmutableBatchEnvelope {
        source_scope: id("scope:x4", SourceScopeId::new),
        section_key: id(section, SectionKey::new),
        section_revision: SectionRevisionId::new(revision).expect("revision is positive"),
        batch_id: id(identity, BatchId::new),
        records: vec![EnvelopeRecord {
            record_id: id(&format!("record:{section}"), RecordId::new),
            entity_id: id(&format!("entity:{section}"), EntityId::new),
            observation_version: ObservationVersion::new(version).expect("version is positive"),
            content: format!("content:{section}:{version}"),
        }],
        optional_detail: None,
    }
}

fn limits() -> GenerationLimits {
    GenerationLimits::bounded(
        CandidateLimits::new(16, 32, 2, 2, 4, 10, 5).expect("limits are non-zero"),
        AggregateLimits::new(2, 24, 48, 4, 4, 8).expect("limits are non-zero"),
    )
}

#[test]
fn alternating_batches_advance_independent_keyed_candidates() {
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
    assert_eq!(
        stager.start_section(start("ships", 1, 1), 1),
        ReceiverDisposition::Received
    );
    assert_eq!(
        stager.start_section(start("stations", 1, 1), 1),
        ReceiverDisposition::Received
    );

    assert_eq!(
        stager.stage_section_batch(batch("ships", 1, "batch:ships", 1), b"ship", 8, 1, 2),
        ReceiverDisposition::Received
    );
    assert_eq!(
        stager.stage_section_batch(
            batch("stations", 1, "batch:stations", 1),
            b"station",
            9,
            1,
            3,
        ),
        ReceiverDisposition::Received
    );
    assert_eq!(stager.candidate_count(), 2);
    assert_eq!(
        stager
            .candidate_usage(&id("ships", SectionKey::new))
            .unwrap()
            .batches,
        1
    );
    assert_eq!(
        stager
            .candidate_usage(&id("stations", SectionKey::new))
            .unwrap()
            .batches,
        1
    );
    assert_eq!(
        stager.invalidate_source_scope(&id("scope:x4", SourceScopeId::new)),
        2
    );
    assert_eq!(stager.aggregate_usage().candidate_count, 0);
}

#[test]
fn exact_replay_is_charge_free_and_changed_bytes_drop_only_that_key() {
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
    let _ = stager.start_section(start("ships", 1, 1), 1);
    let _ = stager.start_section(start("stations", 1, 1), 1);
    let ship_batch = batch("ships", 1, "batch:ships", 1);
    assert_eq!(
        stager.stage_section_batch(ship_batch.clone(), b"ship", 8, 1, 2),
        ReceiverDisposition::Received
    );
    let before = stager.aggregate_usage();
    assert_eq!(
        stager.stage_section_batch(ship_batch.clone(), b"ship", 8, 1, 3),
        ReceiverDisposition::Received
    );
    assert_eq!(stager.aggregate_usage(), before);
    assert_eq!(
        stager.stage_section_batch(ship_batch, b"changed", 8, 1, 4),
        ReceiverDisposition::PermanentlyRejected
    );
    assert!(
        stager
            .candidate_usage(&id("ships", SectionKey::new))
            .is_none()
    );
    assert!(
        stager
            .candidate_usage(&id("stations", SectionKey::new))
            .is_some()
    );
}

#[test]
fn independent_limits_and_expiry_release_exact_aggregate_charges() {
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
    let _ = stager.start_section(start("ships", 1, 1), 1);
    assert_eq!(
        stager.stage_section_batch(batch("ships", 1, "batch:ships", 1), &[b'x'; 16], 32, 4, 2),
        ReceiverDisposition::Received
    );
    let charged = stager.aggregate_usage();
    assert_eq!(charged.raw_bytes, 16);
    assert_eq!(
        stager.stage_section_batch(batch("ships", 1, "batch:over", 2), b"x", 1, 1, 3),
        ReceiverDisposition::PermanentlyRejected
    );
    assert_eq!(stager.aggregate_usage().raw_bytes, 0);

    let _ = stager.start_section(start("stations", 2, 1), 10);
    let _ = stager.stage_section_batch(batch("stations", 2, "batch:stations", 1), b"ok", 2, 1, 11);
    assert_eq!(stager.expire_candidates(17), 1);
    assert_eq!(stager.aggregate_usage().candidate_count, 0);
}

#[test]
fn accepted_versions_reject_regression_and_equal_version_conflicts() {
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
    assert!(stager.record_accepted_entity(
        id("scope:x4", SourceScopeId::new),
        id("entity:ships", EntityId::new),
        ObservationVersion::new(2).expect("version is positive"),
        b"accepted:ships:2",
    ));

    for (version, expected) in [
        (1, ReceiverDisposition::PermanentlyRejected),
        (2, ReceiverDisposition::PermanentlyRejected),
        (3, ReceiverDisposition::Received),
    ] {
        let _ = stager.start_section(start("ships", version, 1), version);
        assert_eq!(
            stager.stage_section_batch(
                batch("ships", version, &format!("batch:{version}"), version),
                format!("bytes:{version}").as_bytes(),
                8,
                1,
                version + 1,
            ),
            expected
        );
    }
}
