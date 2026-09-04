use observation_domain::{SectionKey, SectionRevisionId};
use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    PublicationReceipt, RepositoryDiagnostic, RepositoryError, RevisionRecord, sqlite_read,
};

pub fn load(
    connection: &Connection,
    key: &SectionKey,
    revision: SectionRevisionId,
) -> Result<Option<PublicationReceipt>, RepositoryError> {
    let row: Option<(Vec<u8>, Option<i64>, i64)> = connection
        .query_row(
            "SELECT content_digest, previous_revision, ordinal FROM publication_receipts WHERE section_key=?1 AND revision=?2",
            params![key.as_str(), sqlite_read::sql_u64(revision.get())?],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|_| storage("receipt-read"))?;
    row.map(|(content, previous, ordinal)| {
        Ok(PublicationReceipt {
            section_key: key.clone(),
            revision,
            content_digest: digest(&content)?,
            previous: previous.map(parse_revision).transpose()?,
            ordinal: u64::try_from(ordinal).map_err(|_| corrupt("integer-range"))?,
        })
    })
    .transpose()
}

pub fn load_validated(
    connection: &Connection,
    revision: &RevisionRecord,
) -> Result<PublicationReceipt, RepositoryError> {
    let receipt = load(connection, &revision.section_key, revision.revision)?
        .ok_or(corrupt("receipt-missing"))?;
    let authority = (
        &receipt.section_key,
        receipt.revision,
        receipt.content_digest,
        receipt.previous,
    );
    let expected = (
        &revision.section_key,
        revision.revision,
        revision.content_digest,
        revision.expected_current,
    );
    if authority != expected {
        return Err(corrupt("receipt-integrity-mismatch"));
    }
    Ok(receipt)
}

fn parse_revision(value: i64) -> Result<SectionRevisionId, RepositoryError> {
    u64::try_from(value)
        .ok()
        .and_then(SectionRevisionId::new)
        .ok_or(corrupt("revision-invalid"))
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
