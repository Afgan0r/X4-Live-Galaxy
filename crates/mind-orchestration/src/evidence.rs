use crate::{EvidenceClass, ProviderRequest};
use mind_domain::{AdmissionDecision, AdmissionRejection};
use strategic_state::Faction;

const MAX_FIELD: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceValue {
    Available(String),
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidatorOutcome {
    Accepted,
    Rejected(AdmissionRejection),
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryTransition {
    NotApplicable,
    PausedAwaitingReconciliation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedactedEvidence {
    correlation_id: String,
    faction: Faction,
    snapshot_identity: String,
    snapshot_hash: String,
    cache_identity: EvidenceValue,
    schema_identity: String,
    policy_identity: String,
    prompt_identity: String,
    provider_identity: String,
    model_identity: String,
    generation_identity: EvidenceValue,
    candidate_hash: EvidenceValue,
    candidate_size: Option<usize>,
    validator: ValidatorOutcome,
    usage: EvidenceValue,
    latency: EvidenceValue,
    admitted_goal_id: EvidenceValue,
    admitted_initiative_id: EvidenceValue,
    evidence_class: EvidenceClass,
    recovery: RecoveryTransition,
}

impl RedactedEvidence {
    pub(crate) fn admission(
        request: &ProviderRequest,
        candidate: &[u8],
        decision: &AdmissionDecision,
        evidence_class: EvidenceClass,
    ) -> Self {
        let admitted = match decision {
            AdmissionDecision::Accepted(value) => {
                EvidenceValue::Available(bounded(value.correlation_id()))
            }
            AdmissionDecision::Rejected(_) => EvidenceValue::Unavailable,
        };
        Self::base(
            request,
            evidence_class,
            EvidenceValue::Available(digest(candidate)),
            Some(candidate.len()),
            outcome(decision),
            admitted,
            RecoveryTransition::NotApplicable,
        )
    }

    pub(crate) fn degraded(request: &ProviderRequest, evidence_class: EvidenceClass) -> Self {
        Self::base(
            request,
            evidence_class,
            EvidenceValue::Unavailable,
            None,
            ValidatorOutcome::Unavailable,
            EvidenceValue::Unavailable,
            RecoveryTransition::PausedAwaitingReconciliation,
        )
    }

    fn base(
        request: &ProviderRequest,
        evidence_class: EvidenceClass,
        candidate_hash: EvidenceValue,
        candidate_size: Option<usize>,
        validator: ValidatorOutcome,
        admitted: EvidenceValue,
        recovery: RecoveryTransition,
    ) -> Self {
        let frozen = request.request();
        Self {
            correlation_id: bounded(request.identity()),
            faction: frozen.faction(),
            snapshot_identity: bounded(frozen.snapshot_identity()),
            snapshot_hash: digest(frozen.snapshot_identity().as_bytes()),
            cache_identity: EvidenceValue::Unavailable,
            schema_identity: "schema-v1".into(),
            policy_identity: bounded(frozen.policy_version()),
            prompt_identity: bounded(frozen.prompt_package_hash()),
            provider_identity: bounded(request.metadata().provider_id()),
            model_identity: bounded(request.metadata().model_id()),
            generation_identity: EvidenceValue::Unavailable,
            candidate_hash,
            candidate_size,
            validator,
            usage: EvidenceValue::Unavailable,
            latency: EvidenceValue::Unavailable,
            admitted_goal_id: admitted.clone(),
            admitted_initiative_id: admitted,
            evidence_class,
            recovery,
        }
    }

    #[must_use]
    pub const fn is_redacted_and_bounded(&self) -> bool {
        self.correlation_id.len() <= MAX_FIELD
            && self.snapshot_identity.len() <= MAX_FIELD
            && self.policy_identity.len() <= MAX_FIELD
            && self.prompt_identity.len() <= MAX_FIELD
            && self.provider_identity.len() <= MAX_FIELD
            && self.model_identity.len() <= MAX_FIELD
    }

    #[must_use]
    pub const fn validator(&self) -> ValidatorOutcome {
        self.validator
    }
    #[must_use]
    pub const fn recovery(&self) -> RecoveryTransition {
        self.recovery
    }
    #[must_use]
    pub const fn evidence_class(&self) -> EvidenceClass {
        self.evidence_class
    }
}

const fn outcome(decision: &AdmissionDecision) -> ValidatorOutcome {
    match decision {
        AdmissionDecision::Accepted(_) => ValidatorOutcome::Accepted,
        AdmissionDecision::Rejected(value) => ValidatorOutcome::Rejected(*value),
    }
}
fn bounded(value: &str) -> String {
    value.chars().take(MAX_FIELD).collect()
}
fn digest(value: &[u8]) -> String {
    format!(
        "bytes-v1:{:016x}",
        value
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |state, byte| (state
                ^ u64::from(*byte))
            .wrapping_mul(0x100_0000_01b3))
    )
}
