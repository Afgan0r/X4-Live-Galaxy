use crate::{
    AcceptedProposal, AdmissionDecision, AdmissionRejection, DeliberationRequest, MindAggregate,
    PendingMindCommit, PreemptionRequest, admit,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedPreemption {
    accepted: AcceptedProposal,
    preemption: PreemptionRequest,
    pending: PendingMindCommit,
}

impl AcceptedPreemption {
    #[must_use]
    pub const fn accepted(&self) -> &AcceptedProposal {
        &self.accepted
    }

    #[must_use]
    pub const fn preemption(&self) -> &PreemptionRequest {
        &self.preemption
    }

    #[must_use]
    pub const fn pending(&self) -> &PendingMindCommit {
        &self.pending
    }
}

pub fn admit_preemption(
    request: &DeliberationRequest,
    prior: &MindAggregate,
    bytes: &[u8],
    preemption: PreemptionRequest,
) -> Result<AcceptedPreemption, AdmissionRejection> {
    let AdmissionDecision::Accepted(accepted) = admit(request, prior, bytes) else {
        return Err(AdmissionRejection::Semantic);
    };
    if accepted.command_id() != *preemption.command_id() || !preemption.valid() {
        return Err(AdmissionRejection::Semantic);
    }
    let pending = accepted
        .pending_commit(prior)
        .map_err(|_| AdmissionRejection::CurrentState)?;
    let initiative = pending
        .aggregate()
        .apply_initiative(preemption.command())
        .map_err(|_| AdmissionRejection::CurrentState)?;
    Ok(AcceptedPreemption {
        pending: pending.with_initiative_commit(&initiative),
        accepted: *accepted,
        preemption,
    })
}
