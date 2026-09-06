use std::path::Path;

use observation_application::{
    LifecycleError, LifecycleInput, LifecycleLimits, LifecycleResult, ObservationLifecycle,
    PublicationReconciler,
};
use observation_ingest::{
    AcceptedProjection, DecisionRevisionIndex, GenerationLimits, GenerationStager,
};
use observation_persistence::{
    ObservationRepository, PublicationLimits, SqliteObservationRepository,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductionError {
    InvalidLimits,
    Storage,
    Lifecycle(LifecycleError),
}

pub struct ProductionObservationSession<R = SqliteObservationRepository> {
    lifecycle: ObservationLifecycle<R>,
}

impl ProductionObservationSession {
    pub fn open(
        path: &Path,
        generation_limits: GenerationLimits,
        publication_limits: PublicationLimits,
        lifecycle_limits: LifecycleLimits,
        blocker_limit: usize,
    ) -> Result<Self, ProductionError> {
        let repository = SqliteObservationRepository::open(path, publication_limits)
            .map_err(|_| ProductionError::Storage)?;
        Self::from_repository(
            repository,
            generation_limits,
            lifecycle_limits,
            blocker_limit,
        )
    }
}

impl<R: ObservationRepository> ProductionObservationSession<R> {
    pub fn from_repository(
        repository: R,
        generation_limits: GenerationLimits,
        lifecycle_limits: LifecycleLimits,
        blocker_limit: usize,
    ) -> Result<Self, ProductionError> {
        let index =
            DecisionRevisionIndex::new(blocker_limit).ok_or(ProductionError::InvalidLimits)?;
        Ok(Self {
            lifecycle: ObservationLifecycle::new(
                GenerationStager::new(AcceptedProjection::empty(), generation_limits),
                index,
                repository,
                lifecycle_limits,
            ),
        })
    }

    pub fn submit(&mut self, input: LifecycleInput) -> Result<LifecycleResult, ProductionError> {
        self.lifecycle
            .submit(input)
            .map_err(ProductionError::Lifecycle)
    }

    #[must_use]
    pub fn restore_current_snapshot(&mut self) -> bool {
        self.lifecycle.restore_current_snapshot()
    }
}

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
