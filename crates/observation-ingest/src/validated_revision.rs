use observation_domain::{
    CompletionCoverage, EnvelopeRecord, SectionKey, SectionRevisionId, SourceScopeId,
    SourceSessionIdentity,
};

use crate::CandidateContext;

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
/// Authority created only by live completion or validated hydration.
///
/// ```compile_fail
/// use observation_ingest::{DurableRevisionParts, ValidatedSectionRevision};
/// fn bypass(parts: DurableRevisionParts) -> ValidatedSectionRevision {
///     ValidatedSectionRevision::from_durable_parts(parts)
/// }
/// ```
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

#[must_use]
pub struct DurableRevisionParts {
    pub source_scope: SourceScopeId,
    pub source_session: SourceSessionIdentity,
    pub section_key: SectionKey,
    pub section_revision: SectionRevisionId,
    pub records: Vec<EnvelopeRecord>,
    pub coverage: CompletionCoverage,
    pub context: CandidateContext,
    pub manifest_digest: [u8; 32],
    pub content_digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableRevisionError {
    ContractVersion,
    ContextEvidence,
    Coverage,
    RecordOrder,
    EntityVersion,
    ContentDigest,
    RevisionOrder,
}

impl ValidatedSectionRevision {
    pub fn try_from_durable_parts(
        parts: DurableRevisionParts,
    ) -> Result<Self, DurableRevisionError> {
        crate::hydration_validation::validate(&parts)?;
        Ok(Self {
            source_scope: parts.source_scope,
            source_session: parts.source_session,
            section_key: parts.section_key,
            section_revision: parts.section_revision,
            records: parts.records,
            coverage: parts.coverage,
            context: parts.context,
            manifest_digest: parts.manifest_digest,
            content_digest: parts.content_digest,
        })
    }

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
