use crate::ProductionError;
use observation_application::ObservationLifecycle;
use observation_domain::{
    BatchId, CompleteMessage, ImmutableBatchEnvelope, SectionCompletionEnvelope,
};
use observation_ingest::{ContractVersions, bind_completion_certificate};
use observation_persistence::{CurrentRevision, ObservationRepository};

pub fn is_committed<R: ObservationRepository>(
    lifecycle: &ObservationLifecycle<R>,
    message: &CompleteMessage,
) -> Result<bool, ProductionError> {
    let CompleteMessage::SectionCompletion(done) = message else {
        return Ok(false);
    };
    if done.section_key.as_str() != "ship_core" {
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
    if done.section_revision != revision_id
        || done.source_scope != revision.source_scope
        || done.producer_incarnation != *revision.source_session.producer_incarnation()
        || done.transport_epoch != revision.source_session.transport_epoch()
        || done.canonical_content_digest != revision.content_digest
        || done.ordered_batch_manifest_digest != revision.manifest_digest
        || done.coverage != revision.coverage
        || done.sender_evidence.section_state != context.state()
        || done.sender_evidence.stable_identity != context.stable_identity()
    {
        return false;
    }
    let mut batches = Vec::new();
    for (index, record) in revision.records.iter().enumerate() {
        let ordinal = index + 1;
        let Some(id) = BatchId::new(format!(
            "carrier-b:{}:{}:{ordinal}",
            revision.revision.get(),
            done.transport_epoch.get()
        )) else {
            return false;
        };
        batches.push(ImmutableBatchEnvelope {
            source_scope: revision.source_scope.clone(),
            producer_incarnation: done.producer_incarnation.clone(),
            transport_epoch: done.transport_epoch,
            section_key: revision.section_key.clone(),
            section_revision: revision.revision,
            batch_id: id,
            section_ordinal: ordinal,
            records: vec![record.clone()],
            optional_detail: None,
        });
    }
    let versions: ContractVersions = context.versions();
    bind_completion_certificate(done.clone(), &batches, versions)
        .is_some_and(|bound| bound == *done)
}
