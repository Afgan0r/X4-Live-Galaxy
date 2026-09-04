#![expect(
    clippy::expect_used,
    reason = "test fixtures fail immediately when their invariants are invalid"
)]

use std::{cell::Cell, rc::Rc};

use observation_ingest::{
    CollectionClass, CollectionIntent, CollectionIntentId, CollectionPolicyLimits,
    CompletionDisposition, DeliveredPulse, MonotonicClock, ObservationScheduler, SchedulerOutcome,
    SchedulerSafetyLimits, TransportPolicyLimits, WorkKind,
};

#[derive(Clone)]
struct FakeClock(Rc<Cell<u64>>);

impl MonotonicClock for FakeClock {
    fn now_millis(&self) -> u64 {
        self.0.get()
    }
}

fn scheduler() -> (ObservationScheduler<FakeClock>, FakeClock) {
    let clock = FakeClock(Rc::new(Cell::new(0)));
    let value = ObservationScheduler::new(
        clock.clone(),
        CollectionPolicyLimits::new(10, 2, 4, 1).expect("finite collection limits"),
        TransportPolicyLimits::new(8, 2).expect("finite transport limits"),
        SchedulerSafetyLimits::new(4, 8).expect("finite safety limits"),
    );
    (value, clock)
}

fn id(value: &str) -> CollectionIntentId {
    CollectionIntentId::new(value).expect("non-empty intent identity")
}

fn pulse(value: &mut ObservationScheduler<FakeClock>) -> SchedulerOutcome {
    value.deliver_pulse(DeliveredPulse::new(3))
}

macro_rules! enq {
    ($scheduler:ident, $name:expr, $class:ident, $kind:ident, $work:expr, $urgency:expr) => {{
        let intent = CollectionIntent::new(
            id($name),
            CollectionClass::$class,
            WorkKind::$kind,
            $work,
            $urgency,
        )
        .expect("positive declared work");
        $scheduler.enqueue(intent).is_ok()
    }};
}

#[test]
fn urgency_orders_without_increasing_allowance() {
    assert_priority_order();
    assert_urgency_keeps_allowance();
    assert_pump_first_reserve();
}

fn assert_priority_order() {
    let (mut ranked, _) = scheduler();
    assert!(enq!(ranked, "detail", Detail, Light, 4, u64::MAX));
    assert!(enq!(ranked, "core-low", Core, Light, 4, 1));
    assert!(enq!(ranked, "core-high", Core, Light, 4, 9));
    assert_eq!(
        pulse(&mut ranked).admission().map(|a| a.intent_id()),
        Some(&id("core-high"))
    );
    let (mut tied, _) = scheduler();
    assert!(enq!(tied, "first", Core, Light, 4, 4));
    assert!(enq!(tied, "second", Core, Light, 4, 4));
    assert_eq!(
        pulse(&mut tied).admission().map(|a| a.intent_id()),
        Some(&id("first"))
    );
}

fn assert_urgency_keeps_allowance() {
    let (mut low, _) = scheduler();
    let (mut high, _) = scheduler();
    assert!(enq!(low, "same", Core, Light, 4, 1));
    assert!(enq!(high, "same", Core, Light, 4, u64::MAX));
    let low = pulse(&mut low);
    let high = pulse(&mut high);
    assert_eq!(
        (
            low.remaining_permits(),
            low.admitted_work(),
            low.overrun_debt()
        ),
        (
            high.remaining_permits(),
            high.admitted_work(),
            high.overrun_debt()
        )
    );
}

fn assert_pump_first_reserve() {
    let (mut value, _) = scheduler();
    assert!(value.stage_pending(b"123456".to_vec()).is_ok());
    assert!(enq!(value, "waiting", Core, Light, 4, 3));
    let pumped = value.deliver_pulse(DeliveredPulse::new(8));
    assert_eq!(
        (pumped.pumped_bytes(), pumped.terminal_reserved_bytes()),
        (6, 2)
    );
    assert!(pumped.admission().is_none());
    assert!(value.stage_pending(b"123456".to_vec()).is_ok());
    assert_eq!(
        value.deliver_pulse(DeliveredPulse::new(7)).pumped_bytes(),
        0
    );
    assert_eq!(value.pending_bytes(), Some(&b"123456"[..]));
}

#[test]
fn step_and_heavy_limits_are_exact_and_completion_is_idempotent() {
    let (mut value, _) = scheduler();
    assert!(!enq!(value, "too-large", Core, Light, 5, 0));
    assert!(enq!(value, "heavy", Core, Heavy, 4, 0));
    assert!(enq!(value, "blocked", Core, Heavy, 4, 0));
    let admitted = pulse(&mut value);
    assert_eq!(
        (
            admitted.admitted_work(),
            admitted.remaining_permits(),
            admitted.remaining_heavy_permits()
        ),
        (4, 1, 0)
    );
    assert!(pulse(&mut value).admission().is_none());
    assert_eq!(
        value.complete(&id("unknown"), 4),
        CompletionDisposition::Unknown
    );
    assert_eq!(
        value.complete(&id("heavy"), 4),
        CompletionDisposition::Completed
    );
    assert_eq!(
        value.complete(&id("heavy"), 4),
        CompletionDisposition::Unknown
    );
    assert_eq!(pulse(&mut value).remaining_heavy_permits(), 1);
}

#[test]
fn overrun_debt_repayment_is_bounded_under_frozen_and_coarse_clocks() {
    let (mut value, clock) = scheduler();
    assert!(enq!(value, "overrun", Core, Light, 4, 0));
    assert!(pulse(&mut value).admission().is_some());
    assert_eq!(
        value.complete(&id("overrun"), 10),
        CompletionDisposition::Completed
    );
    assert_eq!(pulse(&mut value).overrun_debt(), 6);
    clock.0.set(100);
    assert_eq!(pulse(&mut value).overrun_debt(), 2);
    assert_eq!(pulse(&mut value).overrun_debt(), 0);
    assert_eq!(pulse(&mut value).remaining_permits(), 1);
}

#[test]
fn policy_dimensions_reject_one_over_without_state_change() {
    let (mut value, _) = scheduler();
    for number in 0..4 {
        assert!(enq!(value, &format!("q{number}"), Detail, Light, 4, 0));
    }
    assert!(!enq!(value, "queue-over", Core, Light, 4, 0));
    let before = pulse(&mut value);
    assert_eq!(
        (before.queued_intents(), before.remaining_permits()),
        (3, 1)
    );
    assert_eq!(
        value.complete(&id("q0"), 13),
        CompletionDisposition::CollectorRejected
    );
    let after = pulse(&mut value);
    assert_eq!(
        (
            after.overrun_debt(),
            after.queued_intents(),
            after.remaining_permits()
        ),
        (8, 3, 1)
    );
}
