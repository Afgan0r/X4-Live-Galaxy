use crate::{
    CausalEvent, CausalKind, Initiative, InitiativeCommand, InitiativeError, InitiativeId,
    InitiativeLifecycle, MindAggregate, PreemptionDisposition,
};

const MAX_HISTORY: usize = 16;
const MAX_EVENTS: usize = 64;
const MAX_COMMANDS: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingInitiativeCommit {
    aggregate: MindAggregate,
    events: Vec<CausalEvent>,
}

impl PendingInitiativeCommit {
    #[must_use]
    pub const fn aggregate(&self) -> &MindAggregate {
        &self.aggregate
    }
    #[must_use]
    pub fn events(&self) -> &[CausalEvent] {
        &self.events
    }
}

pub fn apply(
    prior: &MindAggregate,
    command: InitiativeCommand,
) -> Result<PendingInitiativeCommit, InitiativeError> {
    if let Some((previous, events)) = prior
        .commands
        .iter()
        .find(|(previous, _)| previous.id() == command.id())
    {
        return if *previous == command {
            Ok(PendingInitiativeCommit {
                aggregate: prior.clone(),
                events: events.clone(),
            })
        } else {
            Err(InitiativeError::ContentCollision)
        };
    }
    if prior.commands.len() == MAX_COMMANDS {
        return Err(InitiativeError::CapacityExceeded);
    }
    let mut aggregate = prior.clone();
    let events = transition(&mut aggregate, command)?;
    aggregate.ledger.extend(events.iter().copied());
    aggregate.commands.push((command, events.clone()));
    Ok(PendingInitiativeCommit { aggregate, events })
}

fn transition(
    aggregate: &mut MindAggregate,
    command: InitiativeCommand,
) -> Result<Vec<CausalEvent>, InitiativeError> {
    match command {
        InitiativeCommand::Accept { id, initiative } => accept(aggregate, id, initiative),
        InitiativeCommand::Preempt {
            id,
            predecessor,
            replacement,
            trigger: _,
            disposition,
        } => preempt(aggregate, id, predecessor, replacement, disposition),
        InitiativeCommand::Terminal {
            id,
            initiative,
            state,
        } => terminal(aggregate, id, initiative, state),
    }
}

fn accept(
    aggregate: &mut MindAggregate,
    id: crate::CommandId,
    spec: crate::InitiativeSpec,
) -> Result<Vec<CausalEvent>, InitiativeError> {
    let index = crate::initiative::slot(spec.capability);
    if aggregate.slots[index].is_some() {
        return Err(InitiativeError::ActiveSlot);
    }
    if aggregate.history.len() == MAX_HISTORY || aggregate.ledger.len() + 5 > MAX_EVENTS {
        return Err(InitiativeError::CapacityExceeded);
    }
    let events = crate::initiative::events(
        aggregate.faction,
        id,
        spec.id,
        &[
            CausalKind::Proposal,
            CausalKind::Objection,
            CausalKind::ExecutiveDisposition,
            CausalKind::Validated,
            CausalKind::OwnershipAssigned,
        ],
    );
    let initiative = Initiative {
        spec,
        state: InitiativeLifecycle::Active,
        validating_event: events[3],
    };
    aggregate.slots[index] = Some(initiative);
    aggregate.history.push(initiative);
    Ok(events)
}

fn preempt(
    aggregate: &mut MindAggregate,
    id: crate::CommandId,
    predecessor: InitiativeId,
    replacement: crate::InitiativeSpec,
    disposition: PreemptionDisposition,
) -> Result<Vec<CausalEvent>, InitiativeError> {
    let index = crate::initiative::slot(replacement.capability);
    let Some(active) = aggregate.slots[index] else {
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
    crate::initiative::update_history(&mut aggregate.history, predecessor, state)?;
    let mut events = vec![CausalEvent::preemption(
        aggregate.faction,
        id,
        0,
        predecessor,
        disposition,
    )];
    events.extend(crate::initiative::events(
        aggregate.faction,
        id,
        replacement.id,
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
        validating_event: events[4],
    };
    aggregate.slots[index] = Some(initiative);
    aggregate.history.push(initiative);
    Ok(events)
}

fn terminal(
    aggregate: &mut MindAggregate,
    id: crate::CommandId,
    target: InitiativeId,
    state: InitiativeLifecycle,
) -> Result<Vec<CausalEvent>, InitiativeError> {
    let Some(index) = aggregate
        .slots
        .iter()
        .position(|value| value.is_some_and(|initiative| initiative.id() == target))
    else {
        return Err(InitiativeError::IllegalTerminal);
    };
    let kind = match state {
        InitiativeLifecycle::Completed => CausalKind::Completed,
        InitiativeLifecycle::Cancelled => CausalKind::Cancelled,
        InitiativeLifecycle::Rejected => CausalKind::Rejected,
        InitiativeLifecycle::Failed => CausalKind::Failed,
        InitiativeLifecycle::Active => return Err(InitiativeError::IllegalTerminal),
    };
    crate::initiative::update_history(&mut aggregate.history, target, state)?;
    aggregate.slots[index] = None;
    Ok(crate::initiative::events(
        aggregate.faction,
        id,
        target,
        &[kind],
    ))
}
