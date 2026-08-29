use crate::{
    CausalEvent, CausalKind, Initiative, InitiativeCommand, InitiativeError, InitiativeId,
    InitiativeLifecycle, MindAggregate,
};

mod preemption;

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
    let events = transition(&mut aggregate, command.clone())?;
    aggregate.ledger.extend(events.iter().cloned());
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
        } => preemption::apply(aggregate, id, &predecessor, replacement, disposition),
        InitiativeCommand::Terminal {
            id,
            initiative,
            state,
        } => terminal(aggregate, id, initiative, state),
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "command ownership is consumed by causal evidence"
)]
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
    let events = crate::initiative_events::events(
        aggregate.faction,
        &id,
        &spec.id,
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
        validating_event: events[3].clone(),
    };
    aggregate.slots[index] = Some(initiative.clone());
    aggregate.history.push(initiative);
    Ok(events)
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "command and target ownership are consumed by causal evidence"
)]
fn terminal(
    aggregate: &mut MindAggregate,
    id: crate::CommandId,
    target: InitiativeId,
    state: InitiativeLifecycle,
) -> Result<Vec<CausalEvent>, InitiativeError> {
    let Some(index) = aggregate.slots.iter().position(|value| {
        value
            .as_ref()
            .is_some_and(|initiative| initiative.id() == &target)
    }) else {
        return Err(InitiativeError::IllegalTerminal);
    };
    let kind = match state {
        InitiativeLifecycle::Completed => CausalKind::Completed,
        InitiativeLifecycle::Cancelled => CausalKind::Cancelled,
        InitiativeLifecycle::Rejected => CausalKind::Rejected,
        InitiativeLifecycle::Failed => CausalKind::Failed,
        InitiativeLifecycle::Active => return Err(InitiativeError::IllegalTerminal),
    };
    crate::initiative_events::update_history(&mut aggregate.history, &target, state)?;
    aggregate.slots[index] = None;
    Ok(crate::initiative_events::events(
        aggregate.faction,
        &id,
        &target,
        &[kind],
    ))
}
