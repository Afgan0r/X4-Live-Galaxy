use observation_ingest::{AcceptedProjection, AdmissionOutcome, admit_batch};

const INITIAL_ALPHA: &str = r#"{
    "type": "observation",
    "scope": "runtime:sectors",
    "entity_id": "sector:alpha",
    "observed_at_unix_millis": 1725000000000,
    "version": 1,
    "quality": "fresh",
    "content": "initial-alpha"
}"#;

const INITIAL_BETA: &str = r#"{
    "type": "observation",
    "scope": "runtime:sectors",
    "entity_id": "sector:beta",
    "observed_at_unix_millis": 1725000000000,
    "version": 1,
    "quality": "fresh",
    "content": "initial-beta"
}"#;

const COMPLETE_V1: &str = r#"{
    "type": "complete_marker",
    "scope": "runtime:sectors",
    "version": 1
}"#;

const UPDATED_ALPHA: &str = r#"{
    "type": "observation",
    "scope": "runtime:sectors",
    "entity_id": "sector:alpha",
    "observed_at_unix_millis": 1725000000100,
    "version": 2,
    "quality": "fresh",
    "content": "updated-alpha"
}"#;

const COMPLETE_V2: &str = r#"{
    "type": "complete_marker",
    "scope": "runtime:sectors",
    "version": 2
}"#;

fn initially_accepted() -> AcceptedProjection {
    match admit_batch(
        AcceptedProjection::empty(),
        &[INITIAL_ALPHA, INITIAL_BETA, COMPLETE_V1],
    ) {
        AdmissionOutcome::Accepted(projection) => projection,
        outcome => panic!("initial complete scope must be accepted: {outcome:?}"),
    }
}

#[test]
fn complete_scope_reconciles_only_members_observed_in_its_batch() {
    let initial = initially_accepted();

    let outcome = admit_batch(initial, &[UPDATED_ALPHA, COMPLETE_V2]);

    assert!(matches!(outcome, AdmissionOutcome::Accepted(_)));
    assert_eq!(outcome.snapshot().entity_ids(), vec!["sector:alpha"]);
}

#[test]
fn invalid_completion_marker_rolls_back_every_candidate_change() {
    let initial = initially_accepted();
    let before = initial.snapshot().clone();
    let invalid_marker = r#"{
        "type": "complete_marker",
        "scope": "runtime:sectors",
        "version": 0
    }"#;

    let outcome = admit_batch(initial, &[UPDATED_ALPHA, invalid_marker]);

    assert!(matches!(outcome, AdmissionOutcome::Rejected { .. }));
    assert_eq!(outcome.snapshot(), &before);
}

#[test]
fn replaying_a_complete_batch_is_idempotent() {
    let initial = initially_accepted();
    let first = admit_batch(initial, &[UPDATED_ALPHA, COMPLETE_V2]).into_projection();
    let snapshot = first.snapshot().clone();

    let replay = admit_batch(first, &[UPDATED_ALPHA, COMPLETE_V2]);

    assert!(matches!(replay, AdmissionOutcome::Accepted(_)));
    assert_eq!(replay.snapshot(), &snapshot);
}
