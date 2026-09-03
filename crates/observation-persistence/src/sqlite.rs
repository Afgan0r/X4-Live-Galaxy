use std::path::Path;

use observation_domain::{DecisionSnapshotId, SectionKey};
use observation_ingest::DecisionRevisionSet;
use rusqlite::Connection;

use crate::{
    CurrentRevision, DecisionPinReceipt, DecisionRevisionPin, ObservationRepository,
    PublicationLimits, PublishOutcome, PublishRequest, RepositoryDiagnostic, RepositoryError,
    UnpinOutcome, schema, sqlite_pins, sqlite_read, sqlite_write,
};

pub struct SqliteObservationRepository {
    connection: Connection,
    limits: PublicationLimits,
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
        let repository = Self { connection, limits };
        repository.validate_stored_revisions()?;
        Ok(repository)
    }

    pub fn foreign_keys_enabled(&self) -> Result<bool, RepositoryError> {
        self.connection
            .pragma_query_value(None, "foreign_keys", |row| row.get::<_, i64>(0))
            .map(|value| value == 1)
            .map_err(|_| storage("foreign-keys-query"))
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
            let _ = sqlite_read::load_revision(&self.connection, &key, revision)?
                .ok_or_else(|| storage("revision-missing"))?;
        }
        Ok(())
    }
}

impl ObservationRepository for SqliteObservationRepository {
    fn publish(&mut self, request: PublishRequest) -> PublishOutcome {
        sqlite_write::publish(&mut self.connection, self.limits, &request)
    }

    fn current(&self, key: &SectionKey) -> Result<Option<CurrentRevision>, RepositoryError> {
        sqlite_read::current(&self.connection, key)
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
