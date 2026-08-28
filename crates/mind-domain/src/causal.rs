use crate::CommandId;
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
}
