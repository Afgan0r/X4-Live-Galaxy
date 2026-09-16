use super::{ProductionError, ProductionObservationSession};
use observation_domain::SectionKey;
use observation_persistence::ObservationRepository;

impl<R: ObservationRepository> ProductionObservationSession<R> {
    pub(super) fn validate_detail_dependency(
        &self,
        message: &observation_domain::CompleteMessage,
        now: u64,
    ) -> Result<(), ProductionError> {
        crate::receiver_ship_detail::validate_dependency(
            &self.lifecycle,
            message,
            self.heavy_limits
                .as_ref()
                .map_or(4, |limits| limits.group_members),
            self.heavy_limits
                .as_ref()
                .map_or(64, |limits| limits.max_inner_records),
            self.heavy_limits
                .as_ref()
                .map(|limits| (now, limits.freshness_millis as u64)),
        )
    }
    #[must_use]
    pub fn restore_current_snapshot(&mut self) -> bool {
        self.lifecycle.restore_current_snapshot()
    }
    pub fn configure_heavy(
        &mut self,
        limits: crate::HeavyShipLimits,
    ) -> Result<(), ProductionError> {
        if !limits.valid() {
            return Err(ProductionError::InvalidLimits);
        }
        self.heavy_limits = Some(limits);
        Ok(())
    }
    pub(crate) const fn heavy_limits(&self) -> Option<&crate::HeavyShipLimits> {
        self.heavy_limits.as_ref()
    }
    pub fn heavy_revision_floor(&self) -> Result<u64, ProductionError> {
        if self.heavy_limits.is_none() {
            return self.next_revision(
                &SectionKey::new(self.collection_key()).ok_or(ProductionError::InvalidLimits)?,
            );
        }
        // Scan actual durable section identities, not the resource envelope.
        // This also preserves old groups after a smaller replacement census.
        self.lifecycle
            .current_snapshot()
            .map_err(|_| ProductionError::Storage)?
            .iter()
            .filter(|v| {
                v.revision().section_key.as_str() == "ship_core"
                    || crate::receiver_ship_detail::is_key(v.revision().section_key.as_str())
            })
            .try_fold(1, |floor, v| {
                let next = v
                    .receipt()
                    .revision
                    .get()
                    .checked_add(1)
                    .filter(|n| *n <= observation_ingest::MAX_DURABLE_SECTION_REVISION)
                    .ok_or(ProductionError::RevisionExhausted)?;
                Ok(floor.max(next))
            })
    }
    pub(crate) fn next_ship_key(&self, previous: &str) -> Result<String, ProductionError> {
        if self.heavy_limits.is_none() {
            return Ok(self.collection_key().into());
        }
        let key = SectionKey::new("ship_core").ok_or(ProductionError::InvalidLimits)?;
        let parent = self
            .lifecycle
            .current_revision(&key)
            .map_err(|_| ProductionError::Storage)?
            .ok_or(ProductionError::InvalidLimits)?;
        let members = parent
            .revision()
            .records
            .iter()
            .map(|record| record.entity_id.as_str().to_owned())
            .collect::<Vec<_>>();
        crate::production_ship_cursor::next_member(&members, previous, None)
    }
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
