use observation_persistence::{CurrentRevision, ObservationRepository, RepositoryError};

use crate::ObservationLifecycle;

impl<R: ObservationRepository> ObservationLifecycle<R> {
    pub const fn complete_message_limit(&self) -> usize {
        self.limits.complete_message_bytes.get()
    }

    pub fn current_revision(
        &self,
        key: &observation_domain::SectionKey,
    ) -> Result<Option<CurrentRevision>, RepositoryError> {
        self.repository.current(key)
    }

    pub fn current_snapshot(&self) -> Result<Vec<CurrentRevision>, RepositoryError> {
        self.repository.current_snapshot()
    }

    #[must_use]
    pub fn authoritative_source_session(
        &self,
        scope: &observation_domain::SourceScopeId,
    ) -> Option<&observation_domain::SourceSessionIdentity> {
        self.index.authoritative_source_session(scope)
    }
}
