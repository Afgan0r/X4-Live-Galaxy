#![allow(
    dead_code,
    reason = "each integration-test crate uses a different fixture subset"
)]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "invalid integration fixtures must fail immediately"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use observation_domain::{
    CanonicalizationVersion, CaptureWindow, CompletionCoverage, DigestAlgorithmVersion,
    ObservationPolicyVersion, ObservationSchemaVersion, ProducerIncarnationId, SectionAvailability,
    SectionCompletionEnvelope, SectionCoverage, SectionFreshness, SectionKey, SectionQuality,
    SectionRevisionId, SectionStartEnvelope, SectionState, SourceScopeId, TransportEpoch,
};
use observation_ingest::{
    AcceptedProjection, AggregateLimits, CandidateContext, CandidateLimits, CompletionCurrent,
    CompletionOutcome, ContractVersions, DecisionEligibility, DecisionRevisionIndex,
    DecisionRevisionSet, GenerationLimits, GenerationStager, ReceiverDisposition,
    ValidatedSectionRevision,
};

static NEXT_PATH: AtomicU64 = AtomicU64::new(1);

pub fn key(value: &str) -> SectionKey {
    SectionKey::new(value).expect("fixture key is valid")
}

pub const fn revision(value: u64) -> SectionRevisionId {
    SectionRevisionId::new(value).expect("fixture revision is non-zero")
}

pub fn validated(
    section: &str,
    value: u64,
    expected: Option<SectionRevisionId>,
    dependencies: BTreeMap<SectionKey, SectionRevisionId>,
) -> ValidatedSectionRevision {
    let section_key = key(section);
    let source_scope = SourceScopeId::new("scope:x4").expect("fixture scope is valid");
    let context = CandidateContext::new(
        ContractVersions::new(
            ObservationSchemaVersion::new(1).expect("version is non-zero"),
            ObservationPolicyVersion::new(2).expect("version is non-zero"),
            CanonicalizationVersion::new(3).expect("version is non-zero"),
            DigestAlgorithmVersion::new(1).expect("version is non-zero"),
        ),
        CaptureWindow::new(10, 20).expect("fixture window is ordered"),
        SectionState::with_evidence(
            CaptureWindow::new(10, 20).expect("fixture window is ordered"),
            SectionFreshness::Fresh,
            SectionQuality::KnownEmpty,
            SectionAvailability::Available,
            SectionCoverage::KnownEmpty,
        ),
        dependencies.clone(),
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
            .expect("fixture producer is valid"),
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
            .expect("fixture producer is valid"),
        transport_epoch: TransportEpoch::new(1).expect("epoch is non-zero"),
        section_key,
        section_revision: revision(value),
        record_count: 0,
        coverage: CompletionCoverage::KnownEmpty,
    };
    let certificate = stager
        .completion_certificate(envelope)
        .expect("fixture candidate exists");
    match stager.complete_section(
        &certificate,
        &CompletionCurrent::new(dependencies, expected),
        2,
    ) {
        CompletionOutcome::Validated(revision) => *revision,
        CompletionOutcome::Rejected(reason) => panic!("fixture completion failed: {reason:?}"),
    }
}

pub fn decision_set(revisions: Vec<ValidatedSectionRevision>) -> DecisionRevisionSet {
    let required: Vec<_> = revisions
        .iter()
        .map(|revision| revision.section_key().clone())
        .collect();
    let mut index = DecisionRevisionIndex::new(required.len()).expect("set is non-empty");
    for revision in revisions {
        index.accept(revision, 1);
    }
    match index.eligibility(&required, 1, 1) {
        DecisionEligibility::Eligible(set) => set,
        DecisionEligibility::Blocked(blockers) => panic!("fixture set blocked: {blockers:?}"),
    }
}

pub struct TempDatabase(PathBuf);

impl TempDatabase {
    pub fn new(label: &str) -> Self {
        let ordinal = NEXT_PATH.fetch_add(1, Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!(
            "live-galaxy-{label}-{}-{ordinal}.sqlite3",
            std::process::id()
        )))
    }
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDatabase {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
        for suffix in ["-journal", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", self.0.display()));
        }
    }
}
