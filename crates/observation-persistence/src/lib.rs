#![forbid(unsafe_code)]

mod fake;
mod port;
mod record;
mod recovery;
mod retention;
mod schema;
mod sqlite;
mod sqlite_ambiguity;
mod sqlite_pins;
mod sqlite_publish;
mod sqlite_read;
mod sqlite_receipt;
mod sqlite_reconcile;
mod sqlite_write;
mod sqlite_write_rows;
mod types;

pub use fake::FakeObservationRepository;
pub use port::ObservationRepository;
pub use recovery::{PublicationFailpoint, ReconciliationOutcome, RetentionPolicy, RetentionReport};
pub use schema::{OBSERVATION_REPOSITORY_PROTOCOL_IDENTITY, OBSERVATION_REPOSITORY_SCHEMA_VERSION};
pub use sqlite::SqliteObservationRepository;
pub use types::{
    CurrentRevision, DecisionPinReceipt, DecisionRevisionPin, PublicationLimits,
    PublicationReceipt, PublishAttemptIdentity, PublishOutcome, PublishRequest,
    RepositoryDiagnostic, RepositoryError, RevisionRecord, UnpinOutcome,
};

#[cfg(test)]
pub(crate) mod test_support {
    use std::collections::BTreeMap;

    use observation_domain::{
        CanonicalizationVersion, CaptureWindow, CompletionCoverage, DigestAlgorithmVersion,
        ObservationPolicyVersion, ObservationSchemaVersion, ProducerIncarnationId,
        SectionAvailability, SectionCompletionEnvelope, SectionCoverage, SectionFreshness,
        SectionKey, SectionQuality, SectionRevisionId, SectionStartEnvelope, SectionState,
        SourceScopeId, TransportEpoch,
    };
    use observation_ingest::{
        AcceptedProjection, AggregateLimits, CandidateContext, CandidateLimits, CompletionCurrent,
        CompletionOutcome, ContractVersions, DecisionEligibility, DecisionRevisionIndex,
        DecisionRevisionSet, FinalizationOutcome, GenerationLimits, GenerationStager,
        ReceiverDisposition, ValidatedSectionRevision,
    };

    pub fn key(value: &str) -> SectionKey {
        SectionKey::new(value).expect("test key is valid")
    }

    pub const fn revision(value: u64) -> SectionRevisionId {
        SectionRevisionId::new(value).expect("test revision is non-zero")
    }

    #[expect(
        clippy::too_many_lines,
        reason = "test fixture assembles the complete publication contract"
    )]
    pub fn validated(value: u64, expected: Option<SectionRevisionId>) -> ValidatedSectionRevision {
        let section_key = key("ships");
        let source_scope = SourceScopeId::new("scope:x4").expect("test scope is valid");
        let context = CandidateContext::new(
            ContractVersions::new(
                ObservationSchemaVersion::new(1).expect("version is non-zero"),
                ObservationPolicyVersion::new(2).expect("version is non-zero"),
                CanonicalizationVersion::new(3).expect("version is non-zero"),
                DigestAlgorithmVersion::new(1).expect("version is non-zero"),
            ),
            CaptureWindow::new(10, 20).expect("test window is ordered"),
            SectionState::with_evidence(
                CaptureWindow::new(10, 20).expect("test window is ordered"),
                SectionFreshness::Fresh,
                SectionQuality::KnownEmpty,
                SectionAvailability::Available,
                SectionCoverage::KnownEmpty,
            ),
            BTreeMap::new(),
            expected,
            true,
        );
        let limits = GenerationLimits::bounded(
            CandidateLimits::new(128, 256, 1, 1, 1, 100, 10).expect("limits are non-zero"),
            AggregateLimits::new(1, 128, 256, 1, 1, 1).expect("limits are non-zero"),
        );
        let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits);
        let start = SectionStartEnvelope {
            source_scope: source_scope.clone(),
            producer_incarnation: ProducerIncarnationId::new("producer:1")
                .expect("test producer is valid"),
            transport_epoch: TransportEpoch::new(1).expect("epoch is non-zero"),
            section_key: section_key.clone(),
            section_revision: revision(value),
            expected_records: 0,
        };
        assert_eq!(
            stager.start_section_with_context(start, context, 1),
            ReceiverDisposition::Received
        );
        let envelope = SectionCompletionEnvelope {
            source_scope,
            producer_incarnation: ProducerIncarnationId::new("producer:1")
                .expect("test producer is valid"),
            transport_epoch: TransportEpoch::new(1).expect("epoch is non-zero"),
            section_key,
            section_revision: revision(value),
            batch_count: 0,
            record_count: 0,
            raw_bytes: 0,
            decoded_bytes: 0,
            ordered_batch_manifest_digest: [0; 32],
            canonical_content_digest: [0; 32],
            schema_version: ObservationSchemaVersion::new(1).expect("version is non-zero"),
            policy_version: ObservationPolicyVersion::new(2).expect("version is non-zero"),
            canonicalization_version: CanonicalizationVersion::new(3).expect("version is non-zero"),
            digest_version: DigestAlgorithmVersion::new(1).expect("version is non-zero"),
            coverage: CompletionCoverage::KnownEmpty,
        };
        let envelope = observation_ingest::bind_completion_certificate(
            envelope,
            &[],
            ContractVersions::new(
                ObservationSchemaVersion::new(1).expect("version is non-zero"),
                ObservationPolicyVersion::new(2).expect("version is non-zero"),
                CanonicalizationVersion::new(3).expect("version is non-zero"),
                DigestAlgorithmVersion::new(1).expect("version is non-zero"),
            ),
        )
        .expect("producer certificate binds");
        let certificate = stager
            .completion_certificate(envelope)
            .expect("test candidate exists");
        match stager.complete_section(
            &certificate,
            &CompletionCurrent::new(BTreeMap::new(), expected),
            2,
        ) {
            CompletionOutcome::Validated(revision) => *revision,
            CompletionOutcome::Rejected(reason) => panic!("test completion failed: {reason:?}"),
        }
    }

    pub fn decision_set(revision: ValidatedSectionRevision) -> DecisionRevisionSet {
        let mut index = DecisionRevisionIndex::new(1).expect("blocker limit is non-zero");
        let section = revision.section_key().clone();
        revision
            .context()
            .dependencies()
            .iter()
            .for_each(|(key, value)| {
                index.record_current_pointer(key.clone(), *value);
            });
        if let Some(current) = revision.context().expected_current() {
            index.record_current_pointer(section.clone(), current);
        }
        let accepted = index
            .accept(revision, 1)
            .expect("test revision is authoritative");
        assert_eq!(
            index.finalize_committed(&accepted, 1),
            FinalizationOutcome::Finalized
        );
        match index.eligibility(&[section], 1, 1) {
            DecisionEligibility::Eligible(set) => set,
            DecisionEligibility::Blocked(blockers) => panic!("test set blocked: {blockers:?}"),
        }
    }
}
