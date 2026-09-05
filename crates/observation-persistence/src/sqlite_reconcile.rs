use rusqlite::Connection;

use crate::{
    PublicationLimits, PublishRequest, ReconciliationOutcome, RepositoryDiagnostic, RevisionRecord,
    record, sqlite_read, sqlite_receipt,
};

pub fn classify(
    connection: &Connection,
    request: &PublishRequest,
    limits: PublicationLimits,
) -> ReconciliationOutcome {
    let Some(candidate) = record::normalize(request, limits) else {
        return ambiguous("reconciliation-invalid");
    };
    match sqlite_read::load_revision(connection, &candidate.section_key, candidate.revision) {
        Ok(Some(stored)) => committed(connection, &candidate, &stored),
        Ok(None) => not_committed(connection, request, &candidate),
        Err(_) => ambiguous("reconciliation-corrupt"),
    }
}

fn committed(
    connection: &Connection,
    candidate: &RevisionRecord,
    stored: &RevisionRecord,
) -> ReconciliationOutcome {
    let receipt = sqlite_receipt::load_validated(connection, stored);
    let current = sqlite_read::current_pointer(connection, &candidate.section_key);
    match (receipt, current) {
        (Ok(receipt), Ok(Some(current)))
            if stored == candidate && current == candidate.revision =>
        {
            ReconciliationOutcome::CommittedReplay(receipt)
        }
        (Ok(_), Ok(Some(current))) if stored == candidate && current != candidate.revision => {
            ReconciliationOutcome::Superseded(RepositoryDiagnostic {
                code: "reconciliation-superseded",
            })
        }
        _ => ambiguous("reconciliation-ambiguous"),
    }
}

fn not_committed(
    connection: &Connection,
    request: &PublishRequest,
    candidate: &RevisionRecord,
) -> ReconciliationOutcome {
    let receipt = sqlite_receipt::load(connection, &candidate.section_key, candidate.revision);
    let current = sqlite_read::current_pointer(connection, &candidate.section_key);
    let dependencies_match = request.frozen_dependencies().iter().all(|(key, expected)| {
        sqlite_read::current_pointer(connection, key) == Ok(Some(*expected))
    });
    match (receipt, current) {
        (Ok(None), Ok(current)) if current == candidate.expected_current && dependencies_match => {
            ReconciliationOutcome::ProvenNotCommitted
        }
        _ => ambiguous("reconciliation-ambiguous"),
    }
}

const fn ambiguous(code: &'static str) -> ReconciliationOutcome {
    ReconciliationOutcome::Ambiguous(RepositoryDiagnostic { code })
}
