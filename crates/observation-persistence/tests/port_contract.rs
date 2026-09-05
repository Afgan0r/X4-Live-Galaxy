#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "contract-test setup and mismatches must fail immediately"
)]

mod support;

use std::collections::BTreeMap;

use observation_domain::{ProducerIncarnationId, SourceSessionIdentity, TransportEpoch};
use observation_ingest::DecisionRevisionIndex;
use observation_persistence::{
    FakeObservationRepository, ObservationRepository, PublicationLimits, PublishOutcome,
    PublishRequest, SqliteObservationRepository, UnpinOutcome,
};
use rusqlite::Connection;
use support::{
    RevisionFixture, TempDatabase, decision_set, key, publish_request, revision, validated,
    validated_with,
};

const fn limits() -> PublicationLimits {
    PublicationLimits::new(4, 256).expect("limits are non-zero")
}

fn shared_contract(repository: &mut dyn ObservationRepository) {
    let candidate = validated("ships", 1, None, BTreeMap::new());
    let request = publish_request(candidate.clone());
    let receipt = match repository.publish(request.clone()) {
        PublishOutcome::CommittedNew(receipt) => receipt,
        outcome => panic!("genesis must commit, got {outcome:?}"),
    };
    assert_eq!(
        repository.publish(request),
        PublishOutcome::CommittedReplay(receipt.clone())
    );
    let current = repository
        .current(&key("ships"))
        .expect("read succeeds")
        .expect("current exists");
    assert_eq!(current.revision.content_digest, *candidate.content_digest());
    assert_eq!(current.receipt, receipt);
    assert!(matches!(
        repository.publish(publish_request(validated(
            "ships",
            2,
            None,
            BTreeMap::new()
        ))),
        PublishOutcome::StalePointer(_)
    ));
    pin_contract(repository, candidate);
    cas_contract(repository);
}

fn pin_contract(
    repository: &mut dyn ObservationRepository,
    candidate: observation_ingest::ValidatedSectionRevision,
) {
    let set = decision_set(vec![candidate]);
    let pin = repository.pin_decision(&set).expect("stored revision pins");
    assert_eq!(
        repository
            .load_decision_pin(&pin.decision)
            .expect("pin loads")
            .revisions,
        *set.revisions()
    );
    assert_eq!(repository.unpin_decision(&pin), Ok(UnpinOutcome::Unpinned));
    assert_eq!(
        repository.unpin_decision(&pin),
        Ok(UnpinOutcome::AlreadyAbsent)
    );
}

fn cas_contract(repository: &mut dyn ObservationRepository) {
    assert!(matches!(
        repository.publish(publish_request(validated(
            "ships",
            1,
            Some(revision(1)),
            BTreeMap::new()
        ))),
        PublishOutcome::Conflict(_)
    ));
    let dependency = validated("sectors", 1, None, BTreeMap::new());
    assert!(matches!(
        repository.publish(publish_request(dependency)),
        PublishOutcome::CommittedNew(_)
    ));
    let dependencies = BTreeMap::from([(key("sectors"), revision(1))]);
    assert!(matches!(
        repository.publish(publish_request(validated(
            "ships",
            2,
            Some(revision(1)),
            dependencies
        ))),
        PublishOutcome::CommittedNew(_)
    ));
    let stale = BTreeMap::from([(key("sectors"), revision(2))]);
    assert!(matches!(
        repository.publish(publish_request(validated(
            "ships",
            3,
            Some(revision(2)),
            stale
        ))),
        PublishOutcome::StaleDependency(_)
    ));
    assert_eq!(
        repository
            .current(&key("ships"))
            .expect("read succeeds")
            .expect("current remains")
            .receipt
            .revision,
        revision(2)
    );
}

#[test]
fn fake_and_sqlite_share_the_publication_contract() {
    shared_contract(&mut FakeObservationRepository::new(limits()));
    let database = TempDatabase::new("port-contract");
    let mut sqlite = SqliteObservationRepository::open(database.path(), limits())
        .expect("SQLite repository opens");
    assert_eq!(sqlite.foreign_keys_enabled(), Ok(true));
    shared_contract(&mut sqlite);
    drop(sqlite);
    let connection = Connection::open(database.path()).expect("fixture database opens");
    connection
        .pragma_update(None, "foreign_keys", true)
        .expect("fixture enables foreign keys");
    assert!(
        connection
            .execute(
                "INSERT INTO current_revisions(section_key, revision) VALUES ('orphan', 99)",
                [],
            )
            .is_err()
    );
}

#[test]
fn reconnect_keeps_a_delayed_validated_revision_history_only() {
    let delayed = validated_with(
        "ships",
        1,
        None,
        BTreeMap::new(),
        RevisionFixture::default(),
    );
    let scope = delayed.source_scope().clone();
    let current_session = SourceSessionIdentity::new(
        ProducerIncarnationId::new("producer:2").expect("producer is valid"),
        TransportEpoch::new(2).expect("epoch is non-zero"),
    );
    let mut index = DecisionRevisionIndex::new(1).expect("blocker limit is non-zero");
    let accepted = index
        .accept(delayed.clone(), 3)
        .expect("session A starts authoritative");
    let stale_request = PublishRequest::from_accepted(accepted);
    index.mark_scope_uncertain(&scope, current_session);
    assert!(index.accept(delayed, 4).is_none());
    assert_eq!(index.current_count(), 0);
    assert_eq!(index.history_count(), 0);

    let database = TempDatabase::new("delayed-session");
    let mut repository = SqliteObservationRepository::open(database.path(), limits())
        .expect("SQLite repository opens");
    assert!(matches!(
        repository.publish(stale_request),
        PublishOutcome::PermanentRejection(_)
    ));
    assert_eq!(repository.current(&key("ships")), Ok(None));
    drop(repository);
    let connection = Connection::open(database.path()).expect("fixture database opens");
    for table in ["revisions", "publication_receipts", "current_revisions"] {
        let count: i64 = connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("publication table count is readable");
        assert_eq!(count, 0, "stale authority wrote {table}");
    }
}

#[test]
fn durable_commit_precedes_index_finalization() {
    let mut index = DecisionRevisionIndex::new(1).expect("blocker limit is non-zero");
    let accepted = index
        .prepare_publication(validated("ships", 1, None, BTreeMap::new()))
        .expect("publication prepares");
    let request = PublishRequest::from_accepted(accepted.clone());
    assert_eq!(index.current_count(), 0);

    let database = TempDatabase::new("durable-before-finalize");
    let mut repository = SqliteObservationRepository::open(database.path(), limits())
        .expect("SQLite repository opens");
    assert!(matches!(
        repository.publish(request),
        PublishOutcome::CommittedNew(_)
    ));
    assert_eq!(index.current_count(), 0);
    assert_eq!(
        index.finalize_committed(&accepted, 2),
        observation_ingest::FinalizationOutcome::Finalized
    );
    assert_eq!(index.current_count(), 1);
}
