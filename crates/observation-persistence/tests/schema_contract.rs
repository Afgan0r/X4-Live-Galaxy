mod support;

use std::fs;

use observation_persistence::{
    OBSERVATION_REPOSITORY_PROTOCOL_IDENTITY, OBSERVATION_REPOSITORY_SCHEMA_VERSION,
    PublicationLimits, RepositoryError, SqliteObservationRepository,
};
use rusqlite::{Connection, params};
use support::TempDatabase;

const fn limits() -> PublicationLimits {
    PublicationLimits::new(4, 256).expect("limits are non-zero")
}

#[test]
fn schema_identity_and_dependency_pin_are_exact() {
    assert_eq!(OBSERVATION_REPOSITORY_SCHEMA_VERSION, 1);
    assert_eq!(
        OBSERVATION_REPOSITORY_PROTOCOL_IDENTITY,
        "live_galaxy.observation_repository.v1"
    );
    let manifest = fs::read_to_string("crates/observation-persistence/Cargo.toml").unwrap();
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
