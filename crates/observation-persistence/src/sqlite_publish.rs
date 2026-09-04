use rusqlite::Connection;

use crate::{
    PublicationFailpoint, PublicationLimits, PublishOutcome, PublishRequest, RepositoryDiagnostic,
    sqlite_ambiguity, sqlite_write,
};

pub fn publish(
    connection: &mut Connection,
    limits: PublicationLimits,
    ambiguous: &mut sqlite_ambiguity::AmbiguousSet,
    request: &PublishRequest,
    failpoint: Option<PublicationFailpoint>,
) -> PublishOutcome {
    let identity = (
        request.revision.section_key().clone(),
        request.revision.section_revision(),
    );
    if ambiguous.contains(&identity) {
        return PublishOutcome::Ambiguous(diagnostic("reconciliation-required"));
    }
    if sqlite_ambiguity::mark(connection, &identity).is_err() {
        return PublishOutcome::PermanentRejection(diagnostic("ambiguity-mark"));
    }
    let outcome = sqlite_write::publish_with_failpoint(connection, limits, request, failpoint);
    if matches!(outcome, PublishOutcome::Ambiguous(_)) {
        ambiguous.insert(identity);
        return outcome;
    }
    if sqlite_ambiguity::clear(connection, &identity).is_err() {
        ambiguous.insert(identity);
        return PublishOutcome::Ambiguous(diagnostic("ambiguity-clear"));
    }
    outcome
}

const fn diagnostic(code: &'static str) -> RepositoryDiagnostic {
    RepositoryDiagnostic { code }
}
