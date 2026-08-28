use crate::fact::{FactAvailability, FactFamily, FactReference, ThreatSubject};
use crate::faction::{Capability, FactOwner, Faction};
use crate::packet::StrategicPacket;
use crate::primitive::{
    BilateralPosture, PlanningHorizon, PrimitiveOwner, ShadowPrimitive, ShadowPrimitiveKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionInputs {
    facts: Vec<(FactReference, FactAvailability)>,
    priorities: [Capability; 3],
    primitives: Vec<ShadowPrimitive>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayFingerprint(u64);

pub fn admission_inputs(
    packet: &StrategicPacket,
    primitives: &[ShadowPrimitive],
) -> AdmissionInputs {
    let mut primitives = primitives.to_vec();
    primitives.sort_unstable_by_key(ShadowPrimitive::canonical_key);
    AdmissionInputs {
        facts: packet
            .canonical_facts()
            .into_iter()
            .map(|fact| (fact.reference(), fact.availability()))
            .collect(),
        priorities: packet.profile().priorities(),
        primitives,
    }
}

pub fn replay_fingerprint(
    packet: &StrategicPacket,
    primitives: &[ShadowPrimitive],
) -> ReplayFingerprint {
    let inputs = admission_inputs(packet, primitives);
    let mut bytes = packet.policy_version().as_bytes().to_vec();
    bytes.extend_from_slice(packet.profile_version().as_bytes());
    bytes.push(faction(packet.faction()));
    bytes.push(snapshot(packet.visible_snapshot_id()));
    for (reference, fact_availability) in inputs.facts {
        bytes.extend_from_slice(&reference_bytes(reference));
        bytes.push(availability(fact_availability));
    }
    for priority in inputs.priorities {
        bytes.push(capability(priority));
    }
    for primitive in inputs.primitives {
        bytes.extend_from_slice(&primitive_bytes(&primitive));
    }
    ReplayFingerprint(fnv1a(&bytes))
}

fn primitive_bytes(primitive: &ShadowPrimitive) -> Vec<u8> {
    let mut bytes = vec![
        kind(primitive.kind()),
        owner(primitive.owner()),
        primitive.priority(),
        horizon(primitive.horizon()),
        posture(primitive.posture()),
    ];
    for reference in primitive.evidence() {
        bytes.extend_from_slice(&reference_bytes(*reference));
    }
    bytes
}

const fn reference_bytes(reference: FactReference) -> [u8; 3] {
    [
        fact_owner(reference.owner()),
        family(reference.family()),
        match reference.subject() {
            Some(value) => subject(value),
            None => 0,
        },
    ]
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

const fn faction(value: Faction) -> u8 {
    match value {
        Faction::Zya => 1,
        Faction::Arg => 2,
    }
}
const fn snapshot(value: crate::VisibleSnapshotId) -> u8 {
    value.value()
}
const fn fact_owner(value: FactOwner) -> u8 {
    match value {
        FactOwner::Zya => 1,
        FactOwner::Arg => 2,
        FactOwner::Xen => 3,
        FactOwner::Khk => 4,
    }
}
const fn family(value: FactFamily) -> u8 {
    match value {
        FactFamily::Economic => 1,
        FactFamily::Military => 2,
        FactFamily::Territorial => 3,
        FactFamily::Threat => 4,
    }
}
const fn subject(value: ThreatSubject) -> u8 {
    match value {
        ThreatSubject::Xen => 1,
        ThreatSubject::Khk => 2,
    }
}
const fn availability(value: FactAvailability) -> u8 {
    match value {
        FactAvailability::Available => 1,
        FactAvailability::Unknown => 2,
        FactAvailability::Stale => 3,
        FactAvailability::Inaccessible => 4,
        FactAvailability::Unsupported => 5,
    }
}
const fn capability(value: Capability) -> u8 {
    match value {
        Capability::DefenseAndMilitaryStrategy => 1,
        Capability::EconomyAndLogistics => 2,
        Capability::TerritorialDevelopmentAndInfrastructure => 3,
    }
}
const fn kind(value: ShadowPrimitiveKind) -> u8 {
    match value {
        ShadowPrimitiveKind::DefensiveReadiness => 1,
        ShadowPrimitiveKind::LogisticsAllocationPriority => 2,
        ShadowPrimitiveKind::TerritorialDevelopmentPriority => 3,
        ShadowPrimitiveKind::BilateralPostureDisposition => 4,
    }
}
const fn owner(value: PrimitiveOwner) -> u8 {
    match value {
        PrimitiveOwner::Defense => 1,
        PrimitiveOwner::Economy => 2,
        PrimitiveOwner::Territorial => 3,
        PrimitiveOwner::Executive => 4,
    }
}
const fn horizon(value: PlanningHorizon) -> u8 {
    match value {
        PlanningHorizon::Immediate => 1,
        PlanningHorizon::NearTerm => 2,
        PlanningHorizon::Sustained => 3,
    }
}
const fn posture(value: Option<BilateralPosture>) -> u8 {
    match value {
        None => 0,
        Some(BilateralPosture::PreserveRelations) => 1,
        Some(BilateralPosture::Deescalate) => 2,
        Some(BilateralPosture::IncreasePressure) => 3,
        Some(BilateralPosture::SeekLimitedCoordination) => 4,
    }
}
