use crate::{
    feedback::{CollectionPolicyLimits, TransportPolicyLimits},
    scheduler_budget::SchedulerBudget,
    scheduler_queue::{
        CollectionIntent, CollectionIntentId, CompletionDisposition, IntentQueue,
        SchedulerAdmission, SchedulerSafetyLimits,
    },
};

pub trait MonotonicClock {
    fn now_millis(&self) -> u64;
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeliveredPulse {
    available_bytes: usize,
}

impl DeliveredPulse {
    pub const fn new(available_bytes: usize) -> Self {
        Self { available_bytes }
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerOutcome {
    pumped_bytes: usize,
    terminal_reserved_bytes: usize,
    admission: Option<SchedulerAdmission>,
    remaining_permits: usize,
    remaining_heavy_permits: usize,
    overrun_debt: usize,
    queued_intents: usize,
}

impl SchedulerOutcome {
    #[must_use]
    pub const fn pumped_bytes(&self) -> usize {
        self.pumped_bytes
    }
    #[must_use]
    pub const fn terminal_reserved_bytes(&self) -> usize {
        self.terminal_reserved_bytes
    }
    #[must_use]
    pub const fn admission(&self) -> Option<&SchedulerAdmission> {
        self.admission.as_ref()
    }
    #[must_use]
    pub const fn admitted_work(&self) -> usize {
        match &self.admission {
            Some(admission) => admission.declared_work,
            None => 0,
        }
    }
    #[must_use]
    pub const fn remaining_permits(&self) -> usize {
        self.remaining_permits
    }
    #[must_use]
    pub const fn remaining_heavy_permits(&self) -> usize {
        self.remaining_heavy_permits
    }
    #[must_use]
    pub const fn overrun_debt(&self) -> usize {
        self.overrun_debt
    }
    #[must_use]
    pub const fn queued_intents(&self) -> usize {
        self.queued_intents
    }
}

pub struct ObservationScheduler<C> {
    clock: C,
    transport: TransportPolicyLimits,
    budget: SchedulerBudget,
    pending: Option<Vec<u8>>,
    queue: IntentQueue,
}

impl<C: MonotonicClock> ObservationScheduler<C> {
    pub fn new(
        clock: C,
        collection: CollectionPolicyLimits,
        transport: TransportPolicyLimits,
        safety: SchedulerSafetyLimits,
    ) -> Self {
        let last_refill = clock.now_millis();
        Self {
            clock,
            transport,
            budget: SchedulerBudget::new(collection, safety, last_refill),
            pending: None,
            queue: IntentQueue::new(safety.queue_capacity),
        }
    }

    pub fn enqueue(&mut self, intent: CollectionIntent) -> Result<(), CollectionIntent> {
        if !self.budget.accepts_declared(intent.declared_work()) {
            return Err(intent);
        }
        self.queue.enqueue(intent)
    }

    pub fn complete(
        &mut self,
        intent_id: &CollectionIntentId,
        actual_work: usize,
    ) -> CompletionDisposition {
        self.budget.complete(intent_id, actual_work)
    }

    pub fn stage_pending(&mut self, bytes: Vec<u8>) -> Result<(), Vec<u8>> {
        if bytes.is_empty()
            || bytes.len() > self.transport.max_pump_bytes.get()
            || self.pending.is_some()
        {
            return Err(bytes);
        }
        self.pending = Some(bytes);
        Ok(())
    }

    pub fn deliver_pulse(&mut self, pulse: DeliveredPulse) -> SchedulerOutcome {
        self.budget.refill(self.clock.now_millis());
        let reserve = self
            .transport
            .terminal_reserve
            .get()
            .min(pulse.available_bytes);
        let bulk_capacity = pulse
            .available_bytes
            .saturating_sub(reserve)
            .min(self.transport.max_pump_bytes.get());
        if let Some(pumped_bytes) = self.pump_pending(bulk_capacity) {
            return self.outcome(pumped_bytes, reserve, None);
        }
        let admission = self.select_admission(bulk_capacity);
        self.outcome(0, reserve, admission)
    }

    #[must_use]
    pub fn pending_bytes(&self) -> Option<&[u8]> {
        self.pending.as_deref()
    }

    fn pump_pending(&mut self, bulk_capacity: usize) -> Option<usize> {
        let pending_len = self.pending.as_ref()?.len();
        if pending_len > bulk_capacity {
            return Some(0);
        }
        Some(self.pending.take().map_or(0, |bytes| bytes.len()))
    }

    fn select_admission(&mut self, bulk_capacity: usize) -> Option<SchedulerAdmission> {
        if bulk_capacity == 0 {
            return None;
        }
        let budget = &self.budget;
        let intent = self
            .queue
            .take_best_where(|intent| budget.can_admit(intent))?;
        self.budget.admit(&intent)
    }

    const fn outcome(
        &self,
        pumped_bytes: usize,
        terminal_reserved_bytes: usize,
        admission: Option<SchedulerAdmission>,
    ) -> SchedulerOutcome {
        SchedulerOutcome {
            pumped_bytes,
            terminal_reserved_bytes,
            admission,
            remaining_permits: self.budget.permits(),
            remaining_heavy_permits: self.budget.heavy_permits(),
            overrun_debt: self.budget.debt().amount(),
            queued_intents: self.queue.len(),
        }
    }
}
