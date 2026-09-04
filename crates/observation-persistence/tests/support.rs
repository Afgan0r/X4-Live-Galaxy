#![allow(
    dead_code,
    unused_imports,
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

use observation_domain::{SectionKey, SectionRevisionId};
use observation_ingest::{
    DecisionEligibility, DecisionRevisionIndex, DecisionRevisionSet, ValidatedSectionRevision,
};
use observation_persistence::PublishRequest;

#[path = "support/revisions.rs"]
mod revisions;
pub use revisions::{RevisionFixture, validated, validated_with};

static NEXT_PATH: AtomicU64 = AtomicU64::new(1);

pub fn key(value: &str) -> SectionKey {
    SectionKey::new(value).expect("fixture key is valid")
}

pub const fn revision(value: u64) -> SectionRevisionId {
    SectionRevisionId::new(value).expect("fixture revision is non-zero")
}

pub fn publish_request(revision: ValidatedSectionRevision) -> PublishRequest {
    let mut index = DecisionRevisionIndex::new(1).expect("blocker limit is non-zero");
    let accepted = index
        .accept(revision, 1)
        .expect("fixture revision is authoritative");
    PublishRequest::from_accepted(accepted)
}

pub fn decision_set(revisions: Vec<ValidatedSectionRevision>) -> DecisionRevisionSet {
    let required: Vec<_> = revisions
        .iter()
        .map(|revision| revision.section_key().clone())
        .collect();
    let mut index = DecisionRevisionIndex::new(required.len()).expect("set is non-empty");
    for revision in revisions {
        let _accepted = index
            .accept(revision, 1)
            .expect("fixture revision is authoritative");
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
