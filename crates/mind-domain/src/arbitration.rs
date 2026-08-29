use crate::{
    CommandId, Initiative, InitiativeCommand, InitiativeLifecycle, InitiativeSpec,
    PreemptionDisposition,
};
use serde::{Deserialize, Serialize};

const MAX_REASON: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    pub const fn advance(self) -> Result<Self, ArbitrationError> {
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

    pub const fn finalize(self) -> Result<Self, ArbitrationError> {
        match self {
            Self::DirectAgreement | Self::MaterialObjection { .. } => Ok(Self::FinalDisposition),
            Self::FinalDisposition => Err(ArbitrationError::CycleCap),
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
            || trigger.len() > MAX_REASON
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
    pub const fn command_id(&self) -> &CommandId {
        &self.command
    }

    #[must_use]
    pub const fn decision(&self) -> ExecutiveDecision {
        self.decision
    }

    #[must_use]
    pub fn valid(&self) -> bool {
        self.command.valid()
            && !self.trigger.is_empty()
            && self.trigger.len() <= MAX_REASON
            && self.prior.valid()
            && self.prior.state() == InitiativeLifecycle::Active
            && self.replacement.valid()
            && self.decision == ExecutiveDecision::Approve
            && !self.reason.is_empty()
            && self.reason.len() <= MAX_REASON
    }
}

const fn cycles(state: DialogueState) -> u8 {
    match state {
        DialogueState::MaterialObjection { cycles } => cycles,
        _ => 0,
    }
}
