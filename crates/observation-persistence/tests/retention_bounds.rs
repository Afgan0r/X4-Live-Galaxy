#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "retention fixtures must fail immediately when setup is invalid"
)]

mod support;

use std::collections::BTreeMap;

use observation_persistence::{
    ObservationRepository, PublicationLimits, PublishOutcome, RetentionPolicy,
    SqliteObservationRepository,
};
use support::{TempDatabase, publish_request, revision, validated};

const fn limits() -> PublicationLimits {
    PublicationLimits::new(4, 256).expect("limits are non-zero")
}

fn repository_with_histories(
    label: &str,
    sections: &[&str],
) -> (TempDatabase, SqliteObservationRepository) {
    let database = TempDatabase::new(label);
    let mut repository = SqliteObservationRepository::open(database.path(), limits()).unwrap();
    for section in sections {
        assert!(matches!(
            repository.publish(publish_request(validated(
                section,
                1,
                None,
                BTreeMap::default(),
            ))),
            PublishOutcome::CommittedNew(_)
        ));
        assert!(matches!(
            repository.publish(publish_request(validated(
                section,
                2,
                Some(revision(1)),
                BTreeMap::default(),
            ))),
            PublishOutcome::CommittedNew(_)
        ));
    }
    (database, repository)
}

#[test]
fn receipt_count_caps_unprotected_history_across_sections() {
    let policy = RetentionPolicy::new(2, 2).unwrap();
    let (_exact_database, mut exact) =
        repository_with_histories("retention-exact", &["ships", "stations"]);
    assert_eq!(exact.run_retention(policy).unwrap().deleted_revisions, 0);

    let (_over_database, mut over) =
        repository_with_histories("retention-over", &["ships", "stations", "sectors"]);
    assert_eq!(over.run_retention(policy).unwrap().deleted_revisions, 1);
}
