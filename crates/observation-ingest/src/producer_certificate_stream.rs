use observation_domain::{ImmutableBatchEnvelope, RecordId, SectionCompletionEnvelope};
use sha2::{Digest, Sha256};

use crate::batch_canonical::CanonicalBatch;

/// Constant-sized certificate state for batches whose record IDs arrive in
/// canonical ascending order. Historical unordered producers use the sorter.
pub struct ProducerCertificateStream {
    manifest: Sha256,
    content: Sha256,
    batches: usize,
    records: usize,
    bytes: usize,
    last_record: Option<RecordId>,
}

impl Default for ProducerCertificateStream {
    fn default() -> Self {
        Self {
            manifest: Sha256::new(),
            content: Sha256::new(),
            batches: 0,
            records: 0,
            bytes: 0,
            last_record: None,
        }
    }
}

impl ProducerCertificateStream {
    pub fn append(&mut self, batch: &ImmutableBatchEnvelope) -> Option<()> {
        let ordinal = self.batches.checked_add(1)?;
        if batch.section_ordinal != ordinal || batch.records.is_empty() {
            return None;
        }
        let last = ordered_last(self.last_record.as_ref(), batch)?;
        let canonical = CanonicalBatch::from_envelope(batch)?;
        let bytes = self.bytes.checked_add(canonical.bytes.len())?;
        let records = self.records.checked_add(batch.records.len())?;
        framed(&mut self.manifest, &ordinal.to_be_bytes());
        framed(&mut self.manifest, batch.batch_id.as_str().as_bytes());
        framed(&mut self.manifest, &canonical.digest);
        for record in &batch.records {
            framed(&mut self.content, record.record_id.as_str().as_bytes());
            framed(&mut self.content, record.entity_id.as_str().as_bytes());
            framed(
                &mut self.content,
                &record.observation_version.get().to_be_bytes(),
            );
            framed(&mut self.content, record.content.as_bytes());
        }
        self.last_record = Some(last.clone());
        self.batches = ordinal;
        self.records = records;
        self.bytes = bytes;
        Some(())
    }

    #[must_use]
    pub fn bind(&self, mut completion: SectionCompletionEnvelope) -> SectionCompletionEnvelope {
        completion.batch_count = self.batches;
        completion.record_count = self.records;
        completion.raw_bytes = self.bytes;
        completion.ordered_batch_manifest_digest = self.manifest.clone().finalize().into();
        completion.canonical_content_digest = self.content.clone().finalize().into();
        completion
    }
}

fn ordered_last<'a>(
    prior: Option<&RecordId>,
    batch: &'a ImmutableBatchEnvelope,
) -> Option<&'a RecordId> {
    let first = batch.records.first()?;
    if prior.is_some_and(|value| value >= &first.record_id)
        || batch
            .records
            .windows(2)
            .any(|pair| pair[0].record_id >= pair[1].record_id)
    {
        return None;
    }
    batch.records.last().map(|record| &record.record_id)
}

fn framed(digest: &mut Sha256, value: &[u8]) {
    digest.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    digest.update(value);
}
