const CAPSULE_SCHEMA: &str = "mind-capsule-v1";
const MAX_FIELD_BYTES: usize = 128;
const MAX_NARRATIVE_BYTES: usize = 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerRange {
    first_sequence: u64,
    last_sequence: u64,
    integrity_hash: String,
}

impl LedgerRange {
    #[must_use]
    pub fn new(first_sequence: u64, last_sequence: u64, integrity_hash: &str) -> Self {
        Self {
            first_sequence,
            last_sequence,
            integrity_hash: integrity_hash.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetProfile {
    provider: String,
    model: String,
    context_limit: u32,
    measured_tokens: u32,
    headroom_tokens: u32,
}

impl BudgetProfile {
    #[must_use]
    pub fn new(
        provider: &str,
        model: &str,
        context_limit: u32,
        measured_tokens: u32,
        headroom_tokens: u32,
    ) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            context_limit,
            measured_tokens,
            headroom_tokens,
        }
    }

    const fn eligible(&self) -> bool {
        self.measured_tokens.saturating_add(self.headroom_tokens) >= self.context_limit
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitmentProjection {
    goal: String,
    plan: String,
    posture: String,
    initiative_owner: String,
}

impl CommitmentProjection {
    #[must_use]
    pub fn new(goal: &str, plan: &str, posture: &str, initiative_owner: &str) -> Self {
        Self {
            goal: goal.into(),
            plan: plan.into(),
            posture: posture.into(),
            initiative_owner: initiative_owner.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapsuleError {
    InvalidBudget,
    InvalidField,
    InvalidRange,
    NarrativeTooLarge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capsule {
    range: LedgerRange,
    profile: BudgetProfile,
    commitments: CommitmentProjection,
    narrative: Option<String>,
}

impl Capsule {
    pub fn new(
        range: LedgerRange,
        profile: BudgetProfile,
        commitments: CommitmentProjection,
        narrative: Option<&str>,
    ) -> Result<Self, CapsuleError> {
        validate_range(&range)?;
        validate_profile(&profile)?;
        validate_commitments(&commitments)?;
        let narrative = validate_narrative(narrative)?;
        Ok(Self {
            range,
            profile,
            commitments,
            narrative,
        })
    }

    #[must_use]
    pub const fn eligible(&self) -> bool {
        self.profile.eligible()
    }

    #[must_use]
    pub fn identity(&self) -> String {
        format!(
            "{CAPSULE_SCHEMA}:{}:{}:{}:{}:{}",
            self.range.first_sequence,
            self.range.last_sequence,
            self.range.integrity_hash,
            self.profile.provider,
            self.profile.model
        )
    }

    #[must_use]
    pub fn typed_commitments(&self) -> CommitmentProjection {
        self.commitments.clone()
    }

    #[must_use]
    pub fn narrative(&self) -> Option<&str> {
        self.narrative.as_deref()
    }

    #[must_use]
    pub fn with_narrative(&self, narrative: Option<&str>) -> Self {
        let narrative = validate_narrative(narrative).ok().flatten();
        Self {
            range: self.range.clone(),
            profile: self.profile.clone(),
            commitments: self.commitments.clone(),
            narrative,
        }
    }
}

fn validate_range(range: &LedgerRange) -> Result<(), CapsuleError> {
    if range.first_sequence > range.last_sequence || !bounded(&range.integrity_hash) {
        return Err(CapsuleError::InvalidRange);
    }
    Ok(())
}

fn validate_profile(profile: &BudgetProfile) -> Result<(), CapsuleError> {
    if !bounded(&profile.provider)
        || !bounded(&profile.model)
        || profile.measured_tokens > profile.context_limit
        || profile.headroom_tokens > profile.context_limit
    {
        return Err(CapsuleError::InvalidBudget);
    }
    Ok(())
}

fn validate_commitments(commitments: &CommitmentProjection) -> Result<(), CapsuleError> {
    if [
        &commitments.goal,
        &commitments.plan,
        &commitments.posture,
        &commitments.initiative_owner,
    ]
    .into_iter()
    .all(|field| bounded(field))
    {
        Ok(())
    } else {
        Err(CapsuleError::InvalidField)
    }
}

fn validate_narrative(narrative: Option<&str>) -> Result<Option<String>, CapsuleError> {
    if narrative.is_some_and(|text| text.len() > MAX_NARRATIVE_BYTES) {
        Err(CapsuleError::NarrativeTooLarge)
    } else {
        Ok(narrative.map(str::to_owned))
    }
}

const fn bounded(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_FIELD_BYTES
}
