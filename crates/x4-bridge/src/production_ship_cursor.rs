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

#[cfg(test)]
mod tests {
    use super::next_member;

    #[test]
    fn replacement_parents_resume_identity_and_unfinished_family() {
        let reordered = ["x4:ship:3".into(), "x4:ship:1".into(), "x4:ship:2".into()];
        assert_eq!(
            next_member(&reordered, "ship_core", Some(("x4:ship:1", "ship_cargo"))),
            Ok("ship_crew:g0".into())
        );
        let replaced = ["x4:ship:4".into(), "x4:ship:2".into(), "x4:ship:1".into()];
        assert_eq!(
            next_member(&replaced, "ship_core", Some(("x4:ship:1", "ship_crew"))),
            Ok("ship_loadout:g0".into())
        );
        assert_eq!(
            next_member(&replaced, "ship_core", Some(("x4:ship:1", "ship_loadout"))),
            Ok("ship_cargo:g1".into())
        );
        assert_eq!(
            next_member(&replaced, "ship_core", Some(("x4:ship:3", "ship_cargo"))),
            Ok("ship_cargo:g2".into())
        );
    }
}
