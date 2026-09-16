use crate::{ProductionError, ProductionObservationSession};
use observation_persistence::{RetentionPolicy, RetentionReport};

impl ProductionObservationSession {
    pub fn maintain_heavy_history(&mut self) -> Result<RetentionReport, ProductionError> {
        let Some(limits) = &self.heavy_limits else {
            return Ok(RetentionReport {
                deleted_revisions: 0,
            });
        };
        let sections = limits
            .bridge
            .max_candidate_records
            .checked_mul(3)
            .and_then(|value| value.checked_add(1))
            .ok_or(ProductionError::InvalidLimits)?;
        let receipts = sections
            .checked_mul(limits.retained_revisions)
            .ok_or(ProductionError::InvalidLimits)?;
        let policy = RetentionPolicy::new(limits.retained_revisions, receipts)
            .ok_or(ProductionError::InvalidLimits)?;
        self.lifecycle
            .run_retention(policy)
            .map_err(|_| ProductionError::Storage)
    }
}
