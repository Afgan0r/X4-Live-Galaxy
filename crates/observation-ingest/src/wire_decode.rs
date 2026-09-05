use observation_domain::{
    CanonicalizationVersion, CompleteMessage, ControlEnvelope, DigestAlgorithmVersion, EntityId,
    EnvelopeDecodeError, EnvelopeRecord, ObservationPolicyVersion, ObservationSchemaVersion,
    ObservationVersion, ProducerIncarnationId, RecordId, SectionCompletionEnvelope, SectionKey,
    SectionRevisionId, SourceScopeId, TransportEpoch,
};

use crate::wire::{RawCompletion, RawControl, RawRecord};

pub fn control(
    raw: &RawControl,
    control: ControlEnvelope,
) -> Result<CompleteMessage, EnvelopeDecodeError> {
    EnvelopeDecodeError::require_contract(raw.contract_version)?;
    Ok(CompleteMessage::Control(control))
}

pub fn parse_digest(value: &str) -> Result<[u8; 32], EnvelopeDecodeError> {
    if value.len() != 64 {
        return Err(EnvelopeDecodeError::InvalidShape);
    }
    let mut digest = [0_u8; 32];
    for (index, slot) in digest.iter_mut().enumerate() {
        let offset = index * 2;
        *slot = u8::from_str_radix(&value[offset..offset + 2], 16)
            .map_err(|_| EnvelopeDecodeError::InvalidShape)?;
    }
    Ok(digest)
}

pub fn decode_records(records: Vec<RawRecord>) -> Result<Vec<EnvelopeRecord>, EnvelopeDecodeError> {
    records
        .into_iter()
        .map(|record| {
            Ok(EnvelopeRecord {
                record_id: RecordId::new(record.record_id)
                    .ok_or(EnvelopeDecodeError::InvalidIdentity)?,
                entity_id: EntityId::new(record.entity_id)
                    .ok_or(EnvelopeDecodeError::InvalidIdentity)?,
                observation_version: ObservationVersion::new(record.observation_version)
                    .ok_or(EnvelopeDecodeError::InvalidVersion)?,
                content: (!record.content.is_empty())
                    .then_some(record.content)
                    .ok_or(EnvelopeDecodeError::InvalidShape)?,
            })
        })
        .collect()
}

pub fn decode_completion(raw: RawCompletion) -> Result<CompleteMessage, EnvelopeDecodeError> {
    EnvelopeDecodeError::require_contract(raw.contract_version)?;
    Ok(CompleteMessage::SectionCompletion(
        SectionCompletionEnvelope {
            source_scope: SourceScopeId::new(raw.source_scope)
                .ok_or(EnvelopeDecodeError::InvalidIdentity)?,
            producer_incarnation: ProducerIncarnationId::new(raw.producer_incarnation)
                .ok_or(EnvelopeDecodeError::InvalidIdentity)?,
            transport_epoch: TransportEpoch::new(raw.transport_epoch)
                .ok_or(EnvelopeDecodeError::InvalidVersion)?,
            section_key: SectionKey::new(raw.section_key)
                .ok_or(EnvelopeDecodeError::InvalidIdentity)?,
            section_revision: SectionRevisionId::new(raw.section_revision)
                .ok_or(EnvelopeDecodeError::InvalidVersion)?,
            batch_count: raw.batch_count,
            record_count: raw.record_count,
            raw_bytes: raw.raw_bytes,
            decoded_bytes: raw.decoded_bytes,
            ordered_batch_manifest_digest: parse_digest(&raw.ordered_batch_manifest_digest)?,
            canonical_content_digest: parse_digest(&raw.canonical_content_digest)?,
            schema_version: ObservationSchemaVersion::new(raw.schema_version)
                .ok_or(EnvelopeDecodeError::InvalidVersion)?,
            policy_version: ObservationPolicyVersion::new(raw.policy_version)
                .ok_or(EnvelopeDecodeError::InvalidVersion)?,
            canonicalization_version: CanonicalizationVersion::new(raw.canonicalization_version)
                .ok_or(EnvelopeDecodeError::InvalidVersion)?,
            digest_version: DigestAlgorithmVersion::new(raw.digest_version)
                .ok_or(EnvelopeDecodeError::InvalidVersion)?,
            coverage: SectionCompletionEnvelope::coverage_from_wire(&raw.coverage)
                .ok_or(EnvelopeDecodeError::InvalidShape)?,
        },
    ))
}
