use observation_ingest::ProjectionSnapshot;

use crate::fact::{
    FactAvailability, FactFamily, FactReference, StrategicFact, ThreatSubject, availability, family,
};
use crate::faction::{FactOwner, Faction, is_own, owner};
use crate::packet::{PairedPackets, StrategicPacket};
use crate::policy::VisibilityPolicy;

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
                StrategicFact::new(
                    FactReference::new(owner, family, subject),
                    family,
                    subject,
                    availability(item.quality),
                ),
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
    StrategicPacket::new(
        faction,
        policy.version(),
        facts
            .iter()
            .map(|(owner, static_map, fact)| visible(faction, *owner, *static_map, fact))
            .collect(),
    )
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
