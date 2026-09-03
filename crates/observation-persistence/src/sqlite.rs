use std::path::Path;

use observation_domain::{DecisionSnapshotId, SectionKey};
use observation_ingest::DecisionRevisionSet;
use rusqlite::Connection;

use crate::{
    CurrentRevision, DecisionPinReceipt, DecisionRevisionPin, ObservationRepository,
    PublicationLimits, PublishOutcome, PublishRequest, RepositoryDiagnostic, RepositoryError,
    UnpinOutcome, schema,
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
        Ok(Self { connection, limits })
    }

    #[must_use]
    pub const fn limits(&self) -> PublicationLimits {
        self.limits
    }
}

impl ObservationRepository for SqliteObservationRepository {
    fn publish(&mut self, _: PublishRequest) -> PublishOutcome {
        let _ = &self.connection;
        PublishOutcome::PermanentRejection(RepositoryDiagnostic {
            code: "sqlite-not-implemented",
        })
    }

    fn current(&self, _: &SectionKey) -> Result<Option<CurrentRevision>, RepositoryError> {
        Ok(None)
    }

    fn pin_decision(
        &mut self,
        _: &DecisionRevisionSet,
    ) -> Result<DecisionPinReceipt, RepositoryError> {
        Err(RepositoryError::Storage(RepositoryDiagnostic {
            code: "sqlite-not-implemented",
        }))
    }

    fn load_decision_pin(
        &self,
        _: &DecisionSnapshotId,
    ) -> Result<DecisionRevisionPin, RepositoryError> {
        Err(RepositoryError::MissingRevision(RepositoryDiagnostic {
            code: "pin-not-found",
        }))
    }

    fn unpin_decision(&mut self, _: &DecisionPinReceipt) -> Result<UnpinOutcome, RepositoryError> {
        Ok(UnpinOutcome::AlreadyAbsent)
    }
}

const fn storage(code: &'static str) -> RepositoryError {
    RepositoryError::Storage(RepositoryDiagnostic { code })
}
