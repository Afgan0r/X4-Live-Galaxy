use crate::capsule::{BudgetProfile, CAPSULE_SCHEMA, CommitmentProjection, LedgerRange};

pub fn build(
    range: &LedgerRange,
    profile: &BudgetProfile,
    commitments: &CommitmentProjection,
) -> String {
    let first_sequence = range.first_sequence.to_string();
    let last_sequence = range.last_sequence.to_string();
    let context_limit = profile.context_limit.to_string();
    let measured_tokens = profile.measured_tokens.to_string();
    let headroom_tokens = profile.headroom_tokens.to_string();
    let mut identity = String::new();
    for value in [
        CAPSULE_SCHEMA,
        &first_sequence,
        &last_sequence,
        &range.integrity_hash,
        &profile.provider,
        &profile.model,
        &context_limit,
        &measured_tokens,
        &headroom_tokens,
        &commitments.goal,
        &commitments.plan,
        &commitments.posture,
        &commitments.initiative_owner,
    ] {
        frame(&mut identity, value);
    }
    identity
}

fn frame(identity: &mut String, value: &str) {
    identity.push_str(&value.len().to_string());
    identity.push(':');
    identity.push_str(value);
}
