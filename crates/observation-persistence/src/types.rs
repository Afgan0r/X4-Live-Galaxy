use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use observation_domain::{
    CompletionCoverage, DecisionSnapshotId, EnvelopeRecord, SectionKey, SectionRevisionId,
    SourceScopeId, SourceSessionIdentity,
};
mod publication;
pub use publication::{PublishAttemptIdentity, PublishRequest};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublicationLimits {
    pub(crate) max_records: NonZeroUsize,
    pub(crate) max_content_bytes: NonZeroUsize,
}

impl PublicationLimits {
    #[must_use]
    pub const fn new(max_records: usize, max_content_bytes: usize) -> Option<Self> {
        Some(Self {
            max_records: match NonZeroUsize::new(max_records) {
                Some(value) => value,
                None => return None,
            },
            max_content_bytes: match NonZeroUsize::new(max_content_bytes) {
                Some(value) => value,
                None => return None,
            },
        })
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionRecord {
    pub source_scope: SourceScopeId,
    pub source_session: SourceSessionIdentity,
    pub section_key: SectionKey,
    pub revision: SectionRevisionId,
    pub records: Vec<EnvelopeRecord>,
    pub coverage: CompletionCoverage,
    pub dependencies: BTreeMap<SectionKey, SectionRevisionId>,
    pub expected_current: Option<SectionRevisionId>,
    pub manifest_digest: [u8; 32],
    pub content_digest: [u8; 32],
    pub integrity_digest: [u8; 32],
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
pub struct CurrentRevision {
    pub revision: RevisionRecord,
    pub receipt: PublicationReceipt,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepositoryDiagnostic {
    pub code: &'static str,
}

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
pub struct DecisionPinReceipt {
    pub decision: DecisionSnapshotId,
    pub ordinal: u64,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionRevisionPin {
    pub receipt: DecisionPinReceipt,
    pub revisions: BTreeMap<SectionKey, SectionRevisionId>,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnpinOutcome {
    Unpinned,
    AlreadyAbsent,
    StaleReceipt,
}

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

#[cfg(test)]
mod tests {
    use crate::test_support::validated;
    use observation_ingest::DecisionRevisionIndex;

    use super::{PublishAttemptIdentity, PublishRequest};

    #[test]
    fn publish_attempt_identity_and_retained_bytes_are_exact() {
        let revision = validated(7, Some(crate::test_support::revision(6)));
        let mut index = DecisionRevisionIndex::new(1).expect("blocker limit is non-zero");
        let accepted = index
            .prepare_publication(revision.clone())
            .expect("publication prepares");
        let request = PublishRequest::from_accepted(accepted);
        let expected = PublishAttemptIdentity::new(
            revision.source_scope().clone(),
            revision.source_session().clone(),
            revision.section_key().clone(),
            revision.section_revision(),
            *revision.content_digest(),
            revision.context().expected_current(),
            revision.context().dependencies().clone(),
        );

        assert_eq!(request.attempt_identity(), &expected);
        assert_eq!(request.retained_bytes(), Some(159));
    }
}
