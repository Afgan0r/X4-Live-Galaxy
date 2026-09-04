use crate::{
    feedback::{CollectionPolicyLimits, TransportPolicyLimits},
    scheduler_queue::{
        CollectionIntent, IntentQueue, SchedulerAdmission, SchedulerSafetyLimits, WorkKind,
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
}

pub struct ObservationScheduler<C> {
    clock: C,
    collection: CollectionPolicyLimits,
    transport: TransportPolicyLimits,
    last_refill: u64,
    permits: usize,
    heavy_permits: usize,
    debt: usize,
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
            collection,
            transport,
            last_refill,
            permits: collection.burst.get(),
            heavy_permits: collection.heavy_permits.get(),
            debt: 0,
            pending: None,
            queue: IntentQueue::new(safety.queue_capacity),
        }
    }

    pub fn enqueue(&mut self, intent: CollectionIntent) -> Result<(), CollectionIntent> {
        self.queue.enqueue(intent)
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
        self.refill();
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
        if bulk_capacity == 0 || self.permits == 0 || self.debt > 0 {
            return None;
        }
        let step_work = self.collection.step_work.get();
        let heavy_permits = self.heavy_permits;
        let intent = self.queue.take_best_where(|intent| {
            intent.declared_work() <= step_work
                && (intent.work_kind() == WorkKind::Light || heavy_permits > 0)
        })?;
        self.permits -= 1;
        if intent.work_kind() == WorkKind::Heavy {
            self.heavy_permits -= 1;
        }
        Some(SchedulerAdmission {
            intent_id: intent.id().clone(),
            work_kind: intent.work_kind(),
            declared_work: intent.declared_work(),
        })
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
            remaining_permits: self.permits,
            remaining_heavy_permits: self.heavy_permits,
            overrun_debt: self.debt,
        }
    }

    fn refill(&mut self) {
        let now = self.clock.now_millis();
        let elapsed = now.saturating_sub(self.last_refill);
        let refill = elapsed / self.collection.refill_millis.get();
        if refill == 0 {
            return;
        }
        let refill = usize::try_from(refill).unwrap_or(usize::MAX);
        self.permits = self
            .permits
            .saturating_add(refill)
            .min(self.collection.burst.get());
        self.last_refill = now;
    }
}
