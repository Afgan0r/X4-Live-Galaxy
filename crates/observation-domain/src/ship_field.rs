#[must_use]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SourceEvidenceRef(String);

impl SourceEvidenceRef {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (!value.trim().is_empty()).then_some(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldApplicability {
    Applicable,
    NotApplicable(SourceEvidenceRef),
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldOutcome<T> {
    Value(T),
    Zero,
    Empty,
    Absent,
    Unknown,
    Inaccessible,
    Unsupported,
    Stale,
}
