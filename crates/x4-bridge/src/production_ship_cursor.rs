use crate::ProductionError;

pub(crate) fn next_member(
    members: &[String],
    previous: &str,
    _cursor: Option<(&str, &str)>,
) -> Result<String, ProductionError> {
    if previous == "ship_core" {
        return Ok("ship_cargo:g0".into());
    }
    let (family, group) = previous
        .split_once(":g")
        .ok_or(ProductionError::InvalidLimits)?;
    let group = group
        .parse::<usize>()
        .map_err(|_| ProductionError::InvalidLimits)?;
    match family {
        "ship_cargo" => Ok(format!("ship_crew:g{group}")),
        "ship_crew" => Ok(format!("ship_loadout:g{group}")),
        "ship_loadout" if group + 1 < members.len() => Ok(format!("ship_cargo:g{}", group + 1)),
        "ship_loadout" => Ok("ship_core".into()),
        _ => Err(ProductionError::InvalidLimits),
    }
}
