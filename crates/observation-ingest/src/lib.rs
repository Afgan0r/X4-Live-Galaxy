#![forbid(unsafe_code)]

mod batch;
mod batch_budget;
mod completion;
mod model;
mod runtime_facts;
mod snapshot;
mod wire;

use observation_domain::{
    EntityId, ObservationSource, ObservationTime, ObservationVersion, SectionDescriptor,
    SectionQuality,
};

pub use batch::{
    MAX_BATCH_BYTES, MAX_BATCH_FRAMES, MAX_BATCH_MARKERS, MAX_BATCH_OBSERVATIONS, MAX_BATCH_SCOPES,
    ReceiptClock, SystemReceiptClock, admit_batch, admit_batch_with_receipt_clock, validate_batch,
};
pub use model::{
    AcceptedProjection, AdmissionError, AdmissionOutcome, MAX_REJECTION_EVIDENCE,
    RejectionEvidence, RejectionReason,
};
pub use runtime_facts::{
    RuntimeAsset, RuntimeCapacity, RuntimeFactAvailability, RuntimeFactQuality, RuntimeFacts,
    RuntimeOwnership, RuntimeSector,
};
pub use snapshot::ProjectionSnapshot;
use wire::TracerObservation;
pub use wire::{FrameHeader, inspect_frame};

const MAX_TRACER_PAYLOAD_BYTES: usize = 512;

#[must_use]
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

    #[must_use]
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
