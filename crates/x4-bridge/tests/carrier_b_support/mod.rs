#![allow(
    dead_code,
    unused_imports,
    reason = "publication and recovery tests share fixture subsets"
)]
#![expect(clippy::expect_used, reason = "invalid fixtures must fail immediately")]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use observation_application::{LifecycleContext, LifecycleInput, LifecycleLimits};
use observation_domain::{
    BatchId, CanonicalizationVersion, CaptureWindow, CompletionCoverage, DigestAlgorithmVersion,
    ObservationPolicyVersion, ObservationSchemaVersion, ProducerIncarnationId, SectionAvailability,
    SectionCompletionEnvelope, SectionCoverage, SectionFreshness, SectionKey, SectionQuality,
    SectionRevisionId, SectionState, SourceScopeId, TransportEpoch,
};
use observation_ingest::{
    AggregateLimits, CandidateContext, CandidateLimits, CompletionCurrent, ContractVersions,
    GenerationLimits,
};
use observation_persistence::PublicationLimits;

mod repository;
pub use repository::{AmbiguousRepository, FirstPublish};

static NEXT_PATH: AtomicU64 = AtomicU64::new(1);

pub fn database(label: &str) -> TempDatabase {
    TempDatabase::new(label)
}

pub fn generation_limits() -> GenerationLimits {
    GenerationLimits::bounded(
        CandidateLimits::new(4_096, 16, 16, 32, 100, 10).expect("candidate limits"),
        AggregateLimits::new(4, 16_384, 64, 64, 128).expect("aggregate limits"),
    )
}

pub const fn publication_limits() -> PublicationLimits {
    PublicationLimits::new(16, 8_192).expect("publication limits")
}

pub fn lifecycle_limits() -> LifecycleLimits {
    LifecycleLimits::new(4_096, 16_384, 1_000, 4).expect("lifecycle limits")
}

pub fn context(coverage: SectionCoverage) -> CandidateContext {
    context_at(coverage, None)
}

pub fn context_at(coverage: SectionCoverage, expected: Option<u64>) -> CandidateContext {
    CandidateContext::new(
        versions(),
        CaptureWindow::new(10, 20).expect("capture window"),
        SectionState::with_evidence(
            CaptureWindow::new(10, 20).expect("capture window"),
            SectionFreshness::Fresh,
            SectionQuality::Fresh,
            SectionAvailability::Available,
            coverage,
        ),
        BTreeMap::new(),
        expected.and_then(SectionRevisionId::new),
        true,
    )
}

pub fn input(
    identity: &str,
    bytes: Vec<u8>,
    context: LifecycleContext,
    now: u64,
) -> LifecycleInput {
    LifecycleInput::new(
        TransportEpoch::new(1).expect("epoch"),
        BatchId::new(identity).expect("batch identity"),
        bytes,
        1,
        now,
        context,
    )
}

pub fn start_bytes(section: &str) -> Vec<u8> {
    start_bytes_at(section, 1)
}

pub fn start_bytes_at(section: &str, revision: u64) -> Vec<u8> {
    format!(
        "{{\"type\":\"section_start\",\"contract_version\":2,\"source_scope\":\"scope:x4\",\"producer_incarnation\":\"producer:1\",\"transport_epoch\":1,\"section_key\":\"{section}\",\"section_revision\":{revision},\"expected_records\":0,\"sender_evidence\":{{\"capture_clock\":\"game_time_millis\",\"capture_start_millis\":0,\"capture_end_millis\":0,\"freshness\":\"fresh\",\"quality\":\"unknown\",\"availability\":\"available\",\"coverage\":\"complete\",\"source_epoch\":null,\"source_epoch_status\":\"unknown\",\"source_boundary\":\"unknown\",\"source_consistency\":\"unknown\",\"stable_identity\":false,\"schema_version\":1,\"policy_version\":2,\"canonicalization_version\":3,\"digest_version\":1}}}}"
    )
    .into_bytes()
}

pub fn completion_bytes(section: &str) -> Vec<u8> {
    completion_bytes_at(section, 1)
}

pub fn completion_bytes_at(section: &str, revision: u64) -> Vec<u8> {
    let envelope = SectionCompletionEnvelope {
        source_scope: SourceScopeId::new("scope:x4").expect("scope"),
        producer_incarnation: ProducerIncarnationId::new("producer:1").expect("producer"),
        transport_epoch: TransportEpoch::new(1).expect("epoch"),
        section_key: SectionKey::new(section).expect("section"),
        section_revision: SectionRevisionId::new(revision).expect("revision"),
        batch_count: 0,
        record_count: 0,
        raw_bytes: 0,
        ordered_batch_manifest_digest: [0; 32],
        canonical_content_digest: [0; 32],
        schema_version: ObservationSchemaVersion::new(1).expect("schema"),
        policy_version: ObservationPolicyVersion::new(2).expect("policy"),
        canonicalization_version: CanonicalizationVersion::new(3).expect("canonicalization"),
        digest_version: DigestAlgorithmVersion::new(1).expect("digest"),
        coverage: CompletionCoverage::KnownEmpty,
        sender_evidence: observation_domain::SenderEvidence::legacy_default(),
    };
    let bound = observation_ingest::bind_completion_certificate(envelope, &[], versions())
        .expect("completion binds");
    let manifest = digest_hex(bound.ordered_batch_manifest_digest);
    let content = digest_hex(bound.canonical_content_digest);
    format!(
        "{{\"type\":\"section_completion\",\"contract_version\":2,\"source_scope\":\"scope:x4\",\"producer_incarnation\":\"producer:1\",\"transport_epoch\":1,\"section_key\":\"{section}\",\"section_revision\":{revision},\"batch_count\":0,\"record_count\":0,\"raw_bytes\":0,\"ordered_batch_manifest_digest\":\"{manifest}\",\"canonical_content_digest\":\"{content}\",\"schema_version\":1,\"policy_version\":2,\"canonicalization_version\":3,\"digest_version\":1,\"coverage\":\"known_empty\",\"sender_evidence\":{{\"capture_clock\":\"game_time_millis\",\"capture_start_millis\":0,\"capture_end_millis\":0,\"freshness\":\"fresh\",\"quality\":\"unknown\",\"availability\":\"available\",\"coverage\":\"complete\",\"source_epoch\":null,\"source_epoch_status\":\"unknown\",\"source_boundary\":\"unknown\",\"source_consistency\":\"unknown\",\"stable_identity\":false,\"schema_version\":1,\"policy_version\":2,\"canonicalization_version\":3,\"digest_version\":1}}}}"
    )
    .into_bytes()
}

fn digest_hex(digest: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

pub const fn current() -> CompletionCurrent {
    CompletionCurrent::new(BTreeMap::new(), None)
}

pub const fn current_at(expected: u64) -> CompletionCurrent {
    CompletionCurrent::new(BTreeMap::new(), SectionRevisionId::new(expected))
}

pub fn session_with<R: observation_persistence::ObservationRepository>(
    repository: R,
) -> x4_bridge::ProductionObservationSession<R> {
    x4_bridge::ProductionObservationSession::from_repository(
        repository,
        generation_limits(),
        lifecycle_limits(),
        4,
    )
    .expect("production session opens")
}

const fn versions() -> ContractVersions {
    ContractVersions::new(
        ObservationSchemaVersion::new(1).expect("schema"),
        ObservationPolicyVersion::new(2).expect("policy"),
        CanonicalizationVersion::new(3).expect("canonicalization"),
        DigestAlgorithmVersion::new(1).expect("digest"),
    )
}

pub struct TempDatabase(PathBuf);

impl TempDatabase {
    fn new(label: &str) -> Self {
        let ordinal = NEXT_PATH.fetch_add(1, Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!(
            "live-galaxy-carrier-b-{label}-{}-{ordinal}.sqlite3",
            std::process::id()
        )))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDatabase {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
