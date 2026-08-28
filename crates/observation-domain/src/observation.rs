use crate::{EntityId, ObservationSource, ObservationTime, ObservationVersion};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservationRecordError {
    EmptyContent,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationRecord {
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
            entity_id,
            source,
            observed_at,
            version,
            content,
        })
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
    if accepted.entity_id != candidate.entity_id {
        return DuplicateDecision::DifferentIdentity;
    }

    if accepted.version == candidate.version && accepted.content == candidate.content {
        DuplicateDecision::Idempotent
    } else {
        DuplicateDecision::Conflict
    }
}
