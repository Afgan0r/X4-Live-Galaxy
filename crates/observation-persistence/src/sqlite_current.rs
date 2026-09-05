use observation_domain::SectionKey;
use rusqlite::Connection;

use crate::{CurrentRevision, RepositoryDiagnostic, RepositoryError, sqlite_read};

pub fn snapshot(connection: &Connection) -> Result<Vec<CurrentRevision>, RepositoryError> {
    let mut statement = connection
        .prepare("SELECT section_key FROM current_revisions ORDER BY section_key")
        .map_err(|_| storage("current-snapshot"))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|_| storage("current-snapshot"))?;
    let mut keys = Vec::new();
    for row in rows {
        keys.push(
            SectionKey::new(row.map_err(|_| storage("current-snapshot"))?)
                .ok_or(corrupt("section-key-invalid"))?,
        );
    }
    drop(statement);
    keys.into_iter()
        .map(|key| sqlite_read::current(connection, &key)?.ok_or(corrupt("dangling-current")))
        .collect()
}

const fn corrupt(code: &'static str) -> RepositoryError {
    RepositoryError::Corrupt(RepositoryDiagnostic { code })
}

const fn storage(code: &'static str) -> RepositoryError {
    RepositoryError::Storage(RepositoryDiagnostic { code })
}
