use crate::SourceEvidenceRef;
use std::collections::{BTreeMap, BTreeSet};
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionObservationDisposition {
    Included,
    Excluded,
    Unknown,
}
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionOrigin {
    Vanilla,
    Dlc,
    Player,
    Service,
    Modded,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactionObservationError {
    InvalidIdentity,
    InvalidEvidence,
    DuplicateInventory,
    DuplicateCensus,
}
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactionOriginEvidence {
    id: String,
    origin: FactionOrigin,
    independent: Option<bool>,
    mind_candidate: bool,
    source: SourceEvidenceRef,
}
impl FactionOriginEvidence {
    pub fn new(
        id: impl Into<String>,
        origin: FactionOrigin,
        independent: Option<bool>,
        mind_candidate: bool,
        source: impl Into<String>,
    ) -> Result<Self, FactionObservationError> {
        let id = id.into();
        let source =
            SourceEvidenceRef::new(source).ok_or(FactionObservationError::InvalidEvidence)?;
        token(&id)
            .then_some(Self {
                id,
                origin,
                independent,
                mind_candidate,
                source,
            })
            .ok_or(FactionObservationError::InvalidIdentity)
    }
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub const fn origin(&self) -> FactionOrigin {
        self.origin
    }
    #[must_use]
    pub const fn independent(&self) -> Option<bool> {
        self.independent
    }
    #[must_use]
    pub const fn mind_candidate(&self) -> bool {
        self.mind_candidate
    }
    #[must_use]
    pub const fn source(&self) -> &SourceEvidenceRef {
        &self.source
    }
}
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactionCensusEntry {
    id: String,
    origin: Option<FactionOrigin>,
    disposition: FactionObservationDisposition,
    reason: &'static str,
    mind_candidate: bool,
    source: Option<SourceEvidenceRef>,
}
impl FactionCensusEntry {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub const fn origin(&self) -> Option<FactionOrigin> {
        self.origin
    }
    pub const fn disposition(&self) -> FactionObservationDisposition {
        self.disposition
    }
    #[must_use]
    pub const fn reason(&self) -> &'static str {
        self.reason
    }
    #[must_use]
    pub const fn mind_candidate(&self) -> bool {
        self.mind_candidate
    }
    #[must_use]
    pub const fn source_evidence(&self) -> Option<&SourceEvidenceRef> {
        self.source.as_ref()
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactionObservationRoster {
    discovery_revision: u64,
    entries: Vec<FactionCensusEntry>,
}

impl FactionObservationRoster {
    #[must_use]
    pub const fn discovery_revision(&self) -> u64 {
        self.discovery_revision
    }
    pub fn entries(&self) -> &[FactionCensusEntry] {
        &self.entries
    }
    #[must_use]
    pub fn entry(&self, id: &str) -> Option<&FactionCensusEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }
    #[must_use]
    pub fn has_unknown_blocker(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.disposition == FactionObservationDisposition::Unknown)
    }
    pub fn included_ids(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().filter_map(|entry| {
            (entry.disposition == FactionObservationDisposition::Included)
                .then_some(entry.id.as_str())
        })
    }
}

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
        let item = evidence.get(id.as_str()).copied();
        entries.push(classify(id, item));
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

fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b':' | b'-'))
}
