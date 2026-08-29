use crate::{
    CommandId, DeliberationRequest, MindAggregate, MindCommand, PendingMindCommit, ShadowProposal,
};

const SCHEMA_VERSION: &str = "schema-v1";
const MAX_FACTS: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionDecision {
    Accepted(Box<AcceptedProposal>),
    Rejected(AdmissionRejection),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedProposal {
    proposal: ShadowProposal,
    packet: strategic_state::StrategicPacket,
    snapshot_identity: String,
    policy_version: String,
    prompt_package_hash: String,
    candidate_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionRejection {
    Oversized,
    Decode,
    Schema,
    Semantic,
    Information,
    Safety,
    Budget,
    CurrentState,
}

impl AdmissionDecision {
    #[must_use]
    pub fn pending_commit(
        &self,
        prior: &MindAggregate,
    ) -> Option<Result<PendingMindCommit, crate::MindError>> {
        match self {
            Self::Accepted(accepted) => Some(accepted.pending_commit(prior)),
            Self::Rejected(_) => None,
        }
    }
}

impl AcceptedProposal {
    pub fn pending_commit(
        &self,
        prior: &MindAggregate,
    ) -> Result<PendingMindCommit, crate::MindError> {
        crate::transition(
            prior,
            MindCommand::from_packet(&self.packet, self.command_id()),
        )
    }

    #[must_use]
    pub fn command_id(&self) -> CommandId {
        CommandId::new(&self.proposal.command_id)
    }

    #[must_use]
    pub fn snapshot_identity(&self) -> &str {
        &self.snapshot_identity
    }

    #[must_use]
    pub fn policy_version(&self) -> &str {
        &self.policy_version
    }

    #[must_use]
    pub fn prompt_package_hash(&self) -> &str {
        &self.prompt_package_hash
    }

    #[must_use]
    pub fn correlation_id(&self) -> &str {
        &self.proposal.command_id
    }

    #[must_use]
    pub const fn candidate_bytes(&self) -> usize {
        self.candidate_bytes
    }
}

#[must_use]
pub fn admit(
    request: &DeliberationRequest,
    prior: &MindAggregate,
    bytes: &[u8],
) -> AdmissionDecision {
    if bytes.len() > request.max_candidate_bytes {
        return AdmissionDecision::Rejected(AdmissionRejection::Oversized);
    }
    let proposal: ShadowProposal = match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(_) => return AdmissionDecision::Rejected(AdmissionRejection::Decode),
    };
    if proposal.schema_version != SCHEMA_VERSION {
        return AdmissionDecision::Rejected(AdmissionRejection::Schema);
    }
    if !semantic_valid(&proposal) {
        return AdmissionDecision::Rejected(AdmissionRejection::Semantic);
    }
    if proposal
        .supporting_fact_ids
        .iter()
        .any(|fact| !request.visible_fact_ids.contains(fact))
    {
        return AdmissionDecision::Rejected(AdmissionRejection::Information);
    }
    if !request.allowed_capabilities.contains(&proposal.capability) {
        return AdmissionDecision::Rejected(AdmissionRejection::Safety);
    }
    if proposal.trade_offs.len() > request.max_trade_offs {
        return AdmissionDecision::Rejected(AdmissionRejection::Budget);
    }
    if prior_faction_mismatch(request, prior) {
        return AdmissionDecision::Rejected(AdmissionRejection::CurrentState);
    }
    AdmissionDecision::Accepted(Box::new(AcceptedProposal {
        proposal,
        packet: request.packet.clone(),
        snapshot_identity: request.snapshot_identity.clone(),
        policy_version: request.policy_version.clone(),
        prompt_package_hash: request.prompt_package_hash.clone(),
        candidate_bytes: bytes.len(),
    }))
}

fn semantic_valid(proposal: &ShadowProposal) -> bool {
    (1..=3).contains(&proposal.priority)
        && crate::deliberation::valid(&proposal.horizon)
        && crate::deliberation::valid(&proposal.explanation)
        && crate::deliberation::valid(&proposal.command_id)
        && !proposal.supporting_fact_ids.is_empty()
        && proposal.supporting_fact_ids.len() <= MAX_FACTS
        && proposal
            .supporting_fact_ids
            .iter()
            .all(|fact| crate::deliberation::valid(fact))
        && proposal
            .trade_offs
            .iter()
            .all(|item| crate::deliberation::valid(item))
}

fn prior_faction_mismatch(request: &DeliberationRequest, prior: &MindAggregate) -> bool {
    request.packet.faction() != prior.faction()
}
