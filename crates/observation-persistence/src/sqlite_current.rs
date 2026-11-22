use observation_domain::{SectionKey, SectionRevisionId};
use rusqlite::{Connection, params};

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

pub fn pointers(
    connection: &Connection,
) -> Result<Vec<(SectionKey, SectionRevisionId)>, RepositoryError> {
    let mut statement = connection
        .prepare(
            "SELECT c.section_key, c.revision, r.revision, p.revision
             FROM current_revisions c
             LEFT JOIN revisions r USING(section_key, revision)
             LEFT JOIN publication_receipts p USING(section_key, revision)
             ORDER BY c.section_key",
        )
        .map_err(|_| storage("current-pointers"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, Option<i64>>(3)?,
            ))
        })
        .map_err(|_| storage("current-pointers"))?;
    rows.map(|row| {
        let (key, revision, stored, receipt) = row.map_err(|_| storage("current-pointers"))?;
        pointer(key, revision, stored, receipt)
    })
    .collect()
}

pub fn dependents(
    connection: &Connection,
    parent: &CurrentRevision,
) -> Result<Vec<(SectionKey, SectionRevisionId)>, RepositoryError> {
    let mut statement = connection
        .prepare(
            "SELECT c.section_key, c.revision, r.revision, p.revision
             FROM revision_dependencies d
             JOIN current_revisions c USING(section_key, revision)
             LEFT JOIN revisions r USING(section_key, revision)
             LEFT JOIN publication_receipts p USING(section_key, revision)
             WHERE d.dependency_key=?1 AND d.dependency_revision=?2
               AND (r.source_scope=?3 OR r.source_scope IS NULL)
             ORDER BY c.section_key",
        )
        .map_err(|_| storage("current-dependents"))?;
    let rows = statement
        .query_map(
            params![
                parent.receipt().section_key.as_str(),
                sqlite_read::sql_u64(parent.receipt().revision.get())
                    .map_err(|_| corrupt("current-revision-invalid"))?,
                parent.revision().source_scope.as_str(),
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                ))
            },
        )
        .map_err(|_| storage("current-dependents"))?;
    rows.map(|row| {
        let (key, revision, stored, receipt) = row.map_err(|_| storage("current-dependents"))?;
        pointer(key, revision, stored, receipt)
    })
    .collect()
}

fn pointer(
    key: String,
    revision: i64,
    stored: Option<i64>,
    receipt: Option<i64>,
) -> Result<(SectionKey, SectionRevisionId), RepositoryError> {
    if stored != Some(revision) || receipt != Some(revision) {
        return Err(corrupt("dangling-current"));
    }
    let key = SectionKey::new(key).ok_or(corrupt("section-key-invalid"))?;
    let revision = u64::try_from(revision)
        .ok()
        .and_then(SectionRevisionId::new)
        .ok_or(corrupt("current-revision-invalid"))?;
    Ok((key, revision))
}

const fn corrupt(code: &'static str) -> RepositoryError {
    RepositoryError::Corrupt(RepositoryDiagnostic { code })
}

const fn storage(code: &'static str) -> RepositoryError {
    RepositoryError::Storage(RepositoryDiagnostic { code })
}
