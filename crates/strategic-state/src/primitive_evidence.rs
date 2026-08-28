use crate::fact::{FactAvailability, FactFamily, FactReference};
use crate::packet::StrategicPacket;
use crate::primitive::ShadowPrimitiveError;

const MAX_EVIDENCE_REFERENCES: usize = 8;

pub fn family(
    packet: &StrategicPacket,
    family: FactFamily,
) -> Result<Vec<FactReference>, ShadowPrimitiveError> {
    collect(packet, |fact| fact.family() == family)
}

pub fn threat(packet: &StrategicPacket) -> Result<Vec<FactReference>, ShadowPrimitiveError> {
    collect(packet, |fact| fact.family() == FactFamily::Threat)
}

fn collect(
    packet: &StrategicPacket,
    predicate: impl Fn(&crate::StrategicFact) -> bool,
) -> Result<Vec<FactReference>, ShadowPrimitiveError> {
    let mut evidence = packet
        .facts()
        .iter()
        .filter(|fact| predicate(fact) && fact.availability() == FactAvailability::Available)
        .map(crate::StrategicFact::reference)
        .take(MAX_EVIDENCE_REFERENCES + 1)
        .collect::<Vec<_>>();
    if evidence.len() > MAX_EVIDENCE_REFERENCES {
        return Err(ShadowPrimitiveError::EvidenceLimitExceeded);
    }
    evidence.sort_unstable();
    if evidence.is_empty() {
        return Err(ShadowPrimitiveError::UnavailableRequiredFact);
    }
    Ok(evidence)
}
