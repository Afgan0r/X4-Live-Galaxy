mod support;

use observation_application::{
    LifecycleContext, LifecycleResult, ObservationLifecycle, ReconcileResult,
};
use observation_ingest::{
    DecisionEligibility, DecisionRevisionIndex, EligibilityBlocker, ReceiverDisposition,
};
use observation_persistence::{PublicationReceipt, ReconciliationOutcome};
use support::flow::{input, limits, submit_start_and_batch};
use support::repository_support::{AttemptLog, FirstPublish, RecordingRepository};
use support::{completion_bytes, current, key, repository, revision, stager};

#[test]
fn durable_replay_finalizes_with_the_original_receipt_time() {
    let (_database, repository) = repository("replay-receipt-time");
    let receipt = PublicationReceipt {
        section_key: key("ships"),
        revision: revision(1),
        content_digest: [7; 32],
        previous: None,
        ordinal: 1,
        accepted_at: 1,
    };
    let repository = RecordingRepository::new(
        repository,
        AttemptLog::default(),
        FirstPublish::SkipThenAmbiguous,
    )
    .with_reconciliation(ReconciliationOutcome::CommittedReplay(receipt));
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(2).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    submit_start_and_batch(&mut lifecycle, "ships", &[("record:1", "ship:1")]);
    assert_eq!(
        lifecycle.submit(input(
            "outer:ships:complete",
            completion_bytes("ships", &[("record:1", "ship:1")], "complete"),
            LifecycleContext::Completion(current()),
            102,
        )),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::AmbiguousCommit
        ))
    );
    assert_eq!(
        lifecycle.reconcile_ambiguous(103),
        Ok(LifecycleResult::Reconciled(ReconcileResult::Committed))
    );
    assert_eq!(
        lifecycle.decision_eligibility(&[key("ships")], 102, 100),
        DecisionEligibility::Blocked(vec![EligibilityBlocker::Stale(key("ships"))])
    );
}
