use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use observation_domain::{
    CompletionCoverage, DecisionSnapshotId, EnvelopeRecord, SectionKey, SectionRevisionId,
    SourceScopeId, SourceSessionIdentity,
};
use observation_ingest::ValidatedSectionRevision;

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
pub struct PublishRequest {
    pub revision: ValidatedSectionRevision,
    pub expected_current: Option<SectionRevisionId>,
    pub frozen_dependencies: BTreeMap<SectionKey, SectionRevisionId>,
    pub authoritative_session: SourceSessionIdentity,
}

impl PublishRequest {
    pub fn from_revision(
        revision: ValidatedSectionRevision,
        authoritative_session: SourceSessionIdentity,
    ) -> Self {
        Self {
            expected_current: revision.context().expected_current(),
            frozen_dependencies: revision.context().dependencies().clone(),
            authoritative_session,
            revision,
        }
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
