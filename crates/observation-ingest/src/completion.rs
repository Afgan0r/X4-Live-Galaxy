use observation_domain::{CanonicalObservationKey, ObservationVersion};

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletedScope {
    version: ObservationVersion,
    members: Vec<CanonicalObservationKey>,
}

impl CompletedScope {
    pub fn new(version: ObservationVersion, mut members: Vec<CanonicalObservationKey>) -> Self {
        members.sort();
        members.dedup();
        Self { version, members }
    }

    pub const fn version(&self) -> ObservationVersion {
        self.version
    }

    pub fn is_exact_replay(
        &self,
        version: ObservationVersion,
        members: &[CanonicalObservationKey],
    ) -> bool {
        Self::new(version, members.to_vec()) == *self
    }
}
