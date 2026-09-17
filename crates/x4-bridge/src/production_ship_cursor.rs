use crate::ProductionError;

pub fn next_member(
    members: &[String],
    previous: &str,
    cursor: Option<(&str, &str)>,
) -> Result<String, ProductionError> {
    let mut ordered = members.iter().map(String::as_str).collect::<Vec<_>>();
    ordered.sort_unstable();
    if ordered.is_empty() {
        return Ok("ship_core".into());
    }
    let Some((identity, family)) = cursor else {
        return Ok("ship_cargo:g0".into());
    };
    if let Some(group) = ordered.iter().position(|member| *member == identity) {
        match family {
            "ship_cargo" => return Ok(format!("ship_crew:g{group}")),
            "ship_crew" => return Ok(format!("ship_loadout:g{group}")),
            "ship_loadout" => {}
            _ => return Err(ProductionError::InvalidLimits),
        }
    }
    let group = ordered
        .iter()
        .position(|member| *member > identity)
        .unwrap_or(0);
    if group == 0 && family == "ship_loadout" && previous != "ship_core" {
        return Ok("ship_core".into());
    }
    u16::try_from(group).map_err(|_| ProductionError::InvalidLimits)?;
    Ok(format!("ship_cargo:g{group}"))
}

pub const fn refresh_required(
    now: u64,
    accepted: u64,
    freshness: u64,
    observed: u64,
    margin: u64,
) -> bool {
    let Some(age) = now.checked_sub(accepted) else {
        return true;
    };
    let Some(required) = observed.checked_add(margin) else {
        return true;
    };
    match freshness.checked_sub(age) {
        Some(remaining) => remaining <= required,
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::next_member;

    #[test]
    fn refresh_preserves_progress_across_slow_in_budget_families() {
        let members = ["x4:ship:1".into(), "x4:ship:2".into(), "x4:ship:3".into()];
        assert!(super::refresh_required(29_000, 0, 30_000, 12_000, 50));
        assert!(!super::refresh_required(31_000, 30_000, 30_000, 12_000, 50));
        assert_eq!(
            next_member(&members, "ship_core", Some(("x4:ship:1", "ship_cargo"))),
            Ok("ship_crew:g0".into())
        );
        assert!(super::refresh_required(59_000, 30_000, 30_000, 12_000, 50));
        assert_eq!(
            next_member(&members, "ship_core", Some(("x4:ship:1", "ship_crew"))),
            Ok("ship_loadout:g0".into())
        );
        assert_eq!(
            next_member(&members, "ship_core", Some(("x4:ship:1", "ship_loadout"))),
            Ok("ship_cargo:g1".into())
        );
        assert!(super::refresh_required(90_001, 60_000, 30_000, 0, 50));
    }

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
