use crate::ObservationLifecycle;
use observation_persistence::{
    RepositoryError, RetentionPolicy, RetentionReport, SqliteObservationRepository,
};

impl ObservationLifecycle<SqliteObservationRepository> {
    pub fn run_retention(
        &mut self,
        policy: RetentionPolicy,
    ) -> Result<RetentionReport, RepositoryError> {
        // A retained ambiguous publication must remain available for reconciliation.
        if self.retained.is_some() {
            return Ok(RetentionReport {
                deleted_revisions: 0,
            });
        }
        self.repository.run_retention(policy)
    }
}
