#![forbid(unsafe_code)]

use std::num::NonZeroU64;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntityId(String);

impl EntityId {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (!value.trim().is_empty()).then_some(Self(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ObservationSource {
    X4Runtime,
}

impl ObservationSource {
    pub const fn x4_runtime() -> Self {
        Self::X4Runtime
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObservationTime(u64);

impl ObservationTime {
    pub const fn from_unix_millis(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObservationVersion(NonZeroU64);

impl ObservationVersion {
    pub const fn new(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionQuality {
    Fresh,
    KnownEmpty,
    Unknown,
    Partial,
    Stale,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SectionDescriptor {
    entity_id: EntityId,
    source: ObservationSource,
    observed_at: ObservationTime,
    version: ObservationVersion,
    quality: SectionQuality,
}

impl SectionDescriptor {
    pub const fn new(
        entity_id: EntityId,
        source: ObservationSource,
        observed_at: ObservationTime,
        version: ObservationVersion,
        quality: SectionQuality,
    ) -> Self {
        Self {
            entity_id,
            source,
            observed_at,
            version,
            quality,
        }
    }

    pub const fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }

    pub const fn source(&self) -> &ObservationSource {
        &self.source
    }

    pub const fn observed_at(&self) -> ObservationTime {
        self.observed_at
    }

    pub const fn version(&self) -> ObservationVersion {
        self.version
    }

    pub const fn quality(&self) -> SectionQuality {
        self.quality
    }
}
