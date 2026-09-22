#![forbid(unsafe_code)]

mod absence;
mod completion_envelope;
mod faction_observation;
mod identity;
mod observation;
mod reconciliation;
mod section;
mod section_state;
mod sender_evidence;
mod session;
mod ship_cargo;
mod ship_cargo_decode;
mod ship_core;
mod ship_crew;
mod ship_crew_decode;
mod ship_detail;
mod ship_detail_codec;
mod ship_field;
mod ship_group;
mod ship_loadout;
mod ship_loadout_decode;
mod ship_section;

pub use absence::{AbsenceEvidence, AbsenceTracker, reconcile_qualified_membership};
pub use faction_observation::{
    FactionCensusEntry, FactionObservationDisposition, FactionObservationError,
    FactionObservationRoster, FactionOrigin, FactionOriginEvidence, classify_observation_factions,
};
pub use identity::{
    BatchId, CanonicalizationVersion, CompletionCoverage, ControlEnvelope, DecisionSnapshotId,
    DigestAlgorithmVersion, EntityId, EnvelopeDecodeError, EventId, FrameHeader,
    ObservationPolicyVersion, ObservationSchemaVersion, ObservationSource, ObservationTime,
    ObservationVersion, ProducerIncarnationId, RecordId, SectionKey, SectionRevisionId,
    SourceEpochId, SourceScopeId, TransportEpoch,
};
pub use observation::{
    CompleteMessage, DuplicateDecision, EnvelopeRecord, ImmutableBatchEnvelope, ObservationRecord,
    ObservationRecordError, SectionCompletionEnvelope, SectionStartEnvelope, classify_duplicate,
};
pub use reconciliation::{
    CanonicalObservationKey, CollectionLimit, CollectionSize, CountError, ReconciliationDecision,
    reconcile_membership,
};
pub use section::{CompleteMarker, SectionDescriptor, quality_for_empty_section};
pub use section_state::{
    CaptureWindow, SectionAvailability, SectionCoverage, SectionFreshness, SectionQuality,
    SectionState,
};
pub use sender_evidence::{
    CaptureClock, SenderEvidence, SourceBoundary, SourceConsistency, SourceEpochStatus,
};
pub use session::SourceSessionIdentity;
pub use ship_cargo::{CargoObservation, CargoStorage, CargoWare};
pub use ship_core::{
    ShipClass, ShipCoreError, ShipCoreRecord, ShipIdentity, ShipLocation, ShipOwner, ShipType,
};
pub use ship_crew::{CrewObservation, CrewRole, CrewTier};
pub use ship_detail::{
    ShipDetailDependency, ShipDetailError, ShipRecordConsistency, ShipStaleReason, detail_number,
    detail_outcome, detail_token,
};
pub use ship_field::{FieldApplicability, FieldOutcome, SourceEvidenceRef};
pub use ship_group::{ShipDetailGroup, ShipGroupDescriptor, ShipGroupError};
pub use ship_loadout::{
    InstalledSlot, InstalledSoftware, LoadoutObservation, MissileCargo, ShipUnit, VirtualSlot,
};
pub use ship_section::{ShipSectionIdentity, ShipSectionKind};
