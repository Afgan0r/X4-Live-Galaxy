use crate::{
    CanonicalizationVersion, DigestAlgorithmVersion, ObservationPolicyVersion,
    ObservationSchemaVersion, SectionCoverage, SectionFreshness, SectionState, SourceEpochId,
};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CaptureClock {
    GameTimeMillis,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SourceEpochStatus {
    Known,
    Unknown,
    BoundaryUncertain,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SourceBoundary {
    RuntimeStart,
    GameLoaded,
    LuaReload,
    TransportReconnect,
    Unknown,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SourceConsistency {
    Barrier,
    VersionedManifest,
    EventInterval,
    ObservedCountFillOnly,
    Unknown,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SenderEvidence {
    pub capture_clock: CaptureClock,
    pub section_state: SectionState,
    pub source_epoch: Option<SourceEpochId>,
    pub source_epoch_status: SourceEpochStatus,
    pub source_boundary: SourceBoundary,
    pub source_consistency: SourceConsistency,
    pub stable_identity: bool,
    pub schema_version: ObservationSchemaVersion,
    pub policy_version: ObservationPolicyVersion,
    pub canonicalization_version: CanonicalizationVersion,
    pub digest_version: DigestAlgorithmVersion,
}

impl SenderEvidence {
    pub const fn legacy_default() -> Self {
        Self {
            capture_clock: CaptureClock::GameTimeMillis,
            section_state: SectionState::new(SectionFreshness::Fresh, SectionCoverage::Complete),
            source_epoch: None,
            source_epoch_status: SourceEpochStatus::Unknown,
            source_boundary: SourceBoundary::Unknown,
            source_consistency: SourceConsistency::Unknown,
            stable_identity: false,
            schema_version: schema(1),
            policy_version: policy(2),
            canonicalization_version: canonicalization(3),
            digest_version: digest(1),
        }
    }
}

const fn schema(value: u64) -> ObservationSchemaVersion {
    match ObservationSchemaVersion::new(value) {
        Some(value) => value,
        None => unreachable!(),
    }
}
const fn policy(value: u64) -> ObservationPolicyVersion {
    match ObservationPolicyVersion::new(value) {
        Some(value) => value,
        None => unreachable!(),
    }
}
const fn canonicalization(value: u64) -> CanonicalizationVersion {
    match CanonicalizationVersion::new(value) {
        Some(value) => value,
        None => unreachable!(),
    }
}
const fn digest(value: u64) -> DigestAlgorithmVersion {
    match DigestAlgorithmVersion::new(value) {
        Some(value) => value,
        None => unreachable!(),
    }
}
