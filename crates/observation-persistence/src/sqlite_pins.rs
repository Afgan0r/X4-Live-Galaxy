use std::collections::BTreeMap;

use observation_domain::{DecisionSnapshotId, SectionKey, SectionRevisionId};
use observation_ingest::DecisionRevisionSet;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::{
    DecisionPinReceipt, DecisionRevisionPin, RepositoryDiagnostic, RepositoryError, UnpinOutcome,
    record, sqlite_read,
};

pub fn pin(
    connection: &mut Connection,
    set: &DecisionRevisionSet,
) -> Result<DecisionPinReceipt, RepositoryError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| storage("pin-transaction"))?;
    for (key, revision) in set.revisions() {
        if sqlite_read::load_revision(&transaction, key, *revision)?.is_none() {
            return Err(RepositoryError::MissingRevision(diagnostic(
                "pin-revision-missing",
            )));
        }
    }
    let decision = record::decision_identity(set).ok_or(storage("pin-identity"))?;
    if let Ok(existing) = load(&transaction, &decision) {
        if existing.revisions == *set.revisions() {
            return Ok(existing.receipt);
        }
        return Err(RepositoryError::PinConflict(diagnostic("pin-conflict")));
    }
    let ordinal: i64 = transaction
        .query_row(
            "SELECT COALESCE(MAX(ordinal), 0) + 1 FROM decision_pins",
            [],
            |row| row.get(0),
        )
        .map_err(|_| storage("pin-ordinal"))?;
    transaction
        .execute(
            "INSERT INTO decision_pins(decision_id, ordinal) VALUES (?1, ?2)",
            params![decision.as_str(), ordinal],
        )
        .map_err(|_| storage("pin-insert"))?;
    for (position, (key, revision)) in set.revisions().iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO decision_pin_revisions VALUES (?1, ?2, ?3, ?4)",
                params![
                    decision.as_str(),
                    i64::try_from(position).map_err(|_| storage("integer-range"))?,
                    key.as_str(),
                    sqlite_read::sql_u64(revision.get())?
                ],
            )
            .map_err(|_| storage("pin-link-insert"))?;
    }
    transaction.commit().map_err(|_| storage("pin-commit"))?;
    Ok(DecisionPinReceipt {
        decision,
        ordinal: u64::try_from(ordinal).map_err(|_| storage("integer-range"))?,
    })
}

pub fn load(
    connection: &Connection,
    decision: &DecisionSnapshotId,
) -> Result<DecisionRevisionPin, RepositoryError> {
    let ordinal: Option<i64> = connection
        .query_row(
            "SELECT ordinal FROM decision_pins WHERE decision_id=?1",
            [decision.as_str()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| storage("pin-read"))?;
    let Some(ordinal) = ordinal else {
        return Err(RepositoryError::MissingRevision(diagnostic(
            "pin-not-found",
        )));
    };
    let mut statement = connection.prepare(
        "SELECT position, section_key, revision FROM decision_pin_revisions WHERE decision_id=?1 ORDER BY position"
    ).map_err(|_| storage("pin-links-read"))?;
    let rows = statement
        .query_map([decision.as_str()], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|_| storage("pin-links-read"))?;
    let mut revisions = BTreeMap::new();
    for (position, row) in rows.enumerate() {
        let (stored, key, value) = row.map_err(|_| storage("pin-links-read"))?;
        if stored != i64::try_from(position).map_err(|_| storage("integer-range"))? {
            return Err(corrupt("pin-link-order"));
        }
        let key = SectionKey::new(key).ok_or(corrupt("pin-key"))?;
        let value = u64::try_from(value)
            .ok()
            .and_then(SectionRevisionId::new)
            .ok_or(corrupt("pin-revision"))?;
        revisions.insert(key, value);
    }
    Ok(DecisionRevisionPin {
        receipt: DecisionPinReceipt {
            decision: decision.clone(),
            ordinal: u64::try_from(ordinal).map_err(|_| corrupt("pin-ordinal"))?,
        },
        revisions,
    })
}

pub fn unpin(
    connection: &mut Connection,
    receipt: &DecisionPinReceipt,
) -> Result<UnpinOutcome, RepositoryError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| storage("unpin-transaction"))?;
    let stored: Option<i64> = transaction
        .query_row(
            "SELECT ordinal FROM decision_pins WHERE decision_id=?1",
            [receipt.decision.as_str()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| storage("unpin-read"))?;
    let Some(stored) = stored else {
        return Ok(UnpinOutcome::AlreadyAbsent);
    };
    if u64::try_from(stored).ok() != Some(receipt.ordinal) {
        return Ok(UnpinOutcome::StaleReceipt);
    }
    transaction
        .execute(
            "DELETE FROM decision_pins WHERE decision_id=?1",
            [receipt.decision.as_str()],
        )
        .map_err(|_| storage("unpin-delete"))?;
    transaction.commit().map_err(|_| storage("unpin-commit"))?;
    Ok(UnpinOutcome::Unpinned)
}

const fn diagnostic(code: &'static str) -> RepositoryDiagnostic {
    RepositoryDiagnostic { code }
}
const fn storage(code: &'static str) -> RepositoryError {
    RepositoryError::Storage(diagnostic(code))
}
const fn corrupt(code: &'static str) -> RepositoryError {
    RepositoryError::Corrupt(diagnostic(code))
}
