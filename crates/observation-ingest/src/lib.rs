#![forbid(unsafe_code)]

mod batch;
mod model;
mod wire;

use observation_domain::{
    EntityId, ObservationSource, ObservationTime, ObservationVersion, SectionDescriptor,
    SectionQuality,
};

pub use batch::{admit_batch, validate_batch};
pub use model::{
    AcceptedProjection, AdmissionError, AdmissionOutcome, MAX_REJECTION_EVIDENCE,
    ProjectionSnapshot, RejectionEvidence, RejectionReason,
};
use wire::TracerObservation;

const MAX_TRACER_PAYLOAD_BYTES: usize = 512;

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
        let version =
            ObservationVersion::new(payload.version).ok_or(AdmissionError::InvalidVersion)?;
        Ok(Self {
            section: SectionDescriptor::new(
                entity_id,
                ObservationSource::x4_runtime(),
                ObservationTime::from_unix_millis(payload.observed_at_unix_millis),
                version,
                payload.quality.into(),
            ),
        })
    }

    pub const fn entity_id(&self) -> &EntityId {
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
