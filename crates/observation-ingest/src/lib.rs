#![forbid(unsafe_code)]

use std::collections::{BTreeMap, VecDeque};

use observation_domain::{
    CanonicalObservationKey, CollectionLimit, CompleteMarker, DuplicateDecision, EntityId,
    ObservationRecord, ObservationSource, ObservationTime, ObservationVersion,
    ReconciliationDecision, SectionDescriptor, SectionQuality, classify_duplicate,
    reconcile_membership,
};
use serde::Deserialize;

const MAX_TRACER_PAYLOAD_BYTES: usize = 512;
const MAX_SCOPE_MEMBERS: usize = 64;
pub const MAX_REJECTION_EVIDENCE: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionError {
    InvalidFixture,
    FrameTooLarge,
    InvalidEntityId,
    InvalidVersion,
    InvalidQuality,
    InvalidScope,
    InvalidContent,
    OutOfOrderVersion,
    EqualVersionConflict,
    CollectionLimitExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectionReason {
    MalformedFrame,
    OversizedFrame,
    InvalidIdentity,
    InvalidVersion,
    OutOfOrderVersion,
    EqualVersionConflict,
    CollectionLimitExceeded,
}

impl From<&AdmissionError> for RejectionReason {
    fn from(error: &AdmissionError) -> Self {
        match error {
            AdmissionError::FrameTooLarge => Self::OversizedFrame,
            AdmissionError::InvalidEntityId | AdmissionError::InvalidScope => Self::InvalidIdentity,
            AdmissionError::InvalidVersion => Self::InvalidVersion,
            AdmissionError::OutOfOrderVersion => Self::OutOfOrderVersion,
            AdmissionError::EqualVersionConflict => Self::EqualVersionConflict,
            AdmissionError::CollectionLimitExceeded => Self::CollectionLimitExceeded,
            AdmissionError::InvalidFixture
            | AdmissionError::InvalidQuality
            | AdmissionError::InvalidContent => Self::MalformedFrame,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectionEvidence {
    reason: RejectionReason,
}
impl RejectionEvidence {
    pub const fn reason(&self) -> RejectionReason {
        self.reason
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScopedObservation {
    scope: EntityId,
    record: ObservationRecord,
    quality: SectionQuality,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProjectionSnapshot {
    observations: BTreeMap<EntityId, ScopedObservation>,
}

impl ProjectionSnapshot {
    fn keys_for_scope(&self, scope: &EntityId) -> Vec<CanonicalObservationKey> {
        self.observations
            .values()
            .filter(|item| &item.scope == scope)
            .map(|item| {
                CanonicalObservationKey::new(item.record.entity_id().clone(), item.record.version())
            })
            .collect()
    }

    fn remove_tombstones(&mut self, scope: &EntityId, tombstones: &[CanonicalObservationKey]) {
        self.observations.retain(|_, item| {
            &item.scope != scope
                || !tombstones
                    .iter()
                    .any(|tombstone| tombstone.entity_id() == item.record.entity_id())
        });
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AcceptedProjection {
    snapshot: ProjectionSnapshot,
    rejection_evidence: VecDeque<RejectionEvidence>,
}

impl AcceptedProjection {
    pub fn empty() -> Self {
        Self::default()
    }
    pub const fn snapshot(&self) -> &ProjectionSnapshot {
        &self.snapshot
    }
    pub fn rejection_evidence(&self) -> &VecDeque<RejectionEvidence> {
        &self.rejection_evidence
    }
    fn with_snapshot(snapshot: ProjectionSnapshot) -> Self {
        Self {
            snapshot,
            rejection_evidence: VecDeque::new(),
        }
    }
    fn record_rejection(mut self, reason: RejectionReason) -> Self {
        if self.rejection_evidence.len() == MAX_REJECTION_EVIDENCE {
            let _ = self.rejection_evidence.pop_front();
        }
        self.rejection_evidence
            .push_back(RejectionEvidence { reason });
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionOutcome {
    Accepted(AcceptedProjection),
    Rejected {
        projection: AcceptedProjection,
        evidence: RejectionEvidence,
    },
}

impl AdmissionOutcome {
    pub fn projection(&self) -> &AcceptedProjection {
        match self {
            Self::Accepted(value)
            | Self::Rejected {
                projection: value, ..
            } => value,
        }
    }
    pub fn snapshot(&self) -> &ProjectionSnapshot {
        self.projection().snapshot()
    }
    pub fn rejection_reason(&self) -> Option<RejectionReason> {
        match self {
            Self::Accepted(_) => None,
            Self::Rejected { evidence, .. } => Some(evidence.reason()),
        }
    }
    pub fn into_projection(self) -> AcceptedProjection {
        match self {
            Self::Accepted(value)
            | Self::Rejected {
                projection: value, ..
            } => value,
        }
    }
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

pub fn validate_batch(
    accepted: &AcceptedProjection,
    frames: &[&str],
) -> Result<ProjectionSnapshot, AdmissionError> {
    let mut candidate = accepted.snapshot.clone();
    let mut markers = Vec::new();
    for frame in frames {
        if frame.len() > MAX_TRACER_PAYLOAD_BYTES {
            return Err(AdmissionError::FrameTooLarge);
        }
        match serde_json::from_str(frame).map_err(|_| AdmissionError::InvalidFixture)? {
            WireFrame::Observation(frame) => apply_observation(&mut candidate, frame)?,
            WireFrame::CompleteMarker(frame) => markers.push(complete_marker(frame)?),
        }
    }
    for marker in markers {
        apply_reconciliation(&accepted.snapshot, &mut candidate, marker)?;
    }
    Ok(candidate)
}

pub fn admit_batch(accepted: AcceptedProjection, frames: &[&str]) -> AdmissionOutcome {
    match validate_batch(&accepted, frames) {
        Ok(snapshot) => AdmissionOutcome::Accepted(AcceptedProjection::with_snapshot(snapshot)),
        Err(error) => {
            let evidence = RejectionEvidence {
                reason: RejectionReason::from(&error),
            };
            AdmissionOutcome::Rejected {
                projection: accepted.record_rejection(evidence.reason()),
                evidence,
            }
        }
    }
}

fn apply_observation(
    candidate: &mut ProjectionSnapshot,
    frame: WireObservation,
) -> Result<(), AdmissionError> {
    let entity_id = EntityId::new(frame.entity_id).ok_or(AdmissionError::InvalidEntityId)?;
    let scope = EntityId::new(frame.scope).ok_or(AdmissionError::InvalidScope)?;
    let version = ObservationVersion::new(frame.version).ok_or(AdmissionError::InvalidVersion)?;
    let quality = SectionQuality::from(frame.quality);
    let record = ObservationRecord::new(
        entity_id.clone(),
        ObservationSource::x4_runtime(),
        ObservationTime::from_unix_millis(frame.observed_at_unix_millis),
        version,
        frame.content,
    )
    .map_err(|_| AdmissionError::InvalidContent)?;
    if let Some(previous) = candidate.observations.get(&entity_id) {
        match classify_duplicate(&previous.record, &record) {
            DuplicateDecision::Idempotent => return Ok(()),
            DuplicateDecision::Conflict if version == previous.record.version() => {
                return Err(AdmissionError::EqualVersionConflict);
            }
            DuplicateDecision::Conflict if version < previous.record.version() => {
                return Err(AdmissionError::OutOfOrderVersion);
            }
            DuplicateDecision::Conflict => {}
            DuplicateDecision::DifferentIdentity => return Err(AdmissionError::InvalidEntityId),
        }
    }
    candidate.observations.insert(
        entity_id,
        ScopedObservation {
            scope,
            record,
            quality,
        },
    );
    Ok(())
}

fn complete_marker(frame: WireCompleteMarker) -> Result<CompleteMarker, AdmissionError> {
    let scope = EntityId::new(frame.scope).ok_or(AdmissionError::InvalidScope)?;
    let version = ObservationVersion::new(frame.version).ok_or(AdmissionError::InvalidVersion)?;
    Ok(CompleteMarker::successful(scope, version))
}

fn apply_reconciliation(
    accepted: &ProjectionSnapshot,
    candidate: &mut ProjectionSnapshot,
    marker: CompleteMarker,
) -> Result<(), AdmissionError> {
    let scope = marker.scope();
    let limit =
        CollectionLimit::new(MAX_SCOPE_MEMBERS).ok_or(AdmissionError::CollectionLimitExceeded)?;
    match reconcile_membership(
        &accepted.keys_for_scope(scope),
        candidate.keys_for_scope(scope),
        scope,
        Some(&marker),
        limit,
    ) {
        ReconciliationDecision::Reconciled { tombstones, .. } => {
            candidate.remove_tombstones(scope, &tombstones);
            Ok(())
        }
        ReconciliationDecision::RejectedCollectionLimit => {
            Err(AdmissionError::CollectionLimitExceeded)
        }
        ReconciliationDecision::PreservedIncompleteScope => Ok(()),
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WireFrame {
    Observation(WireObservation),
    CompleteMarker(WireCompleteMarker),
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireObservation {
    scope: String,
    entity_id: String,
    observed_at_unix_millis: u64,
    version: u64,
    quality: TracerQuality,
    content: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireCompleteMarker {
    scope: String,
    version: u64,
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
impl From<TracerQuality> for SectionQuality {
    fn from(value: TracerQuality) -> Self {
        match value {
            TracerQuality::Fresh => Self::Fresh,
            TracerQuality::KnownEmpty => Self::KnownEmpty,
            TracerQuality::Unknown => Self::Unknown,
            TracerQuality::Partial => Self::Partial,
            TracerQuality::Stale => Self::Stale,
            TracerQuality::Unsupported => Self::Unsupported,
        }
    }
}
