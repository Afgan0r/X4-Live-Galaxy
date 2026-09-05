#![allow(dead_code, reason = "contract cases use focused fixture subsets")]
#![expect(
    clippy::expect_used,
    reason = "invalid integration fixtures must fail immediately"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use observation_domain::{
    BatchId, CanonicalizationVersion, CaptureWindow, DigestAlgorithmVersion,
    ObservationPolicyVersion, ObservationSchemaVersion, SectionAvailability, SectionCoverage,
    SectionFreshness, SectionKey, SectionQuality, SectionRevisionId, SectionState, TransportEpoch,
};
use observation_ingest::{
    AcceptedProjection, AggregateLimits, CandidateContext, CandidateLimits, CompletionCurrent,
    ContractVersions, GenerationLimits, GenerationStager,
};
use observation_persistence::{PublicationLimits, SqliteObservationRepository};

#[path = "support/flow.rs"]
pub mod flow;
#[path = "support/repository.rs"]
pub mod repository_support;

static NEXT_PATH: AtomicU64 = AtomicU64::new(1);

pub fn key(value: &str) -> SectionKey {
    SectionKey::new(value).expect("fixture key is valid")
}

pub const fn revision(value: u64) -> SectionRevisionId {
    SectionRevisionId::new(value).expect("fixture revision is non-zero")
}

pub const fn epoch() -> TransportEpoch {
    TransportEpoch::new(1).expect("fixture epoch is non-zero")
}

pub fn batch_id(value: &str) -> BatchId {
    BatchId::new(value).expect("fixture batch identity is valid")
}

pub const fn candidate_context(coverage: SectionCoverage) -> CandidateContext {
    CandidateContext::new(
        ContractVersions::new(
            ObservationSchemaVersion::new(1).expect("version is non-zero"),
            ObservationPolicyVersion::new(2).expect("version is non-zero"),
            CanonicalizationVersion::new(3).expect("version is non-zero"),
            DigestAlgorithmVersion::new(1).expect("version is non-zero"),
        ),
        CaptureWindow::new(10, 20).expect("window is ordered"),
        SectionState::with_evidence(
            CaptureWindow::new(10, 20).expect("window is ordered"),
            SectionFreshness::Fresh,
            SectionQuality::Fresh,
            SectionAvailability::Available,
            coverage,
        ),
        BTreeMap::new(),
        None,
        true,
    )
}

pub const fn current() -> CompletionCurrent {
    CompletionCurrent::new(BTreeMap::new(), None)
}

#[must_use]
pub fn stager() -> GenerationStager {
    let candidate = CandidateLimits::new(4_096, 8_192, 16, 16, 32, 100, 10)
        .expect("candidate limits are non-zero");
    let aggregate = AggregateLimits::new(4, 16_384, 32_768, 64, 64, 128)
        .expect("aggregate limits are non-zero");
    GenerationStager::new(
        AcceptedProjection::empty(),
        GenerationLimits::bounded(candidate, aggregate),
    )
}

#[must_use]
pub fn repository(label: &str) -> (TempDatabase, SqliteObservationRepository) {
    let database = TempDatabase::new(label);
    let limits = PublicationLimits::new(16, 8_192).expect("publication limits are non-zero");
    let repository = SqliteObservationRepository::open(database.path(), limits)
        .expect("SQLite repository opens");
    (database, repository)
}

#[must_use]
pub fn start_bytes(section: &str, expected: usize) -> Vec<u8> {
    format!(
        "{{\"type\":\"section_start\",\"contract_version\":1,\"source_scope\":\"scope:x4\",\"producer_incarnation\":\"producer:1\",\"transport_epoch\":1,\"section_key\":\"{section}\",\"section_revision\":1,\"expected_records\":{expected}}}"
    )
    .into_bytes()
}

#[must_use]
pub fn batch_bytes(section: &str, records: &[(&str, &str)]) -> Vec<u8> {
    let records = records
        .iter()
        .map(|(record, entity)| {
            format!(
                "{{\"record_id\":\"{record}\",\"entity_id\":\"{entity}\",\"observation_version\":1,\"content\":\"content:{entity}\"}}"
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"type\":\"immutable_batch\",\"contract_version\":1,\"source_scope\":\"scope:x4\",\"producer_incarnation\":\"producer:1\",\"transport_epoch\":1,\"section_key\":\"{section}\",\"section_revision\":1,\"batch_id\":\"inner:{section}\",\"records\":[{records}],\"optional_detail\":null}}"
    )
    .into_bytes()
}

#[must_use]
pub fn completion_bytes(section: &str, count: usize, coverage: &str) -> Vec<u8> {
    format!(
        "{{\"type\":\"section_completion\",\"contract_version\":1,\"source_scope\":\"scope:x4\",\"producer_incarnation\":\"producer:1\",\"transport_epoch\":1,\"section_key\":\"{section}\",\"section_revision\":1,\"record_count\":{count},\"coverage\":\"{coverage}\"}}"
    )
    .into_bytes()
}

pub struct TempDatabase(PathBuf);

impl TempDatabase {
    fn new(label: &str) -> Self {
        let ordinal = NEXT_PATH.fetch_add(1, Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!(
            "live-galaxy-app-{label}-{}-{ordinal}.sqlite3",
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
