use observation_domain::{
    BatchId, CompleteMessage, CompletionCoverage, EntityId, EnvelopeRecord, ImmutableBatchEnvelope,
    ObservationVersion, ProducerIncarnationId, RecordId, SectionCompletionEnvelope, SectionKey,
    SectionRevisionId, SectionStartEnvelope, SourceScopeId, TransportEpoch,
};
use observation_ingest::{ContractVersions, bind_completion_certificate, encode_complete_message};

use crate::{ProducerError, ProducerSource, SectionEvidence, TypedFact};

pub struct SectionMessages {
    pub start: Vec<u8>,
    pub batch: Vec<u8>,
    pub completion: Vec<u8>,
}

pub fn assemble(
    source: &ProducerSource,
    evidence: &SectionEvidence,
    fact: &TypedFact,
    revision: u64,
    limit: usize,
) -> Result<SectionMessages, ProducerError> {
    let scope =
        SourceScopeId::new(evidence.source_scope.clone()).ok_or(ProducerError::InvalidInput)?;
    let producer = ProducerIncarnationId::new(source.producer_incarnation.clone())
        .ok_or(ProducerError::InvalidInput)?;
    let epoch = TransportEpoch::new(source.transport_epoch).ok_or(ProducerError::StaleEpoch)?;
    let key = SectionKey::new("carrier_b_realtime_sample").ok_or(ProducerError::InvalidInput)?;
    let revision = SectionRevisionId::new(revision).ok_or(ProducerError::InvalidInput)?;
    let start = SectionStartEnvelope {
        source_scope: scope.clone(),
        producer_incarnation: producer.clone(),
        transport_epoch: epoch,
        section_key: key.clone(),
        section_revision: revision,
        expected_records: 1,
        sender_evidence: evidence.sender.clone(),
    };
    let content = format!(
        "getter={}\nsemantics={}\nraw_value={}",
        fact.getter, fact.semantics, fact.raw_value
    );
    let batch_id = BatchId::new(format!("carrier-b:{}:{}:1", revision.get(), epoch.get()))
        .ok_or(ProducerError::InvalidInput)?;
    let batch = ImmutableBatchEnvelope {
        source_scope: scope.clone(),
        producer_incarnation: producer.clone(),
        transport_epoch: epoch,
        section_key: key.clone(),
        section_revision: revision,
        batch_id,
        section_ordinal: 1,
        records: vec![EnvelopeRecord {
            record_id: RecordId::new(format!("carrier-b:{}:1", revision.get()))
                .ok_or(ProducerError::InvalidInput)?,
            entity_id: EntityId::new(fact.entity_id.clone()).ok_or(ProducerError::InvalidInput)?,
            observation_version: ObservationVersion::new(fact.observation_version)
                .ok_or(ProducerError::InvalidInput)?,
            content,
        }],
        optional_detail: None,
    };
    let completion = completion(evidence, scope, producer, epoch, key, revision, &batch)?;
    Ok(SectionMessages {
        start: encode_complete_message(&CompleteMessage::SectionStart(start), limit)
            .map_err(|_| ProducerError::DataLimit)?,
        batch: encode_complete_message(&CompleteMessage::ImmutableBatch(batch), limit)
            .map_err(|_| ProducerError::DataLimit)?,
        completion: encode_complete_message(&CompleteMessage::SectionCompletion(completion), limit)
            .map_err(|_| ProducerError::DataLimit)?,
    })
}

fn completion(
    evidence: &SectionEvidence,
    scope: SourceScopeId,
    producer: ProducerIncarnationId,
    epoch: TransportEpoch,
    key: SectionKey,
    revision: SectionRevisionId,
    batch: &ImmutableBatchEnvelope,
) -> Result<SectionCompletionEnvelope, ProducerError> {
    let versions = ContractVersions::new(
        evidence.sender.schema_version,
        evidence.sender.policy_version,
        evidence.sender.canonicalization_version,
        evidence.sender.digest_version,
    );
    bind_completion_certificate(
        SectionCompletionEnvelope {
            source_scope: scope,
            producer_incarnation: producer,
            transport_epoch: epoch,
            section_key: key,
            section_revision: revision,
            batch_count: 0,
            record_count: 0,
            raw_bytes: 0,
            ordered_batch_manifest_digest: [0; 32],
            canonical_content_digest: [0; 32],
            schema_version: versions.schema(),
            policy_version: versions.policy(),
            canonicalization_version: versions.canonicalization(),
            digest_version: versions.digest(),
            coverage: CompletionCoverage::PointMeasurement,
            sender_evidence: evidence.sender.clone(),
        },
        core::slice::from_ref(batch),
        versions,
    )
    .ok_or(ProducerError::InvalidInput)
}
