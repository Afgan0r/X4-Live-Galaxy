mod support;

use std::collections::BTreeMap;

use observation_persistence::{
    FakeObservationRepository, ObservationRepository, PublicationLimits, PublishOutcome,
    PublishRequest, SqliteObservationRepository, UnpinOutcome,
};
use support::{TempDatabase, decision_set, key, revision, validated};

fn limits() -> PublicationLimits {
    PublicationLimits::new(4, 256).expect("limits are non-zero")
}

fn shared_contract(repository: &mut dyn ObservationRepository) {
    let candidate = validated("ships", 1, None, BTreeMap::new());
    let request = PublishRequest::from_revision(candidate.clone());
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
        repository.publish(PublishRequest::from_revision(validated(
            "ships",
            2,
            None,
            BTreeMap::new()
        ))),
        PublishOutcome::StalePointer(_)
    ));
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
    assert_eq!(
        repository
            .current(&key("ships"))
            .expect("read succeeds")
            .expect("current remains")
            .receipt
            .revision,
        revision(1)
    );
}

#[test]
fn fake_and_sqlite_share_the_publication_contract() {
    shared_contract(&mut FakeObservationRepository::new(limits()));
    let database = TempDatabase::new("port-contract");
    let mut sqlite = SqliteObservationRepository::open(database.path(), limits())
        .expect("SQLite repository opens");
    shared_contract(&mut sqlite);
}
