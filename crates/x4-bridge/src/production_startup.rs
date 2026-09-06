use std::ffi::OsString;
use std::path::{Path, PathBuf};

use observation_application::LifecycleLimits;
use observation_ingest::GenerationLimits;
use observation_persistence::PublicationLimits;

use crate::{
    DiagnosticError, OperationalHistory, ProductionError, ProductionObservationSession,
    readback_revision,
};

#[derive(Debug)]
pub enum StartupError {
    Diagnostic(DiagnosticError),
    InvalidLimits,
    MissingArgument,
    Production(ProductionError),
    UnknownArgument,
}

impl std::fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for StartupError {}

pub fn run_production(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<Option<String>, StartupError> {
    let parsed = StartupArguments::parse(arguments)?;
    if parsed.readback {
        let section = parsed.section_key.ok_or(StartupError::MissingArgument)?;
        let revision = parsed
            .section_revision
            .ok_or(StartupError::MissingArgument)?;
        return readback_revision(&parsed.data_dir, &section, revision)
            .map(Some)
            .map_err(StartupError::Diagnostic);
    }
    let limits_file = parsed.limits_file.ok_or(StartupError::MissingArgument)?;
    validate_limits(&limits_file)?;
    let mut history =
        OperationalHistory::open(&parsed.data_dir).map_err(StartupError::Diagnostic)?;
    let _session = ProductionObservationSession::open(
        &parsed.data_dir.join("observations.sqlite3"),
        default_generation_limits()?,
        PublicationLimits::new(4_096, 16 * 1_024 * 1_024).ok_or(StartupError::InvalidLimits)?,
        LifecycleLimits::new(16 * 1_024 * 1_024, 32 * 1_024 * 1_024, 5_000, 8)
            .ok_or(StartupError::InvalidLimits)?,
        64,
    )
    .map_err(StartupError::Production)?;
    let _ = history.record("waiting", "peer-absent");
    Ok(None)
}

struct StartupArguments {
    data_dir: PathBuf,
    limits_file: Option<PathBuf>,
    readback: bool,
    section_key: Option<String>,
    section_revision: Option<u64>,
}

impl StartupArguments {
    fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, StartupError> {
        let mut values = arguments.into_iter();
        let mut parsed = Self {
            data_dir: PathBuf::new(),
            limits_file: None,
            readback: false,
            section_key: None,
            section_revision: None,
        };
        while let Some(argument) = values.next() {
            match argument.to_str().ok_or(StartupError::UnknownArgument)? {
                "--readback" => parsed.readback = true,
                "--data-dir" => parsed.data_dir = PathBuf::from(next(&mut values)?),
                "--limits-file" => parsed.limits_file = Some(PathBuf::from(next(&mut values)?)),
                "--section-key" => parsed.section_key = Some(text(&next(&mut values)?)),
                "--section-revision" => parsed.set_revision(&next(&mut values)?)?,
                _ => return Err(StartupError::UnknownArgument),
            }
        }
        if parsed.data_dir.as_os_str().is_empty() {
            return Err(StartupError::MissingArgument);
        }
        Ok(parsed)
    }

    fn set_revision(&mut self, value: &OsString) -> Result<(), StartupError> {
        self.section_revision = Some(revision(value)?);
        Ok(())
    }
}

fn next(values: &mut impl Iterator<Item = OsString>) -> Result<OsString, StartupError> {
    values.next().ok_or(StartupError::MissingArgument)
}

fn text(value: &OsString) -> String {
    value.to_string_lossy().into()
}

fn revision(value: &OsString) -> Result<u64, StartupError> {
    value
        .to_string_lossy()
        .parse()
        .map_err(|_| StartupError::UnknownArgument)
}

fn validate_limits(path: &Path) -> Result<(), StartupError> {
    let contents = std::fs::read_to_string(path).map_err(|_| StartupError::InvalidLimits)?;
    if contents.is_empty() || contents.len() > 4_096 {
        return Err(StartupError::InvalidLimits);
    }
    for line in contents.lines() {
        let (name, value) = line.split_once('=').ok_or(StartupError::InvalidLimits)?;
        if !matches!(
            name,
            "max_records" | "max_content_bytes" | "max_message_bytes" | "max_pending"
        ) || value.parse::<usize>().ok().is_none_or(|value| value == 0)
        {
            return Err(StartupError::InvalidLimits);
        }
    }
    Ok(())
}

fn default_generation_limits() -> Result<GenerationLimits, StartupError> {
    let candidate = observation_ingest::CandidateLimits::new(
        16 * 1_024 * 1_024,
        32 * 1_024 * 1_024,
        4_096,
        4_096,
        8_192,
        5_000,
        64,
    )
    .ok_or(StartupError::InvalidLimits)?;
    let aggregate = observation_ingest::AggregateLimits::new(
        128,
        64 * 1_024 * 1_024,
        128 * 1_024 * 1_024,
        16_384,
        16_384,
        32_768,
    )
    .ok_or(StartupError::InvalidLimits)?;
    Ok(GenerationLimits::bounded(candidate, aggregate))
}
