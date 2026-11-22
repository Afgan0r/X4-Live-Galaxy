use super::{ProductionError, ProductionObservationSession};
use observation_domain::{SectionKey, SectionRevisionId, ShipSectionIdentity};
use observation_persistence::ObservationRepository;

impl<R: ObservationRepository> ProductionObservationSession<R> {
    pub fn heavy_revision_floor(&self) -> Result<u64, ProductionError> {
        if self.heavy_limits.is_none() {
            return self.next_revision(
                &SectionKey::new(self.collection_key()).ok_or(ProductionError::InvalidLimits)?,
            );
        }
        let pointers = self
            .lifecycle
            .current_pointers()
            .map_err(|_| ProductionError::Storage)?;
        from_pointers(&pointers)
    }

    pub fn next_revision(&self, key: &SectionKey) -> Result<u64, ProductionError> {
        self.lifecycle
            .current_revision(key)
            .map_err(|_| ProductionError::Storage)?
            .as_ref()
            .map_or(Ok(1), |value| revision_after(value.receipt().revision))
    }
}

pub(super) fn from_pointers(
    pointers: &[(SectionKey, SectionRevisionId)],
) -> Result<u64, ProductionError> {
    // Include old durable groups that survive a smaller replacement census.
    pointers
        .iter()
        .filter(|(key, _)| {
            key.as_str() == "faction_census" || ShipSectionIdentity::parse(key.as_str()).is_some()
        })
        .try_fold(1, |floor, (_, revision)| {
            revision_after(*revision).map(|next| floor.max(next))
        })
}

fn revision_after(revision: SectionRevisionId) -> Result<u64, ProductionError> {
    revision
        .get()
        .checked_add(1)
        .filter(|next| *next <= observation_ingest::MAX_DURABLE_SECTION_REVISION)
        .ok_or(ProductionError::RevisionExhausted)
}
