use crate::ship_detail_codec::{dependency, field};
use crate::{
    CrewObservation, CrewRole, CrewTier, FieldOutcome, ShipDetailError, detail_number,
    detail_outcome,
};

impl CrewObservation {
    pub fn from_content(content: &str, maximum: usize) -> Result<Self, ShipDetailError> {
        let mut lines = content.lines();
        if lines.next() != Some("profile=ship_crew") {
            return Err(ShipDetailError::InvalidField);
        }
        let dependency = dependency(&mut lines)?;
        let capacity_outcome = field(&mut lines, "capacity_outcome=")?;
        if lines.next() != Some("includepilot=true")
            || lines.next() != Some("includearriving=true")
            || lines.next() != Some("role_coverage=observed_count_fill_only")
        {
            return Err(ShipDetailError::InvalidField);
        }
        let roles_outcome = field(&mut lines, "roles_outcome=")?;
        let capacity = capacity(&mut lines, capacity_outcome)?;
        let roles = roles(lines, maximum)?;
        let record = Self {
            dependency,
            capacity,
            roles: detail_outcome(roles_outcome, roles)?,
        };
        record.validate(maximum)?;
        if record.canonical_content() != content {
            return Err(ShipDetailError::InvalidField);
        }
        Ok(record)
    }
}
fn capacity(
    lines: &mut std::str::Lines<'_>,
    outcome: &str,
) -> Result<FieldOutcome<u32>, ShipDetailError> {
    Ok(match outcome {
        "value" => FieldOutcome::Value(
            u32::try_from(detail_number(field(lines, "capacity_people=")?)?)
                .map_err(|_| ShipDetailError::InvalidNumber)?,
        ),
        "zero" => FieldOutcome::Zero,
        "empty" => FieldOutcome::Empty,
        "absent" => FieldOutcome::Absent,
        "unknown" => FieldOutcome::Unknown,
        "inaccessible" => FieldOutcome::Inaccessible,
        "unsupported" => FieldOutcome::Unsupported,
        "stale" => FieldOutcome::Stale,
        _ => return Err(ShipDetailError::InvalidField),
    })
}
fn roles(lines: std::str::Lines<'_>, maximum: usize) -> Result<Vec<CrewRole>, ShipDetailError> {
    let mut roles = Vec::new();
    for line in lines {
        if let Some(value) = line.strip_prefix("role=") {
            push_role(&mut roles, value, maximum)?;
            continue;
        }
        let value = line
            .strip_prefix("tier=")
            .ok_or(ShipDetailError::InvalidField)?;
        push_tier(&mut roles, value, maximum)?;
    }
    Ok(roles)
}
fn parts(value: &str) -> Result<[&str; 4], ShipDetailError> {
    let mut parts = value.split('|');
    let result = [parts.next(), parts.next(), parts.next(), parts.next()];
    let [Some(a), Some(b), Some(c), Some(d)] = result else {
        return Err(ShipDetailError::InvalidField);
    };
    if parts.next().is_some() {
        return Err(ShipDetailError::InvalidField);
    }
    Ok([a, b, c, d])
}
fn unsigned(value: &str) -> Result<u32, ShipDetailError> {
    u32::try_from(detail_number(value)?).map_err(|_| ShipDetailError::InvalidNumber)
}
fn push_role(
    roles: &mut Vec<CrewRole>,
    value: &str,
    maximum: usize,
) -> Result<(), ShipDetailError> {
    if roles.len() >= maximum {
        return Err(ShipDetailError::ExceededBound);
    }
    let [id, amount, tiers, canhire] = parts(value)?;
    let canhire = match canhire {
        "true" => true,
        "false" => false,
        _ => return Err(ShipDetailError::InvalidField),
    };
    roles.push(CrewRole {
        id: id.to_owned(),
        amount_people: unsigned(amount)?,
        reported_numtiers: unsigned(tiers)?,
        canhire,
        tiers: Vec::new(),
    });
    Ok(())
}
fn push_tier(roles: &mut [CrewRole], value: &str, maximum: usize) -> Result<(), ShipDetailError> {
    let [id, name, threshold, amount] = parts(value)?;
    let role = roles.last_mut().ok_or(ShipDetailError::InvalidField)?;
    if role.id != id || role.tiers.len() >= maximum {
        return Err(ShipDetailError::ExceededBound);
    }
    let skill = threshold
        .parse::<i32>()
        .map_err(|_| ShipDetailError::InvalidNumber)?;
    if skill.to_string() != threshold {
        return Err(ShipDetailError::InvalidNumber);
    }
    role.tiers.push(CrewTier {
        name: name.to_owned(),
        skill_lower_threshold: skill,
        amount_people: unsigned(amount)?,
    });
    Ok(())
}
