use observation_domain::{
    CompletionCoverage, EnvelopeRecord, SectionKey, SectionRevisionId, SourceScopeId,
    SourceSessionIdentity,
};

use crate::CandidateContext;

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedSectionRevision {
    pub(crate) source_scope: SourceScopeId,
    pub(crate) source_session: SourceSessionIdentity,
    pub(crate) section_key: SectionKey,
    pub(crate) section_revision: SectionRevisionId,
    pub(crate) records: Vec<EnvelopeRecord>,
    pub(crate) coverage: CompletionCoverage,
    pub(crate) context: CandidateContext,
    pub(crate) manifest_digest: [u8; 32],
    pub(crate) content_digest: [u8; 32],
}

impl ValidatedSectionRevision {
    pub const fn source_scope(&self) -> &SourceScopeId {
        &self.source_scope
    }
    pub const fn source_session(&self) -> &SourceSessionIdentity {
        &self.source_session
    }
    pub const fn section_key(&self) -> &SectionKey {
        &self.section_key
    }
    pub const fn section_revision(&self) -> SectionRevisionId {
        self.section_revision
    }
    #[must_use]
    pub fn records(&self) -> &[EnvelopeRecord] {
        &self.records
    }
    pub const fn coverage(&self) -> CompletionCoverage {
        self.coverage
    }
    pub const fn context(&self) -> &CandidateContext {
        &self.context
    }
    #[must_use]
    pub const fn manifest_digest(&self) -> &[u8; 32] {
        &self.manifest_digest
    }
    #[must_use]
    pub const fn content_digest(&self) -> &[u8; 32] {
        &self.content_digest
    }
    #[must_use]
    pub const fn is_published(&self) -> bool {
        false
    }
}
