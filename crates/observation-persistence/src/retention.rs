use std::collections::{BTreeMap, BTreeSet};

use observation_domain::{SectionKey, SectionRevisionId};
use rusqlite::{Connection, TransactionBehavior, params};

use crate::{RepositoryDiagnostic, RepositoryError, RetentionPolicy, RetentionReport, sqlite_read};

pub fn run(
    connection: &mut Connection,
    policy: RetentionPolicy,
) -> Result<RetentionReport, RepositoryError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| storage("retention-begin"))?;
    let protected = protected_revisions(&transaction)?;
    let revisions = receipt_revisions(&transaction)?;
    let mut seen = BTreeMap::<SectionKey, usize>::new();
    let mut retained_receipts = 0usize;
    let mut victims = Vec::new();
    for identity in revisions {
        if protected.contains(&identity) {
            continue;
        }
        let count = seen.entry(identity.0.clone()).or_default();
        *count = count.saturating_add(1);
        if *count > policy.history_per_section.get()
            || retained_receipts >= policy.receipt_count.get()
        {
            victims.push(identity);
        } else {
            retained_receipts = retained_receipts.saturating_add(1);
        }
    }
    for (key, revision) in &victims {
        transaction
            .execute(
                "DELETE FROM revisions WHERE section_key=?1 AND revision=?2",
                params![key.as_str(), sqlite_read::sql_u64(revision.get())?],
            )
            .map_err(|_| storage("retention-delete"))?;
    }
    transaction
        .commit()
        .map_err(|_| storage("retention-commit"))?;
    Ok(RetentionReport {
        deleted_revisions: victims.len(),
    })
}

fn protected_revisions(
    connection: &Connection,
) -> Result<BTreeSet<(SectionKey, SectionRevisionId)>, RepositoryError> {
    let sql = "SELECT section_key, revision FROM current_revisions
               UNION SELECT section_key, revision FROM decision_pin_revisions
               UNION SELECT dependency_key, dependency_revision FROM revision_dependencies";
    read_identities(connection, sql)
}

fn receipt_revisions(
    connection: &Connection,
) -> Result<Vec<(SectionKey, SectionRevisionId)>, RepositoryError> {
    let mut statement = connection
        .prepare(
            "SELECT section_key, revision FROM publication_receipts
             ORDER BY ordinal DESC, section_key, revision DESC",
        )
        .map_err(|_| storage("retention-scan"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|_| storage("retention-scan"))?;
    rows.map(|row| parse_identity(row.map_err(|_| storage("retention-scan"))?))
        .collect()
}

fn read_identities(
    connection: &Connection,
    sql: &str,
) -> Result<BTreeSet<(SectionKey, SectionRevisionId)>, RepositoryError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|_| storage("retention-reachability"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|_| storage("retention-reachability"))?;
    rows.map(|row| parse_identity(row.map_err(|_| storage("retention-reachability"))?))
        .collect()
}

fn parse_identity(
    value: (String, i64),
) -> Result<(SectionKey, SectionRevisionId), RepositoryError> {
    let key = SectionKey::new(value.0).ok_or_else(|| corrupt("retention-key"))?;
    let revision = u64::try_from(value.1)
        .ok()
        .and_then(SectionRevisionId::new)
        .ok_or_else(|| corrupt("retention-revision"))?;
    Ok((key, revision))
}

const fn storage(code: &'static str) -> RepositoryError {
    RepositoryError::Storage(RepositoryDiagnostic { code })
}

const fn corrupt(code: &'static str) -> RepositoryError {
    RepositoryError::Corrupt(RepositoryDiagnostic { code })
}
