use crate::feedback::{CollectionPolicyLimits, TransportPolicyLimits};

pub trait MonotonicClock {
    fn now_millis(&self) -> u64;
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeliveredPulse {
    _game_time_urgency: u64,
    downstream_capacity: bool,
}

impl DeliveredPulse {
    pub const fn new(game_time_urgency: u64, downstream_capacity: bool) -> Self {
        Self {
            _game_time_urgency: game_time_urgency,
            downstream_capacity,
        }
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerOutcome {
    pumped_bytes: usize,
    collection_started: bool,
}

impl SchedulerOutcome {
    #[must_use]
    pub const fn pumped_bytes(self) -> usize {
        self.pumped_bytes
    }
    #[must_use]
    pub const fn collection_started(self) -> bool {
        self.collection_started
    }
}

pub struct ObservationScheduler<C> {
    clock: C,
    collection: CollectionPolicyLimits,
    transport: TransportPolicyLimits,
    last_refill: u64,
    permits: usize,
    pending: Option<Vec<u8>>,
}

impl<C: MonotonicClock> ObservationScheduler<C> {
    pub fn new(
        clock: C,
        collection: CollectionPolicyLimits,
        transport: TransportPolicyLimits,
    ) -> Self {
        let last_refill = clock.now_millis();
        Self {
            clock,
            collection,
            transport,
            last_refill,
            permits: collection.burst.get(),
            pending: None,
        }
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
        if self.pending.is_some() && !pulse.downstream_capacity {
            return SchedulerOutcome {
                pumped_bytes: 0,
                collection_started: false,
            };
        }
        if let Some(bytes) = self.pending.take() {
            return SchedulerOutcome {
                pumped_bytes: bytes.len(),
                collection_started: false,
            };
        }
        let finite_policy = self.collection.step_work.get() > 0
            && self.collection.heavy_permits.get() > 0
            && self.transport.terminal_reserve.get() > 0;
        let collection_started = pulse.downstream_capacity && self.permits > 0 && finite_policy;
        if collection_started {
            self.permits -= 1;
        }
        SchedulerOutcome {
            pumped_bytes: 0,
            collection_started,
        }
    }

    pub fn pending_bytes(&self) -> Option<&[u8]> {
        self.pending.as_deref()
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

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::{DeliveredPulse, MonotonicClock, ObservationScheduler};
    use crate::{CollectionPolicyLimits, TransportPolicyLimits};

    #[derive(Clone)]
    struct FakeClock(Rc<Cell<u64>>);

    impl MonotonicClock for FakeClock {
        fn now_millis(&self) -> u64 {
            self.0.get()
        }
    }

    fn scheduler(clock: FakeClock) -> ObservationScheduler<FakeClock> {
        ObservationScheduler::new(
            clock,
            CollectionPolicyLimits::new(10, 1, 2, 1).expect("finite collection limits"),
            TransportPolicyLimits::new(64, 1).expect("finite transport limits"),
        )
    }

    #[test]
    fn pulse_pumps_pending_bytes_before_starting_collection() {
        let clock = FakeClock(Rc::new(Cell::new(0)));
        let mut scheduler = scheduler(clock);
        scheduler
            .stage_pending(b"immutable".to_vec())
            .expect("empty slot accepts bytes");

        let pumped = scheduler.deliver_pulse(DeliveredPulse::new(900, true));
        assert_eq!(pumped.pumped_bytes(), b"immutable".len());
        assert!(!pumped.collection_started());
        assert_eq!(scheduler.pending_bytes(), None);
    }

    #[test]
    fn urgency_never_refills_without_monotonic_real_time() {
        let clock = FakeClock(Rc::new(Cell::new(0)));
        let mut scheduler = scheduler(clock.clone());
        assert!(
            scheduler
                .deliver_pulse(DeliveredPulse::new(1, true))
                .collection_started()
        );
        assert!(
            !scheduler
                .deliver_pulse(DeliveredPulse::new(u64::MAX, true))
                .collection_started()
        );
        clock.0.set(10);
        assert!(
            scheduler
                .deliver_pulse(DeliveredPulse::new(1, true))
                .collection_started()
        );
    }

    #[test]
    fn capacity_loss_retains_exact_pending_bytes_and_blocks_collection() {
        let clock = FakeClock(Rc::new(Cell::new(0)));
        let mut scheduler = scheduler(clock);
        scheduler
            .stage_pending(b"retain-me".to_vec())
            .expect("empty slot accepts bytes");

        let outcome = scheduler.deliver_pulse(DeliveredPulse::new(u64::MAX, false));

        assert_eq!(outcome.pumped_bytes(), 0);
        assert!(!outcome.collection_started());
        assert_eq!(scheduler.pending_bytes(), Some(&b"retain-me"[..]));
    }
}
