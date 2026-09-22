use std::collections::{BTreeMap, BTreeSet};

use crate::faction_observation::{
    FactionCensusEntry, FactionObservationDisposition, FactionObservationError,
    FactionObservationRoster, FactionOrigin, FactionOriginEvidence, token,
};

pub fn classify_observation_factions<I, S>(
    discovery_revision: u64,
    discovered: I,
    inventory: &[FactionOriginEvidence],
) -> Result<FactionObservationRoster, FactionObservationError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut evidence = BTreeMap::new();
    for item in inventory {
        if evidence.insert(item.id.as_str(), item).is_some() {
            return Err(FactionObservationError::DuplicateInventory);
        }
    }
    let mut seen = BTreeSet::new();
    let mut entries = Vec::new();
    for raw in discovered {
        let id = raw.into();
        if !token(&id) {
            return Err(FactionObservationError::InvalidIdentity);
        }
        if !seen.insert(id.clone()) {
            return Err(FactionObservationError::DuplicateCensus);
        }
        entries.push(classify(id.clone(), evidence.get(id.as_str()).copied()));
    }
    Ok(FactionObservationRoster {
        discovery_revision,
        entries,
    })
}

fn classify(id: String, evidence: Option<&FactionOriginEvidence>) -> FactionCensusEntry {
    let Some(evidence) = evidence else {
        return FactionCensusEntry {
            id,
            origin: None,
            disposition: FactionObservationDisposition::Unknown,
            reason: "origin-unresolved",
            mind_candidate: false,
            source: None,
        };
    };
    let (disposition, reason) = match (evidence.origin, evidence.independent) {
        (FactionOrigin::Vanilla | FactionOrigin::Dlc, Some(true)) => (
            FactionObservationDisposition::Included,
            "independent-first-party",
        ),
        (FactionOrigin::Player | FactionOrigin::Modded, _) => (
            FactionObservationDisposition::Excluded,
            "outside-mandatory-coverage",
        ),
        (_, Some(false)) => (FactionObservationDisposition::Excluded, "not-independent"),
        _ => (
            FactionObservationDisposition::Unknown,
            "independence-unresolved",
        ),
    };
    FactionCensusEntry {
        id,
        origin: Some(evidence.origin),
        disposition,
        reason,
        mind_candidate: disposition == FactionObservationDisposition::Included
            && evidence.mind_candidate,
        source: Some(evidence.source.clone()),
    }
}
