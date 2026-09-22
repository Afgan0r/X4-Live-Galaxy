#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShipSectionKind {
    Core,
    Cargo,
    Crew,
    Loadout,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipSectionIdentity {
    kind: ShipSectionKind,
    faction: Option<String>,
    group: Option<u16>,
}

impl ShipSectionIdentity {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        if value == "ship_core" {
            return Some(Self {
                kind: ShipSectionKind::Core,
                faction: None,
                group: None,
            });
        }
        if let Some(faction) = value.strip_prefix("ship_core:").filter(|v| token(v)) {
            return Some(Self {
                kind: ShipSectionKind::Core,
                faction: Some(faction.to_owned()),
                group: None,
            });
        }
        let (kind, suffix) = detail_family(value)?;
        parse_detail(kind, suffix)
    }

    #[must_use]
    pub fn core(faction: Option<&str>) -> Option<Self> {
        if faction.is_some_and(|value| !token(value)) {
            return None;
        }
        Some(Self {
            kind: ShipSectionKind::Core,
            faction: faction.map(str::to_owned),
            group: None,
        })
    }

    #[must_use]
    pub fn detail(kind: ShipSectionKind, faction: Option<&str>, group: u16) -> Option<Self> {
        if kind == ShipSectionKind::Core || faction.is_some_and(|value| !token(value)) {
            return None;
        }
        Some(Self {
            kind,
            faction: faction.map(str::to_owned),
            group: Some(group),
        })
    }

    pub const fn kind(&self) -> ShipSectionKind {
        self.kind
    }
    #[must_use]
    pub fn faction(&self) -> Option<&str> {
        self.faction.as_deref()
    }
    #[must_use]
    pub const fn group(&self) -> Option<u16> {
        self.group
    }
    #[must_use]
    pub fn key(&self) -> String {
        let family = match self.kind {
            ShipSectionKind::Core => "ship_core",
            ShipSectionKind::Cargo => "ship_cargo",
            ShipSectionKind::Crew => "ship_crew",
            ShipSectionKind::Loadout => "ship_loadout",
        };
        match (&self.faction, self.group) {
            (Some(faction), Some(group)) => format!("{family}:{faction}:g{group}"),
            (None, Some(group)) => format!("{family}:g{group}"),
            (Some(faction), None) => format!("{family}:{faction}"),
            (None, None) => family.to_owned(),
        }
    }
}

fn detail_family(value: &str) -> Option<(ShipSectionKind, &str)> {
    for (prefix, kind) in [
        ("ship_cargo", ShipSectionKind::Cargo),
        ("ship_crew", ShipSectionKind::Crew),
        ("ship_loadout", ShipSectionKind::Loadout),
    ] {
        let suffix = value
            .strip_prefix(prefix)
            .and_then(|value| value.strip_prefix(':'));
        if let Some(suffix) = suffix {
            return Some((kind, suffix));
        }
    }
    None
}

fn parse_detail(kind: ShipSectionKind, suffix: &str) -> Option<ShipSectionIdentity> {
    let (faction, raw_group) = match suffix.rsplit_once(":g") {
        Some((faction, group)) => (Some(faction), group),
        None => (None, suffix.strip_prefix('g')?),
    };
    if faction.is_some_and(|value| !token(value)) {
        return None;
    }
    let group = raw_group.parse::<u16>().ok()?;
    if group.to_string() != raw_group {
        return None;
    }
    Some(ShipSectionIdentity {
        kind,
        faction: faction.map(str::to_owned),
        group: Some(group),
    })
}

fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::{ShipSectionIdentity, ShipSectionKind};

    #[test]
    fn parses_legacy_and_faction_scoped_sections_without_ambiguity() {
        for key in [
            "ship_core",
            "ship_core:xenon",
            "ship_cargo:g0",
            "ship_cargo:xenon:g0",
            "ship_crew:khaak:g12",
            "ship_loadout:scaleplate:g65535",
        ] {
            let parsed = ShipSectionIdentity::parse(key).expect("valid section key");
            assert_eq!(parsed.key(), key);
        }
        assert!(ShipSectionIdentity::parse("ship_core:xenon:g0").is_none());
        assert!(ShipSectionIdentity::parse("ship_cargo:xenon:g01").is_none());
        assert!(ShipSectionIdentity::detail(ShipSectionKind::Core, Some("xenon"), 0).is_none());
    }
}
