use crate::ship_detail::outcome_name;
use crate::{FieldOutcome, ShipDetailDependency, ShipDetailError, detail_token};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrewTier {
    pub name: String,
    pub skill_lower_threshold: i32,
    pub amount_people: u32,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrewRole {
    pub id: String,
    pub amount_people: u32,
    pub reported_numtiers: u32,
    pub canhire: bool,
    pub tiers: Vec<CrewTier>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrewObservation {
    pub dependency: ShipDetailDependency,
    pub capacity: FieldOutcome<u32>,
    pub roles: FieldOutcome<Vec<CrewRole>>,
}

impl CrewObservation {
    pub fn validate(&self, maximum: usize) -> Result<(), ShipDetailError> {
        self.dependency.validate()?;
        if let FieldOutcome::Value(roles) = &self.roles {
            validate_roles(roles, maximum)?;
        }
        Ok(())
    }
    #[must_use]
    pub fn canonical_content(&self) -> String {
        use core::fmt::Write as _;
        let mut text = crate::ship_detail_codec::header("ship_crew", &self.dependency);
        let _ = write!(
            text,
            "\ncapacity_outcome={}\nincludepilot=true\nincludearriving=true\nrole_coverage=observed_count_fill_only\nroles_outcome={}",
            outcome_name(&self.capacity),
            outcome_name(&self.roles)
        );
        append_capacity(&mut text, &self.capacity);
        append_roles(&mut text, &self.roles);
        text
    }
}
fn validate_roles(roles: &[CrewRole], maximum: usize) -> Result<(), ShipDetailError> {
    if roles.is_empty() || roles.len() > maximum {
        return Err(ShipDetailError::ExceededBound);
    }
    if roles.windows(2).any(|v| v[0].id >= v[1].id) {
        return Err(ShipDetailError::InvalidField);
    }
    for role in roles {
        validate_role(role, maximum)?;
    }
    Ok(())
}
fn validate_role(role: &CrewRole, maximum: usize) -> Result<(), ShipDetailError> {
    if !detail_token(&role.id)
        || role.tiers.len() > maximum
        || role.reported_numtiers as usize > maximum
        || role.tiers.len() > role.reported_numtiers as usize
    {
        return Err(ShipDetailError::ExceededBound);
    }
    if role
        .tiers
        .iter()
        .any(|v| v.name.is_empty() || v.name.len() > 128 || v.name.contains(['|', '\n', '\r']))
    {
        return Err(ShipDetailError::InvalidField);
    }
    Ok(())
}
fn append_capacity(text: &mut String, capacity: &FieldOutcome<u32>) {
    use core::fmt::Write as _;
    if let FieldOutcome::Value(value) = capacity {
        let _ = write!(text, "\ncapacity_people={value}");
    }
}
fn append_roles(text: &mut String, outcome: &FieldOutcome<Vec<CrewRole>>) {
    let FieldOutcome::Value(roles) = outcome else {
        return;
    };
    for role in roles {
        append_role(text, role);
    }
}
fn append_role(text: &mut String, role: &CrewRole) {
    use core::fmt::Write as _;
    let _ = write!(
        text,
        "\nrole={}|{}|{}|{}",
        role.id, role.amount_people, role.reported_numtiers, role.canhire
    );
    for tier in &role.tiers {
        let _ = write!(
            text,
            "\ntier={}|{}|{}|{}",
            role.id, tier.name, tier.skill_lower_threshold, tier.amount_people
        );
    }
}
