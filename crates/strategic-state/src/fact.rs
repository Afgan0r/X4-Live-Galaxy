use observation_domain::SectionQuality;

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactFamily {
    Economic,
    Military,
    Territorial,
    Threat,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThreatSubject {
    Xen,
    Khk,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    family: FactFamily,
    subject: Option<ThreatSubject>,
    availability: FactAvailability,
}

impl StrategicFact {
    pub(crate) const fn new(
        family: FactFamily,
        subject: Option<ThreatSubject>,
        availability: FactAvailability,
    ) -> Self {
        Self {
            family,
            subject,
            availability,
        }
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
        Self::new(self.family, self.subject, availability)
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
