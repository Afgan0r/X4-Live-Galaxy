use crate::SourceEvidenceRef;
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
    pub(super) id: String,
    pub(super) origin: FactionOrigin,
    pub(super) independent: Option<bool>,
    pub(super) mind_candidate: bool,
    pub(super) source: SourceEvidenceRef,
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
    pub const fn source(&self) -> &SourceEvidenceRef {
        &self.source
    }
}
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactionCensusEntry {
    pub(super) id: String,
    pub(super) origin: Option<FactionOrigin>,
    pub(super) disposition: FactionObservationDisposition,
    pub(super) reason: &'static str,
    pub(super) mind_candidate: bool,
    pub(super) source: Option<SourceEvidenceRef>,
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
    pub(super) discovery_revision: u64,
    pub(super) entries: Vec<FactionCensusEntry>,
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

pub fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b':' | b'-'))
}
