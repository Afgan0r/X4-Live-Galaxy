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
        let Some(limits) = &self.heavy_limits else {
            return self.next_revision(
                &SectionKey::new(self.collection_key()).ok_or(ProductionError::InvalidLimits)?,
            );
        };
        let mut floor = self
            .next_revision(&SectionKey::new("ship_core").ok_or(ProductionError::InvalidLimits)?)?;
        for (group, family) in (0..limits.bridge.max_candidate_records).flat_map(|group| {
            ["ship_cargo", "ship_crew", "ship_loadout"].map(|family| (group, family))
        }) {
            let key = SectionKey::new(format!("{family}:g{group}"))
                .ok_or(ProductionError::InvalidLimits)?;
            floor = floor.max(self.next_revision(&key)?);
        }
        Ok(floor)
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
        if previous == "ship_core" {
            return Ok("ship_cargo:g0".into());
        }
        let (family, group) = previous
            .split_once(":g")
            .ok_or(ProductionError::InvalidLimits)?;
        let group = group
            .parse::<usize>()
            .map_err(|_| ProductionError::InvalidLimits)?;
        match family {
            "ship_cargo" => Ok(format!("ship_crew:g{group}")),
            "ship_crew" => Ok(format!("ship_loadout:g{group}")),
            "ship_loadout" if group + 1 < parent.revision().records.len() => {
                Ok(format!("ship_cargo:g{}", group + 1))
            }
            "ship_loadout" => Ok("ship_core".into()),
            _ => Err(ProductionError::InvalidLimits),
        }
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
