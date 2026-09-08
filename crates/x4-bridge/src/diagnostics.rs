use std::fmt::Write as _;
use std::path::Path;

use observation_domain::{SectionKey, SectionRevisionId};
use observation_persistence::{
    ObservationRepository, PublicationLimits, SqliteObservationRepository,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticError {
    InvalidIdentity,
    MissingRevision,
    RevisionMismatch,
    Storage,
}

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.code())
    }
}

impl std::error::Error for DiagnosticError {}

impl DiagnosticError {
    const fn code(self) -> &'static str {
        match self {
            Self::InvalidIdentity => "invalid-identity",
            Self::MissingRevision => "missing-revision",
            Self::RevisionMismatch => "revision-mismatch",
            Self::Storage => "diagnostic-storage",
        }
    }
}

pub fn readback_revision(
    data_dir: &Path,
    section_key: &str,
    section_revision: u64,
) -> Result<String, DiagnosticError> {
    let key = SectionKey::new(section_key).ok_or(DiagnosticError::InvalidIdentity)?;
    let expected =
        SectionRevisionId::new(section_revision).ok_or(DiagnosticError::InvalidIdentity)?;
    let database = if data_dir.is_dir() {
        data_dir.join("observations.sqlite3")
    } else {
        data_dir.to_path_buf()
    };
    let limits =
        PublicationLimits::new(4_096, 16 * 1_024 * 1_024).ok_or(DiagnosticError::Storage)?;
    let repository = SqliteObservationRepository::open(&database, limits)
        .map_err(|_| DiagnosticError::Storage)?;
    let current = repository
        .current(&key)
        .map_err(|_| DiagnosticError::Storage)?
        .ok_or(DiagnosticError::MissingRevision)?;
    if current.revision().revision != expected {
        return Err(DiagnosticError::RevisionMismatch);
    }
    Ok(render_current(&current))
}

fn render_current(current: &observation_persistence::CurrentRevision) -> String {
    let revision = current.revision();
    let receipt = current.receipt();
    let mut output = format!(
        "{{\"section_key\":\"{}\",\"section_revision\":{},\"records\":[",
        escape(revision.section_key.as_str()),
        revision.revision.get()
    );
    for (index, record) in revision.records.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        let _ = write!(
            output,
            "{{\"record_id\":\"{}\",\"entity_id\":\"{}\",\"observation_version\":{},\"content\":\"{}\"}}",
            escape(record.record_id.as_str()),
            escape(record.entity_id.as_str()),
            record.observation_version.get(),
            escape(&record.content)
        );
    }
    let _ = write!(
        output,
        "],\"receipt\":{{\"ordinal\":{},\"accepted_at\":{}}}}}",
        receipt.ordinal, receipt.accepted_at
    );
    output
}

pub fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value if value.is_control() => {
                let _ = write!(escaped, "\\u{:04x}", u32::from(value));
            }
            value => escaped.push(value),
        }
    }
    escaped
}
