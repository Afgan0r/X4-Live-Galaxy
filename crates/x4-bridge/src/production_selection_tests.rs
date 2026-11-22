use super::ProductionObservationSession;
use observation_application::{LifecycleContext, LifecycleInput, LifecycleLimits, LifecycleResult};
use observation_domain::{
    BatchId, CanonicalizationVersion, CaptureWindow, DigestAlgorithmVersion, FactionOrigin,
    FactionOriginEvidence, ObservationPolicyVersion, ObservationSchemaVersion, SectionAvailability,
    SectionCoverage, SectionFreshness, SectionQuality, SectionState, TransportEpoch,
};
use observation_ingest::{
    AggregateLimits, CandidateContext, CandidateLimits, CompletionCurrent, ContractVersions,
    GenerationLimits, ReceiverDisposition,
};
use observation_persistence::PublicationLimits;

fn open() -> ProductionObservationSession {
    let candidate = CandidateLimits::new(4_096, 16, 16, 32, 100, 10).expect("candidate");
    let aggregate = AggregateLimits::new(4, 16_384, 64, 64, 128).expect("aggregate");
    ProductionObservationSession::open(
        std::path::Path::new(":memory:"),
        GenerationLimits::bounded(candidate, aggregate),
        PublicationLimits::new(16, 8_192).expect("publication"),
        LifecycleLimits::new(4_096, 16_384, 1_000, 4).expect("lifecycle"),
        4,
    )
    .expect("real isolated SQLite repository")
}

fn append(
    receiver: &mut ProductionObservationSession,
    bytes: &[u8],
    label: String,
    time: u64,
    context: LifecycleContext,
    expected: ReceiverDisposition,
) {
    let input = LifecycleInput::new(
        TransportEpoch::new(1).expect("epoch"),
        BatchId::new(label).expect("batch"),
        bytes.to_vec(),
        1,
        time,
        context,
    );
    assert_eq!(
        receiver.submit(input).expect("valid fixture input"),
        LifecycleResult::Disposition(expected),
    );
}

fn empty_context() -> CandidateContext {
    let window = CaptureWindow::new(10, 20).expect("window");
    CandidateContext::new(
        ContractVersions::new(
            ObservationSchemaVersion::new(1).expect("schema"),
            ObservationPolicyVersion::new(2).expect("policy"),
            CanonicalizationVersion::new(3).expect("canonicalization"),
            DigestAlgorithmVersion::new(1).expect("digest"),
        ),
        window,
        SectionState::with_evidence(
            window,
            SectionFreshness::Fresh,
            SectionQuality::Fresh,
            SectionAvailability::Available,
            SectionCoverage::KnownEmpty,
        ),
        std::collections::BTreeMap::new(),
        None,
        true,
    )
}

fn publish_empty(receiver: &mut ProductionObservationSession, key: &str, revision: u64) {
    let evidence = serde_json::json!({
        "capture_clock": "game_time_millis", "capture_start_millis": 0,
        "capture_end_millis": 0, "freshness": "fresh", "quality": "unknown",
        "availability": "available", "coverage": "complete", "source_epoch": null,
        "source_epoch_status": "unknown", "source_boundary": "unknown",
        "source_consistency": "unknown", "stable_identity": false,
        "schema_version": 1, "policy_version": 2, "canonicalization_version": 3,
        "digest_version": 1
    });
    let common = serde_json::json!({
        "contract_version": 2, "source_scope": "scope:x4",
        "producer_incarnation": "producer:1", "transport_epoch": 1,
        "section_key": key, "section_revision": revision, "sender_evidence": evidence
    });
    let mut start = common.clone();
    start["type"] = "section_start".into();
    start["expected_records"] = 0.into();
    append(
        receiver,
        start.to_string().as_bytes(),
        format!("start:{revision}"),
        10,
        LifecycleContext::Start(empty_context()),
        ReceiverDisposition::Received,
    );

    let mut completion = common;
    let empty_sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let fields = serde_json::json!({
        "type": "section_completion", "batch_count": 0, "record_count": 0,
        "raw_bytes": 0, "ordered_batch_manifest_digest": empty_sha256,
        "canonical_content_digest": empty_sha256, "schema_version": 1, "policy_version": 2,
        "canonicalization_version": 3, "digest_version": 1, "coverage": "known_empty"
    });
    for (field, value) in fields.as_object().expect("completion fields") {
        completion[field] = value.clone();
    }
    append(
        receiver,
        completion.to_string().as_bytes(),
        format!("complete:{revision}"),
        11,
        LifecycleContext::Completion(CompletionCurrent::new(
            std::collections::BTreeMap::new(),
            None,
        )),
        ReceiverDisposition::Committed,
    );
}

#[test]
fn successor_carries_floor_from_all_current_pointers() {
    let mut receiver = open();
    publish_empty(&mut receiver, "ship_core:argon", 7);
    publish_empty(&mut receiver, "faction_census", 14);
    publish_empty(&mut receiver, "economy.stations", 99);

    receiver
        .configure_heavy(
            crate::HeavyShipLimits::parse(include_str!(
                "../../../config/heavy-ship-experiment.json"
            ))
            .expect("heavy profile"),
        )
        .expect("configure");
    let origin = FactionOriginEvidence::new(
        "argon",
        FactionOrigin::Vanilla,
        Some(true),
        false,
        "test-inventory",
    )
    .expect("source evidence");
    receiver
        .accept_faction_census(1, ["argon"], &[origin])
        .expect("qualify the active scope");

    let next = receiver
        .next_ship_selection("ship_core:argon", 12)
        .expect("next selection");
    assert_eq!(next.key, "ship_core:argon");
    assert_eq!(next.revision_floor, Some(Ok(15)));
    assert_eq!(receiver.heavy_revision_floor(), Ok(15));
}
