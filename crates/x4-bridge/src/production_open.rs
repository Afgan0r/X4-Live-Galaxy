use std::path::Path;

use observation_application::LifecycleLimits;
use observation_ingest::GenerationLimits;
use observation_persistence::{PublicationLimits, SqliteObservationRepository};

use super::{ProductionError, ProductionObservationSession};

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
