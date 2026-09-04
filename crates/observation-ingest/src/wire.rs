use crate::runtime_facts::RuntimeFacts;
pub use observation_domain::FrameHeader;
use observation_domain::{
    BatchId, CompleteMessage, ControlEnvelope, EntityId, EnvelopeDecodeError, EnvelopeRecord,
    ImmutableBatchEnvelope, ObservationVersion, ProducerIncarnationId, RecordId,
    SectionCompletionEnvelope, SectionKey, SectionRevisionId, SectionStartEnvelope, SourceScopeId,
    TransportEpoch,
};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WireFrame {
    Hello(WireHello),
    Heartbeat(WireHeartbeat),
    RuntimeHealth(WireRuntimeHealth),
    Observation(WireObservation),
    CompleteMarker(WireCompleteMarker),
}
macro_rules! wire_struct {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name { $(pub $field: $ty),* }
    };
}
wire_struct!(WireHello { protocol_major: u16, game_build: String, capabilities: Vec<String>, generation: u64 });
wire_struct!(WireHeartbeat { scope: String, version: u64, generation: Option<u64>, sequence: Option<u64> });
wire_struct!(WireRuntimeHealth { scope: String, version: u64, status: String, generation: Option<u64>, sequence: Option<u64> });
wire_struct!(WireObservation { scope: String, entity_id: String, version: u64, quality: TracerQuality, runtime_facts: RuntimeFacts, generation: Option<u64>, sequence: Option<u64> });
wire_struct!(WireCompleteMarker { scope: String, version: u64, generation: Option<u64>, sequence: Option<u64> });
wire_struct!(TracerObservation {
    entity_id: String,
    observed_at_unix_millis: u64,
    version: u64,
    quality: TracerQuality
});
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TracerQuality {
    Fresh,
    KnownEmpty,
    Unknown,
    Partial,
    Stale,
    Unsupported,
}
impl From<TracerQuality> for observation_domain::SectionQuality {
    fn from(value: TracerQuality) -> Self {
        match value {
            TracerQuality::Fresh => Self::Fresh,
            TracerQuality::KnownEmpty => Self::KnownEmpty,
            TracerQuality::Unknown => Self::Unknown,
            TracerQuality::Partial => Self::Partial,
            TracerQuality::Stale => Self::Stale,
            TracerQuality::Unsupported => Self::Unsupported,
        }
    }
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RawEnvelope {
    SectionStart(RawSectionStart),
    ImmutableBatch(RawBatch),
    SectionCompletion(RawCompletion),
    Handshake(RawControl),
    Demand(RawControl),
    Disposition(RawControl),
    CollectionIntent(RawControl),
    Health(RawControl),
    Reset(RawControl),
}
wire_struct!(RawSectionStart {
    contract_version: u64,
    source_scope: String,
    producer_incarnation: String,
    transport_epoch: u64,
    section_key: String,
    section_revision: u64,
    expected_records: usize
});
wire_struct!(RawBatch { contract_version: u64, source_scope: String, producer_incarnation: String, transport_epoch: u64, section_key: String, section_revision: u64, batch_id: String, records: Vec<RawRecord>, optional_detail: Option<String> });
wire_struct!(RawCompletion {
    contract_version: u64,
    source_scope: String,
    producer_incarnation: String,
    transport_epoch: u64,
    section_key: String,
    section_revision: u64,
    record_count: usize,
    coverage: String
});
wire_struct!(RawControl {
    contract_version: u64
});
wire_struct!(RawRecord {
    record_id: String,
    entity_id: String,
    observation_version: u64,
    content: String
});
pub fn decode_complete_message(
    bytes: &[u8],
    limit: usize,
) -> Result<CompleteMessage, EnvelopeDecodeError> {
    if bytes.len() > limit {
        return Err(EnvelopeDecodeError::MessageTooLarge);
    }
    let raw = serde_json::from_slice(bytes).map_err(|_| EnvelopeDecodeError::InvalidShape)?;
    decode_raw(raw)
}
macro_rules! identity {
    ($ty:ident, $value:expr) => {
        $ty::new($value).ok_or(EnvelopeDecodeError::InvalidIdentity)?
    };
}
macro_rules! number {
    ($ty:ident, $value:expr) => {
        $ty::new($value).ok_or(EnvelopeDecodeError::InvalidVersion)?
    };
}

fn decode_raw(raw: RawEnvelope) -> Result<CompleteMessage, EnvelopeDecodeError> {
    match raw {
        RawEnvelope::SectionStart(raw) => {
            EnvelopeDecodeError::require_contract(raw.contract_version)?;
            Ok(CompleteMessage::SectionStart(SectionStartEnvelope {
                source_scope: identity!(SourceScopeId, raw.source_scope),
                producer_incarnation: identity!(ProducerIncarnationId, raw.producer_incarnation),
                transport_epoch: number!(TransportEpoch, raw.transport_epoch),
                section_key: identity!(SectionKey, raw.section_key),
                section_revision: number!(SectionRevisionId, raw.section_revision),
                expected_records: raw.expected_records,
            }))
        }
        RawEnvelope::ImmutableBatch(raw) => {
            EnvelopeDecodeError::require_contract(raw.contract_version)?;
            Ok(CompleteMessage::ImmutableBatch(ImmutableBatchEnvelope {
                source_scope: identity!(SourceScopeId, raw.source_scope),
                producer_incarnation: identity!(ProducerIncarnationId, raw.producer_incarnation),
                transport_epoch: number!(TransportEpoch, raw.transport_epoch),
                section_key: identity!(SectionKey, raw.section_key),
                section_revision: number!(SectionRevisionId, raw.section_revision),
                batch_id: identity!(BatchId, raw.batch_id),
                records: decode_records(raw.records)?,
                optional_detail: raw.optional_detail,
            }))
        }
        RawEnvelope::SectionCompletion(raw) => {
            EnvelopeDecodeError::require_contract(raw.contract_version)?;
            Ok(CompleteMessage::SectionCompletion(
                SectionCompletionEnvelope {
                    source_scope: identity!(SourceScopeId, raw.source_scope),
                    producer_incarnation: identity!(
                        ProducerIncarnationId,
                        raw.producer_incarnation
                    ),
                    transport_epoch: number!(TransportEpoch, raw.transport_epoch),
                    section_key: identity!(SectionKey, raw.section_key),
                    section_revision: number!(SectionRevisionId, raw.section_revision),
                    record_count: raw.record_count,
                    coverage: SectionCompletionEnvelope::coverage_from_wire(&raw.coverage)
                        .ok_or(EnvelopeDecodeError::InvalidShape)?,
                },
            ))
        }
        RawEnvelope::Handshake(raw) => control(&raw, ControlEnvelope::Handshake),
        RawEnvelope::Demand(raw) => control(&raw, ControlEnvelope::Demand),
        RawEnvelope::Disposition(raw) => control(&raw, ControlEnvelope::Disposition),
        RawEnvelope::CollectionIntent(raw) => control(&raw, ControlEnvelope::CollectionIntent),
        RawEnvelope::Health(raw) => control(&raw, ControlEnvelope::Health),
        RawEnvelope::Reset(raw) => control(&raw, ControlEnvelope::Reset),
    }
}

fn control(
    raw: &RawControl,
    control: ControlEnvelope,
) -> Result<CompleteMessage, EnvelopeDecodeError> {
    EnvelopeDecodeError::require_contract(raw.contract_version)?;
    Ok(CompleteMessage::Control(control))
}
fn decode_records(records: Vec<RawRecord>) -> Result<Vec<EnvelopeRecord>, EnvelopeDecodeError> {
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
