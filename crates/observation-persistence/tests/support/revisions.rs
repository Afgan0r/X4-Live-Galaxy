use super::*;
use observation_domain::{
    BatchId, CanonicalizationVersion, CaptureWindow, CompletionCoverage, DigestAlgorithmVersion,
    ImmutableBatchEnvelope, ObservationPolicyVersion, ObservationSchemaVersion,
    ProducerIncarnationId, SectionAvailability, SectionCompletionEnvelope, SectionCoverage,
    SectionFreshness, SectionQuality, SectionStartEnvelope, SectionState, SourceScopeId,
    TransportEpoch,
};
use observation_ingest::{
    AcceptedProjection, AggregateLimits, CandidateContext, CandidateLimits, CompletionCurrent,
    CompletionOutcome, ContractVersions, GenerationLimits, GenerationStager, ReceiverDisposition,
};

#[derive(Clone, Copy)]
pub struct RevisionFixture {
    pub source_scope: &'static str,
    pub producer_incarnation: &'static str,
    pub transport_epoch: u64,
    pub coverage: SectionCoverage,
    pub quality: SectionQuality,
    pub capture_start: u64,
    pub batch_id: Option<&'static str>,
}

impl Default for RevisionFixture {
    fn default() -> Self {
        Self {
            source_scope: "scope:x4",
            producer_incarnation: "producer:1",
            transport_epoch: 1,
            coverage: SectionCoverage::KnownEmpty,
            quality: SectionQuality::KnownEmpty,
            capture_start: 10,
            batch_id: None,
        }
    }
}

pub fn validated(
    section: &str,
    value: u64,
    expected: Option<SectionRevisionId>,
    dependencies: BTreeMap<SectionKey, SectionRevisionId>,
) -> ValidatedSectionRevision {
    validated_with(
        section,
        value,
        expected,
        dependencies,
        RevisionFixture::default(),
    )
}

pub fn validated_with(
    section: &str,
    value: u64,
    expected: Option<SectionRevisionId>,
    dependencies: BTreeMap<SectionKey, SectionRevisionId>,
    fixture: RevisionFixture,
) -> ValidatedSectionRevision {
    let section_key = key(section);
    let source_scope = SourceScopeId::new(fixture.source_scope).expect("fixture scope is valid");
    let context = fixture_context(fixture, dependencies.clone(), expected);
    let limits = GenerationLimits::bounded(
        CandidateLimits::new(1_024, 2_048, 1, 1, 1, 100, 10).expect("limits are non-zero"),
        AggregateLimits::new(1, 1_024, 2_048, 1, 1, 1).expect("limits are non-zero"),
    );
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits);
    let producer =
        ProducerIncarnationId::new(fixture.producer_incarnation).expect("producer is valid");
    let epoch = TransportEpoch::new(fixture.transport_epoch).expect("epoch is non-zero");
    let start = SectionStartEnvelope {
        source_scope: source_scope.clone(),
        producer_incarnation: producer.clone(),
        transport_epoch: epoch,
        section_key: section_key.clone(),
        section_revision: revision(value),
        expected_records: 0,
    };
    assert_eq!(
        stager.start_section_with_context(start, context, 1),
        ReceiverDisposition::Received
    );
    if let Some(batch_id) = fixture.batch_id {
        let batch = ImmutableBatchEnvelope {
            source_scope: source_scope.clone(),
            producer_incarnation: producer.clone(),
            transport_epoch: epoch,
            section_key: section_key.clone(),
            section_revision: revision(value),
            batch_id: BatchId::new(batch_id).expect("batch identity is valid"),
            records: Vec::new(),
            optional_detail: None,
        };
        assert_eq!(
            stager.stage_section_batch(batch, 1, 2),
            ReceiverDisposition::Received
        );
    }
    let envelope = SectionCompletionEnvelope {
        source_scope,
        producer_incarnation: producer,
        transport_epoch: epoch,
        section_key,
        section_revision: revision(value),
        record_count: 0,
        coverage: terminal_coverage(fixture.coverage),
    };
    let certificate = stager
        .completion_certificate(envelope)
        .expect("fixture candidate exists");
    match stager.complete_section(
        &certificate,
        &CompletionCurrent::new(dependencies, expected),
        3,
    ) {
        CompletionOutcome::Validated(revision) => *revision,
        CompletionOutcome::Rejected(reason) => panic!("fixture completion failed: {reason:?}"),
    }
}

const fn fixture_context(
    fixture: RevisionFixture,
    dependencies: BTreeMap<SectionKey, SectionRevisionId>,
    expected: Option<SectionRevisionId>,
) -> CandidateContext {
    let window = CaptureWindow::new(fixture.capture_start, 20).expect("window is ordered");
    CandidateContext::new(
        ContractVersions::new(
            ObservationSchemaVersion::new(1).expect("version is non-zero"),
            ObservationPolicyVersion::new(2).expect("version is non-zero"),
            CanonicalizationVersion::new(3).expect("version is non-zero"),
            DigestAlgorithmVersion::new(1).expect("version is non-zero"),
        ),
        window,
        SectionState::with_evidence(
            window,
            SectionFreshness::Fresh,
            fixture.quality,
            SectionAvailability::Available,
            fixture.coverage,
        ),
        dependencies,
        expected,
        true,
    )
}

const fn terminal_coverage(value: SectionCoverage) -> CompletionCoverage {
    match value {
        SectionCoverage::Complete => CompletionCoverage::Complete,
        SectionCoverage::KnownEmpty => CompletionCoverage::KnownEmpty,
        SectionCoverage::Partial => CompletionCoverage::Partial,
        SectionCoverage::Unknown => CompletionCoverage::Unknown,
        SectionCoverage::Unsupported => CompletionCoverage::Unsupported,
    }
}
