use crate::{
    CausalKind, MIND_CHECKPOINT_SCHEMA_VERSION, MindAggregate, MindCheckpointError,
    MindCheckpointState, MindCommand, transition,
};
use strategic_state::{BilateralPosture, FactionProfile};
const MAX_HISTORY: usize = 16;
const MAX_EVENTS: usize = 64;
const MAX_COMMANDS: usize = 16;
pub fn restore(state: &MindCheckpointState) -> Result<MindAggregate, MindCheckpointError> {
    if state.schema_version != MIND_CHECKPOINT_SCHEMA_VERSION {
        return Err(MindCheckpointError::UnsupportedSchema);
    }
    let aggregate = state.commit.aggregate();
    if aggregate.history.len() > MAX_HISTORY
        || aggregate.ledger.len() > MAX_EVENTS
        || aggregate.commands.len() > MAX_COMMANDS
    {
        return Err(MindCheckpointError::CapacityExceeded);
    }
    if !valid_core(aggregate)
        || aggregate.slots.iter().flatten().any(|value| !value.valid())
        || aggregate.history.iter().any(|value| !value.valid())
        || aggregate.ledger.iter().any(|value| !value.valid())
        || state.commit.events.iter().any(|value| !value.valid())
        || aggregate.commands.iter().any(|(command, events)| {
            !command.valid() || events.len() > 6 || events.iter().any(|value| !value.valid())
        })
    {
        return Err(MindCheckpointError::Invalid);
    }
    let mind_event = &state.commit.events[0];
    if mind_event.kind != CausalKind::MindUpdated || mind_event.faction != aggregate.faction {
        return Err(MindCheckpointError::Invalid);
    }
    let base = transition(
        &MindAggregate::empty(aggregate.faction),
        MindCommand {
            faction: aggregate.faction,
            doctrine_version: aggregate.doctrine_version.clone(),
            priorities: aggregate.priorities,
            posture: aggregate.posture,
            id: mind_event.command.clone(),
        },
    )
    .map_err(|_| MindCheckpointError::Invalid)?;
    if base.events() != &state.commit.events || !same_core(base.aggregate(), aggregate) {
        return Err(MindCheckpointError::ReplayMismatch);
    }
    let mut replay = base.aggregate().clone();
    for (command, events) in &aggregate.commands {
        let committed = replay
            .apply_initiative(command.clone())
            .map_err(|_| MindCheckpointError::Invalid)?;
        if committed.events() != events {
            return Err(MindCheckpointError::ReplayMismatch);
        }
        replay = committed.aggregate().clone();
    }
    if replay != *aggregate {
        return Err(MindCheckpointError::ReplayMismatch);
    }
    Ok(replay)
}

fn valid_core(aggregate: &MindAggregate) -> bool {
    let profile = FactionProfile::for_faction(aggregate.faction);
    aggregate.doctrine_version == profile.version()
        && aggregate.priorities == profile.priorities()
        && aggregate
            .motives
            .iter()
            .all(|value| crate::initiative::valid(value))
        && aggregate.posture == posture(aggregate.priorities[0])
}

fn same_core(left: &MindAggregate, right: &MindAggregate) -> bool {
    left.faction == right.faction
        && left.doctrine_version == right.doctrine_version
        && left.motives == right.motives
        && left.priorities == right.priorities
        && left.goals == right.goals
        && left.short_term_plans == right.short_term_plans
        && left.long_term_plans == right.long_term_plans
        && left.posture == right.posture
}

const fn posture(priority: strategic_state::Capability) -> BilateralPosture {
    match priority {
        strategic_state::Capability::DefenseAndMilitaryStrategy => {
            BilateralPosture::IncreasePressure
        }
        strategic_state::Capability::EconomyAndLogistics => {
            BilateralPosture::SeekLimitedCoordination
        }
        strategic_state::Capability::TerritorialDevelopmentAndInfrastructure => {
            BilateralPosture::PreserveRelations
        }
    }
}
