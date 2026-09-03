use observation_domain::EnvelopeRecord;
use sha2::{Digest, Sha256};

use crate::completion_types::Candidate;

pub fn candidate_material(candidate: &Candidate) -> ([u8; 32], Vec<EnvelopeRecord>) {
    let mut batches: Vec<_> = candidate.batches.values().collect();
    batches.sort_by_key(|batch| batch.ordinal);
    let mut manifest = Sha256::new();
    let mut records = Vec::new();
    for batch in batches {
        framed(&mut manifest, &batch.ordinal.to_be_bytes());
        framed(&mut manifest, batch.envelope.batch_id.as_str().as_bytes());
        framed(&mut manifest, &batch.digest);
        records.extend(batch.envelope.records.clone());
    }
    records.sort_by(|left, right| left.record_id.cmp(&right.record_id));
    (manifest.finalize().into(), records)
}

pub fn content_digest(records: &[EnvelopeRecord]) -> [u8; 32] {
    let mut digest = Sha256::new();
    for record in records {
        framed(&mut digest, record.record_id.as_str().as_bytes());
        framed(&mut digest, record.entity_id.as_str().as_bytes());
        framed(&mut digest, &record.observation_version.get().to_be_bytes());
        framed(&mut digest, record.content.as_bytes());
    }
    digest.finalize().into()
}

fn framed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}
