#![forbid(unsafe_code)]

mod checkpoint;
mod fake_port;
mod port;

pub use checkpoint::{CheckpointEnvelope, GAME_PROTOCOL_IDENTITY, SCHEMA_VERSION};
pub use fake_port::FakeCheckpointPort;
pub use port::{CheckpointAck, CheckpointPort, CompatibilityStatus, PortError};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointCursor {
    pub(crate) sequence: u64,
    pub(crate) integrity_hash: String,
}

impl CheckpointCursor {
    #[must_use]
    pub const fn new(sequence: u64, integrity_hash: String) -> Self {
        Self {
            sequence,
            integrity_hash,
        }
    }
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
    #[must_use]
    pub fn integrity_hash(&self) -> &str {
        &self.integrity_hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointDraft {
    pub(crate) snapshot: String,
    pub(crate) tick: String,
    pub(crate) replay: String,
    pub(crate) admission: String,
    pub(crate) report: String,
}

impl CheckpointDraft {
    #[must_use]
    pub fn new(snapshot: &str, tick: &str, replay: &str, admission: &str, report: &str) -> Self {
        Self {
            snapshot: snapshot.into(),
            tick: tick.into(),
            replay: replay.into(),
            admission: admission.into(),
            report: report.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointError {
    InvalidHash,
    InvalidIdentity,
    Malformed,
    Oversized,
    SequenceMismatch,
}
