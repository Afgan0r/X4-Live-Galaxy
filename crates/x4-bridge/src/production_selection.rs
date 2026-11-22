use super::{ProductionError, ProductionObservationSession};
use observation_domain::SectionKey;
use observation_persistence::{CurrentRevision, ObservationRepository};
#[cfg(test)]
#[path = "production_selection_tests.rs"]
mod selection_tests;
#[cfg(test)]
#[path = "production_ship_timing_tests.rs"]
mod timing_tests;

pub struct NextShipSelection {
    pub key: String,
    // An exhausted revision is reported only after the schedule admits the key.
    pub revision_floor: Option<Result<u64, ProductionError>>,
}

impl<R: ObservationRepository> ProductionObservationSession<R> {
    pub(crate) fn ship_receipt_completed(
        &mut self,
        message: &observation_domain::CompleteMessage,
        now: u64,
    ) {
        self.ship_timing.completed(message, now);
    }
    pub(crate) fn ship_intent_issued(
        &mut self,
        identity: &observation_ingest::CarrierIdentity,
        key: &str,
        revision: u64,
        now: u64,
    ) {
        if let Some(scope) = &self.ship_scope {
            self.ship_timing.issued(identity, key, revision, now, scope);
        }
    }
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
        self.faction_census_mode = true;
        Ok(())
    }
    pub(crate) const fn heavy_limits(&self) -> Option<&crate::HeavyShipLimits> {
        self.heavy_limits.as_ref()
    }
    pub(crate) fn stale_ship_parent(&self, now: u64) -> bool {
        let Some(limits) = self.heavy_limits.as_ref() else {
            return false;
        };
        let Some(key) = SectionKey::new(self.collection_key()) else {
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
    pub fn next_ship_key(&mut self, previous: &str, now: u64) -> Result<String, ProductionError> {
        self.next_ship_selection(previous, now)
            .map(|selection| selection.key)
    }

    pub(crate) fn next_ship_selection(
        &mut self,
        previous: &str,
        now: u64,
    ) -> Result<NextShipSelection, ProductionError> {
        if self.heavy_limits.is_none() || previous == "faction_census" {
            return Ok(NextShipSelection {
                key: self.collection_key(),
                revision_floor: None,
            });
        }
        let key = SectionKey::new(self.collection_key()).ok_or(ProductionError::InvalidLimits)?;
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
        let pointers = self
            .lifecycle
            .current_pointers()
            .map_err(|_| ProductionError::Storage)?;
        let detail = self.latest_current_detail(&parent)?;
        let cursor = detail
            .as_ref()
            .and_then(crate::production_ship_cursor::durable_cursor);
        let limits = self
            .heavy_limits
            .as_ref()
            .ok_or(ProductionError::InvalidLimits)?;
        let previous_legacy = crate::production_ship_cursor::legacy_key(previous)
            .ok_or(ProductionError::InvalidLimits)?;
        let next = crate::production_ship_cursor::next_member(
            &members,
            &previous_legacy,
            cursor,
            limits.group_members,
        )?;
        let observed = self.ship_timing.observed(&next);
        let margin = limits.rate_interval_millis as u64;
        let revision_floor = Some(super::revision_floor::from_pointers(&pointers));
        if crate::production_ship_cursor::refresh_required(
            now,
            parent.receipt().accepted_at,
            limits.freshness_millis as u64,
            observed,
            margin,
        ) {
            return Ok(NextShipSelection {
                key: self.collection_key(),
                revision_floor,
            });
        }
        if next == "ship_core" && (previous_legacy != "ship_core" || members.is_empty()) {
            self.rotate_ship_scope();
        }
        Ok(NextShipSelection {
            key: self.scoped_key(&next)?,
            revision_floor,
        })
    }

    fn latest_current_detail(
        &self,
        parent: &CurrentRevision,
    ) -> Result<Option<CurrentRevision>, ProductionError> {
        let candidates = self
            .lifecycle
            .current_dependents(parent)
            .map_err(|_| ProductionError::Storage)?;
        let Some((key, revision)) = candidates
            .into_iter()
            .filter(|(key, _)| crate::receiver_ship_detail::is_key(key.as_str()))
            .max_by(|(key, revision), (other_key, other_revision)| {
                revision.cmp(other_revision).then(key.cmp(other_key))
            })
        else {
            return Ok(None);
        };
        let detail = self
            .lifecycle
            .current_revision(&key)
            .map_err(|_| ProductionError::Storage)?
            .ok_or(ProductionError::Storage)?;
        if detail.receipt().revision != revision
            || detail.revision().source_scope != parent.revision().source_scope
            || detail
                .revision()
                .dependencies
                .get(&parent.receipt().section_key)
                != Some(&parent.receipt().revision)
        {
            return Err(ProductionError::Storage);
        }
        Ok(Some(detail))
    }
}
