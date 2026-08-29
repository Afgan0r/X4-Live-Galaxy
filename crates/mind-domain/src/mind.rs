use crate::{CausalEvent, CausalKind, Initiative, InitiativeCommand, PendingInitiativeCommit};
use serde::{Deserialize, Serialize};
use strategic_state::{BilateralPosture, Capability, Faction, StrategicPacket};
const MAX_TEXT: usize = 256;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandId(String);
impl CommandId {
    #[must_use]
    pub fn new(value: &str) -> Self {
        Self(value.into())
    }
    pub(crate) const fn valid(&self) -> bool {
        !self.0.is_empty() && self.0.len() <= MAX_TEXT
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MindError {
    FactionMismatch,
    UnsupportedProfile,
    Initiative(crate::InitiativeError),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MindCommand {
    pub(crate) faction: Faction,
    pub(crate) doctrine_version: String,
    pub(crate) priorities: [Capability; 3],
    pub(crate) posture: BilateralPosture,
    pub(crate) id: CommandId,
}
impl MindCommand {
    #[must_use]
    pub fn from_packet(packet: &StrategicPacket, id: CommandId) -> Self {
        let posture = match packet.profile().priorities()[0] {
            Capability::DefenseAndMilitaryStrategy => BilateralPosture::IncreasePressure,
            Capability::EconomyAndLogistics => BilateralPosture::SeekLimitedCoordination,
            Capability::TerritorialDevelopmentAndInfrastructure => {
                BilateralPosture::PreserveRelations
            }
        };
        Self {
            faction: packet.faction(),
            doctrine_version: packet.profile_version().into(),
            priorities: packet.profile().priorities(),
            posture,
            id,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MindAggregate {
    pub(crate) faction: Faction,
    pub(crate) doctrine_version: String,
    pub(crate) motives: [String; 2],
    pub(crate) priorities: [Capability; 3],
    pub(crate) goals: [Capability; 3],
    pub(crate) short_term_plans: [Capability; 3],
    pub(crate) long_term_plans: [Capability; 3],
    pub(crate) posture: BilateralPosture,
    pub(crate) slots: [Option<Initiative>; 3],
    pub(crate) history: Vec<Initiative>,
    pub(crate) ledger: Vec<CausalEvent>,
    pub(crate) commands: Vec<(InitiativeCommand, Vec<CausalEvent>)>,
}
impl MindAggregate {
    #[must_use]
    pub const fn empty(faction: Faction) -> Self {
        Self {
            faction,
            doctrine_version: String::new(),
            motives: [String::new(), String::new()],
            priorities: Capability::ALL,
            goals: Capability::ALL,
            short_term_plans: Capability::ALL,
            long_term_plans: Capability::ALL,
            posture: BilateralPosture::PreserveRelations,
            slots: [None, None, None],
            history: Vec::new(),
            ledger: Vec::new(),
            commands: Vec::new(),
        }
    }
    #[must_use]
    pub fn doctrine_version(&self) -> &str {
        &self.doctrine_version
    }
    #[must_use]
    pub const fn motives(&self) -> &[String; 2] {
        &self.motives
    }
    pub const fn priorities(&self) -> &[Capability; 3] {
        &self.priorities
    }
    pub const fn goals(&self) -> &[Capability; 3] {
        &self.goals
    }
    pub const fn short_term_plans(&self) -> &[Capability; 3] {
        &self.short_term_plans
    }
    pub const fn long_term_plans(&self) -> &[Capability; 3] {
        &self.long_term_plans
    }
    #[must_use]
    pub const fn posture(&self) -> BilateralPosture {
        self.posture
    }
    #[must_use]
    pub const fn active_initiative(&self, capability: Capability) -> Option<&Initiative> {
        self.slots[crate::initiative::slot(capability)].as_ref()
    }
    #[must_use]
    pub fn initiative_history(&self) -> &[Initiative] {
        &self.history
    }
    #[must_use]
    pub fn causal_events(&self) -> &[CausalEvent] {
        &self.ledger
    }
    pub fn apply_initiative(
        &self,
        command: InitiativeCommand,
    ) -> Result<PendingInitiativeCommit, MindError> {
        crate::ledger::apply(self, command).map_err(MindError::Initiative)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingMindCommit {
    pub(crate) aggregate: MindAggregate,
    pub(crate) events: [CausalEvent; 1],
}
impl PendingMindCommit {
    #[must_use]
    pub const fn aggregate(&self) -> &MindAggregate {
        &self.aggregate
    }
    #[must_use]
    pub const fn events(&self) -> &[CausalEvent; 1] {
        &self.events
    }
    #[must_use]
    pub fn checkpoint_state(&self) -> crate::MindCheckpointState {
        crate::MindCheckpointState::from_pending_commit(self)
    }
}
pub fn transition(
    prior: &MindAggregate,
    command: MindCommand,
) -> Result<PendingMindCommit, MindError> {
    if prior.faction != command.faction {
        return Err(MindError::FactionMismatch);
    }
    if command.doctrine_version != "doctrine-v1" {
        return Err(MindError::UnsupportedProfile);
    }
    let motives = match command.priorities[0] {
        Capability::DefenseAndMilitaryStrategy => {
            ["protect territory".into(), "sustain pressure".into()]
        }
        Capability::EconomyAndLogistics => ["sustain economy".into(), "coordinate defense".into()],
        Capability::TerritorialDevelopmentAndInfrastructure => {
            ["expand infrastructure".into(), "protect territory".into()]
        }
    };
    Ok(PendingMindCommit {
        aggregate: MindAggregate {
            faction: command.faction,
            doctrine_version: command.doctrine_version,
            motives,
            priorities: command.priorities,
            goals: command.priorities,
            short_term_plans: command.priorities,
            long_term_plans: command.priorities,
            posture: command.posture,
            slots: prior.slots.clone(),
            history: prior.history.clone(),
            ledger: prior.ledger.clone(),
            commands: prior.commands.clone(),
        },
        events: [CausalEvent::new(
            CausalKind::MindUpdated,
            command.faction,
            command.id,
            0,
        )],
    })
}
