use super::CheckpointEnvelope;

impl CheckpointEnvelope {
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}
