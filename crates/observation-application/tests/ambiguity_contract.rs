mod support;

use observation_application::{
    LifecycleContext, LifecycleError, LifecycleLimits, LifecycleResult, ObservationLifecycle,
    ReconcileResult,
};
use observation_domain::SectionCoverage;
use observation_ingest::{DecisionRevisionIndex, ReceiverDisposition};
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
            completion_bytes("ships", 1, "complete"),
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
