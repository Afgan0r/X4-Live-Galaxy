use observation_ingest::{
    AcceptedProjection, AdmissionOutcome, GenerationLimits, GenerationProgress,
    GenerationStager, admit_batch,
};

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

fn roomy_limits() -> GenerationLimits {
    GenerationLimits::new(1_000_000, 1_000).expect("test limits must be valid")
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
fn streamed_generation_commits_129_members_only_at_its_terminal_marker() {
    let initial = initially_accepted();
    let before = initial.snapshot().clone();
    let mut stager = GenerationStager::new(initial, roomy_limits());

    for index in 0..129_u64 {
        let frame = streamed_observation(
            &format!("sector:streamed_{index:03}"),
            2,
            7,
            index + 1,
        );
        assert_eq!(
            stager.stage_frame_at(&frame, 1_725_000_000_200),
            GenerationProgress::Staged
        );
        assert_eq!(stager.accepted().snapshot(), &before);
    }

    let marker = streamed_marker(2, 7, 130);
    assert_eq!(
        stager.stage_frame_at(&marker, 1_725_000_000_200),
        GenerationProgress::Admitted
    );
    assert_eq!(stager.accepted().snapshot().entity_ids().len(), 129);
    assert_eq!(stager.admitted_generation_count(), 1);
}

#[test]
fn streamed_generation_replay_is_idempotent() {
    let frame = streamed_observation("sector:streamed", 2, 7, 1);
    let marker = streamed_marker(2, 7, 2);
    let mut stager = GenerationStager::new(initially_accepted(), roomy_limits());

    assert_eq!(
        stager.stage_frame_at(&frame, 1_725_000_000_200),
        GenerationProgress::Staged
    );
    assert_eq!(
        stager.stage_frame_at(&marker, 1_725_000_000_200),
        GenerationProgress::Admitted
    );
    let once = stager.accepted().snapshot().clone();

    assert_eq!(
        stager.stage_frame_at(&frame, 1_725_000_000_200),
        GenerationProgress::Staged
    );
    assert_eq!(
        stager.stage_frame_at(&marker, 1_725_000_000_200),
        GenerationProgress::Replay
    );
    assert_eq!(stager.accepted().snapshot(), &once);
    assert_eq!(stager.admitted_generation_count(), 1);
}

#[test]
fn streamed_generation_drops_gaps_and_mixed_identity_without_partial_admission() {
    let initial = initially_accepted();
    let before = initial.snapshot().clone();
    let mut stager = GenerationStager::new(initial, roomy_limits());
    let first = streamed_observation("sector:streamed_a", 2, 7, 1);
    let gap = streamed_observation("sector:streamed_b", 2, 7, 3);

    assert_eq!(
        stager.stage_frame_at(&first, 1_725_000_000_200),
        GenerationProgress::Staged
    );
    assert!(matches!(
        stager.stage_frame_at(&gap, 1_725_000_000_200),
        GenerationProgress::Rejected(_)
    ));
    assert_eq!(stager.accepted().snapshot(), &before);

    let restart = streamed_observation("sector:streamed_a", 2, 8, 1);
    let mixed = streamed_marker(3, 8, 2);
    assert_eq!(
        stager.stage_frame_at(&restart, 1_725_000_000_200),
        GenerationProgress::Staged
    );
    assert!(matches!(
        stager.stage_frame_at(&mixed, 1_725_000_000_200),
        GenerationProgress::Rejected(_)
    ));
    assert_eq!(stager.accepted().snapshot(), &before);
    assert_eq!(stager.admitted_generation_count(), 0);
}

#[test]
fn streamed_generation_drops_exhausted_candidate_and_can_restart() {
    let initial = initially_accepted();
    let before = initial.snapshot().clone();
    let limits = GenerationLimits::new(2_048, 1).expect("test limits must be valid");
    let mut stager = GenerationStager::new(initial, limits);
    let first = streamed_observation("sector:streamed_a", 2, 7, 1);
    let second = streamed_observation("sector:streamed_b", 2, 7, 2);

    assert_eq!(
        stager.stage_frame_at(&first, 1_725_000_000_200),
        GenerationProgress::Staged
    );
    assert!(matches!(
        stager.stage_frame_at(&second, 1_725_000_000_200),
        GenerationProgress::Rejected(_)
    ));
    assert_eq!(stager.accepted().snapshot(), &before);

    let restart = streamed_observation("sector:streamed_c", 2, 8, 1);
    assert_eq!(
        stager.stage_frame_at(&restart, 1_725_000_000_200),
        GenerationProgress::Staged
    );
}
