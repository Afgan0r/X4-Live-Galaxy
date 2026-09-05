use crate::{
    EntityId, ObservationSource, ObservationTime, ObservationVersion, SectionCoverage,
    SectionFreshness, SectionQuality, SectionState,
};

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
