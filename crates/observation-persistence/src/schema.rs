use observation_domain::CompletionCoverage;
use rusqlite::Connection;

use crate::{RepositoryDiagnostic, RepositoryError};

pub const OBSERVATION_REPOSITORY_SCHEMA_VERSION: u32 = 4;
pub const OBSERVATION_REPOSITORY_PROTOCOL_IDENTITY: &str = "live_galaxy.observation_repository.v4";

const CREATE_SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS repository_metadata (
  singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
  schema_version INTEGER NOT NULL,
  protocol_identity TEXT NOT NULL
);
INSERT OR IGNORE INTO repository_metadata VALUES (1, 4, 'live_galaxy.observation_repository.v4');
CREATE TABLE IF NOT EXISTS revisions (
  section_key TEXT NOT NULL, revision INTEGER NOT NULL, source_scope TEXT NOT NULL,
  producer_incarnation TEXT NOT NULL, transport_epoch INTEGER NOT NULL,
  accepted_at INTEGER NOT NULL,
  coverage TEXT NOT NULL, manifest_digest BLOB NOT NULL, content_digest BLOB NOT NULL,
  integrity_digest BLOB NOT NULL, context_token TEXT NOT NULL, expected_current INTEGER,
  PRIMARY KEY (section_key, revision)
);
CREATE TABLE IF NOT EXISTS revision_dependencies (
  section_key TEXT NOT NULL, revision INTEGER NOT NULL, dependency_key TEXT NOT NULL,
  dependency_revision INTEGER NOT NULL,
  PRIMARY KEY (section_key, revision, dependency_key),
  FOREIGN KEY (section_key, revision) REFERENCES revisions(section_key, revision) ON DELETE CASCADE,
  FOREIGN KEY (dependency_key, dependency_revision) REFERENCES revisions(section_key, revision)
);
CREATE TABLE IF NOT EXISTS revision_records (
  section_key TEXT NOT NULL, revision INTEGER NOT NULL, position INTEGER NOT NULL,
  record_id TEXT NOT NULL, entity_id TEXT NOT NULL, observation_version INTEGER NOT NULL,
  content TEXT NOT NULL, PRIMARY KEY (section_key, revision, position),
  UNIQUE (section_key, revision, record_id),
  FOREIGN KEY (section_key, revision) REFERENCES revisions(section_key, revision) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS publication_receipts (
  section_key TEXT NOT NULL, revision INTEGER NOT NULL, content_digest BLOB NOT NULL,
  previous_revision INTEGER, ordinal INTEGER NOT NULL UNIQUE, accepted_at INTEGER NOT NULL,
  integrity_digest BLOB NOT NULL,
  PRIMARY KEY (section_key, revision),
  FOREIGN KEY (section_key, revision) REFERENCES revisions(section_key, revision) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS current_revisions (
  section_key TEXT PRIMARY KEY, revision INTEGER NOT NULL,
  FOREIGN KEY (section_key, revision) REFERENCES revisions(section_key, revision)
);
CREATE TABLE IF NOT EXISTS ambiguous_publications (
  section_key TEXT NOT NULL, revision INTEGER NOT NULL,
  PRIMARY KEY (section_key, revision)
);
CREATE TABLE IF NOT EXISTS decision_pins (
  decision_id TEXT PRIMARY KEY, ordinal INTEGER NOT NULL UNIQUE
);
CREATE TABLE IF NOT EXISTS decision_pin_revisions (
  decision_id TEXT NOT NULL, position INTEGER NOT NULL, section_key TEXT NOT NULL,
  revision INTEGER NOT NULL, PRIMARY KEY (decision_id, position),
  UNIQUE (decision_id, section_key),
  FOREIGN KEY (decision_id) REFERENCES decision_pins(decision_id) ON DELETE CASCADE,
  FOREIGN KEY (section_key, revision) REFERENCES revisions(section_key, revision)
);
";

pub fn initialize(connection: &Connection) -> Result<(), RepositoryError> {
    connection
        .execute_batch(CREATE_SCHEMA)
        .map_err(|_| storage("schema-initialize"))?;
    let identity: (u32, String) = connection
        .query_row(
            "SELECT schema_version, protocol_identity FROM repository_metadata WHERE singleton=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| storage("schema-metadata"))?;
    if identity
        != (
            OBSERVATION_REPOSITORY_SCHEMA_VERSION,
            OBSERVATION_REPOSITORY_PROTOCOL_IDENTITY.to_owned(),
        )
    {
        return Err(RepositoryError::Corrupt(RepositoryDiagnostic {
            code: "schema-incompatible",
        }));
    }
    validate_foreign_keys(connection)?;
    Ok(())
}

pub fn validate_foreign_keys(connection: &Connection) -> Result<(), RepositoryError> {
    let mut statement = connection
        .prepare("PRAGMA foreign_key_check")
        .map_err(|_| storage("foreign-key-check"))?;
    if statement
        .exists([])
        .map_err(|_| storage("foreign-key-check"))?
    {
        return Err(RepositoryError::Corrupt(RepositoryDiagnostic {
            code: "foreign-key-violation",
        }));
    }
    Ok(())
}

pub const fn coverage_name(coverage: CompletionCoverage) -> &'static str {
    match coverage {
        CompletionCoverage::Complete => "complete",
        CompletionCoverage::KnownEmpty => "known_empty",
        CompletionCoverage::Partial => "partial",
        CompletionCoverage::Unknown => "unknown",
        CompletionCoverage::Unsupported => "unsupported",
    }
}

pub fn parse_coverage(value: &str) -> Result<CompletionCoverage, RepositoryError> {
    match value {
        "complete" => Ok(CompletionCoverage::Complete),
        "known_empty" => Ok(CompletionCoverage::KnownEmpty),
        "partial" => Ok(CompletionCoverage::Partial),
        "unknown" => Ok(CompletionCoverage::Unknown),
        "unsupported" => Ok(CompletionCoverage::Unsupported),
        _ => Err(RepositoryError::Corrupt(RepositoryDiagnostic {
            code: "coverage-invalid",
        })),
    }
}

const fn storage(code: &'static str) -> RepositoryError {
    RepositoryError::Storage(RepositoryDiagnostic { code })
}
