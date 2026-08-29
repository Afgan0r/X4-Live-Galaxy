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
    current_snapshot_identity: &str,
    bytes: &[u8],
    preemption: PreemptionRequest,
) -> Result<AcceptedPreemption, AdmissionRejection> {
    let accepted = match admit(request, prior, current_snapshot_identity, bytes) {
        AdmissionDecision::Accepted(accepted) => accepted,
        AdmissionDecision::Rejected(rejection) => return Err(rejection),
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
