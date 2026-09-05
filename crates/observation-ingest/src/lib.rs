#![forbid(unsafe_code)]
mod accepted_versions;
mod batch;
mod batch_budget;
mod batch_canonical;
mod candidate_limits;
mod completed_scope;
mod completion;
mod completion_digest;
mod completion_types;
mod eligibility;
mod feedback;
mod generation;
mod generation_inspect;
mod legacy_candidate;
mod legacy_generation;
mod model;
mod producer_completion;
mod runtime_facts;
mod scheduler;
mod scheduler_budget;
mod scheduler_queue;
mod snapshot;
mod validated_revision;
mod wire;
mod wire_decode;
pub use batch::{
    MAX_BATCH_BYTES, MAX_BATCH_FRAMES, MAX_BATCH_MARKERS, MAX_BATCH_OBSERVATIONS, MAX_BATCH_SCOPES,
    admit_batch, admit_batch_with_receipt_clock, validate_batch,
};
pub use batch_budget::{AggregateUsage, CandidateUsage};
pub use candidate_limits::{AggregateLimits, CandidateLimits, GenerationLimits};
pub use completed_scope::CompletedScope;
pub use completion_types::{
    CandidateContext, CompletionCertificate, CompletionCurrent, CompletionOutcome, ContractVersions,
};
pub use eligibility::{
    AcceptedPublication, DecisionEligibility, DecisionRevisionIndex, DecisionRevisionSet,
    EligibilityBlocker, FinalizationOutcome,
};
pub use feedback::{
    AmbiguityResolution, ApplicationContextIdentity, CollectionPolicyLimits, DeliveryStage,
    FeedbackError, ImmutableApplicationBatch, ReceiverDisposition, SlotAdmission, SlotTurnover,
    StopAndWaitSlot, TransportPolicyLimits,
};
pub use generation::GenerationStager;
pub use model::{
    AcceptedProjection, AdmissionError, AdmissionOutcome, GenerationProgress,
    MAX_REJECTION_EVIDENCE, ReceiptClock, RejectionEvidence, RejectionReason, SystemReceiptClock,
};
pub use observation_domain::{
    CompleteMessage, CompletionCoverage, ControlEnvelope, EnvelopeDecodeError, EnvelopeRecord,
    FrameHeader, ImmutableBatchEnvelope, SectionCompletionEnvelope, SectionStartEnvelope,
    TransportEpoch,
};
use observation_domain::{
    EntityId, ObservationSource, ObservationTime, ObservationVersion, SectionDescriptor,
    SectionQuality,
};
pub use producer_completion::bind_completion_certificate;
pub use runtime_facts::{
    RuntimeAsset, RuntimeCapacity, RuntimeFactAvailability, RuntimeFactQuality, RuntimeFacts,
    RuntimeOwnership, RuntimeSector,
};
pub use scheduler::{DeliveredPulse, MonotonicClock, ObservationScheduler, SchedulerOutcome};
pub use scheduler_queue::{
    CollectionClass, CollectionIntent, CollectionIntentId, CompletionDisposition,
    SchedulerAdmission, SchedulerSafetyLimits, WorkKind,
};
pub use snapshot::ProjectionSnapshot;
pub use validated_revision::{DurableRevisionParts, ValidatedSectionRevision};
use wire::TracerObservation;
pub use wire::decode_complete_message;
const MAX_TRACER_PAYLOAD_BYTES: usize = 512;
pub fn inspect_frame(payload: &str) -> Result<FrameHeader, AdmissionError> {
    if payload.len() > 2_048 {
        return Err(AdmissionError::FrameTooLarge);
    }
    match serde_json::from_str(payload).map_err(|_| AdmissionError::InvalidFixture)? {
        wire::WireFrame::Hello(frame) => Ok(FrameHeader::Hello {
            protocol_major: frame.protocol_major,
            game_build: frame.game_build,
            capabilities: frame.capabilities,
            generation: frame.generation,
        }),
        wire::WireFrame::Heartbeat(frame) => frame_header("heartbeat", frame),
        wire::WireFrame::RuntimeHealth(frame) => {
            if frame.status.is_empty() {
                return Err(AdmissionError::InvalidFixture);
            }
            frame_header("runtime_health", frame)
        }
        wire::WireFrame::Observation(frame) => frame_header("observation", frame),
        wire::WireFrame::CompleteMarker(frame) => frame_header("complete_marker", frame),
    }
}

trait HeaderFields {
    fn into_fields(self) -> (String, u64, Option<u64>, Option<u64>);
}
macro_rules! header_fields {
    ($($ty:ty),+ $(,)?) => {$(
        impl HeaderFields for $ty {
            fn into_fields(self) -> (String, u64, Option<u64>, Option<u64>) {
                (self.scope, self.version, self.generation, self.sequence)
            }
        }
    )+};
}
header_fields!(
    wire::WireHeartbeat,
    wire::WireRuntimeHealth,
    wire::WireObservation,
    wire::WireCompleteMarker
);

fn frame_header(
    kind: &'static str,
    frame: impl HeaderFields,
) -> Result<FrameHeader, AdmissionError> {
    let (scope, version, generation, sequence) = frame.into_fields();
    let _ = EntityId::new(scope.clone()).ok_or(AdmissionError::InvalidScope)?;
    let _ = ObservationVersion::new(version).ok_or(AdmissionError::InvalidVersion)?;
    Ok(FrameHeader::Data {
        kind,
        scope,
        version,
        generation: generation.ok_or(AdmissionError::InvalidFixture)?,
        sequence: sequence.ok_or(AdmissionError::InvalidFixture)?,
    })
}
fn validate_telemetry(scope: String, version: u64) -> Result<(), AdmissionError> {
    let _ = EntityId::new(scope).ok_or(AdmissionError::InvalidScope)?;
    let _ = ObservationVersion::new(version).ok_or(AdmissionError::InvalidVersion)?;
    Ok(())
}
fn validate_health(frame: wire::WireRuntimeHealth) -> Result<(), AdmissionError> {
    validate_telemetry(frame.scope, frame.version)?;
    (!frame.status.is_empty())
        .then_some(())
        .ok_or(AdmissionError::InvalidFixture)
}
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
