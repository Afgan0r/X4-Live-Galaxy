use observation_ingest::{AcceptedPublication, FinalizationOutcome};
use observation_persistence::{
    PublishAttemptIdentity, PublishRequest, ReconciliationOutcome, SqliteObservationRepository,
};

use crate::{AttemptState, LifecycleLimits, ReconcileResult};

pub trait PublicationReconciler {
    fn reconcile_publication(&mut self, request: &PublishRequest) -> ReconciliationOutcome;
}

impl PublicationReconciler for SqliteObservationRepository {
    fn reconcile_publication(&mut self, request: &PublishRequest) -> ReconciliationOutcome {
        Self::reconcile_publication(self, request)
    }
}

#[must_use]
#[derive(Clone, Debug)]
pub struct RetainedPublicationAttempt {
    pub(crate) request: PublishRequest,
    pub(crate) identity: PublishAttemptIdentity,
    pub(crate) authority: AcceptedPublication,
    pub(crate) started_at: u64,
    pub(crate) retained_bytes: usize,
    pub(crate) reconcile_count: usize,
    pub(crate) state: AttemptState,
}

impl RetainedPublicationAttempt {
    pub(crate) fn new(
        request: PublishRequest,
        authority: AcceptedPublication,
        started_at: u64,
        retained_bytes: usize,
    ) -> Self {
        Self {
            identity: request.attempt_identity().clone(),
            request,
            authority,
            started_at,
            retained_bytes,
            reconcile_count: 0,
            state: AttemptState::Ambiguous,
        }
    }

    pub const fn state(&self) -> AttemptState {
        self.state
    }

    pub const fn identity(&self) -> &PublishAttemptIdentity {
        &self.identity
    }

    #[must_use]
    pub const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub(crate) const fn reconciliation_allowed(&self, limits: LifecycleLimits, now: u64) -> bool {
        self.reconcile_count < limits.reconcile_attempts.get()
            && now.saturating_sub(self.started_at) < limits.ambiguous_age_millis.get()
    }

    pub(crate) const fn record_reconciliation(&mut self) {
        self.reconcile_count = self.reconcile_count.saturating_add(1);
    }

    pub(crate) const fn finalized(outcome: FinalizationOutcome) -> bool {
        matches!(
            outcome,
            FinalizationOutcome::Finalized | FinalizationOutcome::AlreadyFinalized
        )
    }

    pub(crate) const fn reconciled(result: ReconcileResult) -> crate::LifecycleResult {
        crate::LifecycleResult::Reconciled(result)
    }
}
