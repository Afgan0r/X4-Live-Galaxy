#![expect(
    clippy::expect_used,
    reason = "test fixtures fail immediately when their invariants are invalid"
)]

use std::{cell::Cell, rc::Rc};

use observation_ingest::{
    CollectionClass, CollectionIntent, CollectionIntentId, CollectionPolicyLimits, DeliveredPulse,
    MonotonicClock, ObservationScheduler, SchedulerSafetyLimits, TransportPolicyLimits, WorkKind,
};

#[derive(Clone)]
struct FakeClock(Rc<Cell<u64>>);

impl MonotonicClock for FakeClock {
    fn now_millis(&self) -> u64 {
        self.0.get()
    }
}

fn scheduler() -> ObservationScheduler<FakeClock> {
    ObservationScheduler::new(
        FakeClock(Rc::new(Cell::new(0))),
        CollectionPolicyLimits::new(10, 1, 4, 1).expect("collection limits are finite"),
        TransportPolicyLimits::new(8, 2).expect("transport limits are finite"),
        SchedulerSafetyLimits::new(4, 8).expect("safety limits are finite"),
    )
}

fn intent(id: &str, class: CollectionClass, urgency: u64) -> CollectionIntent {
    CollectionIntent::new(
        CollectionIntentId::new(id).expect("intent identity is non-empty"),
        class,
        WorkKind::Light,
        4,
        urgency,
    )
    .expect("declared work is positive")
}

#[test]
fn urgency_orders_without_increasing_allowance() {
    assert_priority_order();
    assert_urgency_keeps_allowance();
    assert_pump_first_reserve();
}

fn assert_priority_order() {
    let mut ranked = scheduler();
    assert!(
        ranked
            .enqueue(intent("detail", CollectionClass::Detail, u64::MAX))
            .is_ok()
    );
    assert!(
        ranked
            .enqueue(intent("core-low", CollectionClass::Core, 1))
            .is_ok()
    );
    assert!(
        ranked
            .enqueue(intent("core-high", CollectionClass::Core, 9))
            .is_ok()
    );
    let selected = ranked.deliver_pulse(DeliveredPulse::new(3));
    assert_eq!(
        selected
            .admission()
            .map(|admission| admission.intent_id().as_str()),
        Some("core-high")
    );

    let mut tied = scheduler();
    assert!(
        tied.enqueue(intent("first", CollectionClass::Core, 4))
            .is_ok()
    );
    assert!(
        tied.enqueue(intent("second", CollectionClass::Core, 4))
            .is_ok()
    );
    assert_eq!(
        tied.deliver_pulse(DeliveredPulse::new(3))
            .admission()
            .map(|admission| admission.intent_id().as_str()),
        Some("first")
    );
}

fn assert_urgency_keeps_allowance() {
    let mut low = scheduler();
    let mut high = scheduler();
    assert!(
        low.enqueue(intent("same", CollectionClass::Core, 1))
            .is_ok()
    );
    assert!(
        high.enqueue(intent("same", CollectionClass::Core, u64::MAX))
            .is_ok()
    );
    let low = low.deliver_pulse(DeliveredPulse::new(3));
    let high = high.deliver_pulse(DeliveredPulse::new(3));
    assert_eq!(low.remaining_permits(), high.remaining_permits());
    assert_eq!(low.admitted_work(), high.admitted_work());
    assert_eq!(
        low.remaining_heavy_permits(),
        high.remaining_heavy_permits()
    );
    assert_eq!(
        low.terminal_reserved_bytes(),
        high.terminal_reserved_bytes()
    );
    assert_eq!(low.overrun_debt(), high.overrun_debt());
}

fn assert_pump_first_reserve() {
    let mut pump_first = scheduler();
    pump_first
        .stage_pending(b"123456".to_vec())
        .expect("pending message fits the pump ceiling");
    assert!(
        pump_first
            .enqueue(intent("waiting", CollectionClass::Core, 3))
            .is_ok()
    );
    let pumped = pump_first.deliver_pulse(DeliveredPulse::new(8));
    assert_eq!(pumped.pumped_bytes(), 6);
    assert_eq!(pumped.terminal_reserved_bytes(), 2);
    assert!(pumped.admission().is_none());

    pump_first
        .stage_pending(b"123456".to_vec())
        .expect("slot was released after the exact pump");
    let retained = pump_first.deliver_pulse(DeliveredPulse::new(7));
    assert_eq!(retained.pumped_bytes(), 0);
    assert!(retained.admission().is_none());
    assert_eq!(pump_first.pending_bytes(), Some(&b"123456"[..]));
}
