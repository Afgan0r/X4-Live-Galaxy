use crate::ProductionError;
use observation_application::ObservationLifecycle;
use observation_domain::{
    BatchId, CompleteMessage, ImmutableBatchEnvelope, SectionCompletionEnvelope,
};
use observation_ingest::bind_completion_certificate;
use observation_persistence::{CurrentRevision, ObservationRepository};

pub fn is_committed<R: ObservationRepository>(
    lifecycle: &ObservationLifecycle<R>,
    message: &CompleteMessage,
) -> Result<bool, ProductionError> {
    let CompleteMessage::SectionCompletion(done) = message else {
        return Ok(false);
    };
    if done.section_key.as_str() != "ship_core"
        && !crate::receiver_ship_detail::is_key(done.section_key.as_str())
    {
        return Ok(false);
    }
    let current = lifecycle
        .current_revision(&done.section_key)
        .map_err(|_| ProductionError::Storage)?;
    Ok(current.as_ref().is_some_and(|v| matches(done, v)))
}

fn matches(done: &SectionCompletionEnvelope, current: &CurrentRevision) -> bool {
    let revision = current.revision();
    let revision_id = revision.revision;
    let context = revision
        .context
        .candidate(revision.dependencies.clone(), revision.expected_current);
    let versions = context.versions();
    if done.section_revision != revision_id
        || done.source_scope != revision.source_scope
        || done.producer_incarnation != *revision.source_session.producer_incarnation()
        || done.transport_epoch != revision.source_session.transport_epoch()
        || done.canonical_content_digest != revision.content_digest
        || done.ordered_batch_manifest_digest != revision.manifest_digest
        || done.coverage != revision.coverage
        || done.sender_evidence.section_state != context.state()
        || done.sender_evidence.stable_identity != context.stable_identity()
        || done.record_count != revision.records.len()
        || done.schema_version != versions.schema()
        || done.policy_version != versions.policy()
        || done.canonicalization_version != versions.canonicalization()
        || done.digest_version != versions.digest()
    {
        return false;
    }
    if let Some((batch_count, raw_bytes)) = revision.context.completion_counts() {
        return done.batch_count == batch_count && done.raw_bytes == raw_bytes;
    }
    let Some(batches) = legacy_batches(done, current) else {
        return false;
    };
    bind_completion_certificate(done.clone(), &batches, versions)
        .is_some_and(|bound| bound == *done)
}

fn legacy_batches(
    done: &SectionCompletionEnvelope,
    current: &CurrentRevision,
) -> Option<Vec<ImmutableBatchEnvelope>> {
    let revision = current.revision();
    let mut batches = Vec::new();
    for (index, record) in revision.records.iter().enumerate() {
        let ordinal = index.checked_add(1)?;
        batches.push(envelope(done, vec![record.clone()], ordinal)?);
    }
    Some(batches)
}

fn envelope(
    done: &SectionCompletionEnvelope,
    records: Vec<observation_domain::EnvelopeRecord>,
    ordinal: usize,
) -> Option<ImmutableBatchEnvelope> {
    Some(ImmutableBatchEnvelope {
        source_scope: done.source_scope.clone(),
        producer_incarnation: done.producer_incarnation.clone(),
        transport_epoch: done.transport_epoch,
        section_key: done.section_key.clone(),
        section_revision: done.section_revision,
        batch_id: BatchId::new(format!(
            "carrier-b:{}:{}:{ordinal}",
            done.section_revision.get(),
            done.transport_epoch.get()
        ))?,
        section_ordinal: ordinal,
        records,
        optional_detail: None,
    })
}
