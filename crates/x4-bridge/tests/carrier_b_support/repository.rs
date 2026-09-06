use observation_application::PublicationReconciler;
use observation_domain::{DecisionSnapshotId, SectionKey};
use observation_ingest::DecisionRevisionSet;
use observation_persistence::{
    CurrentRevision, DecisionPinReceipt, DecisionRevisionPin, ObservationRepository,
    PublishOutcome, PublishRequest, ReconciliationOutcome, RepositoryDiagnostic, RepositoryError,
    SqliteObservationRepository, UnpinOutcome,
};

#[derive(Clone, Copy)]
pub enum FirstPublish {
    CommitThenAmbiguous,
    SkipThenAmbiguous,
}

pub struct AmbiguousRepository {
    inner: SqliteObservationRepository,
    first: Option<FirstPublish>,
}

impl AmbiguousRepository {
    pub const fn new(inner: SqliteObservationRepository, first: FirstPublish) -> Self {
        Self {
            inner,
            first: Some(first),
        }
    }
}

impl ObservationRepository for AmbiguousRepository {
    fn publish(&mut self, request: PublishRequest) -> PublishOutcome {
        let Some(first) = self.first.take() else {
            return self.inner.publish(request);
        };
        let outcome = match first {
            FirstPublish::CommitThenAmbiguous => Some(self.inner.publish(request)),
            FirstPublish::SkipThenAmbiguous => None,
        };
        if outcome.as_ref().is_some_and(|outcome| {
            !matches!(
                outcome,
                PublishOutcome::CommittedNew(_) | PublishOutcome::CommittedReplay(_)
            )
        }) {
            return outcome.expect("checked present outcome");
        }
        PublishOutcome::Ambiguous(RepositoryDiagnostic {
            code: "injected-response-loss",
        })
    }

    fn current(&self, key: &SectionKey) -> Result<Option<CurrentRevision>, RepositoryError> {
        self.inner.current(key)
    }

    fn current_snapshot(&self) -> Result<Vec<CurrentRevision>, RepositoryError> {
        self.inner.current_snapshot()
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

impl PublicationReconciler for AmbiguousRepository {
    fn reconcile_publication(&mut self, request: &PublishRequest) -> ReconciliationOutcome {
        self.inner.reconcile_publication(request)
    }
}
