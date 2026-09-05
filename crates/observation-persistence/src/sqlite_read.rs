use std::collections::BTreeMap;

use observation_domain::{
    EntityId, EnvelopeRecord, ObservationVersion, ProducerIncarnationId, RecordId, SectionKey,
    SectionRevisionId, SourceScopeId, SourceSessionIdentity, TransportEpoch,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    CurrentRevision, RepositoryDiagnostic, RepositoryError, RevisionRecord, record, schema,
    sqlite_receipt,
};

pub fn current(
    connection: &Connection,
    key: &SectionKey,
) -> Result<Option<CurrentRevision>, RepositoryError> {
    let Some(revision) = current_pointer(connection, key)? else {
        return Ok(None);
    };
    let record = load_revision(connection, key, revision)?.ok_or(corrupt("dangling-current"))?;
    let receipt = sqlite_receipt::load_validated(connection, &record)?;
    Ok(Some(CurrentRevision {
        revision: record,
        receipt,
    }))
}

pub fn current_pointer(
    connection: &Connection,
    key: &SectionKey,
) -> Result<Option<SectionRevisionId>, RepositoryError> {
    let value: Option<i64> = connection
        .query_row(
            "SELECT revision FROM current_revisions WHERE section_key=?1",
            [key.as_str()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| storage("current-read"))?;
    value.map(revision).transpose()
}

pub fn load_revision(
    connection: &Connection,
    key: &SectionKey,
    revision_id: SectionRevisionId,
) -> Result<Option<RevisionRecord>, RepositoryError> {
    type Header = (
        String,
        String,
        i64,
        i64,
        String,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        String,
        Option<i64>,
    );
    let header: Option<Header> = connection.query_row(
        "SELECT source_scope, producer_incarnation, transport_epoch, accepted_at, coverage, manifest_digest, content_digest, integrity_digest, context_token, expected_current FROM revisions WHERE section_key=?1 AND revision=?2",
        params![key.as_str(), sql_u64(revision_id.get())?],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?)),
    ).optional().map_err(|_| storage("revision-read"))?;
    let Some((
        source,
        producer,
        epoch,
        accepted_at,
        coverage,
        manifest,
        content_bytes,
        integrity,
        context_payload,
        expected,
    )) = header
    else {
        return Ok(None);
    };
    let records = load_records(connection, key, revision_id)?;
    let stored_digest = digest(&content_bytes)?;
    if record::content_digest(&records) != stored_digest {
        return Err(corrupt("content-digest-mismatch"));
    }
    let record = RevisionRecord {
        source_scope: SourceScopeId::new(source).ok_or(corrupt("source-invalid"))?,
        source_session: SourceSessionIdentity::new(
            ProducerIncarnationId::new(producer).ok_or(corrupt("producer-invalid"))?,
            TransportEpoch::new(rust_u64(epoch)?).ok_or(corrupt("epoch-invalid"))?,
        ),
        section_key: key.clone(),
        revision: revision_id,
        accepted_at: rust_u64(accepted_at)?,
        records,
        coverage: schema::parse_coverage(&coverage)?,
        dependencies: load_dependencies(connection, key, revision_id)?,
        expected_current: expected.map(revision).transpose()?,
        manifest_digest: digest(&manifest)?,
        content_digest: stored_digest,
        integrity_digest: digest(&integrity)?,
        context: crate::PersistedContext::parse(&context_payload)
            .ok_or(corrupt("context-invalid"))?,
    };
    if record::integrity_digest(&record) != record.integrity_digest {
        return Err(corrupt("integrity-digest-mismatch"));
    }
    Ok(Some(record))
}

fn load_records(
    connection: &Connection,
    key: &SectionKey,
    revision_id: SectionRevisionId,
) -> Result<Vec<EnvelopeRecord>, RepositoryError> {
    let mut statement = connection.prepare(
        "SELECT position, record_id, entity_id, observation_version, content FROM revision_records WHERE section_key=?1 AND revision=?2 ORDER BY position"
    ).map_err(|_| storage("records-read"))?;
    let rows = statement
        .query_map(params![key.as_str(), sql_u64(revision_id.get())?], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|_| storage("records-read"))?;
    let mut records = Vec::new();
    for (position, row) in rows.enumerate() {
        let (stored, record_id, entity_id, version, content) =
            row.map_err(|_| storage("records-read"))?;
        if stored != sql_usize(position)? {
            return Err(corrupt("record-order"));
        }
        records.push(EnvelopeRecord {
            record_id: RecordId::new(record_id).ok_or(corrupt("record-id"))?,
            entity_id: EntityId::new(entity_id).ok_or(corrupt("entity-id"))?,
            observation_version: ObservationVersion::new(rust_u64(version)?)
                .ok_or(corrupt("record-version"))?,
            content,
        });
    }
    Ok(records)
}

fn load_dependencies(
    connection: &Connection,
    key: &SectionKey,
    revision_id: SectionRevisionId,
) -> Result<BTreeMap<SectionKey, SectionRevisionId>, RepositoryError> {
    let mut statement = connection.prepare(
        "SELECT dependency_key, dependency_revision FROM revision_dependencies WHERE section_key=?1 AND revision=?2 ORDER BY dependency_key"
    ).map_err(|_| storage("dependencies-read"))?;
    let rows = statement
        .query_map(params![key.as_str(), sql_u64(revision_id.get())?], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|_| storage("dependencies-read"))?;
    let mut dependencies = BTreeMap::new();
    for row in rows {
        let (key, value) = row.map_err(|_| storage("dependencies-read"))?;
        dependencies.insert(
            SectionKey::new(key).ok_or(corrupt("dependency-key"))?,
            revision(value)?,
        );
    }
    Ok(dependencies)
}

pub fn sql_u64(value: u64) -> Result<i64, RepositoryError> {
    i64::try_from(value).map_err(|_| corrupt("integer-range"))
}
fn sql_usize(value: usize) -> Result<i64, RepositoryError> {
    i64::try_from(value).map_err(|_| corrupt("integer-range"))
}
fn rust_u64(value: i64) -> Result<u64, RepositoryError> {
    u64::try_from(value).map_err(|_| corrupt("integer-range"))
}
fn revision(value: i64) -> Result<SectionRevisionId, RepositoryError> {
    SectionRevisionId::new(rust_u64(value)?).ok_or(corrupt("revision-invalid"))
}
fn digest(value: &[u8]) -> Result<[u8; 32], RepositoryError> {
    value.try_into().map_err(|_| corrupt("digest-length"))
}
const fn corrupt(code: &'static str) -> RepositoryError {
    RepositoryError::Corrupt(RepositoryDiagnostic { code })
}
const fn storage(code: &'static str) -> RepositoryError {
    RepositoryError::Storage(RepositoryDiagnostic { code })
}
