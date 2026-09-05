use std::collections::BTreeMap;

use observation_domain::{SectionKey, SectionRevisionId, SourceScopeId, SourceSessionIdentity};
use observation_ingest::{AcceptedPublication, ValidatedSectionRevision};

#[must_use]
#[derive(Clone, Debug)]
pub struct PublishRequest {
    accepted: AcceptedPublication,
    attempt_identity: PublishAttemptIdentity,
    accepted_at: u64,
}

impl PublishRequest {
    pub fn from_accepted(accepted: AcceptedPublication, accepted_at: u64) -> Self {
        let revision = accepted.revision();
        let attempt_identity = PublishAttemptIdentity::new(
            revision.source_scope().clone(),
            revision.source_session().clone(),
            revision.section_key().clone(),
            revision.section_revision(),
            *revision.content_digest(),
            revision.context().expected_current(),
            revision.context().dependencies().clone(),
        );
        Self {
            accepted,
            attempt_identity,
            accepted_at,
        }
    }

    pub(crate) const fn revision(&self) -> &ValidatedSectionRevision {
        self.accepted.revision()
    }

    pub(crate) const fn expected_current(&self) -> Option<SectionRevisionId> {
        self.revision().context().expected_current()
    }

    pub(crate) const fn frozen_dependencies(&self) -> &BTreeMap<SectionKey, SectionRevisionId> {
        self.revision().context().dependencies()
    }

    pub(crate) fn is_authoritative(&self) -> bool {
        self.accepted.is_authoritative()
    }

    pub(crate) const fn accepted_at(&self) -> u64 {
        self.accepted_at
    }

    pub const fn attempt_identity(&self) -> &PublishAttemptIdentity {
        &self.attempt_identity
    }

    #[must_use]
    pub fn retained_bytes(&self) -> Option<usize> {
        let revision = self.revision();
        let fixed_revision_bytes = 32usize
            .checked_add(4 * size_of::<u64>())?
            .checked_add(2 * size_of::<u64>())?;
        revision.records().iter().try_fold(
            self.attempt_identity
                .retained_bytes()?
                .checked_add(fixed_revision_bytes)?,
            |total, record| {
                total
                    .checked_add(record.record_id.as_str().len())?
                    .checked_add(record.entity_id.as_str().len())?
                    .checked_add(size_of::<u64>())?
                    .checked_add(record.content.len())
            },
        )
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishAttemptIdentity {
    source_scope: SourceScopeId,
    source_session: SourceSessionIdentity,
    section_key: SectionKey,
    revision: SectionRevisionId,
    content_digest: [u8; 32],
    expected_current: Option<SectionRevisionId>,
    dependencies: BTreeMap<SectionKey, SectionRevisionId>,
}

impl PublishAttemptIdentity {
    pub const fn new(
        source_scope: SourceScopeId,
        source_session: SourceSessionIdentity,
        section_key: SectionKey,
        revision: SectionRevisionId,
        content_digest: [u8; 32],
        expected_current: Option<SectionRevisionId>,
        dependencies: BTreeMap<SectionKey, SectionRevisionId>,
    ) -> Self {
        Self {
            source_scope,
            source_session,
            section_key,
            revision,
            content_digest,
            expected_current,
            dependencies,
        }
    }

    fn retained_bytes(&self) -> Option<usize> {
        self.dependencies.iter().try_fold(
            self.source_scope
                .as_str()
                .len()
                .checked_add(self.source_session.producer_incarnation().as_str().len())?
                .checked_add(size_of::<u64>())?
                .checked_add(self.section_key.as_str().len())?
                .checked_add(size_of::<u64>())?
                .checked_add(self.content_digest.len())?
                .checked_add(size_of::<u64>())?,
            |total, (key, _)| {
                total
                    .checked_add(key.as_str().len())?
                    .checked_add(size_of::<u64>())
            },
        )
    }
}
