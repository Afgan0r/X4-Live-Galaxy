use std::path::Path;

use observation_application::{
    LifecycleInput, LifecycleLimits, LifecycleResult, ObservationLifecycle,
};
use observation_ingest::{
    AcceptedProjection, DecisionRevisionIndex, GenerationLimits, GenerationStager,
};
use observation_persistence::{PublicationLimits, SqliteObservationRepository};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductionError {
    InvalidLimits,
    Storage,
    NotReady,
}

pub struct ProductionObservationSession {
    lifecycle: ObservationLifecycle<SqliteObservationRepository>,
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
        let index = DecisionRevisionIndex::new(blocker_limit)
            .ok_or(ProductionError::InvalidLimits)?;
        Ok(Self {
            lifecycle: ObservationLifecycle::new(
                GenerationStager::new(AcceptedProjection::empty(), generation_limits),
                index,
                repository,
                lifecycle_limits,
            ),
        })
    }

    pub fn submit(
        &mut self,
        _input: LifecycleInput,
    ) -> Result<LifecycleResult, ProductionError> {
        let _ = &self.lifecycle;
        Err(ProductionError::NotReady)
    }
}
