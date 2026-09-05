use observation_ingest::{DurableRevisionError, DurableRevisionParts, ValidatedSectionRevision};

use crate::{CurrentRevision, RevisionRecord};

impl RevisionRecord {
    pub fn hydrate(&self) -> Result<ValidatedSectionRevision, DurableRevisionError> {
        ValidatedSectionRevision::try_from_durable_parts(DurableRevisionParts {
            source_scope: self.source_scope.clone(),
            source_session: self.source_session.clone(),
            section_key: self.section_key.clone(),
            section_revision: self.revision,
            records: self.records.clone(),
            coverage: self.coverage,
            context: self
                .context
                .candidate(self.dependencies.clone(), self.expected_current),
            manifest_digest: self.manifest_digest,
            content_digest: self.content_digest,
        })
    }
}

impl CurrentRevision {
    pub fn hydrate(&self) -> Result<ValidatedSectionRevision, DurableRevisionError> {
        self.revision.hydrate()
    }
}
