use std::cell::RefCell;
use std::rc::Rc;

use observation_application::PublicationReconciler;
use observation_domain::{DecisionSnapshotId, SectionKey};
use observation_ingest::DecisionRevisionSet;
use observation_persistence::{
    CurrentRevision, DecisionPinReceipt, DecisionRevisionPin, ObservationRepository,
    PublicationFailpoint, PublishAttemptIdentity, PublishOutcome, PublishRequest,
    ReconciliationOutcome, RepositoryDiagnostic, RepositoryError, SqliteObservationRepository,
    UnpinOutcome,
};

#[derive(Clone, Default)]
pub struct AttemptLog(Rc<RefCell<Vec<PublishAttemptIdentity>>>);

impl AttemptLog {
    #[must_use]
    pub fn values(&self) -> Vec<PublishAttemptIdentity> {
        self.0.borrow().clone()
    }
}

pub struct RecordingRepository {
    inner: SqliteObservationRepository,
    log: AttemptLog,
    first: FirstPublish,
}

#[derive(Clone, Copy)]
pub enum FirstPublish {
    Normal,
    CommitThenAmbiguous,
    SkipThenAmbiguous,
}

impl RecordingRepository {
    pub const fn new(
        inner: SqliteObservationRepository,
        log: AttemptLog,
        first: FirstPublish,
    ) -> Self {
        Self { inner, log, first }
    }
}

impl ObservationRepository for RecordingRepository {
    fn publish(&mut self, request: PublishRequest) -> PublishOutcome {
        self.log
            .0
            .borrow_mut()
            .push(request.attempt_identity().clone());
        let first = std::mem::replace(&mut self.first, FirstPublish::Normal);
        match first {
            FirstPublish::Normal => self.inner.publish(request),
            FirstPublish::CommitThenAmbiguous => self
                .inner
                .publish_with_failpoint(&request, PublicationFailpoint::AfterCommitBeforeResponse),
            FirstPublish::SkipThenAmbiguous => PublishOutcome::Ambiguous(RepositoryDiagnostic {
                code: "scripted-unknown",
            }),
        }
    }

    fn current(&self, key: &SectionKey) -> Result<Option<CurrentRevision>, RepositoryError> {
        self.inner.current(key)
    }

    fn pin_decision(
        &mut self,
        set: &DecisionRevisionSet,
    ) -> Result<DecisionPinReceipt, RepositoryError> {
        self.inner.pin_decision(set)
    }

    fn load_decision_pin(
        &self,
        decision: &DecisionSnapshotId,
    ) -> Result<DecisionRevisionPin, RepositoryError> {
        self.inner.load_decision_pin(decision)
    }

    fn unpin_decision(
        &mut self,
        receipt: &DecisionPinReceipt,
    ) -> Result<UnpinOutcome, RepositoryError> {
        self.inner.unpin_decision(receipt)
    }
}

impl PublicationReconciler for RecordingRepository {
    fn reconcile_publication(&mut self, request: &PublishRequest) -> ReconciliationOutcome {
        self.inner.reconcile_publication(request)
    }
}
