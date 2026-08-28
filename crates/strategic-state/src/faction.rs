#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Faction {
    Zya,
    Arg,
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
