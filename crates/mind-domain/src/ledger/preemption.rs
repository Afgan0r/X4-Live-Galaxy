use crate::{
    CausalEvent, CausalKind, Initiative, InitiativeError, InitiativeId, InitiativeLifecycle,
    MindAggregate, PreemptionDisposition,
};

const MAX_HISTORY: usize = 16;
const MAX_EVENTS: usize = 64;

#[expect(
    clippy::needless_pass_by_value,
    reason = "command ownership is consumed by causal evidence"
)]
pub(super) fn apply(
    aggregate: &mut MindAggregate,
    id: crate::CommandId,
    predecessor: &InitiativeId,
    replacement: crate::InitiativeSpec,
    disposition: PreemptionDisposition,
) -> Result<Vec<CausalEvent>, InitiativeError> {
    let index = crate::initiative::slot(replacement.capability);
    let Some(ref active) = aggregate.slots[index] else {
        return Err(InitiativeError::StalePredecessor);
    };
    if active.id() != predecessor {
        return Err(InitiativeError::StalePredecessor);
    }
    if aggregate.history.len() == MAX_HISTORY || aggregate.ledger.len() + 6 > MAX_EVENTS {
        return Err(InitiativeError::CapacityExceeded);
    }
    let state = match disposition {
        PreemptionDisposition::Cancelled => InitiativeLifecycle::Cancelled,
        PreemptionDisposition::Rejected => InitiativeLifecycle::Rejected,
    };
    crate::initiative_events::update_history(&mut aggregate.history, predecessor, state)?;
    let mut events = vec![CausalEvent::preemption(
        aggregate.faction,
        id.clone(),
        0,
        predecessor.clone(),
        disposition,
    )];
    events.extend(crate::initiative_events::events(
        aggregate.faction,
        &id,
        &replacement.id,
        &[
            CausalKind::Proposal,
            CausalKind::Objection,
            CausalKind::ExecutiveDisposition,
            CausalKind::Validated,
            CausalKind::OwnershipAssigned,
        ],
    ));
    let initiative = Initiative {
        spec: replacement,
        state: InitiativeLifecycle::Active,
        validating_event: events[4].clone(),
    };
    aggregate.slots[index] = Some(initiative.clone());
    aggregate.history.push(initiative);
    Ok(events)
}
