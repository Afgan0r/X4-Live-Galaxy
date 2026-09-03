#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::{
        CollectionPolicyLimits, DeliveredPulse, MonotonicClock, ObservationScheduler,
        TransportPolicyLimits,
    };

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
