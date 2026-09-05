use std::collections::BTreeMap;

use observation_domain::{
    CanonicalObservationKey, EntityId, ObservationRecord, ObservationVersion, SectionQuality,
};

use crate::completed_scope::CompletedScope;

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopedObservation {
    pub scope: EntityId,
    pub record: ObservationRecord,
    pub quality: SectionQuality,
}

#[must_use]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProjectionSnapshot {
    pub observations: BTreeMap<EntityId, ScopedObservation>,
    completed_scopes: BTreeMap<EntityId, CompletedScope>,
}

impl ProjectionSnapshot {
    pub fn entity_ids(&self) -> Vec<&str> {
        self.observations.keys().map(EntityId::as_str).collect()
    }

    #[must_use]
    pub fn keys_for_scope(&self, scope: &EntityId) -> Vec<CanonicalObservationKey> {
        self.observations
            .values()
            .filter(|item| &item.scope == scope)
            .map(|item| {
                CanonicalObservationKey::new(item.record.entity_id().clone(), item.record.version())
            })
            .collect()
    }

    pub fn remove_tombstones(&mut self, scope: &EntityId, tombstones: &[CanonicalObservationKey]) {
        self.observations.retain(|_, item| {
            &item.scope != scope
                || !tombstones
                    .iter()
                    .any(|tombstone| tombstone.entity_id() == item.record.entity_id())
        });
    }

    #[must_use]
    pub fn completed_scope(&self, scope: &EntityId) -> Option<&CompletedScope> {
        self.completed_scopes.get(scope)
    }

    pub fn record_completion(
        &mut self,
        scope: EntityId,
        version: ObservationVersion,
        members: Vec<CanonicalObservationKey>,
    ) {
        self.completed_scopes
            .insert(scope, CompletedScope::new(version, members));
    }
}
