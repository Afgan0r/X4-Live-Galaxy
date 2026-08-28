#![forbid(unsafe_code)]

use observation_domain::{
    EntityId, ObservationSource, ObservationTime, ObservationVersion, SectionDescriptor,
    SectionQuality,
};
use serde::Deserialize;

const MAX_TRACER_PAYLOAD_BYTES: usize = 512;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionError {
    InvalidFixture,
    FrameTooLarge,
    InvalidEntityId,
    InvalidVersion,
    InvalidQuality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedSnapshot {
    section: SectionDescriptor,
}

impl AcceptedSnapshot {
    pub fn from_tracer_payload(payload: &str) -> Result<Self, AdmissionError> {
        if payload.len() > MAX_TRACER_PAYLOAD_BYTES {
            return Err(AdmissionError::FrameTooLarge);
        }

        let payload: TracerObservation =
            serde_json::from_str(payload).map_err(|_| AdmissionError::InvalidFixture)?;
        let entity_id = EntityId::new(payload.entity_id).ok_or(AdmissionError::InvalidEntityId)?;
        let observed_at = ObservationTime::from_unix_millis(payload.observed_at_unix_millis);
        let version =
            ObservationVersion::new(payload.version).ok_or(AdmissionError::InvalidVersion)?;
        let quality = match payload.quality {
            TracerQuality::Fresh => SectionQuality::Fresh,
            TracerQuality::KnownEmpty => SectionQuality::KnownEmpty,
            TracerQuality::Unknown => SectionQuality::Unknown,
            TracerQuality::Partial => SectionQuality::Partial,
            TracerQuality::Stale => SectionQuality::Stale,
            TracerQuality::Unsupported => SectionQuality::Unsupported,
        };

        Ok(Self {
            section: SectionDescriptor::new(
                entity_id,
                ObservationSource::x4_runtime(),
                observed_at,
                version,
                quality,
            ),
        })
    }

    pub fn entity_id(&self) -> &EntityId {
        self.section.entity_id()
    }

    pub const fn version(&self) -> ObservationVersion {
        self.section.version()
    }

    pub const fn section_quality_name(&self) -> &'static str {
        match self.section.quality() {
            SectionQuality::Fresh => "fresh",
            SectionQuality::KnownEmpty => "known_empty",
            SectionQuality::Unknown => "unknown",
            SectionQuality::Partial => "partial",
            SectionQuality::Stale => "stale",
            SectionQuality::Unsupported => "unsupported",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TracerObservation {
    entity_id: String,
    observed_at_unix_millis: u64,
    version: u64,
    quality: TracerQuality,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum TracerQuality {
    Fresh,
    KnownEmpty,
    Unknown,
    Partial,
    Stale,
    Unsupported,
}
