#![forbid(unsafe_code)]

mod identity;
mod observation;
mod reconciliation;
mod section;

pub use identity::{
    BatchId, CanonicalizationVersion, DecisionSnapshotId, DigestAlgorithmVersion, EntityId,
    EventId, ObservationPolicyVersion, ObservationSchemaVersion, ObservationSource,
    ObservationTime, ObservationVersion, ProducerIncarnationId, RecordId, SectionKey,
    SectionRevisionId, SourceScopeId, TransportEpoch,
};
pub use observation::{
    DuplicateDecision, ObservationRecord, ObservationRecordError, classify_duplicate,
};
pub use reconciliation::{
    CanonicalObservationKey, CollectionLimit, CollectionSize, CountError, ReconciliationDecision,
    reconcile_membership,
};
pub use section::{
    CompleteMarker, SectionCoverage, SectionDescriptor, SectionFreshness, SectionQuality,
    SectionState, quality_for_empty_section,
};
