use rusqlite::{Connection, TransactionBehavior, params};

use crate::{
    PublicationFailpoint, PublicationLimits, PublicationReceipt, PublishOutcome, PublishRequest,
    RepositoryDiagnostic, RepositoryError, RevisionRecord, record, sqlite_read, sqlite_receipt,
    sqlite_write_rows,
};

pub fn publish_with_failpoint(
    connection: &mut Connection,
    limits: PublicationLimits,
    request: &PublishRequest,
    failpoint: Option<PublicationFailpoint>,
) -> PublishOutcome {
    let Some(revision) = record::normalize(request, limits) else {
        return rejection("invalid-revision");
    };
    let Ok(transaction) = connection.transaction_with_behavior(TransactionBehavior::Immediate)
    else {
        return rejection("transaction-begin");
    };
    let receipt = match publish_in_transaction(&transaction, request, &revision, failpoint) {
        Ok(WriteOutcome::Replay(receipt)) => return PublishOutcome::CommittedReplay(receipt),
        Ok(WriteOutcome::New(receipt)) => receipt,
        Err(outcome) => return outcome,
    };
    if failpoint == Some(PublicationFailpoint::BeforeCommit) {
        return rejection("failpoint-before-commit");
    }
    match transaction.commit() {
        Ok(())
            if matches!(
                failpoint,
                Some(
                    PublicationFailpoint::CommitResultUnknown
                        | PublicationFailpoint::AfterCommitBeforeResponse
                )
            ) =>
        {
            PublishOutcome::Ambiguous(diagnostic("commit-result-unknown"))
        }
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
    failpoint: Option<PublicationFailpoint>,
) -> Result<WriteOutcome, PublishOutcome> {
    if let Some(existing) =
        sqlite_read::load_revision(connection, &revision.section_key, revision.revision)
            .map_err(storage)?
    {
        if existing != *revision {
            return Err(PublishOutcome::Conflict(diagnostic("content-conflict")));
        }
        let receipt = sqlite_receipt::load_validated(connection, revision).map_err(storage)?;
        return Ok(WriteOutcome::Replay(receipt));
    }
    if sqlite_read::current_pointer(connection, &revision.section_key).map_err(storage)?
        != request.expected_current()
    {
        return Err(PublishOutcome::StalePointer(diagnostic("stale-pointer")));
    }
    for (key, expected) in request.frozen_dependencies() {
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
    if failpoint == Some(PublicationFailpoint::BeforeContent) {
        return Err(rejection("failpoint-before-content"));
    }
    sqlite_write_rows::insert_revision(connection, revision).map_err(storage)?;
    if failpoint == Some(PublicationFailpoint::AfterContent) {
        return Err(rejection("failpoint-after-content"));
    }
    let receipt = PublicationReceipt {
        section_key: revision.section_key.clone(),
        revision: revision.revision,
        content_digest: revision.content_digest,
        previous: revision.expected_current,
        ordinal: u64::try_from(ordinal).map_err(|_| rejection("receipt-ordinal"))?,
    };
    let receipt_integrity = sqlite_receipt::integrity_digest(&receipt);
    connection.execute(
        "INSERT INTO publication_receipts(section_key, revision, content_digest, previous_revision, ordinal, integrity_digest) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![revision.section_key.as_str(), sqlite_read::sql_u64(revision.revision.get()).map_err(storage)?, revision.content_digest.as_slice(), revision.expected_current.map(|value| sqlite_read::sql_u64(value.get())).transpose().map_err(storage)?, ordinal, receipt_integrity.as_slice()],
    ).map_err(|_| rejection("receipt-insert"))?;
    if failpoint == Some(PublicationFailpoint::AfterReceipt) {
        return Err(rejection("failpoint-after-receipt"));
    }
    sqlite_write_rows::update_current(connection, revision).map_err(storage)?;
    if failpoint == Some(PublicationFailpoint::AfterPointer) {
        return Err(rejection("failpoint-after-pointer"));
    }
    Ok(WriteOutcome::New(receipt))
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
