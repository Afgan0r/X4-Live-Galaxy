use rusqlite::{Connection, TransactionBehavior, params};

use crate::{
    PublicationLimits, PublicationReceipt, PublishOutcome, PublishRequest, RepositoryDiagnostic,
    RepositoryError, RevisionRecord, record, schema, sqlite_read,
};

pub fn publish(
    connection: &mut Connection,
    limits: PublicationLimits,
    request: &PublishRequest,
) -> PublishOutcome {
    let Some(revision) = record::normalize(request, limits) else {
        return rejection("invalid-revision");
    };
    let Ok(transaction) = connection.transaction_with_behavior(TransactionBehavior::Immediate)
    else {
        return rejection("transaction-begin");
    };
    let receipt = match publish_in_transaction(&transaction, request, &revision) {
        Ok(WriteOutcome::Replay(receipt)) => return PublishOutcome::CommittedReplay(receipt),
        Ok(WriteOutcome::New(receipt)) => receipt,
        Err(outcome) => return outcome,
    };
    match transaction.commit() {
        Ok(()) => PublishOutcome::CommittedNew(receipt),
        Err(_) => PublishOutcome::Ambiguous(diagnostic("commit-result-unknown")),
    }
}

enum WriteOutcome {
    New(PublicationReceipt),
    Replay(PublicationReceipt),
}

fn publish_in_transaction(
    connection: &Connection,
    request: &PublishRequest,
    revision: &RevisionRecord,
) -> Result<WriteOutcome, PublishOutcome> {
    if let Some(existing) =
        sqlite_read::load_revision(connection, &revision.section_key, revision.revision)
            .map_err(storage)?
    {
        if existing != *revision {
            return Err(PublishOutcome::Conflict(diagnostic("content-conflict")));
        }
        let receipt =
            sqlite_read::load_receipt(connection, &revision.section_key, revision.revision)
                .map_err(storage)?
                .ok_or_else(|| rejection("receipt-missing"))?;
        return Ok(WriteOutcome::Replay(receipt));
    }
    if sqlite_read::current_pointer(connection, &revision.section_key).map_err(storage)?
        != request.expected_current
    {
        return Err(PublishOutcome::StalePointer(diagnostic("stale-pointer")));
    }
    for (key, expected) in &request.frozen_dependencies {
        if sqlite_read::current_pointer(connection, key).map_err(storage)? != Some(*expected) {
            return Err(PublishOutcome::StaleDependency(diagnostic(
                "stale-dependency",
            )));
        }
    }
    let ordinal: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(ordinal), 0) + 1 FROM publication_receipts",
            [],
            |row| row.get(0),
        )
        .map_err(|_| rejection("receipt-ordinal"))?;
    insert_revision(connection, revision).map_err(storage)?;
    connection.execute(
        "INSERT INTO publication_receipts(section_key, revision, content_digest, previous_revision, ordinal) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![revision.section_key.as_str(), sqlite_read::sql_u64(revision.revision.get()).map_err(storage)?, revision.content_digest.as_slice(), revision.expected_current.map(|value| sqlite_read::sql_u64(value.get())).transpose().map_err(storage)?, ordinal],
    ).map_err(|_| rejection("receipt-insert"))?;
    update_current(connection, revision).map_err(storage)?;
    Ok(WriteOutcome::New(PublicationReceipt {
        section_key: revision.section_key.clone(),
        revision: revision.revision,
        content_digest: revision.content_digest,
        previous: revision.expected_current,
        ordinal: u64::try_from(ordinal).map_err(|_| rejection("receipt-ordinal"))?,
    }))
}

fn insert_revision(
    connection: &Connection,
    revision: &RevisionRecord,
) -> Result<(), RepositoryError> {
    connection.execute(
        "INSERT INTO revisions(section_key, revision, source_scope, coverage, manifest_digest, content_digest, integrity_digest, context_token, expected_current) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![revision.section_key.as_str(), sqlite_read::sql_u64(revision.revision.get())?, revision.source_scope.as_str(), schema::coverage_name(revision.coverage), revision.manifest_digest.as_slice(), revision.content_digest.as_slice(), revision.integrity_digest.as_slice(), &revision.context_token, revision.expected_current.map(|value| sqlite_read::sql_u64(value.get())).transpose()?],
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

fn update_current(
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

const fn storage(_: RepositoryError) -> PublishOutcome {
    rejection("storage-precommit")
}
const fn rejection(code: &'static str) -> PublishOutcome {
    PublishOutcome::PermanentRejection(diagnostic(code))
}
const fn diagnostic(code: &'static str) -> RepositoryDiagnostic {
    RepositoryDiagnostic { code }
}
const fn error(code: &'static str) -> RepositoryError {
    RepositoryError::Storage(diagnostic(code))
}
