use crate::{CommandId, InitiativeId, PreemptionDisposition};
use serde::{Deserialize, Serialize};
use strategic_state::Faction;
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum CausalKind {
    MindUpdated,
    Proposal,
    Objection,
    ExecutiveDisposition,
    Validated,
    OwnershipAssigned,
    Preempted,
    Completed,
    Cancelled,
    Rejected,
    Failed,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CausalEvent {
    pub(crate) kind: CausalKind,
    pub(crate) faction: Faction,
    pub(crate) command: CommandId,
    pub(crate) sequence: u8,
    pub(crate) initiative: Option<InitiativeId>,
    pub(crate) disposition: Option<PreemptionDisposition>,
}
impl CausalEvent {
    pub(crate) const fn new(
        kind: CausalKind,
        faction: Faction,
        command: CommandId,
        sequence: u8,
    ) -> Self {
        Self {
            kind,
            faction,
            command,
            sequence,
            initiative: None,
            disposition: None,
        }
    }
    pub(crate) const fn for_initiative(
        kind: CausalKind,
        faction: Faction,
        command: CommandId,
        sequence: u8,
        initiative: InitiativeId,
    ) -> Self {
        Self {
            kind,
            faction,
            command,
            sequence,
            initiative: Some(initiative),
            disposition: None,
        }
    }
    pub(crate) const fn preemption(
        faction: Faction,
        command: CommandId,
        sequence: u8,
        initiative: InitiativeId,
        disposition: PreemptionDisposition,
    ) -> Self {
        Self {
            kind: CausalKind::Preempted,
            faction,
            command,
            sequence,
            initiative: Some(initiative),
            disposition: Some(disposition),
        }
    }
    #[must_use]
    pub const fn kind(&self) -> CausalKind {
        self.kind
    }
    #[must_use]
    pub const fn command(&self) -> &CommandId {
        &self.command
    }
    #[must_use]
    pub const fn initiative(&self) -> Option<&InitiativeId> {
        self.initiative.as_ref()
    }
    #[must_use]
    pub const fn disposition(&self) -> Option<PreemptionDisposition> {
        self.disposition
    }

    pub(crate) fn valid(&self) -> bool {
        let shape = match self.kind {
            CausalKind::MindUpdated => self.initiative.is_none() && self.disposition.is_none(),
            CausalKind::Preempted => self.initiative.is_some() && self.disposition.is_some(),
            _ => self.initiative.is_some() && self.disposition.is_none(),
        };
        shape
            && self.command.valid()
            && self.initiative.as_ref().is_none_or(InitiativeId::valid)
            && self.sequence < 6
    }
}
