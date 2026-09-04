use std::collections::BTreeSet;
use std::path::Path;

use observation_domain::{DecisionSnapshotId, SectionKey};
use observation_ingest::DecisionRevisionSet;
use rusqlite::Connection;

use crate::{
    CurrentRevision, DecisionPinReceipt, DecisionRevisionPin, ObservationRepository,
    PublicationLimits, PublishOutcome, PublishRequest, RepositoryDiagnostic, RepositoryError,
    UnpinOutcome, record, retention, schema, sqlite_pins, sqlite_read, sqlite_write,
};
use crate::{PublicationFailpoint, ReconciliationOutcome, RetentionPolicy, RetentionReport};

pub struct SqliteObservationRepository {
    connection: Connection,
    limits: PublicationLimits,
    ambiguous: BTreeSet<(SectionKey, observation_domain::SectionRevisionId)>,
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
        let repository = Self {
            connection,
            limits,
            ambiguous: BTreeSet::new(),
        };
        repository.validate_stored_revisions()?;
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
        let identity = (
            request.revision.section_key().clone(),
            request.revision.section_revision(),
        );
        let outcome = sqlite_write::publish_with_failpoint(
            &mut self.connection,
            self.limits,
            request,
            Some(failpoint),
        );
        if matches!(outcome, PublishOutcome::Ambiguous(_)) {
            self.ambiguous.insert(identity);
        }
        outcome
    }

    pub fn reconcile_publication(&mut self, request: &PublishRequest) -> ReconciliationOutcome {
        if schema::validate_foreign_keys(&self.connection).is_err()
            || self.validate_stored_revisions().is_err()
        {
            return ambiguous("reconciliation-corrupt");
        }
        let Some(candidate) = record::normalize(request, self.limits) else {
            return ambiguous("reconciliation-invalid");
        };
        let identity = (candidate.section_key.clone(), candidate.revision);
        let receipt =
            sqlite_read::load_receipt(&self.connection, &candidate.section_key, candidate.revision);
        let current = sqlite_read::current_pointer(&self.connection, &candidate.section_key);
        let outcome = match (receipt, current) {
            (Ok(Some(receipt)), Ok(Some(current)))
                if current == candidate.revision
                    && receipt.content_digest == candidate.content_digest =>
            {
                ReconciliationOutcome::CommittedReplay(receipt)
            }
            (Ok(None), Ok(current))
                if current == candidate.expected_current && self.dependencies_match(request) =>
            {
                ReconciliationOutcome::ProvenNotCommitted
            }
            _ => ambiguous("reconciliation-ambiguous"),
        };
        if !matches!(outcome, ReconciliationOutcome::Ambiguous(_)) {
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

    fn dependencies_match(&self, request: &PublishRequest) -> bool {
        request.frozen_dependencies.iter().all(|(key, expected)| {
            sqlite_read::current_pointer(&self.connection, key) == Ok(Some(*expected))
        })
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
            let _ = sqlite_read::load_receipt(&self.connection, &key, revision)?
                .ok_or(corrupt("receipt-missing"))?;
        }
        Ok(())
    }
}

impl ObservationRepository for SqliteObservationRepository {
    fn publish(&mut self, request: PublishRequest) -> PublishOutcome {
        if self.ambiguous.contains(&(
            request.revision.section_key().clone(),
            request.revision.section_revision(),
        )) {
            return PublishOutcome::Ambiguous(RepositoryDiagnostic {
                code: "reconciliation-required",
            });
        }
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

const fn ambiguous(code: &'static str) -> ReconciliationOutcome {
    ReconciliationOutcome::Ambiguous(RepositoryDiagnostic { code })
}

const fn corrupt(code: &'static str) -> RepositoryError {
    RepositoryError::Corrupt(RepositoryDiagnostic { code })
}
