#![forbid(unsafe_code)]

use observation_domain::{
    EntityId, ObservationSource, ObservationTime, ObservationVersion, SectionDescriptor,
    SectionQuality,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionError {
    InvalidFixture,
    InvalidEntityId,
    InvalidVersion,
    InvalidQuality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedSnapshot {
    section: SectionDescriptor,
}

impl AcceptedSnapshot {
    pub fn from_tracer_payload(payload: &str) -> Result<Self, AdmissionError> {
        let entity_id = EntityId::new(required_string(payload, "entity_id")?)
            .ok_or(AdmissionError::InvalidEntityId)?;
        let observed_at =
            ObservationTime::from_unix_millis(required_u64(payload, "observed_at_unix_millis")?);
        let version = ObservationVersion::new(required_u64(payload, "version")?)
            .ok_or(AdmissionError::InvalidVersion)?;
        let quality = match required_string(payload, "quality")?.as_str() {
            "fresh" => SectionQuality::Fresh,
            "known_empty" => SectionQuality::KnownEmpty,
            "unknown" => SectionQuality::Unknown,
            "partial" => SectionQuality::Partial,
            "stale" => SectionQuality::Stale,
            "unsupported" => SectionQuality::Unsupported,
            _ => return Err(AdmissionError::InvalidQuality),
        };

        Ok(Self {
            section: SectionDescriptor::new(
                entity_id,
                ObservationSource::x4_runtime(),
                observed_at,
                version,
                quality,
            ),
        })
    }

    pub fn entity_id(&self) -> &EntityId {
        self.section.entity_id()
    }

    pub const fn version(&self) -> ObservationVersion {
        self.section.version()
    }
}

fn required_string(payload: &str, field: &str) -> Result<String, AdmissionError> {
    let marker = format!("\"{field}\"");
    let after_field = payload
        .split_once(&marker)
        .and_then(|(_, suffix)| suffix.split_once(':'))
        .map(|(_, value)| value.trim_start())
        .ok_or(AdmissionError::InvalidFixture)?;
    let quoted = after_field
        .strip_prefix('"')
        .ok_or(AdmissionError::InvalidFixture)?;
    let value = quoted
        .split_once('"')
        .map(|(value, _)| value)
        .ok_or(AdmissionError::InvalidFixture)?;

    if value.contains('\\') {
        return Err(AdmissionError::InvalidFixture);
    }

    Ok(value.to_owned())
}

fn required_u64(payload: &str, field: &str) -> Result<u64, AdmissionError> {
    let marker = format!("\"{field}\"");
    let value = payload
        .split_once(&marker)
        .and_then(|(_, suffix)| suffix.split_once(':'))
        .map(|(_, value)| value.trim_start())
        .ok_or(AdmissionError::InvalidFixture)?;
    let number = value
        .split(|character: char| !character.is_ascii_digit())
        .next()
        .filter(|number| !number.is_empty())
        .ok_or(AdmissionError::InvalidFixture)?;

    number.parse().map_err(|_| AdmissionError::InvalidFixture)
}
