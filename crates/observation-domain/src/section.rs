use crate::{EntityId, ObservationSource, ObservationTime, ObservationVersion};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionQuality {
    Fresh,
    KnownEmpty,
    Unknown,
    Partial,
    Stale,
    Unsupported,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionFreshness {
    Fresh,
    Stale,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionCoverage {
    Complete,
    KnownEmpty,
    Unknown,
    Partial,
    Unsupported,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionAvailability {
    Available,
    Unavailable,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CaptureWindow {
    start_millis: u64,
    end_millis: u64,
}

impl CaptureWindow {
    #[must_use]
    pub const fn new(start_millis: u64, end_millis: u64) -> Option<Self> {
        if start_millis <= end_millis {
            Some(Self {
                start_millis,
                end_millis,
            })
        } else {
            None
        }
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SectionState {
    capture_window: CaptureWindow,
    freshness: SectionFreshness,
    quality: SectionQuality,
    availability: SectionAvailability,
    coverage: SectionCoverage,
}

impl SectionState {
    pub const fn new(freshness: SectionFreshness, coverage: SectionCoverage) -> Self {
        Self {
            capture_window: CaptureWindow {
                start_millis: 0,
                end_millis: 0,
            },
            freshness,
            quality: SectionQuality::Unknown,
            availability: SectionAvailability::Available,
            coverage,
        }
    }

    pub const fn with_evidence(
        capture_window: CaptureWindow,
        freshness: SectionFreshness,
        quality: SectionQuality,
        availability: SectionAvailability,
        coverage: SectionCoverage,
    ) -> Self {
        Self {
            capture_window,
            freshness,
            quality,
            availability,
            coverage,
        }
    }

    pub const fn capture_window(self) -> CaptureWindow {
        self.capture_window
    }

    pub const fn freshness(self) -> SectionFreshness {
        self.freshness
    }

    pub const fn coverage(self) -> SectionCoverage {
        self.coverage
    }

    pub const fn quality(self) -> SectionQuality {
        self.quality
    }

    pub const fn availability(self) -> SectionAvailability {
        self.availability
    }
}

#[must_use]
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

#[must_use]
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
