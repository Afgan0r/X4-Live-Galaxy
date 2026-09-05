use observation_domain::EnvelopeRecord;
use sha2::{Digest, Sha256};

use crate::{batch_canonical::CanonicalBatch, completion_types::Candidate};

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

pub struct ProducerMaterial {
    pub batch_count: usize,
    pub record_count: usize,
    pub raw_bytes: usize,
    pub decoded_bytes: usize,
    pub manifest: [u8; 32],
    pub content: [u8; 32],
}

pub fn producer_material(
    batches: &[observation_domain::ImmutableBatchEnvelope],
) -> Option<ProducerMaterial> {
    let mut canonical = batches
        .iter()
        .map(|batch| CanonicalBatch::from_envelope(batch).map(|value| (batch, value)))
        .collect::<Option<Vec<_>>>()?;
    canonical.sort_by_key(|(batch, _)| batch.section_ordinal);
    if canonical
        .iter()
        .enumerate()
        .any(|(index, (batch, _))| batch.section_ordinal != index + 1)
    {
        return None;
    }
    let mut manifest = Sha256::new();
    let mut records = Vec::new();
    let mut raw_bytes = 0_usize;
    let mut decoded_bytes = 0_usize;
    for (batch, value) in canonical {
        framed(&mut manifest, &batch.section_ordinal.to_be_bytes());
        framed(&mut manifest, batch.batch_id.as_str().as_bytes());
        framed(&mut manifest, &value.digest);
        raw_bytes = raw_bytes.checked_add(value.bytes.len())?;
        decoded_bytes = decoded_bytes.checked_add(value.decoded_bytes)?;
        records.extend(batch.records.clone());
    }
    records.sort_by(|left, right| left.record_id.cmp(&right.record_id));
    Some(ProducerMaterial {
        batch_count: batches.len(),
        record_count: records.len(),
        raw_bytes,
        decoded_bytes,
        manifest: manifest.finalize().into(),
        content: content_digest(&records),
    })
}

fn framed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}
