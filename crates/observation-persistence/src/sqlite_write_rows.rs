use rusqlite::{Connection, params};

use crate::{RepositoryDiagnostic, RepositoryError, RevisionRecord, schema, sqlite_read};

pub fn insert_revision(
    connection: &Connection,
    revision: &RevisionRecord,
) -> Result<(), RepositoryError> {
    connection.execute(
        "INSERT INTO revisions(section_key, revision, source_scope, producer_incarnation, transport_epoch, coverage, manifest_digest, content_digest, integrity_digest, context_token, expected_current) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![revision.section_key.as_str(), sqlite_read::sql_u64(revision.revision.get())?, revision.source_scope.as_str(), revision.source_session.producer_incarnation().as_str(), sqlite_read::sql_u64(revision.source_session.transport_epoch().get())?, schema::coverage_name(revision.coverage), revision.manifest_digest.as_slice(), revision.content_digest.as_slice(), revision.integrity_digest.as_slice(), &revision.context_token, revision.expected_current.map(|value| sqlite_read::sql_u64(value.get())).transpose()?],
    ).map_err(|_| error("revision-insert"))?;
    for (position, item) in revision.records.iter().enumerate() {
        connection
            .execute(
                "INSERT INTO revision_records VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    revision.section_key.as_str(),
                    sqlite_read::sql_u64(revision.revision.get())?,
                    i64::try_from(position).map_err(|_| error("integer-range"))?,
                    item.record_id.as_str(),
                    item.entity_id.as_str(),
                    sqlite_read::sql_u64(item.observation_version.get())?,
                    &item.content
                ],
            )
            .map_err(|_| error("record-insert"))?;
    }
    for (key, value) in &revision.dependencies {
        connection
            .execute(
                "INSERT INTO revision_dependencies VALUES (?1, ?2, ?3, ?4)",
                params![
                    revision.section_key.as_str(),
                    sqlite_read::sql_u64(revision.revision.get())?,
                    key.as_str(),
                    sqlite_read::sql_u64(value.get())?
                ],
            )
            .map_err(|_| error("dependency-insert"))?;
    }
    Ok(())
}

pub fn update_current(
    connection: &Connection,
    revision: &RevisionRecord,
) -> Result<(), RepositoryError> {
    let changed = if let Some(expected) = revision.expected_current {
        connection.execute(
            "UPDATE current_revisions SET revision=?1 WHERE section_key=?2 AND revision=?3",
            params![
                sqlite_read::sql_u64(revision.revision.get())?,
                revision.section_key.as_str(),
                sqlite_read::sql_u64(expected.get())?
            ],
        )
    } else {
        connection.execute(
            "INSERT INTO current_revisions(section_key, revision) VALUES (?1, ?2)",
            params![
                revision.section_key.as_str(),
                sqlite_read::sql_u64(revision.revision.get())?
            ],
        )
    }
    .map_err(|_| error("current-update"))?;
    if changed != 1 {
        return Err(error("current-cas"));
    }
    Ok(())
}

const fn error(code: &'static str) -> RepositoryError {
    RepositoryError::Storage(RepositoryDiagnostic { code })
}
