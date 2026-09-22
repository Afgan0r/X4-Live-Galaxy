use super::{ProductionError, ProductionObservationSession};
use observation_application::{LifecycleResult, PublicationReconciler};
use observation_persistence::ObservationRepository;

impl<R: ObservationRepository + PublicationReconciler> ProductionObservationSession<R> {
    pub fn reconcile_ambiguous(&mut self, now: u64) -> Result<LifecycleResult, ProductionError> {
        self.lifecycle
            .reconcile_ambiguous(now)
            .map_err(ProductionError::Lifecycle)
    }

    pub fn retry_proven_not_committed(&mut self) -> Result<LifecycleResult, ProductionError> {
        self.lifecycle
            .retry_proven_not_committed()
            .map_err(ProductionError::Lifecycle)
    }
}
