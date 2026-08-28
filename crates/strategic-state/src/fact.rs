use observation_domain::SectionQuality;

use crate::faction::FactOwner;

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FactFamily {
    Economic,
    Military,
    Territorial,
    Threat,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ThreatSubject {
    Xen,
    Khk,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FactAvailability {
    Available,
    Unknown,
    Stale,
    Inaccessible,
    Unsupported,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrategicFact {
    reference: FactReference,
    family: FactFamily,
    subject: Option<ThreatSubject>,
    availability: FactAvailability,
}

impl StrategicFact {
    pub(crate) const fn new(
        reference: FactReference,
        family: FactFamily,
        subject: Option<ThreatSubject>,
        availability: FactAvailability,
    ) -> Self {
        Self {
            reference,
            family,
            subject,
            availability,
        }
    }

    #[must_use]
    pub const fn reference(&self) -> FactReference {
        self.reference
    }

    pub const fn family(&self) -> FactFamily {
        self.family
    }

    #[must_use]
    pub const fn subject(&self) -> Option<ThreatSubject> {
        self.subject
    }

    pub const fn availability(&self) -> FactAvailability {
        self.availability
    }

    pub(crate) const fn with_availability(&self, availability: FactAvailability) -> Self {
        Self::new(self.reference, self.family, self.subject, availability)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FactReference {
    owner: FactOwner,
    family: FactFamily,
    subject: Option<ThreatSubject>,
    source_fingerprint: u64,
}

impl FactReference {
    #[must_use]
    pub const fn new(
        owner: FactOwner,
        family: FactFamily,
        subject: Option<ThreatSubject>,
        source_fingerprint: u64,
    ) -> Self {
        Self {
            owner,
            family,
            subject,
            source_fingerprint,
        }
    }

    #[must_use]
    pub const fn owner(self) -> FactOwner {
        self.owner
    }

    pub const fn family(self) -> FactFamily {
        self.family
    }

    #[must_use]
    pub const fn subject(self) -> Option<ThreatSubject> {
        self.subject
    }
    #[must_use]
    pub const fn source_fingerprint(self) -> u64 {
        self.source_fingerprint
    }
}

pub fn family(value: Option<&str>) -> Option<FactFamily> {
    match value {
        Some("economy") => Some(FactFamily::Economic),
        Some("military") => Some(FactFamily::Military),
        Some("territorial") => Some(FactFamily::Territorial),
        Some("threat") => Some(FactFamily::Threat),
        _ => None,
    }
}

pub const fn availability(quality: SectionQuality) -> FactAvailability {
    match quality {
        SectionQuality::Fresh | SectionQuality::KnownEmpty => FactAvailability::Available,
        SectionQuality::Unknown | SectionQuality::Partial => FactAvailability::Unknown,
        SectionQuality::Stale => FactAvailability::Stale,
        SectionQuality::Unsupported => FactAvailability::Unsupported,
    }
}
