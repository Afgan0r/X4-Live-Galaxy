#![forbid(unsafe_code)]

mod absence;
mod completion_envelope;
mod identity;
mod observation;
mod reconciliation;
mod section;
mod session;

pub use absence::{AbsenceEvidence, AbsenceTracker, reconcile_qualified_membership};
pub use identity::{
    BatchId, CanonicalizationVersion, CompletionCoverage, ControlEnvelope, DecisionSnapshotId,
    DigestAlgorithmVersion, EntityId, EnvelopeDecodeError, EventId, FrameHeader,
    ObservationPolicyVersion, ObservationSchemaVersion, ObservationSource, ObservationTime,
    ObservationVersion, ProducerIncarnationId, RecordId, SectionKey, SectionRevisionId,
    SourceScopeId, TransportEpoch,
};
pub use observation::{
    CompleteMessage, DuplicateDecision, EnvelopeRecord, ImmutableBatchEnvelope, ObservationRecord,
    ObservationRecordError, SectionCompletionEnvelope, SectionStartEnvelope, classify_duplicate,
};
pub use reconciliation::{
    CanonicalObservationKey, CollectionLimit, CollectionSize, CountError, ReconciliationDecision,
    reconcile_membership,
};
pub use section::{
    CaptureWindow, CompleteMarker, SectionAvailability, SectionCoverage, SectionDescriptor,
    SectionFreshness, SectionQuality, SectionState, quality_for_empty_section,
};
pub use session::SourceSessionIdentity;
