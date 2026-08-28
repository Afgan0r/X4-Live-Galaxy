use observation_ingest::ProjectionSnapshot;

use crate::fact::{
    FactAvailability, FactFamily, StrategicFact, ThreatSubject, availability, family,
};
use crate::faction::{FactOwner, Faction, is_own, owner};
use crate::policy::VisibilityPolicy;

const POLICY_VERSION: &str = "visibility-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrategicPacket {
    faction: Faction,
    policy_version: &'static str,
    facts: Vec<StrategicFact>,
}

impl StrategicPacket {
    pub const fn faction(&self) -> Faction {
        self.faction
    }
    #[must_use]
    pub const fn facts(&self) -> &Vec<StrategicFact> {
        &self.facts
    }
    #[must_use]
    pub const fn policy_version(&self) -> &'static str {
        self.policy_version
    }
    pub fn availability(&self, family: FactFamily) -> FactAvailability {
        self.facts
            .iter()
            .find(|fact| fact.family() == family)
            .map_or(FactAvailability::Unknown, StrategicFact::availability)
    }
    #[must_use]
    pub fn has_shared_threat(&self, threat: ThreatSubject) -> bool {
        self.facts
            .iter()
            .any(|fact| fact.family() == FactFamily::Threat && fact.subject() == Some(threat))
    }
    #[must_use]
    pub fn has_observed_threat(&self, threat: ThreatSubject) -> bool {
        self.facts.iter().any(|fact| {
            fact.subject() == Some(threat) && fact.availability() != FactAvailability::Unsupported
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairedPackets {
    zya: StrategicPacket,
    arg: StrategicPacket,
}

impl PairedPackets {
    #[must_use]
    pub const fn policy_version(&self) -> &'static str {
        POLICY_VERSION
    }
    #[must_use]
    pub const fn packet(&self, faction: Faction) -> &StrategicPacket {
        match faction {
            Faction::Zya => &self.zya,
            Faction::Arg => &self.arg,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PacketLimits {
    facts: usize,
    primitives: usize,
}

impl PacketLimits {
    #[must_use]
    pub const fn new(facts: usize, primitives: usize) -> Self {
        Self { facts, primitives }
    }
    #[must_use]
    pub const fn tracer() -> Self {
        Self::new(32, 4)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DerivationError {
    FactLimitExceeded,
    PrimitiveLimitExceeded,
    UnsupportedFact,
}

pub fn derive_packets(
    snapshot: &ProjectionSnapshot,
    limits: PacketLimits,
) -> Result<PairedPackets, DerivationError> {
    derive_with_policy(snapshot, VisibilityPolicy::v1(), limits)
}

pub fn derive_with_policy(
    snapshot: &ProjectionSnapshot,
    policy: VisibilityPolicy,
    limits: PacketLimits,
) -> Result<PairedPackets, DerivationError> {
    if snapshot.observations.len() > limits.facts {
        return Err(DerivationError::FactLimitExceeded);
    }
    if limits.primitives == 0 {
        return Err(DerivationError::PrimitiveLimitExceeded);
    }
    let facts = snapshot
        .observations
        .values()
        .map(|item| {
            let mut parts = item.record.entity_id().as_str().split(':');
            let owner = owner(parts.next()).ok_or(DerivationError::UnsupportedFact)?;
            let family = family(parts.next()).ok_or(DerivationError::UnsupportedFact)?;
            let kind = parts.next().ok_or(DerivationError::UnsupportedFact)?;
            let subject = match kind {
                "XEN" => Some(ThreatSubject::Xen),
                "KHK" => Some(ThreatSubject::Khk),
                _ => None,
            };
            if parts.next().is_some() {
                return Err(DerivationError::UnsupportedFact);
            }
            Ok((
                owner,
                kind == "resource_map",
                StrategicFact::new(family, subject, availability(item.quality)),
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PairedPackets {
        zya: packet(Faction::Zya, policy, &facts),
        arg: packet(Faction::Arg, policy, &facts),
    })
}

fn packet(
    faction: Faction,
    policy: VisibilityPolicy,
    facts: &[(FactOwner, bool, StrategicFact)],
) -> StrategicPacket {
    StrategicPacket {
        faction,
        policy_version: policy.version(),
        facts: facts
            .iter()
            .map(|(owner, static_map, fact)| visible(faction, *owner, *static_map, fact))
            .collect(),
    }
}

const fn visible(
    faction: Faction,
    owner: FactOwner,
    static_map: bool,
    fact: &StrategicFact,
) -> StrategicFact {
    let own = is_own(faction, owner);
    let availability = if matches!(fact.family(), FactFamily::Threat) || own || static_map {
        fact.availability()
    } else {
        FactAvailability::Inaccessible
    };
    fact.with_availability(availability)
}
