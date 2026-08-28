use std::collections::{BTreeMap, VecDeque};

use observation_domain::{CanonicalObservationKey, EntityId, ObservationRecord, SectionQuality};

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
    pub const fn new(reason: RejectionReason) -> Self {
        Self { reason }
    }

    pub const fn reason(&self) -> RejectionReason {
        self.reason
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopedObservation {
    pub scope: EntityId,
    pub record: ObservationRecord,
    pub quality: SectionQuality,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProjectionSnapshot {
    pub observations: BTreeMap<EntityId, ScopedObservation>,
}

impl ProjectionSnapshot {
    pub fn entity_ids(&self) -> Vec<&str> {
        self.observations.keys().map(EntityId::as_str).collect()
    }

    pub fn keys_for_scope(&self, scope: &EntityId) -> Vec<CanonicalObservationKey> {
        self.observations
            .values()
            .filter(|item| &item.scope == scope)
            .map(|item| {
                CanonicalObservationKey::new(item.record.entity_id().clone(), item.record.version())
            })
            .collect()
    }

    pub fn remove_tombstones(&mut self, scope: &EntityId, tombstones: &[CanonicalObservationKey]) {
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
    pub snapshot: ProjectionSnapshot,
    rejection_evidence: VecDeque<RejectionEvidence>,
}

impl AcceptedProjection {
    pub fn empty() -> Self {
        Self::default()
    }

    pub const fn snapshot(&self) -> &ProjectionSnapshot {
        &self.snapshot
    }

    pub const fn rejection_evidence(&self) -> &VecDeque<RejectionEvidence> {
        &self.rejection_evidence
    }

    pub const fn with_snapshot(snapshot: ProjectionSnapshot) -> Self {
        Self {
            snapshot,
            rejection_evidence: VecDeque::new(),
        }
    }

    #[must_use]
    pub fn record_rejection(mut self, reason: RejectionReason) -> Self {
        if self.rejection_evidence.len() == MAX_REJECTION_EVIDENCE {
            let _ = self.rejection_evidence.pop_front();
        }
        self.rejection_evidence
            .push_back(RejectionEvidence::new(reason));
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
    pub const fn projection(&self) -> &AcceptedProjection {
        match self {
            Self::Accepted(value)
            | Self::Rejected {
                projection: value, ..
            } => value,
        }
    }

    pub const fn snapshot(&self) -> &ProjectionSnapshot {
        self.projection().snapshot()
    }

    pub const fn rejection_reason(&self) -> Option<RejectionReason> {
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
