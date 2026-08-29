use crate::{
    CommandId, Initiative, InitiativeCommand, InitiativeLifecycle, InitiativeSpec,
    PreemptionDisposition,
};

const MAX_REASON: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutiveDecision {
    Approve,
    Revise,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialogueState {
    DirectAgreement,
    MaterialObjection { cycles: u8 },
    FinalDisposition,
}

impl DialogueState {
    pub fn advance(self) -> Result<Self, ArbitrationError> {
        match self {
            Self::DirectAgreement => Ok(Self::FinalDisposition),
            Self::MaterialObjection { cycles } if cycles < 2 => {
                Ok(Self::MaterialObjection { cycles: cycles + 1 })
            }
            Self::MaterialObjection { .. } | Self::FinalDisposition => {
                Err(ArbitrationError::CycleCap)
            }
        }
    }

    #[must_use]
    pub const fn cycles(self) -> u8 {
        cycles(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArbitrationError {
    CycleCap,
    IncompleteCausalRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreemptionRequest {
    command: CommandId,
    trigger: String,
    prior: Initiative,
    disposition: PreemptionDisposition,
    replacement: InitiativeSpec,
    decision: ExecutiveDecision,
    reason: String,
}

impl PreemptionRequest {
    #[expect(
        clippy::too_many_arguments,
        reason = "the constructor requires every causal field at the admission boundary"
    )]
    pub fn new(
        command: CommandId,
        trigger: &str,
        prior: Initiative,
        disposition: PreemptionDisposition,
        replacement: InitiativeSpec,
        decision: ExecutiveDecision,
        reason: &str,
    ) -> Result<Self, ArbitrationError> {
        if trigger.is_empty()
            || reason.is_empty()
            || reason.len() > MAX_REASON
            || prior.state() != InitiativeLifecycle::Active
            || decision != ExecutiveDecision::Approve
        {
            return Err(ArbitrationError::IncompleteCausalRecord);
        }
        Ok(Self {
            command,
            trigger: trigger.into(),
            prior,
            disposition,
            replacement,
            decision,
            reason: reason.into(),
        })
    }

    #[must_use]
    pub fn command(&self) -> InitiativeCommand {
        InitiativeCommand::preempt(
            self.command.clone(),
            self.prior.id().clone(),
            self.replacement.clone(),
            &self.trigger,
            self.disposition,
        )
    }

    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    #[must_use]
    pub const fn decision(&self) -> ExecutiveDecision {
        self.decision
    }
}

const fn cycles(state: DialogueState) -> u8 {
    match state {
        DialogueState::MaterialObjection { cycles } => cycles,
        _ => 0,
    }
}
