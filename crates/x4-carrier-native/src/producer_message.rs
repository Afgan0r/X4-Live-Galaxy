use observation_domain::{
    BatchId, CompleteMessage, CompletionCoverage, EntityId, EnvelopeRecord, ImmutableBatchEnvelope,
    ObservationVersion, ProducerIncarnationId, RecordId, SectionCompletionEnvelope, SectionKey,
    SectionRevisionId, SectionStartEnvelope, SourceScopeId, TransportEpoch,
};
use observation_ingest::{ContractVersions, bind_completion_certificate, encode_complete_message};

use crate::producer_types::PreparedRecord;
use crate::{ProducerError, ProducerSource, SectionEvidence};

pub struct SectionMessages {
    pub section_key: String,
    pub certificate: observation_ingest::ProducerCertificateStream,
}

pub fn assemble(
    source: &ProducerSource,
    evidence: &SectionEvidence,
    section_key: &str,
    expected_records: usize,
    revision: u64,
    limit: usize,
) -> Result<Vec<u8>, ProducerError> {
    let scope =
        SourceScopeId::new(evidence.source_scope.clone()).ok_or(ProducerError::InvalidInput)?;
    let producer = ProducerIncarnationId::new(source.producer_incarnation.clone())
        .ok_or(ProducerError::InvalidInput)?;
    let epoch = TransportEpoch::new(source.transport_epoch).ok_or(ProducerError::StaleEpoch)?;
    let key = SectionKey::new(section_key).ok_or(ProducerError::InvalidInput)?;
    let revision = SectionRevisionId::new(revision).ok_or(ProducerError::InvalidInput)?;
    let start = SectionStartEnvelope {
        source_scope: scope.clone(),
        producer_incarnation: producer.clone(),
        transport_epoch: epoch,
        section_key: key.clone(),
        section_revision: revision,
        expected_records,
        sender_evidence: evidence.sender.clone(),
    };
    encode_complete_message(&CompleteMessage::SectionStart(start), limit)
        .map_err(|_| ProducerError::DataLimit)
}

pub(super) fn batch(
    source: &ProducerSource,
    scope: &SourceScopeId,
    records: &[PreparedRecord],
    section_key: &str,
    revision: SectionRevisionId,
    ordinal: usize,
    first_record_ordinal: usize,
) -> Result<ImmutableBatchEnvelope, ProducerError> {
    Ok(ImmutableBatchEnvelope {
        source_scope: scope.clone(),
        producer_incarnation: ProducerIncarnationId::new(source.producer_incarnation.clone())
            .ok_or(ProducerError::InvalidInput)?,
        transport_epoch: TransportEpoch::new(source.transport_epoch)
            .ok_or(ProducerError::StaleEpoch)?,
        section_key: SectionKey::new(section_key).ok_or(ProducerError::InvalidInput)?,
        section_revision: revision,
        batch_id: BatchId::new(format!(
            "carrier-b:{}:{}:{ordinal}",
            revision.get(),
            source.transport_epoch
        ))
        .ok_or(ProducerError::InvalidInput)?,
        section_ordinal: ordinal,
        records: records
            .iter()
            .enumerate()
            .map(|(offset, record)| {
                let record_ordinal = first_record_ordinal
                    .checked_add(offset)
                    .ok_or(ProducerError::DataLimit)?;
                Ok(EnvelopeRecord {
                    // Certificate order is lexical: variable-width ordinals fail at 10.
                    record_id: RecordId::new(
                        if section_key.starts_with("ship_") || section_key == "faction_census" {
                            format!("carrier-b:{}:{record_ordinal:020}", revision.get())
                        } else {
                            format!("carrier-b:{}:{record_ordinal}", revision.get())
                        },
                    )
                    .ok_or(ProducerError::InvalidInput)?,
                    entity_id: EntityId::new(record.entity_id.clone())
                        .ok_or(ProducerError::InvalidInput)?,
                    observation_version: ObservationVersion::new(revision.get())
                        .ok_or(ProducerError::InvalidInput)?,
                    content: record.content.clone(),
                })
            })
            .collect::<Result<Vec<_>, ProducerError>>()?,
        optional_detail: None,
    })
}

pub(super) fn completion(
    evidence: &SectionEvidence,
    scope: SourceScopeId,
    producer: ProducerIncarnationId,
    epoch: TransportEpoch,
    key: SectionKey,
    revision: SectionRevisionId,
    certificate: &observation_ingest::ProducerCertificateStream,
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
            coverage: match evidence.sender.section_state.coverage() {
                observation_domain::SectionCoverage::Complete => CompletionCoverage::Complete,
                observation_domain::SectionCoverage::KnownEmpty => CompletionCoverage::KnownEmpty,
                observation_domain::SectionCoverage::Partial => CompletionCoverage::Partial,
                observation_domain::SectionCoverage::Unknown => CompletionCoverage::Unknown,
                observation_domain::SectionCoverage::Unsupported => CompletionCoverage::Unsupported,
                observation_domain::SectionCoverage::PointMeasurement => {
                    CompletionCoverage::PointMeasurement
                }
            },
            sender_evidence: evidence.sender.clone(),
        },
        &[],
        versions,
    )
    .map(|envelope| certificate.bind(envelope))
    .ok_or(ProducerError::InvalidInput)
}
