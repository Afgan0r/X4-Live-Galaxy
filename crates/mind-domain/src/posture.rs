use serde::{Deserialize, Serialize};

const MAX_FACTS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadowPosture {
    Maintain,
    DeEscalate,
    Intensify,
    LimitedThreatDrivenCoordination,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostureCandidate {
    pub posture: ShadowPosture,
    pub fact_ids: Vec<String>,
    pub effect: PostureEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostureEffect {
    ShadowOnly,
    Negotiation,
    X4Command,
    ReportIntent,
    RelationshipChange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostureRejection {
    Information,
    MissingVisibleThreat,
    ExternalEffect,
}

pub fn admit_posture(
    candidate: &PostureCandidate,
    visible_fact_ids: &[String],
) -> Result<ShadowPosture, PostureRejection> {
    if candidate.fact_ids.is_empty()
        || candidate.fact_ids.len() > MAX_FACTS
        || candidate.fact_ids.windows(2).any(|pair| pair[0] >= pair[1])
        || candidate
            .fact_ids
            .iter()
            .any(|fact| !visible_fact_ids.contains(fact))
    {
        return Err(PostureRejection::Information);
    }
    if candidate.effect != PostureEffect::ShadowOnly {
        return Err(PostureRejection::ExternalEffect);
    }
    if candidate.posture == ShadowPosture::LimitedThreatDrivenCoordination
        && !candidate
            .fact_ids
            .iter()
            .any(|fact| fact.split(':').any(|part| part == "threat"))
    {
        return Err(PostureRejection::MissingVisibleThreat);
    }
    Ok(candidate.posture)
}
