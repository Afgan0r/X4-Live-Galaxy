mod support;

use observation_application::{
    LifecycleContext, LifecycleError, LifecycleLimits, LifecycleResult, ObservationLifecycle,
    ReconcileResult,
};
use observation_domain::SectionCoverage;
use observation_ingest::{DecisionRevisionIndex, ReceiverDisposition};
use observation_persistence::{ReconciliationOutcome, RepositoryDiagnostic};
use support::flow::{input, submit_start_and_batch};
use support::repository_support::{AttemptLog, FirstPublish, RecordingRepository};
use support::{candidate_context, completion_bytes, current, repository, stager, start_bytes};

#[test]
fn ambiguity_limits_preserve_the_blocked_attempt() {
    let (_database, repository) = repository("bounded-ambiguity");
    let log = AttemptLog::default();
    let repository =
        RecordingRepository::new(repository, log.clone(), FirstPublish::CommitThenAmbiguous);
    let limits = LifecycleLimits::new(4_096, 16_384, 1, 1).expect("limits are non-zero");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits,
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
        lifecycle.reconcile_ambiguous(5),
        Ok(LifecycleResult::Reconciled(ReconcileResult::StillAmbiguous))
    );
    assert_eq!(
        lifecycle.submit(input(
            "outer:blocked",
            start_bytes("stations", 0),
            LifecycleContext::Start(candidate_context(SectionCoverage::KnownEmpty)),
            6,
        )),
        Err(LifecycleError::BlockedAmbiguous)
    );
    assert_eq!(log.values().len(), 1);
}

#[test]
fn superseded_ambiguity_releases_exact_replay_and_allows_turnover() {
    let (_database, repository) = repository("superseded-ambiguity");
    let log = AttemptLog::default();
    let repository =
        RecordingRepository::new(repository, log.clone(), FirstPublish::SkipThenAmbiguous)
            .with_reconciliation(ReconciliationOutcome::Superseded(RepositoryDiagnostic {
                code: "historical-revision",
            }));
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        LifecycleLimits::new(4_096, 16_384, 100, 4).expect("limits are non-zero"),
    );
    submit_start_and_batch(&mut lifecycle, "ships", &[("record:1", "ship:1")]);
    let completion = completion_bytes("ships", &[("record:1", "ship:1")], "complete");
    assert_eq!(
        lifecycle.submit(input(
            "outer:ships:complete",
            completion.clone(),
            LifecycleContext::Completion(current()),
            3,
        )),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::AmbiguousCommit
        ))
    );
    assert_eq!(
        lifecycle.reconcile_ambiguous(4),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::TimedOutOrSuperseded
        ))
    );
    assert_eq!(
        lifecycle.submit(input(
            "outer:ships:complete",
            completion,
            LifecycleContext::Completion(current()),
            5,
        )),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::TimedOutOrSuperseded
        ))
    );
    assert_eq!(
        lifecycle.submit(input(
            "outer:stations:start",
            start_bytes("stations", 0),
            LifecycleContext::Start(candidate_context(SectionCoverage::KnownEmpty)),
            6,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert_eq!(log.values().len(), 1);
}
