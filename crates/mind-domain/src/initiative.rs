use crate::{CausalEvent, CommandId};
use serde::{Deserialize, Serialize};
use strategic_state::Capability;

const MAX_TEXT: usize = 256;
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitiativeId(String);
impl InitiativeId {
    #[must_use]
    pub fn new(value: &str) -> Self {
        Self(value.into())
    }
    pub(crate) fn valid(&self) -> bool {
        valid(&self.0)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum InitiativeLifecycle {
    Active,
    Completed,
    Cancelled,
    Rejected,
    Failed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitiativeSpec {
    pub(crate) id: InitiativeId,
    pub(crate) capability: Capability,
    pub(crate) objective: String,
    pub(crate) evidence: String,
    pub(crate) priority: u8,
}
impl InitiativeSpec {
    #[must_use]
    pub fn new(
        id: InitiativeId,
        capability: Capability,
        objective: &str,
        evidence: &str,
        priority: u8,
    ) -> Self {
        Self {
            id,
            capability,
            objective: objective.into(),
            evidence: evidence.into(),
            priority,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.id.valid() && valid(&self.objective) && valid(&self.evidence)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Initiative {
    pub(crate) spec: InitiativeSpec,
    pub(crate) state: InitiativeLifecycle,
    pub(crate) validating_event: CausalEvent,
}
impl Initiative {
    #[must_use]
    pub const fn id(&self) -> &InitiativeId {
        &self.spec.id
    }
    #[must_use]
    pub const fn state(&self) -> InitiativeLifecycle {
        self.state
    }

    pub(crate) fn valid(&self) -> bool {
        self.spec.valid()
            && self.validating_event.valid()
            && self.validating_event.kind == crate::CausalKind::Validated
            && self.validating_event.initiative.as_ref() == Some(&self.spec.id)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind")]
pub enum InitiativeCommand {
    Accept {
        id: CommandId,
        initiative: InitiativeSpec,
    },
    Preempt {
        id: CommandId,
        predecessor: InitiativeId,
        replacement: InitiativeSpec,
        trigger: String,
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
    pub fn preempt(
        id: CommandId,
        predecessor: InitiativeId,
        replacement: InitiativeSpec,
        trigger: &str,
        disposition: PreemptionDisposition,
    ) -> Self {
        Self::Preempt {
            id,
            predecessor,
            replacement,
            trigger: trigger.into(),
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
    pub(crate) const fn id(&self) -> &CommandId {
        match self {
            Self::Accept { id, .. } | Self::Preempt { id, .. } | Self::Terminal { id, .. } => id,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Accept { id, initiative } => id.valid() && initiative.valid(),
            Self::Preempt {
                id,
                predecessor,
                replacement,
                trigger,
                ..
            } => id.valid() && predecessor.valid() && replacement.valid() && valid(trigger),
            Self::Terminal { id, initiative, .. } => id.valid() && initiative.valid(),
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
pub const fn valid(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_TEXT
}
