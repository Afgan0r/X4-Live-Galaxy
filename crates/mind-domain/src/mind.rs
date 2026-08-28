use crate::{CausalEvent, CausalKind};
use strategic_state::{BilateralPosture, Capability, Faction, StrategicPacket};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandId(&'static str);

impl CommandId {
    #[must_use]
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MindError {
    FactionMismatch,
    UnsupportedProfile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MindCommand {
    faction: Faction,
    doctrine_version: &'static str,
    priorities: [Capability; 3],
    posture: BilateralPosture,
    id: CommandId,
}

impl MindCommand {
    #[must_use]
    pub const fn from_packet(packet: &StrategicPacket, id: CommandId) -> Self {
        let posture = match packet.profile().priorities()[0] {
            Capability::DefenseAndMilitaryStrategy => BilateralPosture::IncreasePressure,
            Capability::EconomyAndLogistics => BilateralPosture::SeekLimitedCoordination,
            Capability::TerritorialDevelopmentAndInfrastructure => {
                BilateralPosture::PreserveRelations
            }
        };
        Self {
            faction: packet.faction(),
            doctrine_version: packet.profile_version(),
            priorities: packet.profile().priorities(),
            posture,
            id,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MindAggregate {
    faction: Faction,
    doctrine_version: &'static str,
    motives: [&'static str; 2],
    priorities: [Capability; 3],
    goals: [Capability; 3],
    short_term_plans: [Capability; 3],
    long_term_plans: [Capability; 3],
    posture: BilateralPosture,
}

impl MindAggregate {
    #[must_use]
    pub const fn empty(faction: Faction) -> Self {
        Self {
            faction,
            doctrine_version: "",
            motives: ["", ""],
            priorities: Capability::ALL,
            goals: Capability::ALL,
            short_term_plans: Capability::ALL,
            long_term_plans: Capability::ALL,
            posture: BilateralPosture::PreserveRelations,
        }
    }
    #[must_use]
    pub const fn doctrine_version(&self) -> &'static str {
        self.doctrine_version
    }
    #[must_use]
    pub const fn motives(&self) -> &[&'static str; 2] {
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingMindCommit {
    aggregate: MindAggregate,
    events: [CausalEvent; 1],
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
        Capability::DefenseAndMilitaryStrategy => ["protect territory", "sustain pressure"],
        Capability::EconomyAndLogistics => ["sustain economy", "coordinate defense"],
        Capability::TerritorialDevelopmentAndInfrastructure => {
            ["expand infrastructure", "protect territory"]
        }
    };
    let aggregate = MindAggregate {
        faction: command.faction,
        doctrine_version: command.doctrine_version,
        motives,
        priorities: command.priorities,
        goals: command.priorities,
        short_term_plans: command.priorities,
        long_term_plans: command.priorities,
        posture: command.posture,
    };
    Ok(PendingMindCommit {
        aggregate,
        events: [CausalEvent::new(
            CausalKind::MindUpdated,
            command.faction,
            command.id,
            0,
        )],
    })
}
