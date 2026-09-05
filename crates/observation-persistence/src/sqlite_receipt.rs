use observation_domain::{SectionKey, SectionRevisionId};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};

use crate::{
    PublicationReceipt, RepositoryDiagnostic, RepositoryError, RevisionRecord, sqlite_read,
};

type StoredReceipt = (Vec<u8>, Option<i64>, i64, i64, Vec<u8>);

pub fn load(
    connection: &Connection,
    key: &SectionKey,
    revision: SectionRevisionId,
) -> Result<Option<PublicationReceipt>, RepositoryError> {
    let row: Option<StoredReceipt> = connection
        .query_row(
            "SELECT content_digest, previous_revision, ordinal, accepted_at, integrity_digest FROM publication_receipts WHERE section_key=?1 AND revision=?2",
            params![key.as_str(), sqlite_read::sql_u64(revision.get())?],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .optional()
        .map_err(|_| storage("receipt-read"))?;
    row.map(|(content, previous, ordinal, accepted_at, integrity)| {
        let receipt = PublicationReceipt {
            section_key: key.clone(),
            revision,
            content_digest: digest(&content)?,
            previous: previous.map(parse_revision).transpose()?,
            ordinal: u64::try_from(ordinal).map_err(|_| corrupt("integer-range"))?,
            accepted_at: u64::try_from(accepted_at).map_err(|_| corrupt("integer-range"))?,
        };
        if integrity_digest(&receipt) != digest(&integrity)? {
            return Err(corrupt("receipt-integrity-mismatch"));
        }
        Ok(receipt)
    })
    .transpose()
}

pub fn integrity_digest(receipt: &PublicationReceipt) -> [u8; 32] {
    let mut digest = Sha256::new();
    hash(&mut digest, receipt.section_key.as_str().as_bytes());
    hash(&mut digest, &receipt.revision.get().to_be_bytes());
    hash(&mut digest, &receipt.content_digest);
    hash(
        &mut digest,
        &receipt
            .previous
            .map_or(0, SectionRevisionId::get)
            .to_be_bytes(),
    );
    hash(&mut digest, &receipt.ordinal.to_be_bytes());
    hash(&mut digest, &receipt.accepted_at.to_be_bytes());
    digest.finalize().into()
}

fn hash(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
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
        receipt.accepted_at,
    );
    let expected = (
        &revision.section_key,
        revision.revision,
        revision.content_digest,
        revision.expected_current,
        revision.accepted_at,
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
