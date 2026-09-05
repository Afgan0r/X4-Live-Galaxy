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
    DecisionEligibility, DecisionRevisionIndex, DecisionRevisionSet, FinalizationOutcome,
    ValidatedSectionRevision,
};
use observation_persistence::{
    ObservationRepository, PublicationLimits, PublishOutcome, PublishRequest,
    SqliteObservationRepository,
};

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
    PublishRequest::from_accepted(accepted, 3)
}

pub fn decision_set(revisions: Vec<ValidatedSectionRevision>) -> DecisionRevisionSet {
    let required: Vec<_> = revisions
        .iter()
        .map(|revision| revision.section_key().clone())
        .collect();
    let mut index = DecisionRevisionIndex::new(required.len()).expect("set is non-empty");
    for item in &revisions {
        item.context()
            .dependencies()
            .iter()
            .for_each(|(key, value)| {
                index.record_current_pointer(key.clone(), *value);
            });
        if let Some(current) = item.context().expected_current() {
            index.record_current_pointer(item.section_key().clone(), current);
        }
    }
    for item in revisions {
        let accepted = index
            .accept(item, 1)
            .expect("fixture revision is authoritative");
        assert_eq!(
            index.finalize_committed(&accepted, 1),
            FinalizationOutcome::Finalized
        );
    }
    match index.eligibility(&required, 1, 1) {
        DecisionEligibility::Eligible(set) => set,
        DecisionEligibility::Blocked(blockers) => panic!("fixture set blocked: {blockers:?}"),
    }
}

pub fn assert_durable_commit_precedes_index_finalization(limits: PublicationLimits) {
    let mut index = DecisionRevisionIndex::new(1).expect("blocker limit is non-zero");
    let accepted = index
        .prepare_publication(validated("ships", 1, None, BTreeMap::new()))
        .expect("publication prepares");
    let request = PublishRequest::from_accepted(accepted.clone(), 3);
    assert_eq!(index.current_count(), 0);
    let database = TempDatabase::new("durable-before-finalize");
    let mut repository = SqliteObservationRepository::open(database.path(), limits)
        .expect("SQLite repository opens");
    assert!(matches!(
        repository.publish(request),
        PublishOutcome::CommittedNew(_)
    ));
    assert_eq!(index.current_count(), 0);
    assert_eq!(
        index.finalize_committed(&accepted, 2),
        FinalizationOutcome::Finalized
    );
    assert_eq!(index.current_count(), 1);
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
