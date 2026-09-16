use super::{ProductionError, ProductionObservationSession};
use observation_domain::SectionKey;
use observation_persistence::ObservationRepository;

impl<R: ObservationRepository> ProductionObservationSession<R> {
    pub fn next_revision(&self, key: &SectionKey) -> Result<u64, ProductionError> {
        let current = self
            .lifecycle
            .current_revision(key)
            .map_err(|_| ProductionError::Storage)?;
        current.map_or(Ok(1), |value| {
            let next = value
                .revision()
                .revision
                .get()
                .checked_add(1)
                .ok_or(ProductionError::RevisionExhausted)?;
            (next <= observation_ingest::MAX_DURABLE_SECTION_REVISION)
                .then_some(next)
                .ok_or(ProductionError::RevisionExhausted)
        })
    }

    pub fn select_ship_core(&mut self, faction: &str) -> Result<(), ProductionError> {
        self.ship_scope = Some(crate::receiver_ship::scope(faction)?);
        self.last_received = None;
        Ok(())
    }

    pub const fn collection_key(&self) -> &'static str {
        if self.ship_scope.is_some() {
            "ship_core"
        } else {
            "carrier_b_realtime_sample"
        }
    }
}
