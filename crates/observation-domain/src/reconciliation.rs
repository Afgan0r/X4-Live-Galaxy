use std::num::NonZeroUsize;

use crate::{CompleteMarker, EntityId, ObservationVersion};

#[must_use]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalObservationKey {
    entity_id: EntityId,
    version: ObservationVersion,
}

impl CanonicalObservationKey {
    pub const fn new(entity_id: EntityId, version: ObservationVersion) -> Self {
        Self { entity_id, version }
    }

    pub const fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }

    pub const fn version(&self) -> ObservationVersion {
        self.version
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CollectionLimit(NonZeroUsize);

impl CollectionLimit {
    #[must_use]
    pub const fn new(value: usize) -> Option<Self> {
        match NonZeroUsize::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CollectionSize(usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountError {
    ExceedsPlatformCapacity,
}

impl CollectionSize {
    pub fn from_u128(value: u128) -> Result<Self, CountError> {
        usize::try_from(value)
            .map(Self)
            .map_err(|_| CountError::ExceedsPlatformCapacity)
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconciliationDecision {
    PreservedIncompleteScope,
    RejectedCollectionLimit,
    AwaitingSecondAbsence {
        members: Vec<CanonicalObservationKey>,
    },
    Reconciled {
        members: Vec<CanonicalObservationKey>,
        tombstones: Vec<CanonicalObservationKey>,
    },
}

pub fn reconcile_membership(
    previous: &[CanonicalObservationKey],
    mut observed: Vec<CanonicalObservationKey>,
    scope: &EntityId,
    marker: Option<&CompleteMarker>,
    limit: CollectionLimit,
) -> ReconciliationDecision {
    if !matches!(marker, Some(marker) if marker.scope() == scope) {
        return ReconciliationDecision::PreservedIncompleteScope;
    }

    if observed.len() > limit.get() {
        return ReconciliationDecision::RejectedCollectionLimit;
    }

    observed.sort();
    let mut tombstones: Vec<_> = previous
        .iter()
        .filter(|prior| {
            !observed
                .iter()
                .any(|current| current.entity_id() == prior.entity_id())
        })
        .cloned()
        .collect();
    tombstones.sort();

    ReconciliationDecision::Reconciled {
        members: observed,
        tombstones,
    }
}
