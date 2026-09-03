use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use observation_domain::{
    DecisionSnapshotId, EnvelopeRecord, SectionKey, SectionRevisionId, SourceScopeId,
};
use observation_ingest::{DecisionRevisionSet, ValidatedSectionRevision};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublicationLimits {
    max_records: NonZeroUsize,
    max_content_bytes: NonZeroUsize,
}

impl PublicationLimits {
    pub const fn new(max_records: usize, max_content_bytes: usize) -> Option<Self> {
        Some(Self {
            max_records: match NonZeroUsize::new(max_records) { Some(value) => value, None => return None },
            max_content_bytes: match NonZeroUsize::new(max_content_bytes) { Some(value) => value, None => return None },
        })
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishRequest {
    pub revision: ValidatedSectionRevision,
    pub expected_current: Option<SectionRevisionId>,
    pub frozen_dependencies: BTreeMap<SectionKey, SectionRevisionId>,
}

impl PublishRequest {
    pub fn from_revision(revision: ValidatedSectionRevision) -> Self {
        Self {
            expected_current: revision.context().expected_current(),
            frozen_dependencies: revision.context().dependencies().clone(),
            revision,
        }
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionRecord {
    pub source_scope: SourceScopeId,
    pub section_key: SectionKey,
    pub revision: SectionRevisionId,
    pub records: Vec<EnvelopeRecord>,
    pub manifest_digest: [u8; 32],
    pub content_digest: [u8; 32],
    pub context_token: String,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicationReceipt {
    pub section_key: SectionKey,
    pub revision: SectionRevisionId,
    pub content_digest: [u8; 32],
    pub previous: Option<SectionRevisionId>,
    pub ordinal: u64,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrentRevision { pub revision: RevisionRecord, pub receipt: PublicationReceipt }

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepositoryDiagnostic { pub code: &'static str }

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepositoryError {
    MissingRevision(RepositoryDiagnostic),
    PinConflict(RepositoryDiagnostic),
    StalePin(RepositoryDiagnostic),
    Corrupt(RepositoryDiagnostic),
    Storage(RepositoryDiagnostic),
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionPinReceipt { pub decision: DecisionSnapshotId, pub ordinal: u64 }

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionRevisionPin {
    pub receipt: DecisionPinReceipt,
    pub revisions: BTreeMap<SectionKey, SectionRevisionId>,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnpinOutcome { Unpinned, AlreadyAbsent, StaleReceipt }

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PublishOutcome {
    CommittedNew(PublicationReceipt),
    CommittedReplay(PublicationReceipt),
    Conflict(RepositoryDiagnostic),
    StalePointer(RepositoryDiagnostic),
    StaleDependency(RepositoryDiagnostic),
    PermanentRejection(RepositoryDiagnostic),
    Ambiguous(RepositoryDiagnostic),
}

pub trait ObservationRepository {
    fn publish(&mut self, request: PublishRequest) -> PublishOutcome;
    fn current(&self, key: &SectionKey) -> Result<Option<CurrentRevision>, RepositoryError>;
    fn pin_decision(&mut self, set: &DecisionRevisionSet) -> Result<DecisionPinReceipt, RepositoryError>;
    fn load_decision_pin(&self, decision: &DecisionSnapshotId) -> Result<DecisionRevisionPin, RepositoryError>;
    fn unpin_decision(&mut self, receipt: &DecisionPinReceipt) -> Result<UnpinOutcome, RepositoryError>;
}

pub struct FakeObservationRepository { limits: PublicationLimits }

impl FakeObservationRepository {
    pub const fn new(limits: PublicationLimits) -> Self { Self { limits } }
}

impl ObservationRepository for FakeObservationRepository {
    fn publish(&mut self, _: PublishRequest) -> PublishOutcome {
        let _ = self.limits;
        PublishOutcome::PermanentRejection(RepositoryDiagnostic { code: "not-implemented" })
    }
    fn current(&self, _: &SectionKey) -> Result<Option<CurrentRevision>, RepositoryError> { Ok(None) }
    fn pin_decision(&mut self, _: &DecisionRevisionSet) -> Result<DecisionPinReceipt, RepositoryError> {
        Err(RepositoryError::MissingRevision(RepositoryDiagnostic { code: "not-implemented" }))
    }
    fn load_decision_pin(&self, _: &DecisionSnapshotId) -> Result<DecisionRevisionPin, RepositoryError> {
        Err(RepositoryError::MissingRevision(RepositoryDiagnostic { code: "pin-not-found" }))
    }
    fn unpin_decision(&mut self, _: &DecisionPinReceipt) -> Result<UnpinOutcome, RepositoryError> {
        Ok(UnpinOutcome::AlreadyAbsent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{key, validated};

    #[test]
    fn publishes_genesis_and_replays_exact_receipt() {
        let limits = PublicationLimits::new(4, 256).expect("limits are non-zero");
        let mut repository = FakeObservationRepository::new(limits);
        let request = PublishRequest::from_revision(validated(1, None));
        let first = repository.publish(request.clone());
        assert!(matches!(first, PublishOutcome::CommittedNew(_)));
        assert!(matches!(repository.publish(request), PublishOutcome::CommittedReplay(_)));
        assert!(repository.current(&key("ships")).is_ok_and(|value| value.is_some()));
    }
}
