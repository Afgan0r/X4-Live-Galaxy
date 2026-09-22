use crate::ProductionError;
use observation_domain::{ShipSectionIdentity, ShipSectionKind};

pub fn durable_cursor(
    value: &observation_persistence::CurrentRevision,
) -> Option<(&str, &'static str)> {
    let record = value.revision().records.first()?;
    let family = match ShipSectionIdentity::parse(value.revision().section_key.as_str())?.kind() {
        ShipSectionKind::Cargo => "ship_cargo",
        ShipSectionKind::Crew => "ship_crew",
        ShipSectionKind::Loadout => "ship_loadout",
        ShipSectionKind::Core => return None,
    };
    Some((record.entity_id.as_str(), family))
}

pub fn legacy_key(value: &str) -> Option<String> {
    let section = ShipSectionIdentity::parse(value)?;
    let legacy = match section.kind() {
        ShipSectionKind::Core => ShipSectionIdentity::core(None)?,
        kind => ShipSectionIdentity::detail(kind, None, section.group()?)?,
    };
    Some(legacy.key())
}

pub fn next_member(
    members: &[String],
    previous: &str,
    cursor: Option<(&str, &str)>,
    group_members: usize,
) -> Result<String, ProductionError> {
    if group_members == 0 {
        return Err(ProductionError::InvalidLimits);
    }
    let mut ordered = members.iter().map(String::as_str).collect::<Vec<_>>();
    ordered.sort_unstable();
    if ordered.is_empty() {
        return Ok("ship_core".into());
    }
    let Some((identity, family)) = cursor else {
        return Ok("ship_cargo:g0".into());
    };
    if let Some(position) = ordered.iter().position(|member| *member == identity) {
        let group = position / group_members;
        return next_family(family, group, group_members, ordered.len());
    }
    let position = ordered
        .iter()
        .position(|member| *member > identity)
        .unwrap_or(0);
    let group = position / group_members;
    if position == 0 && family == "ship_loadout" && previous != "ship_core" {
        return Ok("ship_core".into());
    }
    u16::try_from(group).map_err(|_| ProductionError::InvalidLimits)?;
    Ok(format!("ship_cargo:g{group}"))
}

fn next_family(
    family: &str,
    group: usize,
    group_members: usize,
    member_count: usize,
) -> Result<String, ProductionError> {
    match family {
        "ship_cargo" => Ok(format!("ship_crew:g{group}")),
        "ship_crew" => Ok(format!("ship_loadout:g{group}")),
        "ship_loadout" => {
            let next = group.saturating_add(1);
            Ok(if next.saturating_mul(group_members) < member_count {
                format!("ship_cargo:g{next}")
            } else {
                "ship_core".into()
            })
        }
        _ => Err(ProductionError::InvalidLimits),
    }
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
    fn full_parent_group_advances_by_family_not_by_ship() {
        let members = ["x4:ship:1".into(), "x4:ship:2".into(), "x4:ship:3".into()];
        let full_group = members.len();
        assert_eq!(
            next_member(&members, "ship_core", None, full_group),
            Ok("ship_cargo:g0".into())
        );
        assert_eq!(
            next_member(
                &members,
                "ship_cargo:g0",
                Some(("x4:ship:1", "ship_cargo")),
                full_group,
            ),
            Ok("ship_crew:g0".into())
        );
        assert_eq!(
            next_member(
                &members,
                "ship_crew:g0",
                Some(("x4:ship:1", "ship_crew")),
                full_group,
            ),
            Ok("ship_loadout:g0".into())
        );
        assert_eq!(
            next_member(
                &members,
                "ship_loadout:g0",
                Some(("x4:ship:1", "ship_loadout")),
                full_group,
            ),
            Ok("ship_core".into())
        );
    }

    #[test]
    fn refresh_preserves_progress_across_slow_in_budget_families() {
        let members = ["x4:ship:1".into(), "x4:ship:2".into(), "x4:ship:3".into()];
        assert!(super::refresh_required(29_000, 0, 30_000, 12_000, 50));
        assert!(!super::refresh_required(31_000, 30_000, 30_000, 12_000, 50));
        assert_eq!(
            next_member(&members, "ship_core", Some(("x4:ship:1", "ship_cargo")), 1),
            Ok("ship_crew:g0".into())
        );
        assert!(super::refresh_required(59_000, 30_000, 30_000, 12_000, 50));
        assert_eq!(
            next_member(&members, "ship_core", Some(("x4:ship:1", "ship_crew")), 1),
            Ok("ship_loadout:g0".into())
        );
        assert_eq!(
            next_member(
                &members,
                "ship_core",
                Some(("x4:ship:1", "ship_loadout")),
                1
            ),
            Ok("ship_cargo:g1".into())
        );
        assert!(super::refresh_required(90_001, 60_000, 30_000, 0, 50));
    }

    #[test]
    fn replacement_parents_resume_identity_and_unfinished_family() {
        let reordered = ["x4:ship:3".into(), "x4:ship:1".into(), "x4:ship:2".into()];
        assert_eq!(
            next_member(
                &reordered,
                "ship_core",
                Some(("x4:ship:1", "ship_cargo")),
                1
            ),
            Ok("ship_crew:g0".into())
        );
        let replaced = ["x4:ship:4".into(), "x4:ship:2".into(), "x4:ship:1".into()];
        assert_eq!(
            next_member(&replaced, "ship_core", Some(("x4:ship:1", "ship_crew")), 1),
            Ok("ship_loadout:g0".into())
        );
        assert_eq!(
            next_member(
                &replaced,
                "ship_core",
                Some(("x4:ship:1", "ship_loadout")),
                1
            ),
            Ok("ship_cargo:g1".into())
        );
        assert_eq!(
            next_member(&replaced, "ship_core", Some(("x4:ship:3", "ship_cargo")), 1),
            Ok("ship_cargo:g2".into())
        );
    }
}
