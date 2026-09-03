#![forbid(unsafe_code)]
mod batch;
mod batch_budget;
mod completion;
mod feedback;
mod generation;
mod model;
mod runtime_facts;
mod scheduler;
mod snapshot;
mod wire;
pub use batch::{
    MAX_BATCH_BYTES, MAX_BATCH_FRAMES, MAX_BATCH_MARKERS, MAX_BATCH_OBSERVATIONS, MAX_BATCH_SCOPES,
    admit_batch, admit_batch_with_receipt_clock, validate_batch,
};
pub use batch_budget::{AggregateLimits, AggregateUsage, CandidateLimits, CandidateUsage};
pub use feedback::{
    CollectionPolicyLimits, DeliveryStage, FeedbackError, ImmutableApplicationBatch,
    ReceiverDisposition, SlotAdmission, StopAndWaitSlot, TransportPolicyLimits,
};
pub use generation::GenerationStager;
pub use model::{
    AcceptedProjection, AdmissionError, AdmissionOutcome, MAX_REJECTION_EVIDENCE,
    RejectionEvidence, RejectionReason,
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
pub use runtime_facts::{
    RuntimeAsset, RuntimeCapacity, RuntimeFactAvailability, RuntimeFactQuality, RuntimeFacts,
    RuntimeOwnership, RuntimeSector,
};
pub use scheduler::{DeliveredPulse, MonotonicClock, ObservationScheduler, SchedulerOutcome};
pub use snapshot::ProjectionSnapshot;
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
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GenerationLimits {
    pub(crate) max_staged_bytes: usize,
    pub(crate) max_work_units: usize,
    pub(crate) candidate: CandidateLimits,
    pub(crate) aggregate: AggregateLimits,
}
impl GenerationLimits {
    pub const fn new(max_staged_bytes: usize, max_work_units: usize) -> Self {
        let candidate = CandidateLimits {
            raw_bytes: match std::num::NonZeroUsize::new(max_staged_bytes) {
                Some(value) => value,
                None => std::num::NonZeroUsize::MIN,
            },
            decoded_bytes: match std::num::NonZeroUsize::new(max_staged_bytes) {
                Some(value) => value,
                None => std::num::NonZeroUsize::MIN,
            },
            records: match std::num::NonZeroUsize::new(max_work_units) {
                Some(value) => value,
                None => std::num::NonZeroUsize::MIN,
            },
            batches: match std::num::NonZeroUsize::new(max_work_units) {
                Some(value) => value,
                None => std::num::NonZeroUsize::MIN,
            },
            work: match std::num::NonZeroUsize::new(max_work_units) {
                Some(value) => value,
                None => std::num::NonZeroUsize::MIN,
            },
            age_millis: std::num::NonZeroU64::MAX,
            inactivity_millis: std::num::NonZeroU64::MAX,
        };
        let aggregate = AggregateLimits {
            candidates: candidate.records,
            raw_bytes: candidate.raw_bytes,
            decoded_bytes: candidate.decoded_bytes,
            records: candidate.records,
            batches: candidate.batches,
            work: candidate.work,
        };
        Self {
            max_staged_bytes,
            max_work_units,
            candidate,
            aggregate,
        }
    }

    pub const fn bounded(candidate: CandidateLimits, aggregate: AggregateLimits) -> Self {
        Self {
            max_staged_bytes: candidate.raw_bytes.get(),
            max_work_units: candidate.work.get(),
            candidate,
            aggregate,
        }
    }
}
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationProgress {
    Staged,
    Admitted,
    Replay,
    Rejected(RejectionReason),
}
pub trait ReceiptClock {
    fn receipt_unix_millis(&self) -> Result<u64, AdmissionError>;
}
pub struct SystemReceiptClock;
impl ReceiptClock for SystemReceiptClock {
    fn receipt_unix_millis(&self) -> Result<u64, AdmissionError> {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok())
            .filter(|value| *value > 0)
            .ok_or(AdmissionError::ReceiptClockUnavailable)
    }
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
