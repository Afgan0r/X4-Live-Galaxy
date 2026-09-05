use std::num::{NonZeroU64, NonZeroUsize};

use observation_domain::{BatchId, TransportEpoch};
use observation_ingest::{
    ApplicationContextIdentity, CandidateContext, CompletionCurrent, ReceiverDisposition,
};

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleInput {
    pub(crate) epoch: TransportEpoch,
    pub(crate) identity: BatchId,
    pub(crate) bytes: Vec<u8>,
    pub(crate) work: usize,
    pub(crate) now: u64,
    pub(crate) context: LifecycleContext,
}

impl LifecycleInput {
    pub const fn new(
        epoch: TransportEpoch,
        identity: BatchId,
        bytes: Vec<u8>,
        work: usize,
        now: u64,
        context: LifecycleContext,
    ) -> Self {
        Self {
            epoch,
            identity,
            bytes,
            work,
            now,
            context,
        }
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleContext {
    Start(CandidateContext),
    Batch,
    Completion(CompletionCurrent),
}

impl LifecycleContext {
    pub(crate) fn replay_identity(&self) -> ApplicationContextIdentity {
        match self {
            Self::Start(context) => ApplicationContextIdentity::Start(context.clone()),
            Self::Batch => ApplicationContextIdentity::Batch,
            Self::Completion(current) => ApplicationContextIdentity::Completion(current.clone()),
        }
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LifecycleLimits {
    pub(crate) complete_message_bytes: NonZeroUsize,
    pub(crate) retained_attempt_bytes: NonZeroUsize,
    pub(crate) ambiguous_age_millis: NonZeroU64,
    pub(crate) reconcile_attempts: NonZeroUsize,
}

impl LifecycleLimits {
    #[must_use]
    pub fn new(
        max_complete_message_bytes: usize,
        max_retained_attempt_bytes: usize,
        max_ambiguous_age_millis: u64,
        max_reconcile_attempts: usize,
    ) -> Option<Self> {
        Some(Self {
            complete_message_bytes: NonZeroUsize::new(max_complete_message_bytes)?,
            retained_attempt_bytes: NonZeroUsize::new(max_retained_attempt_bytes)?,
            ambiguous_age_millis: NonZeroU64::new(max_ambiguous_age_millis)?,
            reconcile_attempts: NonZeroUsize::new(max_reconcile_attempts)?,
        })
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleResult {
    Disposition(ReceiverDisposition),
    Reconciled(ReconcileResult),
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReconcileResult {
    Committed,
    ProvenNotCommitted,
    StillAmbiguous,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttemptState {
    Ambiguous,
    RetryEligible,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleError {
    DecodeRejected,
    ContextMismatch,
    SlotInvariant,
    BlockedAmbiguous,
    RetainedLimit,
    CompletionRejected,
    AuthorityRejected,
    FinalizationBlocked,
    RetryNotEligible,
}
