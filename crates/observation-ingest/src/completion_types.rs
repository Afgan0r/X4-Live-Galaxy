use std::collections::BTreeMap;

use observation_domain::{
    BatchId, CanonicalizationVersion, CaptureWindow, DigestAlgorithmVersion,
    ObservationPolicyVersion, ObservationSchemaVersion, SectionCompletionEnvelope, SectionKey,
    SectionRevisionId, SectionStartEnvelope, SectionState,
};

use crate::CandidateUsage;
use crate::RejectionReason;
use crate::wire::WireObservation;

#[derive(Clone)]
pub struct StagedBatch {
    pub ordinal: usize,
    pub canonical_bytes: Vec<u8>,
    pub digest: [u8; 32],
    pub envelope: observation_domain::ImmutableBatchEnvelope,
}

pub struct Candidate {
    pub start: SectionStartEnvelope,
    pub usage: CandidateUsage,
    pub started_at: u64,
    pub last_progress_at: u64,
    pub batches: BTreeMap<BatchId, StagedBatch>,
    pub legacy_identity: Option<(String, u64, u64)>,
    pub next_sequence: u64,
    pub legacy_frames: Vec<(WireObservation, u64)>,
    pub context: Option<CandidateContext>,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContractVersions {
    pub(crate) schema: ObservationSchemaVersion,
    pub(crate) policy: ObservationPolicyVersion,
    pub(crate) canonicalization: CanonicalizationVersion,
    pub(crate) digest: DigestAlgorithmVersion,
}

impl ContractVersions {
    pub const fn new(
        schema: ObservationSchemaVersion,
        policy: ObservationPolicyVersion,
        canonicalization: CanonicalizationVersion,
        digest: DigestAlgorithmVersion,
    ) -> Self {
        Self {
            schema,
            policy,
            canonicalization,
            digest,
        }
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateContext {
    pub(crate) versions: ContractVersions,
    capture_window: CaptureWindow,
    state: SectionState,
    pub(crate) dependencies: BTreeMap<SectionKey, SectionRevisionId>,
    pub(crate) expected_current: Option<SectionRevisionId>,
    stable_identity: bool,
}

impl CandidateContext {
    pub const fn new(
        versions: ContractVersions,
        capture_window: CaptureWindow,
        state: SectionState,
        dependencies: BTreeMap<SectionKey, SectionRevisionId>,
        expected_current: Option<SectionRevisionId>,
        stable_identity: bool,
    ) -> Self {
        Self {
            versions,
            capture_window,
            state,
            dependencies,
            expected_current,
            stable_identity,
        }
    }
    pub const fn capture_window(&self) -> CaptureWindow {
        self.capture_window
    }
    pub const fn state(&self) -> SectionState {
        self.state
    }
    pub const fn versions(&self) -> ContractVersions {
        self.versions
    }
    #[must_use]
    pub const fn stable_identity(&self) -> bool {
        self.stable_identity
    }
    #[must_use]
    pub const fn dependencies(&self) -> &BTreeMap<SectionKey, SectionRevisionId> {
        &self.dependencies
    }
    #[must_use]
    pub const fn expected_current(&self) -> Option<SectionRevisionId> {
        self.expected_current
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionCurrent {
    pub(crate) dependencies: BTreeMap<SectionKey, SectionRevisionId>,
    pub(crate) current_pointer: Option<SectionRevisionId>,
}

impl CompletionCurrent {
    pub const fn new(
        dependencies: BTreeMap<SectionKey, SectionRevisionId>,
        current_pointer: Option<SectionRevisionId>,
    ) -> Self {
        Self {
            dependencies,
            current_pointer,
        }
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionCertificate {
    pub envelope: SectionCompletionEnvelope,
    pub batch_count: usize,
    pub record_count: usize,
    pub raw_bytes: usize,
    pub decoded_bytes: usize,
    pub ordered_batch_manifest_digest: [u8; 32],
    pub canonical_content_digest: [u8; 32],
    pub versions: ContractVersions,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompletionOutcome {
    Validated(Box<crate::ValidatedSectionRevision>),
    Rejected(RejectionReason),
}
