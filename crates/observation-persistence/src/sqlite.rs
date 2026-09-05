use std::path::Path;

use observation_domain::{DecisionSnapshotId, SectionKey};
use observation_ingest::DecisionRevisionSet;
use rusqlite::Connection;

use crate::{
    CurrentRevision, DecisionPinReceipt, DecisionRevisionPin, ObservationRepository,
    PublicationLimits, PublishOutcome, PublishRequest, RepositoryDiagnostic, RepositoryError,
    UnpinOutcome, retention, schema, sqlite_ambiguity, sqlite_current, sqlite_pins, sqlite_publish,
    sqlite_read, sqlite_receipt, sqlite_reconcile,
};
use crate::{PublicationFailpoint, ReconciliationOutcome, RetentionPolicy, RetentionReport};

pub struct SqliteObservationRepository {
    connection: Connection,
    limits: PublicationLimits,
    ambiguous: sqlite_ambiguity::AmbiguousSet,
}

impl SqliteObservationRepository {
    pub fn open(
        path: impl AsRef<Path>,
        limits: PublicationLimits,
    ) -> Result<Self, RepositoryError> {
        let connection = Connection::open(path).map_err(|_| storage("open"))?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(|_| storage("foreign-keys-enable"))?;
        let enabled: i64 = connection
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .map_err(|_| storage("foreign-keys-query"))?;
        if enabled != 1 {
            return Err(storage("foreign-keys-disabled"));
        }
        schema::initialize(&connection)?;
        let ambiguous = sqlite_ambiguity::load(&connection)?;
        let repository = Self {
            connection,
            limits,
            ambiguous,
        };
        repository.validate_stored_revisions()?;
        sqlite_pins::validate_all(&repository.connection)?;
        Ok(repository)
    }

    pub fn foreign_keys_enabled(&self) -> Result<bool, RepositoryError> {
        self.connection
            .pragma_query_value(None, "foreign_keys", |row| row.get::<_, i64>(0))
            .map(|value| value == 1)
            .map_err(|_| storage("foreign-keys-query"))
    }

    pub fn publish_with_failpoint(
        &mut self,
        request: &PublishRequest,
        failpoint: PublicationFailpoint,
    ) -> PublishOutcome {
        sqlite_publish::publish(
            &mut self.connection,
            self.limits,
            &mut self.ambiguous,
            request,
            Some(failpoint),
        )
    }

    pub fn reconcile_publication(&mut self, request: &PublishRequest) -> ReconciliationOutcome {
        if schema::validate_foreign_keys(&self.connection).is_err()
            || self.validate_stored_revisions().is_err()
        {
            return ambiguous("reconciliation-corrupt");
        }
        let identity = (
            request.revision().section_key().clone(),
            request.revision().section_revision(),
        );
        let outcome = sqlite_reconcile::classify(&self.connection, request, self.limits);
        let definitive = !matches!(outcome, ReconciliationOutcome::Ambiguous(_));
        if definitive && sqlite_ambiguity::clear(&self.connection, &identity).is_err() {
            return ambiguous("reconciliation-barrier-clear");
        }
        if definitive {
            self.ambiguous.remove(&identity);
        }
        outcome
    }

    pub fn run_retention(
        &mut self,
        policy: RetentionPolicy,
    ) -> Result<RetentionReport, RepositoryError> {
        retention::run(&mut self.connection, policy)
    }

    fn validate_stored_revisions(&self) -> Result<(), RepositoryError> {
        let mut statement = self
            .connection
            .prepare("SELECT section_key, revision FROM revisions ORDER BY section_key, revision")
            .map_err(|_| storage("revision-scan"))?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|_| storage("revision-scan"))?;
        for row in rows {
            let (key, revision) = row.map_err(|_| storage("revision-scan"))?;
            let key = SectionKey::new(key).ok_or_else(|| storage("section-key-invalid"))?;
            let revision = u64::try_from(revision)
                .ok()
                .and_then(observation_domain::SectionRevisionId::new)
                .ok_or_else(|| storage("revision-invalid"))?;
            let record = sqlite_read::load_revision(&self.connection, &key, revision)?
                .ok_or_else(|| storage("revision-missing"))?;
            let _ = sqlite_receipt::load_validated(&self.connection, &record)?;
        }
        Ok(())
    }
}

impl ObservationRepository for SqliteObservationRepository {
    fn publish(&mut self, request: PublishRequest) -> PublishOutcome {
        sqlite_publish::publish(
            &mut self.connection,
            self.limits,
            &mut self.ambiguous,
            &request,
            None,
        )
    }

    fn current(&self, key: &SectionKey) -> Result<Option<CurrentRevision>, RepositoryError> {
        sqlite_read::current(&self.connection, key)
    }

    fn current_snapshot(&self) -> Result<Vec<CurrentRevision>, RepositoryError> {
        sqlite_current::snapshot(&self.connection)
    }

    fn pin_decision(
        &mut self,
        set: &DecisionRevisionSet,
    ) -> Result<DecisionPinReceipt, RepositoryError> {
        sqlite_pins::pin(&mut self.connection, set)
    }

    fn load_decision_pin(
        &self,
        decision: &DecisionSnapshotId,
    ) -> Result<DecisionRevisionPin, RepositoryError> {
        sqlite_pins::load(&self.connection, decision)
    }

    fn unpin_decision(
        &mut self,
        receipt: &DecisionPinReceipt,
    ) -> Result<UnpinOutcome, RepositoryError> {
        sqlite_pins::unpin(&mut self.connection, receipt)
    }
}

const fn storage(code: &'static str) -> RepositoryError {
    RepositoryError::Storage(RepositoryDiagnostic { code })
}

const fn ambiguous(code: &'static str) -> ReconciliationOutcome {
    ReconciliationOutcome::Ambiguous(RepositoryDiagnostic { code })
}
