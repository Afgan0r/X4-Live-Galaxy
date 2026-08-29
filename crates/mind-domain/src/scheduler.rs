use strategic_state::Faction;

const MAX_EVENT_ID: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FactionTrigger {
    StrategicTick(u64),
    RelevantEvent(String),
    Interrupted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerBounds {
    max_calls: u8,
    max_retries: u8,
    cooldown_ticks: u64,
}

impl SchedulerBounds {
    #[must_use]
    pub const fn ci() -> Self {
        Self {
            max_calls: 1,
            max_retries: 1,
            cooldown_ticks: 1,
        }
    }

    #[must_use]
    pub const fn valid(self) -> bool {
        self.max_calls > 0 && self.max_retries > 0 && self.cooldown_ticks > 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestEligibility {
    Eligible(FactionTrigger),
    Coalesced,
    Cooldown,
    PausedAwaitingReconciliation,
    Reconciled,
    RejectedBounds,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FactionState {
    outstanding: bool,
    paused_at: Option<u64>,
    last_tick: Option<u64>,
}

impl FactionState {
    const EMPTY: Self = Self {
        outstanding: false,
        paused_at: None,
        last_tick: None,
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliberationScheduler {
    bounds: SchedulerBounds,
    factions: [FactionState; 2],
}

impl DeliberationScheduler {
    #[must_use]
    pub const fn new(bounds: SchedulerBounds) -> Self {
        Self {
            bounds,
            factions: [FactionState::EMPTY, FactionState::EMPTY],
        }
    }

    pub fn eligibility(&mut self, faction: Faction, trigger: FactionTrigger) -> RequestEligibility {
        if !self.bounds.valid() || !valid_trigger(&trigger) {
            return RequestEligibility::RejectedBounds;
        }
        let state = &mut self.factions[index(faction)];
        if state.paused_at.is_some() {
            return RequestEligibility::PausedAwaitingReconciliation;
        }
        if state.outstanding {
            return RequestEligibility::Coalesced;
        }
        let tick = tick(&trigger);
        if state
            .last_tick
            .is_some_and(|last| tick <= last + self.bounds.cooldown_ticks)
        {
            return RequestEligibility::Cooldown;
        }
        state.outstanding = true;
        state.last_tick = Some(tick);
        RequestEligibility::Eligible(trigger)
    }

    pub const fn timeout(&mut self, faction: Faction, observation: u64) -> RequestEligibility {
        let state = &mut self.factions[index(faction)];
        state.outstanding = false;
        state.paused_at = Some(observation);
        RequestEligibility::PausedAwaitingReconciliation
    }

    pub fn reconcile(&mut self, faction: Faction, observation: u64) -> RequestEligibility {
        let state = &mut self.factions[index(faction)];
        if state.paused_at.is_none_or(|paused| observation <= paused) {
            return RequestEligibility::PausedAwaitingReconciliation;
        }
        state.paused_at = None;
        state.last_tick = Some(observation);
        RequestEligibility::Reconciled
    }

    #[must_use]
    pub fn outstanding_count(&self, faction: Faction) -> u8 {
        u8::from(self.factions[index(faction)].outstanding)
    }
}

const fn index(faction: Faction) -> usize {
    match faction {
        Faction::Zya => 0,
        Faction::Arg => 1,
    }
}

const fn valid_trigger(trigger: &FactionTrigger) -> bool {
    match trigger {
        FactionTrigger::StrategicTick(_) | FactionTrigger::Interrupted => true,
        FactionTrigger::RelevantEvent(id) => !id.is_empty() && id.len() <= MAX_EVENT_ID,
    }
}

const fn tick(trigger: &FactionTrigger) -> u64 {
    match trigger {
        FactionTrigger::StrategicTick(value) => *value,
        FactionTrigger::RelevantEvent(_) | FactionTrigger::Interrupted => 0,
    }
}
