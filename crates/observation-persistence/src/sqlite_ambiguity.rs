use std::collections::BTreeSet;

use observation_domain::{SectionKey, SectionRevisionId};
use rusqlite::{Connection, params};

use crate::{RepositoryDiagnostic, RepositoryError, sqlite_read};

pub type AmbiguousSet = BTreeSet<(SectionKey, SectionRevisionId)>;

pub fn load(connection: &Connection) -> Result<AmbiguousSet, RepositoryError> {
    let mut statement = connection
        .prepare("SELECT section_key, revision FROM ambiguous_publications ORDER BY section_key")
        .map_err(|_| storage("ambiguity-read"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|_| storage("ambiguity-read"))?;
    rows.map(|row| {
        let (key, revision) = row.map_err(|_| storage("ambiguity-read"))?;
        let key = SectionKey::new(key).ok_or(corrupt("ambiguity-key"))?;
        let revision = u64::try_from(revision)
            .ok()
            .and_then(SectionRevisionId::new)
            .ok_or(corrupt("ambiguity-revision"))?;
        Ok((key, revision))
    })
    .collect()
}

pub fn mark(
    connection: &Connection,
    identity: &(SectionKey, SectionRevisionId),
) -> Result<(), RepositoryError> {
    connection
        .execute(
            "INSERT OR IGNORE INTO ambiguous_publications(section_key, revision) VALUES (?1, ?2)",
            params![identity.0.as_str(), sqlite_read::sql_u64(identity.1.get())?],
        )
        .map(|_| ())
        .map_err(|_| storage("ambiguity-mark"))
}

pub fn clear(
    connection: &Connection,
    identity: &(SectionKey, SectionRevisionId),
) -> Result<(), RepositoryError> {
    connection
        .execute(
            "DELETE FROM ambiguous_publications WHERE section_key=?1 AND revision=?2",
            params![identity.0.as_str(), sqlite_read::sql_u64(identity.1.get())?],
        )
        .map(|_| ())
        .map_err(|_| storage("ambiguity-clear"))
}

const fn corrupt(code: &'static str) -> RepositoryError {
    RepositoryError::Corrupt(RepositoryDiagnostic { code })
}

const fn storage(code: &'static str) -> RepositoryError {
    RepositoryError::Storage(RepositoryDiagnostic { code })
}
