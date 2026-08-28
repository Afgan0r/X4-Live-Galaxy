use crate::{CommandId, InitiativeId, PreemptionDisposition};
use strategic_state::Faction;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CausalEvent {
    kind: CausalKind,
    faction: Faction,
    command: CommandId,
    sequence: u8,
    initiative: Option<InitiativeId>,
    disposition: Option<PreemptionDisposition>,
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
    pub const fn kind(self) -> CausalKind {
        self.kind
    }

    #[must_use]
    pub const fn command(self) -> CommandId {
        self.command
    }

    #[must_use]
    pub const fn initiative(self) -> Option<InitiativeId> {
        self.initiative
    }

    #[must_use]
    pub const fn disposition(self) -> Option<PreemptionDisposition> {
        self.disposition
    }
}
