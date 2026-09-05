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
        let receipt = &self.receipt;
        let revision = &self.revision;
        if receipt.section_key != revision.section_key
            || receipt.revision != revision.revision
            || receipt.content_digest != revision.content_digest
            || receipt.previous != revision.expected_current
            || receipt.accepted_at != revision.accepted_at
        {
            return Err(DurableRevisionError::ReceiptBinding);
        }
        self.revision.hydrate()
    }
}

#[cfg(test)]
mod tests {
    use crate::{CurrentRevision, PublicationReceipt, record, test_support};
    use observation_ingest::{DecisionRevisionIndex, DurableRevisionError};
    #[test]
    fn receipt_must_bind_the_hydrated_revision() {
        let revision = test_support::validated(1, None);
        let mut index = DecisionRevisionIndex::new(1).expect("limit is non-zero");
        let accepted = index
            .prepare_publication(revision)
            .expect("revision prepares");
        let request = crate::PublishRequest::from_accepted(accepted, 7);
        let record = record::normalize(
            &request,
            crate::PublicationLimits::new(4, 256).expect("limits are non-zero"),
        )
        .expect("revision normalizes");
        let current = CurrentRevision {
            receipt: PublicationReceipt {
                section_key: record.section_key.clone(),
                revision: record.revision,
                content_digest: record.content_digest,
                previous: record.expected_current,
                ordinal: 1,
                accepted_at: 8,
            },
            revision: record,
        };
        assert_eq!(current.hydrate(), Err(DurableRevisionError::ReceiptBinding));
    }
}
