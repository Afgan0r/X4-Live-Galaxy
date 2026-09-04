use crate::{
    feedback::CollectionPolicyLimits,
    scheduler_queue::{
        CollectionIntent, CollectionIntentId, CompletionDisposition, SchedulerAdmission,
        SchedulerSafetyLimits, WorkKind,
    },
};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OverrunDebt(usize);

impl OverrunDebt {
    #[must_use]
    pub const fn amount(self) -> usize {
        self.0
    }
}

struct InFlightAdmission {
    intent_id: CollectionIntentId,
    work_kind: WorkKind,
    declared_work: usize,
}

pub struct SchedulerBudget {
    limits: CollectionPolicyLimits,
    max_overrun_debt: usize,
    last_refill: u64,
    permits: usize,
    heavy_permits: usize,
    debt: OverrunDebt,
    in_flight: Vec<InFlightAdmission>,
}

impl SchedulerBudget {
    #[must_use]
    pub const fn new(
        limits: CollectionPolicyLimits,
        safety: SchedulerSafetyLimits,
        last_refill: u64,
    ) -> Self {
        Self {
            limits,
            max_overrun_debt: safety.max_overrun_debt.get(),
            last_refill,
            permits: limits.burst.get(),
            heavy_permits: limits.heavy_permits.get(),
            debt: OverrunDebt(0),
            in_flight: Vec::new(),
        }
    }

    pub(crate) const fn accepts_declared(&self, work: usize) -> bool {
        work <= self.limits.step_work.get()
    }

    pub(crate) fn can_admit(&self, intent: &CollectionIntent) -> bool {
        self.permits > 0
            && self.debt.0 == 0
            && self.in_flight.len() < self.limits.burst.get()
            && self.accepts_declared(intent.declared_work())
            && (intent.work_kind() == WorkKind::Light || self.heavy_permits > 0)
    }

    pub(crate) fn admit(&mut self, intent: &CollectionIntent) -> Option<SchedulerAdmission> {
        if !self.can_admit(intent) {
            return None;
        }
        let remaining_permits = self.permits.checked_sub(1)?;
        let remaining_heavy = match intent.work_kind() {
            WorkKind::Light => self.heavy_permits,
            WorkKind::Heavy => self.heavy_permits.checked_sub(1)?,
        };
        let admission = SchedulerAdmission {
            intent_id: intent.id().clone(),
            work_kind: intent.work_kind(),
            declared_work: intent.declared_work(),
        };
        self.permits = remaining_permits;
        self.heavy_permits = remaining_heavy;
        self.in_flight.push(InFlightAdmission {
            intent_id: intent.id().clone(),
            work_kind: intent.work_kind(),
            declared_work: intent.declared_work(),
        });
        Some(admission)
    }

    pub(crate) fn complete(
        &mut self,
        intent_id: &CollectionIntentId,
        actual_work: usize,
    ) -> CompletionDisposition {
        let Some(index) = self
            .in_flight
            .iter()
            .position(|admission| &admission.intent_id == intent_id)
        else {
            return CompletionDisposition::Unknown;
        };
        let admission = self.in_flight.remove(index);
        if admission.work_kind == WorkKind::Heavy {
            self.heavy_permits =
                bounded_increment(self.heavy_permits, self.limits.heavy_permits.get());
        }
        let overrun = actual_work.saturating_sub(admission.declared_work);
        let Some(debt) = self.debt.0.checked_add(overrun) else {
            self.debt = OverrunDebt(self.max_overrun_debt);
            return CompletionDisposition::CollectorRejected;
        };
        if debt > self.max_overrun_debt {
            self.debt = OverrunDebt(self.max_overrun_debt);
            return CompletionDisposition::CollectorRejected;
        }
        self.debt = OverrunDebt(debt);
        CompletionDisposition::Completed
    }

    pub(crate) fn refill(&mut self, now: u64) {
        let interval = self.limits.refill_millis.get();
        let quanta = now.saturating_sub(self.last_refill) / interval;
        if quanta == 0 {
            return;
        }
        if self.debt.0 > 0 {
            let available = usize::try_from(quanta).map_or(usize::MAX, |value| value);
            let repaid = available.min(self.debt.0).min(self.limits.step_work.get());
            self.debt.0 -= repaid;
            self.advance_refill(repaid, quanta, interval, now);
            return;
        }
        self.permits = bounded_increment(self.permits, self.limits.burst.get());
        self.advance_refill(1, quanta, interval, now);
    }

    fn advance_refill(&mut self, amount: usize, quanta: u64, interval: u64, now: u64) {
        let accounted = u64::try_from(amount).map_or(quanta, |value| value.min(quanta));
        self.last_refill = self
            .last_refill
            .saturating_add(accounted.saturating_mul(interval))
            .min(now);
    }

    pub(crate) const fn permits(&self) -> usize {
        self.permits
    }

    pub(crate) const fn heavy_permits(&self) -> usize {
        self.heavy_permits
    }

    pub(crate) const fn debt(&self) -> OverrunDebt {
        self.debt
    }
}

fn bounded_increment(current: usize, maximum: usize) -> usize {
    match current.checked_add(1) {
        Some(value) => value.min(maximum),
        None => maximum,
    }
}
