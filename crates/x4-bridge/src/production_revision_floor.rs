use super::{ProductionError, ProductionObservationSession};
use observation_domain::{SectionKey, ShipSectionIdentity};
use observation_persistence::{CurrentRevision, ObservationRepository};

impl<R: ObservationRepository> ProductionObservationSession<R> {
    pub fn heavy_revision_floor(&self) -> Result<u64, ProductionError> {
        if self.heavy_limits.is_none() {
            return self.next_revision(
                &SectionKey::new(self.collection_key()).ok_or(ProductionError::InvalidLimits)?,
            );
        }
        let snapshot = self
            .lifecycle
            .current_snapshot()
            .map_err(|_| ProductionError::Storage)?;
        from_snapshot(&snapshot)
    }

    pub fn next_revision(&self, key: &SectionKey) -> Result<u64, ProductionError> {
        self.lifecycle
            .current_revision(key)
            .map_err(|_| ProductionError::Storage)?
            .as_ref()
            .map_or(Ok(1), revision_after)
    }
}

pub(super) fn from_snapshot(snapshot: &[CurrentRevision]) -> Result<u64, ProductionError> {
    // Include old durable groups that survive a smaller replacement census.
    snapshot
        .iter()
        .filter(|current| {
            let key = current.revision().section_key.as_str();
            key == "faction_census" || ShipSectionIdentity::parse(key).is_some()
        })
        .try_fold(1, |floor, current| {
            revision_after(current).map(|next| floor.max(next))
        })
}

fn revision_after(current: &CurrentRevision) -> Result<u64, ProductionError> {
    current
        .receipt()
        .revision
        .get()
        .checked_add(1)
        .filter(|next| *next <= observation_ingest::MAX_DURABLE_SECTION_REVISION)
        .ok_or(ProductionError::RevisionExhausted)
}
