#![forbid(unsafe_code)]

use std::num::NonZeroU64;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntityId(String);

impl EntityId {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (!value.trim().is_empty()).then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventId(String);

impl EventId {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (!value.trim().is_empty()).then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
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

    pub const fn get(self) -> u64 {
        self.0.get()
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionFreshness {
    Fresh,
    Stale,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionCoverage {
    Complete,
    KnownEmpty,
    Unknown,
    Partial,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SectionState {
    freshness: SectionFreshness,
    coverage: SectionCoverage,
}

impl SectionState {
    pub const fn new(freshness: SectionFreshness, coverage: SectionCoverage) -> Self {
        Self {
            freshness,
            coverage,
        }
    }

    pub const fn freshness(self) -> SectionFreshness {
        self.freshness
    }

    pub const fn coverage(self) -> SectionCoverage {
        self.coverage
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteMarker {
    scope: EntityId,
    version: ObservationVersion,
}

impl CompleteMarker {
    pub const fn successful(scope: EntityId, version: ObservationVersion) -> Self {
        Self { scope, version }
    }

    pub const fn scope(&self) -> &EntityId {
        &self.scope
    }

    pub const fn version(&self) -> ObservationVersion {
        self.version
    }
}

pub fn quality_for_empty_section(
    scope: &EntityId,
    marker: Option<&CompleteMarker>,
) -> SectionState {
    match marker {
        Some(marker) if marker.scope() == scope => {
            SectionState::new(SectionFreshness::Fresh, SectionCoverage::KnownEmpty)
        }
        Some(_) => SectionState::new(SectionFreshness::Fresh, SectionCoverage::Partial),
        None => SectionState::new(SectionFreshness::Fresh, SectionCoverage::Unknown),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservationRecordError {
    EmptyContent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationRecord {
    entity_id: EntityId,
    source: ObservationSource,
    observed_at: ObservationTime,
    version: ObservationVersion,
    content: String,
}

impl ObservationRecord {
    pub fn new(
        entity_id: EntityId,
        source: ObservationSource,
        observed_at: ObservationTime,
        version: ObservationVersion,
        content: impl Into<String>,
    ) -> Result<Self, ObservationRecordError> {
        let content = content.into();
        if content.is_empty() {
            return Err(ObservationRecordError::EmptyContent);
        }

        Ok(Self {
            entity_id,
            source,
            observed_at,
            version,
            content,
        })
    }

    pub const fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }

    pub const fn source(&self) -> ObservationSource {
        self.source
    }

    pub const fn observed_at(&self) -> ObservationTime {
        self.observed_at
    }

    pub const fn version(&self) -> ObservationVersion {
        self.version
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DuplicateDecision {
    Idempotent,
    Conflict,
    DifferentIdentity,
}

pub fn classify_duplicate(
    accepted: &ObservationRecord,
    candidate: &ObservationRecord,
) -> DuplicateDecision {
    if accepted.entity_id != candidate.entity_id {
        return DuplicateDecision::DifferentIdentity;
    }

    if accepted.version == candidate.version && accepted.content == candidate.content {
        DuplicateDecision::Idempotent
    } else {
        DuplicateDecision::Conflict
    }
}

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
