#[must_use]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Faction {
    Zya,
    Arg,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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
pub struct FactionProfile {
    version: &'static str,
    labels: [&'static str; 3],
    priorities: [Capability; 3],
}

impl FactionProfile {
    pub const ZYA: Self = Self {
        version: "doctrine-v1",
        labels: [
            "ZYA Defense & Military Strategy",
            "ZYA Economy & Logistics",
            "ZYA Territorial Development & Infrastructure",
        ],
        priorities: [
            Capability::DefenseAndMilitaryStrategy,
            Capability::TerritorialDevelopmentAndInfrastructure,
            Capability::EconomyAndLogistics,
        ],
    };

    pub const ARG: Self = Self {
        version: "doctrine-v1",
        labels: [
            "ARG Defense & Military Strategy",
            "ARG Economy & Logistics",
            "ARG Territorial Development & Infrastructure",
        ],
        priorities: [
            Capability::EconomyAndLogistics,
            Capability::DefenseAndMilitaryStrategy,
            Capability::TerritorialDevelopmentAndInfrastructure,
        ],
    };

    #[must_use]
    pub const fn for_faction(faction: Faction) -> Self {
        match faction {
            Faction::Zya => Self::ZYA,
            Faction::Arg => Self::ARG,
        }
    }

    #[must_use]
    pub const fn version(self) -> &'static str {
        self.version
    }

    pub const fn priorities(self) -> [Capability; 3] {
        self.priorities
    }

    #[must_use]
    pub const fn label(self, capability: Capability) -> &'static str {
        match capability {
            Capability::DefenseAndMilitaryStrategy => self.labels[0],
            Capability::EconomyAndLogistics => self.labels[1],
            Capability::TerritorialDevelopmentAndInfrastructure => self.labels[2],
        }
    }

    #[must_use]
    pub const fn is_live_galaxy_product_policy(self) -> bool {
        true
    }

    #[must_use]
    pub const fn labels_are_official_x4_names(self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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
