use observation_ingest::{AcceptedProjection, AdmissionOutcome, admit_batch};

fn observation(entity_id: &str, _game_time: u64, version: u64) -> String {
    let asset_id = format!("asset:ship:{version}:{entity_id}");
    format!(
        r#"{{
    "type": "observation",
    "scope": "runtime:sectors",
    "entity_id": "{entity_id}",
    "version": {version},
    "quality": "fresh",
    "runtime_facts": {{
        "r": "x4_runtime",
        "g": 42,
        "q": "fresh",
        "a": "available",
        "s": [{{"i": "{entity_id}"}}],
        "x": [{{"i": "{asset_id}", "p": "{entity_id}"}}],
        "c": [{{"i": "capacity:storage:{version}:{entity_id}", "p": "{asset_id}", "v": 42}}],
        "o": [{{"i": "ownership:ship:{version}:{entity_id}", "p": "{asset_id}", "n": "faction:argon"}}]
    }}
}}"#
    )
}

const COMPLETE_V1: &str = r#"{
    "type": "complete_marker",
    "scope": "runtime:sectors",
    "version": 1
}"#;

const COMPLETE_V2: &str = r#"{
    "type": "complete_marker",
    "scope": "runtime:sectors",
    "version": 2
}"#;

fn initially_accepted() -> AcceptedProjection {
    let initial_alpha = observation("sector:alpha", 1_725_000_000_000, 1);
    let initial_beta = observation("sector:beta", 1_725_000_000_000, 1);
    let outcome = admit_batch(
        AcceptedProjection::empty(),
        &[&initial_alpha, &initial_beta, COMPLETE_V1],
    );
    assert!(
        matches!(outcome, AdmissionOutcome::Accepted(_)),
        "initial complete scope must be accepted: {outcome:?}"
    );
    outcome.into_projection()
}

#[test]
fn complete_scope_reconciles_only_members_observed_in_its_batch() {
    let initial = initially_accepted();
    let updated_alpha = observation("sector:alpha", 1_725_000_000_100, 2);

    let outcome = admit_batch(initial, &[&updated_alpha, COMPLETE_V2]);

    assert!(matches!(outcome, AdmissionOutcome::Accepted(_)));
    assert_eq!(outcome.snapshot().entity_ids(), vec!["sector:alpha"]);
}

#[test]
fn invalid_completion_marker_rolls_back_every_candidate_change() {
    let initial = initially_accepted();
    let before = initial.snapshot().clone();
    let updated_alpha = observation("sector:alpha", 1_725_000_000_100, 2);
    let invalid_marker = r#"{
        "type": "complete_marker",
        "scope": "runtime:sectors",
        "version": 0
    }"#;

    let outcome = admit_batch(initial, &[&updated_alpha, invalid_marker]);

    assert!(matches!(outcome, AdmissionOutcome::Rejected { .. }));
    assert_eq!(outcome.snapshot(), &before);
}

#[test]
fn replaying_a_complete_batch_is_idempotent() {
    let initial = initially_accepted();
    let updated_alpha = observation("sector:alpha", 1_725_000_000_100, 2);
    let first = admit_batch(initial, &[&updated_alpha, COMPLETE_V2]).into_projection();
    let snapshot = first.snapshot().clone();

    let replay = admit_batch(first, &[&updated_alpha, COMPLETE_V2]);

    assert!(matches!(replay, AdmissionOutcome::Accepted(_)));
    assert_eq!(replay.snapshot(), &snapshot);
}

#[test]
fn stale_marker_only_batch_preserves_the_completed_snapshot() {
    let initial = initially_accepted();
    let updated_alpha = observation("sector:alpha", 1_725_000_000_100, 2);
    let current = admit_batch(initial, &[&updated_alpha, COMPLETE_V2]).into_projection();
    let before = current.snapshot().clone();

    let outcome = admit_batch(current, &[COMPLETE_V1]);

    assert!(matches!(outcome, AdmissionOutcome::Rejected { .. }));
    assert_eq!(outcome.snapshot(), &before);
}

#[test]
fn mixed_version_marker_batch_preserves_the_accepted_snapshot() {
    let initial = initially_accepted();
    let before = initial.snapshot().clone();
    let updated_alpha = observation("sector:alpha", 1_725_000_000_100, 2);
    let initial_beta = observation("sector:beta", 1_725_000_000_000, 1);

    let outcome = admit_batch(initial, &[&updated_alpha, &initial_beta, COMPLETE_V2]);

    assert!(matches!(outcome, AdmissionOutcome::Rejected { .. }));
    assert_eq!(outcome.snapshot(), &before);
}

#[test]
fn equal_version_marker_only_batch_preserves_the_completed_snapshot() {
    let updated_alpha = observation("sector:alpha", 1_725_000_000_100, 2);
    let completed =
        admit_batch(initially_accepted(), &[&updated_alpha, COMPLETE_V2]).into_projection();
    let before = completed.snapshot().clone();

    let outcome = admit_batch(completed, &[COMPLETE_V2]);

    assert!(matches!(outcome, AdmissionOutcome::Rejected { .. }));
    assert_eq!(outcome.snapshot(), &before);
}

#[test]
fn equal_version_changed_membership_preserves_the_completed_snapshot() {
    let updated_alpha = observation("sector:alpha", 1_725_000_000_100, 2);
    let added_gamma = observation("sector:gamma", 1_725_000_000_100, 2);
    let completed =
        admit_batch(initially_accepted(), &[&updated_alpha, COMPLETE_V2]).into_projection();
    let before = completed.snapshot().clone();

    let outcome = admit_batch(completed, &[&updated_alpha, &added_gamma, COMPLETE_V2]);

    assert!(matches!(outcome, AdmissionOutcome::Rejected { .. }));
    assert_eq!(outcome.snapshot(), &before);
}
