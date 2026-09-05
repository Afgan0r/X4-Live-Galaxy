#![expect(
    clippy::expect_used,
    reason = "contract-test setup and mismatches must fail immediately"
)]

mod support;

use observation_application::{
    LifecycleContext, LifecycleError, LifecycleResult, ObservationLifecycle, ReconcileResult,
};
use observation_domain::SectionCoverage;
use observation_ingest::{DecisionRevisionIndex, ReceiverDisposition};
use support::flow::{input, limits, submit_empty, submit_section, submit_start_and_batch};
use support::repository_support::{AttemptLog, FirstPublish, RecordingRepository};
use support::{candidate_context, completion_bytes, current, repository, stager, start_bytes};

#[test]
fn decode_first_ship_tracer() {
    let (_database, repository) = repository("ship-tracer");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    let invalid = input(
        "outer:invalid",
        br#"{"type":"section_start"}"#.to_vec(),
        LifecycleContext::Start(candidate_context(SectionCoverage::Complete)),
        1,
    );
    assert!(lifecycle.submit(invalid).is_err());
    assert_eq!(
        lifecycle.submit(input(
            "outer:invalid",
            start_bytes("ships", 0),
            LifecycleContext::Start(candidate_context(SectionCoverage::KnownEmpty)),
            2,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
}

#[test]
fn ambiguous_attempt_reconciles_exact_request_without_reassembly() {
    let (_database, repository) = repository("ambiguous");
    let log = AttemptLog::default();
    let repository =
        RecordingRepository::new(repository, log.clone(), FirstPublish::CommitThenAmbiguous);
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    submit_start_and_batch(&mut lifecycle, "ships", &[("record:1", "ship:1")]);
    let completion = input(
        "outer:ships:complete",
        completion_bytes("ships", &[("record:1", "ship:1")], "complete"),
        LifecycleContext::Completion(current()),
        3,
    );
    assert_eq!(
        lifecycle.submit(completion),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::AmbiguousCommit
        ))
    );
    assert_eq!(
        lifecycle.submit(input(
            "outer:blocked",
            start_bytes("stations", 0),
            LifecycleContext::Start(candidate_context(SectionCoverage::KnownEmpty)),
            4,
        )),
        Err(LifecycleError::BlockedAmbiguous)
    );
    assert_eq!(
        lifecycle.reconcile_ambiguous(5),
        Ok(LifecycleResult::Reconciled(ReconcileResult::Committed))
    );
    assert_eq!(log.values().len(), 1);
}

#[test]
fn proven_not_committed_retries_the_exact_request() {
    let (_database, repository) = repository("retry");
    let log = AttemptLog::default();
    let repository =
        RecordingRepository::new(repository, log.clone(), FirstPublish::SkipThenAmbiguous);
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    submit_start_and_batch(&mut lifecycle, "ships", &[("record:1", "ship:1")]);
    assert_eq!(
        lifecycle.submit(input(
            "outer:ships:complete",
            completion_bytes("ships", &[("record:1", "ship:1")], "complete"),
            LifecycleContext::Completion(current()),
            3,
        )),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::AmbiguousCommit
        ))
    );
    assert_eq!(
        lifecycle.reconcile_ambiguous(4),
        Ok(LifecycleResult::Reconciled(
            ReconcileResult::ProvenNotCommitted
        ))
    );
    assert_eq!(
        lifecycle.retry_proven_not_committed(),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
    );
    let attempts = log.values();
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0], attempts[1]);
}

#[test]
fn ship_and_known_empty_station_traverse_same_lifecycle() {
    let (_database, repository) = repository("ship-station");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    submit_section(&mut lifecycle, "ships", &[("record:1", "ship:1")]);
    submit_empty(&mut lifecycle, "stations");
}

#[test]
fn adjacent_identities_survive_permuted_input_with_canonical_replay() {
    let forward = run_permutation(
        "permuted-forward",
        &[("record:2", "ship:2"), ("record:1", "ship:1")],
    );
    let reverse = run_permutation(
        "permuted-reverse",
        &[("record:1", "ship:1"), ("record:2", "ship:2")],
    );
    assert_eq!(forward, reverse);
}

fn run_permutation(
    label: &str,
    records: &[(&str, &str)],
) -> Vec<observation_persistence::PublishAttemptIdentity> {
    let (_database, repository) = repository(label);
    let log = AttemptLog::default();
    let repository = RecordingRepository::new(repository, log.clone(), FirstPublish::Normal);
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    submit_section(&mut lifecycle, "ships", records);
    assert_eq!(
        lifecycle.submit(input(
            "outer:ships:complete",
            completion_bytes("ships", records, "complete"),
            LifecycleContext::Completion(current()),
            3,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
    );
    assert_eq!(log.values().len(), 1);
    log.values()
}
