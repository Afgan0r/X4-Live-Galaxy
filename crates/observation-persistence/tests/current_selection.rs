mod support;

use std::collections::BTreeMap;

use observation_ingest::ValidatedSectionRevision;
use observation_persistence::{
    ObservationRepository, PublicationLimits, PublishOutcome, RepositoryDiagnostic,
    RepositoryError, SqliteObservationRepository,
};
use support::{
    RevisionFixture, TempDatabase, key, publish_request, revision, validated, validated_with,
};

fn publish(repository: &mut SqliteObservationRepository, value: ValidatedSectionRevision) {
    assert!(matches!(
        repository.publish(publish_request(value)),
        PublishOutcome::CommittedNew(_)
    ));
}

#[test]
fn dependent_heads_require_exact_parent_revision_and_scope() {
    let mut repository = SqliteObservationRepository::open(
        ":memory:",
        PublicationLimits::new(8, 1_024).expect("limits"),
    )
    .expect("SQLite");
    let parent_key = key("ship_core:argon");
    let cargo_key = key("ship_cargo:argon:g0");
    let crew_key = key("ship_crew:argon:g0");
    let another_scope = RevisionFixture {
        source_scope: "scope:other",
        ..RevisionFixture::default()
    };

    publish(
        &mut repository,
        validated(parent_key.as_str(), 1, None, BTreeMap::new()),
    );
    let initial_parent = repository
        .current(&parent_key)
        .expect("read")
        .expect("parent");
    let dependency = BTreeMap::from([(parent_key.clone(), revision(1))]);
    publish(
        &mut repository,
        validated(cargo_key.as_str(), 2, None, dependency.clone()),
    );
    publish(
        &mut repository,
        validated_with(crew_key.as_str(), 3, None, dependency, another_scope),
    );
    assert_eq!(
        repository.current_dependents(&initial_parent),
        Ok(vec![(cargo_key.clone(), revision(2))])
    );

    publish(
        &mut repository,
        validated(parent_key.as_str(), 4, Some(revision(1)), BTreeMap::new()),
    );
    let refreshed = repository
        .current(&parent_key)
        .expect("read")
        .expect("refreshed parent");
    assert_eq!(repository.current_dependents(&refreshed), Ok(Vec::new()));
    assert_eq!(
        repository.current_pointers(),
        Ok(vec![
            (cargo_key, revision(2)),
            (parent_key, revision(4)),
            (crew_key, revision(3))
        ]),
    );
}

#[test]
fn current_pointer_without_receipt_fails_closed() {
    let data = TempDatabase::new("current-pointer-without-receipt");
    let mut repository = SqliteObservationRepository::open(
        data.path(),
        PublicationLimits::new(8, 1_024).expect("limits"),
    )
    .expect("SQLite");
    publish(
        &mut repository,
        validated("ship_core:argon", 1, None, BTreeMap::new()),
    );
    let parent_key = key("ship_core:argon");
    let parent = repository
        .current(&parent_key)
        .expect("read")
        .expect("parent");
    let cargo = key("ship_cargo:argon:g0");
    publish(
        &mut repository,
        validated(
            cargo.as_str(),
            2,
            None,
            BTreeMap::from([(parent_key, revision(1))]),
        ),
    );
    assert_eq!(
        repository.current_dependents(&parent),
        Ok(vec![(cargo, revision(2))])
    );
    let other = rusqlite::Connection::open(data.path()).expect("corruption seam");
    assert_eq!(
        other
            .execute(
                "DELETE FROM publication_receipts WHERE section_key='ship_cargo:argon:g0'",
                []
            )
            .expect("delete dependent receipt"),
        1
    );
    let missing = Err(RepositoryError::Corrupt(RepositoryDiagnostic {
        code: "dangling-current",
    }));
    assert_eq!(repository.current_pointers(), missing);
    assert_eq!(repository.current_dependents(&parent), missing);
}
