use observation_domain::{ImmutableBatchEnvelope, SectionCompletionEnvelope};

use crate::{ContractVersions, completion_digest::producer_material};

#[must_use]
pub fn bind_completion_certificate(
    mut envelope: SectionCompletionEnvelope,
    batches: &[ImmutableBatchEnvelope],
    versions: ContractVersions,
) -> Option<SectionCompletionEnvelope> {
    let material = producer_material(batches)?;
    envelope.batch_count = material.batch_count;
    envelope.record_count = material.record_count;
    envelope.raw_bytes = material.raw_bytes;
    envelope.decoded_bytes = material.decoded_bytes;
    envelope.ordered_batch_manifest_digest = material.manifest;
    envelope.canonical_content_digest = material.content;
    envelope.schema_version = versions.schema;
    envelope.policy_version = versions.policy;
    envelope.canonicalization_version = versions.canonicalization;
    envelope.digest_version = versions.digest;
    Some(envelope)
}
