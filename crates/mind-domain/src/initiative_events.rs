use crate::{CausalEvent, CausalKind, CommandId, Initiative, InitiativeError, InitiativeId};
use strategic_state::Faction;

pub fn events(
    faction: Faction,
    command: &CommandId,
    initiative: &InitiativeId,
    kinds: &[CausalKind],
) -> Vec<CausalEvent> {
    kinds
        .iter()
        .zip(0u8..)
        .map(|(kind, sequence)| {
            CausalEvent::for_initiative(
                *kind,
                faction,
                command.clone(),
                sequence,
                initiative.clone(),
            )
        })
        .collect()
}
pub fn update_history(
    history: &mut [Initiative],
    id: &InitiativeId,
    state: crate::InitiativeLifecycle,
) -> Result<(), InitiativeError> {
    let Some(initiative) = history.iter_mut().find(|initiative| initiative.id() == id) else {
        return Err(InitiativeError::StalePredecessor);
    };
    initiative.state = state;
    Ok(())
}
