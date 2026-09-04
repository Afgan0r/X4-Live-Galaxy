#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "crash fixtures must fail immediately when setup or recovery is invalid"
)]

mod support;

use std::collections::BTreeMap;

use observation_persistence::{
    ObservationRepository, PublicationFailpoint, PublicationLimits, PublishOutcome, PublishRequest,
    ReconciliationOutcome, RetentionPolicy, SqliteObservationRepository,
};
use support::{TempDatabase, decision_set, key, revision, validated};

const fn limits() -> PublicationLimits {
    PublicationLimits::new(4, 256).expect("limits are non-zero")
}

fn precommit_cut(cut: PublicationFailpoint) {
    let database = TempDatabase::new("precommit-cut");
    let request = PublishRequest::from_revision(validated("ships", 1, None, BTreeMap::default()));
    let mut repository = SqliteObservationRepository::open(database.path(), limits()).unwrap();
    assert!(matches!(
        repository.publish_with_failpoint(&request, cut),
        PublishOutcome::PermanentRejection(_)
    ));
    drop(repository);
    let reopened = SqliteObservationRepository::open(database.path(), limits()).unwrap();
    assert_eq!(reopened.current(&key("ships")), Ok(None));
}

#[test]
fn every_precommit_cut_reopens_the_old_complete_state() {
    for cut in [
        PublicationFailpoint::BeforeContent,
        PublicationFailpoint::AfterContent,
        PublicationFailpoint::AfterReceipt,
        PublicationFailpoint::AfterPointer,
        PublicationFailpoint::BeforeCommit,
    ] {
        precommit_cut(cut);
    }
}

#[test]
fn ambiguous_commit_requires_reconciliation_and_replays_once() {
    for cut in [
        PublicationFailpoint::CommitResultUnknown,
        PublicationFailpoint::AfterCommitBeforeResponse,
    ] {
        let database = TempDatabase::new("ambiguous-cut");
        let request =
            PublishRequest::from_revision(validated("ships", 1, None, BTreeMap::default()));
        let mut repository = SqliteObservationRepository::open(database.path(), limits()).unwrap();
        assert!(matches!(
            repository.publish_with_failpoint(&request, cut),
            PublishOutcome::Ambiguous(_)
        ));
        assert!(matches!(
            repository.publish(request.clone()),
            PublishOutcome::Ambiguous(_)
        ));
        assert!(matches!(
            repository.reconcile_publication(&request),
            ReconciliationOutcome::CommittedReplay(_)
        ));
        drop(repository);
        let mut reopened = SqliteObservationRepository::open(database.path(), limits()).unwrap();
        let replay = reopened.publish(request);
        assert!(matches!(replay, PublishOutcome::CommittedReplay(receipt) if receipt.ordinal == 1));
    }
}

#[test]
fn reconciliation_proves_an_absent_attempt_retryable() {
    let database = TempDatabase::new("retryable");
    let request = PublishRequest::from_revision(validated("ships", 1, None, BTreeMap::default()));
    let mut repository = SqliteObservationRepository::open(database.path(), limits()).unwrap();
    assert_eq!(
        repository.reconcile_publication(&request),
        ReconciliationOutcome::ProvenNotCommitted
    );
    assert!(matches!(
        repository.publish(request),
        PublishOutcome::CommittedNew(_)
    ));
}

#[test]
fn retention_preserves_current_and_pinned_history_until_unpinned() {
    let database = TempDatabase::new("retention");
    let mut repository = SqliteObservationRepository::open(database.path(), limits()).unwrap();
    let first = validated("ships", 1, None, BTreeMap::default());
    assert!(matches!(
        repository.publish(PublishRequest::from_revision(first.clone())),
        PublishOutcome::CommittedNew(_)
    ));
    let pin = repository.pin_decision(&decision_set(vec![first])).unwrap();
    for value in 2..=4 {
        assert!(matches!(
            repository.publish(PublishRequest::from_revision(validated(
                "ships",
                value,
                Some(revision(value - 1)),
                BTreeMap::default(),
            ))),
            PublishOutcome::CommittedNew(_)
        ));
    }
    let policy = RetentionPolicy::new(1, 2).unwrap();
    assert_eq!(
        repository.run_retention(policy).unwrap().deleted_revisions,
        1
    );
    assert_eq!(
        repository.load_decision_pin(&pin.decision).unwrap().receipt,
        pin
    );
    assert!(repository.unpin_decision(&pin).is_ok());
    assert_eq!(
        repository.run_retention(policy).unwrap().deleted_revisions,
        1
    );
    assert_eq!(
        repository
            .current(&key("ships"))
            .unwrap()
            .unwrap()
            .receipt
            .revision,
        revision(4)
    );
}
