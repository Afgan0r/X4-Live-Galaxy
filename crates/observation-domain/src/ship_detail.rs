use crate::{
    CaptureWindow, ObservationPolicyVersion, SectionRevisionId, ShipIdentity, ShipOwner,
    SourceEvidenceRef,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipDetailDependency {
    pub identity: ShipIdentity,
    pub owner: ShipOwner,
    pub core_revision: SectionRevisionId,
    pub member_revision: SectionRevisionId,
    pub policy: ObservationPolicyVersion,
    pub capture: CaptureWindow,
    pub source: SourceEvidenceRef,
    pub consistency: ShipRecordConsistency,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShipRecordConsistency {
    Consistent,
    PossiblyStale(ShipStaleReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShipStaleReason {
    LocationChanged,
    OwnerChanged,
    Missing,
    CoreChanged,
}

impl ShipRecordConsistency {
    pub fn from_fields(value: &str, reason: &str) -> Result<Self, ShipDetailError> {
        match (value, reason) {
            ("consistent", "none") => Ok(Self::Consistent),
            ("possibly_stale", "location_changed") => {
                Ok(Self::PossiblyStale(ShipStaleReason::LocationChanged))
            }
            ("possibly_stale", "owner_changed") => {
                Ok(Self::PossiblyStale(ShipStaleReason::OwnerChanged))
            }
            ("possibly_stale", "missing") => Ok(Self::PossiblyStale(ShipStaleReason::Missing)),
            ("possibly_stale", "core_changed") => {
                Ok(Self::PossiblyStale(ShipStaleReason::CoreChanged))
            }
            _ => Err(ShipDetailError::InvalidField),
        }
    }

    #[must_use]
    pub const fn fields(self) -> (&'static str, &'static str) {
        match self {
            Self::Consistent => ("consistent", "none"),
            Self::PossiblyStale(ShipStaleReason::LocationChanged) => {
                ("possibly_stale", "location_changed")
            }
            Self::PossiblyStale(ShipStaleReason::OwnerChanged) => {
                ("possibly_stale", "owner_changed")
            }
            Self::PossiblyStale(ShipStaleReason::Missing) => ("possibly_stale", "missing"),
            Self::PossiblyStale(ShipStaleReason::CoreChanged) => ("possibly_stale", "core_changed"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShipDetailError {
    InvalidField,
    InvalidNumber,
    ExceededBound,
    WrongDependency,
}

impl ShipDetailDependency {
    pub fn validate(&self) -> Result<(), ShipDetailError> {
        if self.core_revision != self.member_revision
            || self.policy.get() != 2
            || self.source.as_str() != "x4-9.00-steam-23660954-ship-detail-source-v1"
        {
            return Err(ShipDetailError::WrongDependency);
        }
        Ok(())
    }
}

#[must_use]
pub fn detail_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b':' | b'-'))
}

pub fn detail_number(value: &str) -> Result<u64, ShipDetailError> {
    let number = value
        .parse::<u64>()
        .map_err(|_| ShipDetailError::InvalidNumber)?;
    if number.to_string() != value || number > 9_007_199_254_740_991 {
        return Err(ShipDetailError::InvalidNumber);
    }
    Ok(number)
}

pub fn detail_outcome<T>(
    name: &str,
    values: Vec<T>,
) -> Result<crate::FieldOutcome<Vec<T>>, ShipDetailError> {
    use crate::FieldOutcome;
    match (name, values.is_empty()) {
        ("value", false) => Ok(FieldOutcome::Value(values)),
        ("empty", true) => Ok(FieldOutcome::Empty),
        ("zero", true) => Ok(FieldOutcome::Zero),
        ("absent", true) => Ok(FieldOutcome::Absent),
        ("unknown", true) => Ok(FieldOutcome::Unknown),
        ("inaccessible", true) => Ok(FieldOutcome::Inaccessible),
        ("unsupported", true) => Ok(FieldOutcome::Unsupported),
        ("stale", true) => Ok(FieldOutcome::Stale),
        _ => Err(ShipDetailError::InvalidField),
    }
}

pub const fn outcome_name<T>(outcome: &crate::FieldOutcome<T>) -> &'static str {
    use crate::FieldOutcome;
    match outcome {
        FieldOutcome::Value(_) => "value",
        FieldOutcome::Zero => "zero",
        FieldOutcome::Empty => "empty",
        FieldOutcome::Absent => "absent",
        FieldOutcome::Unknown => "unknown",
        FieldOutcome::Inaccessible => "inaccessible",
        FieldOutcome::Unsupported => "unsupported",
        FieldOutcome::Stale => "stale",
    }
}
