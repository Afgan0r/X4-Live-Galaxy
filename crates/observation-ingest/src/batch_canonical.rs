use observation_domain::ImmutableBatchEnvelope;
use sha2::{Digest, Sha256};

pub struct CanonicalBatch {
    pub bytes: Vec<u8>,
    pub decoded_bytes: usize,
    pub digest: [u8; 32],
}

impl CanonicalBatch {
    pub fn from_envelope(batch: &ImmutableBatchEnvelope) -> Option<Self> {
        let mut bytes = Vec::new();
        framed(&mut bytes, b"observation-batch-v1");
        framed(&mut bytes, batch.source_scope.as_str().as_bytes());
        framed(&mut bytes, batch.producer_incarnation.as_str().as_bytes());
        framed(&mut bytes, &batch.transport_epoch.get().to_be_bytes());
        framed(&mut bytes, batch.section_key.as_str().as_bytes());
        framed(&mut bytes, &batch.section_revision.get().to_be_bytes());
        framed(&mut bytes, batch.batch_id.as_str().as_bytes());
        framed(
            &mut bytes,
            &u64::try_from(batch.records.len()).ok()?.to_be_bytes(),
        );
        for record in &batch.records {
            framed(&mut bytes, record.record_id.as_str().as_bytes());
            framed(&mut bytes, record.entity_id.as_str().as_bytes());
            framed(&mut bytes, &record.observation_version.get().to_be_bytes());
            framed(&mut bytes, record.content.as_bytes());
        }
        match &batch.optional_detail {
            Some(detail) => {
                bytes.push(1);
                framed(&mut bytes, detail.as_bytes());
            }
            None => bytes.push(0),
        }
        let decoded_bytes = bytes.len().checked_add(
            batch
                .records
                .len()
                .checked_mul(std::mem::size_of::<observation_domain::EnvelopeRecord>())?,
        )?;
        let digest = Sha256::digest(&bytes).into();
        Some(Self {
            bytes,
            decoded_bytes,
            digest,
        })
    }
}

fn framed(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    bytes.extend_from_slice(value);
}
