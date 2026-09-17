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
    pub(crate) fn stale_ship_parent(&self, now: u64) -> bool {
        let Some(limits) = self.heavy_limits.as_ref() else {
            return false;
        };
        let Some(key) = SectionKey::new("ship_core") else {
            return false;
        };
        self.lifecycle
            .current_revision(&key)
            .ok()
            .flatten()
            .is_some_and(|parent| {
                now.saturating_sub(parent.receipt().accepted_at) >= limits.freshness_millis as u64
            })
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
    pub(crate) fn next_ship_key(
        &self,
        previous: &str,
        now: u64,
    ) -> Result<String, ProductionError> {
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
        let snapshot = self
            .lifecycle
            .current_snapshot()
            .map_err(|_| ProductionError::Storage)?;
        let details = snapshot
            .iter()
            .filter(|value| {
                crate::receiver_ship_detail::is_key(value.revision().section_key.as_str())
                    && value.revision().source_scope == parent.revision().source_scope
            })
            .collect::<Vec<_>>();
        let last = details
            .iter()
            .max_by_key(|value| value.receipt().revision.get());
        let cursor = last.and_then(|value| durable_cursor(value));
        let next = crate::production_ship_cursor::next_member(&members, previous, cursor)?;
        let family = next.split_once(":g").map_or("", |value| value.0);
        let observed = family_duration(&details, family);
        let limits = self
            .heavy_limits
            .as_ref()
            .ok_or(ProductionError::InvalidLimits)?;
        let margin = limits.rate_interval_millis as u64;
        if crate::production_ship_cursor::refresh_required(
            now,
            parent.receipt().accepted_at,
            limits.freshness_millis as u64,
            observed,
            margin,
        ) {
            return Ok("ship_core".into());
        }
        Ok(next)
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

fn capture_duration(content: &str) -> Option<u64> {
    let field = |key: &str| {
        content
            .lines()
            .find_map(|line| line.strip_prefix(key))?
            .parse::<u64>()
            .ok()
    };
    field("capture_end=")?.checked_sub(field("capture_start=")?)
}

fn durable_cursor(value: &observation_persistence::CurrentRevision) -> Option<(&str, &str)> {
    let record = value.revision().records.first()?;
    let (family, _) = value.revision().section_key.as_str().split_once(":g")?;
    Some((record.entity_id.as_str(), family))
}

fn family_duration(details: &[&observation_persistence::CurrentRevision], family: &str) -> u64 {
    details
        .iter()
        .filter(|value| value.revision().section_key.as_str().starts_with(family))
        .filter_map(|value| value.revision().records.first())
        .filter_map(|record| capture_duration(&record.content))
        .max()
        .unwrap_or(0)
}
