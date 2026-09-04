use crate::{
    BatchId, CompletionCoverage, ControlEnvelope, EntityId, ObservationSource, ObservationTime,
    ObservationVersion, ProducerIncarnationId, RecordId, SectionKey, SectionRevisionId,
    SourceScopeId, TransportEpoch,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservationRecordError {
    EmptyContent,
}

#[must_use]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObservationRecord {
    source_scope: Option<SourceScopeId>,
    record_id: Option<RecordId>,
    entity_id: EntityId,
    source: ObservationSource,
    observed_at: ObservationTime,
    version: ObservationVersion,
    content: String,
}

impl ObservationRecord {
    pub fn new(
        entity_id: EntityId,
        source: ObservationSource,
        observed_at: ObservationTime,
        version: ObservationVersion,
        content: impl Into<String>,
    ) -> Result<Self, ObservationRecordError> {
        let content = content.into();
        if content.is_empty() {
            return Err(ObservationRecordError::EmptyContent);
        }

        Ok(Self {
            source_scope: None,
            record_id: None,
            entity_id,
            source,
            observed_at,
            version,
            content,
        })
    }

    pub fn scoped(
        source_scope: SourceScopeId,
        record_id: RecordId,
        entity_id: EntityId,
        source: ObservationSource,
        observed_at: ObservationTime,
        version: ObservationVersion,
        content: impl Into<String>,
    ) -> Result<Self, ObservationRecordError> {
        let mut record = Self::new(entity_id, source, observed_at, version, content)?;
        record.source_scope = Some(source_scope);
        record.record_id = Some(record_id);
        Ok(record)
    }

    #[must_use]
    pub const fn source_scope(&self) -> Option<&SourceScopeId> {
        self.source_scope.as_ref()
    }

    #[must_use]
    pub const fn record_id(&self) -> Option<&RecordId> {
        self.record_id.as_ref()
    }

    pub const fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }

    pub const fn source(&self) -> ObservationSource {
        self.source
    }

    pub const fn observed_at(&self) -> ObservationTime {
        self.observed_at
    }

    pub const fn version(&self) -> ObservationVersion {
        self.version
    }
    #[must_use]
    pub fn replay_fingerprint(&self) -> u64 {
        let mut bytes = Vec::new();
        if let Some(scope) = &self.source_scope {
            framed(&mut bytes, scope.as_str().as_bytes());
        }
        if let Some(record_id) = &self.record_id {
            framed(&mut bytes, record_id.as_str().as_bytes());
        }
        framed(&mut bytes, self.entity_id.as_str().as_bytes());
        bytes.push(match self.source {
            ObservationSource::X4Runtime => 1,
        });
        bytes.extend_from_slice(&self.observed_at.unix_millis().to_le_bytes());
        bytes.extend_from_slice(&self.version.get().to_le_bytes());
        framed(&mut bytes, self.content.as_bytes());
        bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
        })
    }
}

fn framed(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value);
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DuplicateDecision {
    Idempotent,
    Conflict,
    DifferentIdentity,
}

pub fn classify_duplicate(
    accepted: &ObservationRecord,
    candidate: &ObservationRecord,
) -> DuplicateDecision {
    if accepted.source_scope != candidate.source_scope
        || accepted.record_id != candidate.record_id
        || accepted.entity_id != candidate.entity_id
    {
        return DuplicateDecision::DifferentIdentity;
    }

    if accepted.version == candidate.version && accepted.content == candidate.content {
        DuplicateDecision::Idempotent
    } else {
        DuplicateDecision::Conflict
    }
}

macro_rules! envelope_struct {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name { $(pub $field: $ty),* }
    };
}
envelope_struct!(SectionStartEnvelope {
    source_scope: SourceScopeId,
    producer_incarnation: ProducerIncarnationId,
    transport_epoch: TransportEpoch,
    section_key: SectionKey,
    section_revision: SectionRevisionId,
    expected_records: usize
});
envelope_struct!(EnvelopeRecord {
    record_id: RecordId,
    entity_id: EntityId,
    observation_version: ObservationVersion,
    content: String
});
envelope_struct!(ImmutableBatchEnvelope { source_scope: SourceScopeId, producer_incarnation: ProducerIncarnationId, transport_epoch: TransportEpoch, section_key: SectionKey, section_revision: SectionRevisionId, batch_id: BatchId, records: Vec<EnvelopeRecord>, optional_detail: Option<String> });
envelope_struct!(SectionCompletionEnvelope {
    source_scope: SourceScopeId,
    producer_incarnation: ProducerIncarnationId,
    transport_epoch: TransportEpoch,
    section_key: SectionKey,
    section_revision: SectionRevisionId,
    record_count: usize,
    coverage: CompletionCoverage
});

impl SectionCompletionEnvelope {
    #[must_use]
    pub fn coverage_from_wire(value: &str) -> Option<CompletionCoverage> {
        match value {
            "complete" => Some(CompletionCoverage::Complete),
            "known_empty" => Some(CompletionCoverage::KnownEmpty),
            "partial" => Some(CompletionCoverage::Partial),
            "unknown" => Some(CompletionCoverage::Unknown),
            "unsupported" => Some(CompletionCoverage::Unsupported),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_qualified_known_empty(&self) -> bool {
        self.record_count == 0 && matches!(self.coverage, CompletionCoverage::KnownEmpty)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompleteMessage {
    SectionStart(SectionStartEnvelope),
    ImmutableBatch(ImmutableBatchEnvelope),
    SectionCompletion(SectionCompletionEnvelope),
    Control(ControlEnvelope),
}
