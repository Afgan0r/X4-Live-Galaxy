use std::num::{NonZeroU64, NonZeroUsize};

mod slot;
mod slot_transition;

pub use slot::{
    AmbiguityResolution, FeedbackError, ImmutableApplicationBatch, SlotAdmission, SlotTurnover,
    StopAndWaitSlot,
};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CollectionPolicyLimits {
    pub(crate) refill_millis: NonZeroU64,
    pub(crate) burst: NonZeroUsize,
    pub(crate) step_work: NonZeroUsize,
    pub(crate) heavy_permits: NonZeroUsize,
}

impl CollectionPolicyLimits {
    #[must_use]
    pub fn new(
        refill_millis: u64,
        burst: usize,
        step_work: usize,
        heavy_permits: usize,
    ) -> Option<Self> {
        Some(Self {
            refill_millis: NonZeroU64::new(refill_millis)?,
            burst: NonZeroUsize::new(burst)?,
            step_work: NonZeroUsize::new(step_work)?,
            heavy_permits: NonZeroUsize::new(heavy_permits)?,
        })
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportPolicyLimits {
    pub(crate) max_pump_bytes: NonZeroUsize,
    pub(crate) terminal_reserve: NonZeroUsize,
}

impl TransportPolicyLimits {
    #[must_use]
    pub fn new(max_pump_bytes: usize, terminal_reserve: usize) -> Option<Self> {
        Some(Self {
            max_pump_bytes: NonZeroUsize::new(max_pump_bytes)?,
            terminal_reserve: NonZeroUsize::new(terminal_reserve)?,
        })
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryStage {
    CollectionProgress,
    LocalHandoff,
    Received,
    Committed,
    TerminalRejection,
    AmbiguousPublication,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiverDisposition {
    CapacityUnavailable,
    Received,
    Committed,
    TimedOutOrSuperseded,
    StaleEpoch,
    PermanentlyRejected,
    AmbiguousCommit,
}
