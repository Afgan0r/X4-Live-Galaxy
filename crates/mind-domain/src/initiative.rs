use crate::{CausalEvent, CausalKind, CommandId};
use strategic_state::{Capability, Faction};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct InitiativeId(&'static str);

impl InitiativeId {
    #[must_use]
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitiativeLifecycle {
    Active,
    Completed,
    Cancelled,
    Rejected,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreemptionDisposition {
    Cancelled,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitiativeError {
    ActiveSlot,
    StalePredecessor,
    ContentCollision,
    IllegalTerminal,
    CapacityExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InitiativeSpec {
    pub(crate) id: InitiativeId,
    pub(crate) capability: Capability,
    pub(crate) objective: &'static str,
    pub(crate) evidence: &'static str,
    pub(crate) priority: u8,
}

impl InitiativeSpec {
    #[must_use]
    pub const fn new(
        id: InitiativeId,
        capability: Capability,
        objective: &'static str,
        evidence: &'static str,
        priority: u8,
    ) -> Self {
        Self {
            id,
            capability,
            objective,
            evidence,
            priority,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Initiative {
    pub(crate) spec: InitiativeSpec,
    pub(crate) state: InitiativeLifecycle,
    pub(crate) validating_event: CausalEvent,
}

impl Initiative {
    #[must_use]
    pub const fn id(&self) -> InitiativeId {
        self.spec.id
    }
    #[must_use]
    pub const fn state(&self) -> InitiativeLifecycle {
        self.state
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitiativeCommand {
    Accept {
        id: CommandId,
        initiative: InitiativeSpec,
    },
    Preempt {
        id: CommandId,
        predecessor: InitiativeId,
        replacement: InitiativeSpec,
        trigger: &'static str,
        disposition: PreemptionDisposition,
    },
    Terminal {
        id: CommandId,
        initiative: InitiativeId,
        state: InitiativeLifecycle,
    },
}

impl InitiativeCommand {
    #[must_use]
    pub const fn accept(id: CommandId, initiative: InitiativeSpec) -> Self {
        Self::Accept { id, initiative }
    }
    #[must_use]
    pub const fn preempt(
        id: CommandId,
        predecessor: InitiativeId,
        replacement: InitiativeSpec,
        trigger: &'static str,
        disposition: PreemptionDisposition,
    ) -> Self {
        Self::Preempt {
            id,
            predecessor,
            replacement,
            trigger,
            disposition,
        }
    }
    #[must_use]
    pub const fn complete(id: CommandId, initiative: InitiativeId) -> Self {
        Self::Terminal {
            id,
            initiative,
            state: InitiativeLifecycle::Completed,
        }
    }
    #[must_use]
    pub const fn cancel(id: CommandId, initiative: InitiativeId) -> Self {
        Self::Terminal {
            id,
            initiative,
            state: InitiativeLifecycle::Cancelled,
        }
    }
    #[must_use]
    pub const fn reject(id: CommandId, initiative: InitiativeId) -> Self {
        Self::Terminal {
            id,
            initiative,
            state: InitiativeLifecycle::Rejected,
        }
    }
    #[must_use]
    pub const fn fail(id: CommandId, initiative: InitiativeId) -> Self {
        Self::Terminal {
            id,
            initiative,
            state: InitiativeLifecycle::Failed,
        }
    }
    pub(crate) const fn id(self) -> CommandId {
        match self {
            Self::Accept { id, .. } | Self::Preempt { id, .. } | Self::Terminal { id, .. } => id,
        }
    }
}

pub const fn slot(capability: Capability) -> usize {
    match capability {
        Capability::DefenseAndMilitaryStrategy => 0,
        Capability::EconomyAndLogistics => 1,
        Capability::TerritorialDevelopmentAndInfrastructure => 2,
    }
}

pub fn events(
    faction: Faction,
    command: CommandId,
    initiative: InitiativeId,
    kinds: &[CausalKind],
) -> Vec<CausalEvent> {
    kinds
        .iter()
        .zip(0u8..)
        .map(|(kind, sequence)| {
            CausalEvent::for_initiative(*kind, faction, command, sequence, initiative)
        })
        .collect()
}

pub fn update_history(
    history: &mut [Initiative],
    id: InitiativeId,
    state: InitiativeLifecycle,
) -> Result<(), InitiativeError> {
    let Some(initiative) = history.iter_mut().find(|initiative| initiative.id() == id) else {
        return Err(InitiativeError::StalePredecessor);
    };
    initiative.state = state;
    Ok(())
}
