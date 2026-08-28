use observation_ingest::{
    AcceptedProjection, AdmissionError, AdmissionOutcome, MAX_REJECTION_EVIDENCE, RejectionReason,
    admit_batch,
};

fn accepted_projection() -> AcceptedProjection {
    let payload = r#"{
        "type": "observation",
        "scope": "runtime:sectors",
        "entity_id": "sector:alpha",
        "observed_at_unix_millis": 1725000000000,
        "version": 2,
        "quality": "fresh",
        "content": "accepted"
    }"#;
    let marker = r#"{
        "type": "complete_marker",
        "scope": "runtime:sectors",
        "version": 2
    }"#;

    match admit_batch(AcceptedProjection::empty(), &[payload, marker]) {
        AdmissionOutcome::Accepted(projection) => projection,
        outcome @ AdmissionOutcome::Rejected { .. } => {
            panic!("fixture must create an accepted projection: {outcome:?}")
        }
    }
}

#[test]
fn hostile_fixture_frames_preserve_the_last_accepted_projection() {
    let accepted = accepted_projection();
    let accepted_snapshot = accepted.snapshot().clone();
    let malformed = include_str!("../../../tests/fixtures/malformed-envelope.json");
    let oversized = include_str!("../../../tests/fixtures/oversized-envelope.json");
    let reordered = include_str!("../../../tests/fixtures/reordered-duplicate-sequence.json");

    for hostile_batch in [
        vec![malformed],
        vec![oversized],
        vec![reordered],
        vec![
            r#"{
            "type": "observation",
            "scope": "runtime:sectors",
            "entity_id": "sector:alpha",
            "observed_at_unix_millis": 1725000000000,
            "version": 1,
            "quality": "fresh",
            "content": "stale"
        }"#,
        ],
        vec![
            r#"{
            "type": "observation",
            "scope": "runtime:sectors",
            "entity_id": "sector:alpha",
            "observed_at_unix_millis": 1725000000000,
            "version": 2,
            "quality": "fresh",
            "content": "equal-version-conflict"
        }"#,
        ],
    ] {
        let outcome = admit_batch(accepted.clone(), &hostile_batch);
        assert_eq!(outcome.snapshot(), &accepted_snapshot);
        assert!(matches!(outcome, AdmissionOutcome::Rejected { .. }));
    }
}

#[test]
fn exact_duplicate_replay_is_idempotent_without_replacing_the_snapshot() {
    let accepted = accepted_projection();
    let accepted_snapshot = accepted.snapshot().clone();
    let replay = r#"{
        "type": "observation",
        "scope": "runtime:sectors",
        "entity_id": "sector:alpha",
        "observed_at_unix_millis": 1725000000000,
        "version": 2,
        "quality": "fresh",
        "content": "accepted"
    }"#;

    let outcome = admit_batch(accepted, &[replay]);

    assert!(matches!(outcome, AdmissionOutcome::Accepted(_)));
    assert_eq!(outcome.snapshot(), &accepted_snapshot);
    assert_eq!(outcome.rejection_reason(), None);
}

#[test]
fn rejection_evidence_is_bounded_and_never_retains_raw_payloads() {
    let mut accepted = accepted_projection();
    let malformed = include_str!("../../../tests/fixtures/malformed-envelope.json");

    for _ in 0..=MAX_REJECTION_EVIDENCE {
        let outcome = admit_batch(accepted, &[malformed]);
        assert_eq!(
            outcome.rejection_reason(),
            Some(RejectionReason::MalformedFrame)
        );
        accepted = outcome.into_projection();
    }

    assert_eq!(accepted.rejection_evidence().len(), MAX_REJECTION_EVIDENCE);
    assert!(
        accepted
            .rejection_evidence()
            .iter()
            .all(|evidence| evidence.reason() == RejectionReason::MalformedFrame)
    );
}

#[test]
fn validate_batch_rejects_duplicate_and_out_of_order_sequences() {
    let accepted = accepted_projection();
    let reordered = include_str!("../../../tests/fixtures/reordered-duplicate-sequence.json");

    assert_eq!(
        observation_ingest::validate_batch(&accepted, &[reordered]),
        Err(AdmissionError::OutOfOrderVersion)
    );
}
