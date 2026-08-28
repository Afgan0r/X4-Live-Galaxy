#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Faction {
    Zya,
    Arg,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    DefenseAndMilitaryStrategy,
    EconomyAndLogistics,
    TerritorialDevelopmentAndInfrastructure,
}

impl Capability {
    pub const ALL: [Self; 3] = [
        Self::DefenseAndMilitaryStrategy,
        Self::EconomyAndLogistics,
        Self::TerritorialDevelopmentAndInfrastructure,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactOwner {
    Zya,
    Arg,
    Xen,
    Khk,
}

pub fn owner(value: Option<&str>) -> Option<FactOwner> {
    match value {
        Some("ZYA") => Some(FactOwner::Zya),
        Some("ARG") => Some(FactOwner::Arg),
        Some("XEN") => Some(FactOwner::Xen),
        Some("KHK") => Some(FactOwner::Khk),
        _ => None,
    }
}

pub const fn is_own(faction: Faction, owner: FactOwner) -> bool {
    matches!(
        (faction, owner),
        (Faction::Zya, FactOwner::Zya) | (Faction::Arg, FactOwner::Arg)
    )
}
