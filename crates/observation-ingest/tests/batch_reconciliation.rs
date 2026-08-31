use observation_ingest::{
    AcceptedProjection, AdmissionOutcome, GenerationLimits, GenerationProgress,
    GenerationStager as Stager, admit_batch,
};
const RECEIPT: u64 = 1_725_000_000_200;
const fn rejected(progress: GenerationProgress) -> bool {
    matches!(progress, GenerationProgress::Rejected(_))
}
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
fn streamed_observation(entity_id: &str, version: u64, generation: u64, sequence: u64) -> String {
    let asset_id = format!("asset:station:{entity_id}");
    format!(
        r#"{{"type":"observation","scope":"runtime:sectors","entity_id":"{entity_id}","version":{version},"quality":"fresh","runtime_facts":{{"r":"x4_runtime","g":42,"q":"fresh","a":"available","s":[{{"i":"{entity_id}"}}],"x":[{{"i":"{asset_id}","p":"{entity_id}"}}],"c":[{{"i":"capacity:station:{entity_id}","p":"{asset_id}","v":42}}],"o":[{{"i":"ownership:station:{entity_id}","p":"{asset_id}","n":"faction:argon"}}]}},"generation":{generation},"sequence":{sequence}}}"#
    )
}
fn streamed_marker(version: u64, generation: u64, sequence: u64) -> String {
    format!(
        r#"{{"type":"complete_marker","scope":"runtime:sectors","version":{version},"generation":{generation},"sequence":{sequence}}}"#
    )
}
const fn roomy_limits() -> GenerationLimits {
    GenerationLimits::new(1_000_000, 1_000)
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
#[test]
fn streamed_generation_commits_129_members_once_at_its_terminal_marker() {
    let mut stager = Stager::new(initially_accepted(), roomy_limits());
    let before = stager.accepted().snapshot().clone();
    for index in 0..129_u64 {
        let id = format!("sector:streamed_{index:03}");
        let frame = streamed_observation(&id, 2, 7, index + 1);
        assert_eq!(
            stager.stage_frame_at(&frame, RECEIPT),
            GenerationProgress::Staged
        );
        assert_eq!(stager.accepted().snapshot(), &before);
    }
    let marker = streamed_marker(2, 7, 130);
    assert_eq!(
        stager.stage_frame_at(&marker, RECEIPT),
        GenerationProgress::Admitted
    );
    assert_eq!(stager.accepted().snapshot().entity_ids().len(), 129);
    assert_eq!(stager.admitted_generation_count(), 1);
    let once = stager.accepted().snapshot().clone();
    stager = Stager::resume(stager.accepted().clone(), roomy_limits(), 7);
    for index in 0..129_u64 {
        let id = format!("sector:streamed_{index:03}");
        let frame = streamed_observation(&id, 2, 7, index + 1);
        let _ = stager.stage_frame_at(&frame, RECEIPT);
    }
    assert_eq!(
        stager.stage_frame_at(&marker, RECEIPT),
        GenerationProgress::Replay
    );
    assert_eq!(stager.accepted().snapshot(), &once);
    let stale = streamed_observation("sector:stale", 3, 6, 1);
    assert!(rejected(stager.stage_frame_at(&stale, RECEIPT)));
    assert_eq!(stager.accepted().snapshot(), &once);
}
#[test]
fn streamed_generation_drops_invalid_or_exhausted_candidates() {
    let mut stager = Stager::new(initially_accepted(), roomy_limits());
    let before = stager.accepted().snapshot().clone();
    let first = streamed_observation("sector:streamed_a", 2, 7, 1);
    let gap = streamed_observation("sector:streamed_b", 2, 7, 3);
    let _ = stager.stage_frame_at(&first, RECEIPT);
    assert!(rejected(stager.stage_frame_at(&gap, RECEIPT)));
    assert_eq!(stager.accepted().snapshot(), &before);
    let restart = streamed_observation("sector:streamed_a", 2, 8, 1);
    let mixed = streamed_marker(3, 8, 2);
    let _ = stager.stage_frame_at(&restart, RECEIPT);
    assert!(rejected(stager.stage_frame_at(&mixed, RECEIPT)));
    assert_eq!(stager.accepted().snapshot(), &before);
    let invalid = streamed_observation("sector:invalid", 2, 9, 1).replace("\"v\":42", "\"v\":-1");
    assert!(rejected(stager.stage_frame_at(&invalid, RECEIPT)));
    assert_eq!(stager.accepted().snapshot(), &before);
    let mut stager = Stager::new(stager.accepted().clone(), GenerationLimits::new(2_048, 1));
    let first = streamed_observation("sector:streamed_a", 2, 7, 1);
    let second = streamed_observation("sector:streamed_b", 2, 7, 2);
    let _ = stager.stage_frame_at(&first, RECEIPT);
    assert!(rejected(stager.stage_frame_at(&second, RECEIPT)));
    assert_eq!(stager.accepted().snapshot(), &before);
}
