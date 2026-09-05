use std::collections::{BTreeMap, VecDeque};

use crate::runtime_facts::RuntimeFacts;
use crate::snapshot::ProjectionSnapshot;
use observation_domain::EntityId;

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
    ReceiptClockUnavailable,
    OutOfOrderVersion,
    EqualVersionConflict,
    CollectionLimitExceeded,
    CompletionMismatch,
    DependencyChanged,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectionReason {
    MalformedFrame,
    OversizedFrame,
    InvalidIdentity,
    InvalidVersion,
    OutOfOrderVersion,
    EqualVersionConflict,
    CollectionLimitExceeded,
    CompletionMismatch,
    DependencyChanged,
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
            AdmissionError::CompletionMismatch => Self::CompletionMismatch,
            AdmissionError::DependencyChanged => Self::DependencyChanged,
            AdmissionError::InvalidFixture
            | AdmissionError::InvalidQuality
            | AdmissionError::InvalidContent
            | AdmissionError::ReceiptClockUnavailable => Self::MalformedFrame,
        }
    }
}

#[must_use]
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

#[must_use]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AcceptedProjection {
    pub snapshot: ProjectionSnapshot,
    pub(crate) runtime_facts: BTreeMap<EntityId, RuntimeFacts>,
    rejection_evidence: VecDeque<RejectionEvidence>,
}

impl AcceptedProjection {
    pub fn empty() -> Self {
        Self::default()
    }

    pub const fn snapshot(&self) -> &ProjectionSnapshot {
        &self.snapshot
    }

    #[must_use]
    pub fn runtime_facts(&self, sector_id: &str) -> Option<&RuntimeFacts> {
        EntityId::new(sector_id.to_owned()).and_then(|id| self.runtime_facts.get(&id))
    }

    #[must_use]
    pub const fn rejection_evidence(&self) -> &VecDeque<RejectionEvidence> {
        &self.rejection_evidence
    }

    pub const fn with_snapshot(snapshot: ProjectionSnapshot) -> Self {
        Self {
            snapshot,
            runtime_facts: BTreeMap::new(),
            rejection_evidence: VecDeque::new(),
        }
    }

    pub(crate) const fn with_runtime_facts(
        snapshot: ProjectionSnapshot,
        runtime_facts: BTreeMap<EntityId, RuntimeFacts>,
    ) -> Self {
        Self {
            snapshot,
            runtime_facts,
            rejection_evidence: VecDeque::new(),
        }
    }

    pub fn record_rejection(mut self, reason: RejectionReason) -> Self {
        if self.rejection_evidence.len() == MAX_REJECTION_EVIDENCE {
            let _ = self.rejection_evidence.pop_front();
        }
        self.rejection_evidence
            .push_back(RejectionEvidence::new(reason));
        self
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionOutcome {
    Accepted(AcceptedProjection),
    Rejected {
        projection: AcceptedProjection,
        evidence: RejectionEvidence,
    },
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

    #[must_use]
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
