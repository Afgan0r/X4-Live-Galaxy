use observation_ingest::ProjectionSnapshot;

use crate::fact::{
    FactAvailability, FactFamily, StrategicFact, ThreatSubject, availability, family,
};
use crate::faction::{FactOwner, Faction, is_own, owner};

const POLICY_VERSION: &str = "visibility-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrategicPacket {
    faction: Faction,
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
            let subject = match parts.next() {
                Some("XEN") => Some(ThreatSubject::Xen),
                Some("KHK") => Some(ThreatSubject::Khk),
                Some(_) => None,
                None => return Err(DerivationError::UnsupportedFact),
            };
            if parts.next().is_some() {
                return Err(DerivationError::UnsupportedFact);
            }
            Ok((
                owner,
                StrategicFact::new(family, subject, availability(item.quality)),
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PairedPackets {
        zya: packet(Faction::Zya, &facts),
        arg: packet(Faction::Arg, &facts),
    })
}

fn packet(faction: Faction, facts: &[(FactOwner, StrategicFact)]) -> StrategicPacket {
    StrategicPacket {
        faction,
        facts: facts
            .iter()
            .map(|(owner, fact)| visible(faction, *owner, fact))
            .collect(),
    }
}

const fn visible(faction: Faction, owner: FactOwner, fact: &StrategicFact) -> StrategicFact {
    let own = is_own(faction, owner);
    let availability = if matches!(fact.family(), FactFamily::Threat) || own {
        fact.availability()
    } else {
        FactAvailability::Inaccessible
    };
    fact.with_availability(availability)
}
