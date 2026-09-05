#![expect(
    clippy::expect_used,
    reason = "schema fixtures must fail immediately when setup or corruption injection fails"
)]

mod support;

use std::collections::BTreeMap;
use std::fs;

use observation_persistence::{
    OBSERVATION_REPOSITORY_PROTOCOL_IDENTITY, OBSERVATION_REPOSITORY_SCHEMA_VERSION,
    ObservationRepository, PublicationLimits, PublishOutcome, RepositoryError,
    SqliteObservationRepository,
};
use rusqlite::{Connection, params};
use support::{TempDatabase, publish_request, validated};

const fn limits() -> PublicationLimits {
    PublicationLimits::new(4, 256).expect("limits are non-zero")
}

#[test]
fn schema_identity_and_dependency_pin_are_exact() {
    assert_eq!(OBSERVATION_REPOSITORY_SCHEMA_VERSION, 4);
    assert_eq!(
        OBSERVATION_REPOSITORY_PROTOCOL_IDENTITY,
        "live_galaxy.observation_repository.v4"
    );
    let manifest =
        fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .unwrap();
    assert!(manifest.contains(
        "rusqlite = { version = \"=0.40.2\", default-features = false, features = [\"bundled\"] }"
    ));
}

#[test]
fn open_rejects_incompatible_or_unreadable_database() {
    let incompatible = TempDatabase::new("incompatible");
    let connection = Connection::open(incompatible.path()).unwrap();
    connection.execute_batch(
        "CREATE TABLE repository_metadata(singleton INTEGER PRIMARY KEY, schema_version INTEGER NOT NULL, protocol_identity TEXT NOT NULL);
         INSERT INTO repository_metadata VALUES(1, 2, 'unsupported');",
    ).unwrap();
    drop(connection);
    assert!(matches!(
        SqliteObservationRepository::open(incompatible.path(), limits()),
        Err(RepositoryError::Corrupt(_))
    ));
    let unreadable = TempDatabase::new("unreadable");
    fs::write(unreadable.path(), b"not a sqlite database").unwrap();
    assert!(SqliteObservationRepository::open(unreadable.path(), limits()).is_err());
}

#[test]
fn open_rejects_an_orphan_built_with_foreign_keys_disabled() {
    let database = TempDatabase::new("orphan");
    drop(SqliteObservationRepository::open(database.path(), limits()).unwrap());
    let connection = Connection::open(database.path()).unwrap();
    connection
        .pragma_update(None, "foreign_keys", false)
        .unwrap();
    connection
        .execute(
            "INSERT INTO current_revisions(section_key, revision) VALUES (?1, ?2)",
            params!["ships", 99_i64],
        )
        .unwrap();
    drop(connection);
    assert!(matches!(
        SqliteObservationRepository::open(database.path(), limits()),
        Err(RepositoryError::Corrupt(_))
    ));
}

#[test]
fn open_rejects_tampered_digest_and_partial_row_set() {
    for (label, mutation) in [
        (
            "tampered",
            "UPDATE revisions SET content_digest=zeroblob(32)",
        ),
        (
            "receipt-digest",
            "UPDATE publication_receipts SET content_digest=zeroblob(32)",
        ),
        (
            "receipt-predecessor",
            "UPDATE publication_receipts SET previous_revision=99",
        ),
        (
            "receipt-ordinal",
            "UPDATE publication_receipts SET ordinal=99",
        ),
        ("partial", "DELETE FROM publication_receipts"),
    ] {
        let database = TempDatabase::new(label);
        let mut repository = SqliteObservationRepository::open(database.path(), limits()).unwrap();
        assert!(matches!(
            repository.publish(publish_request(validated(
                "ships",
                1,
                None,
                BTreeMap::default(),
            ))),
            PublishOutcome::CommittedNew(_)
        ));
        drop(repository);
        let connection = Connection::open(database.path()).unwrap();
        connection
            .pragma_update(None, "foreign_keys", false)
            .unwrap();
        connection.execute(mutation, []).unwrap();
        drop(connection);
        assert!(matches!(
            SqliteObservationRepository::open(database.path(), limits()),
            Err(RepositoryError::Corrupt(_))
        ));
    }
}
