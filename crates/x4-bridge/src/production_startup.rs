use std::ffi::OsString;
use std::path::PathBuf;

use observation_application::LifecycleLimits;
use observation_ingest::GenerationLimits;
use observation_persistence::PublicationLimits;

use crate::{
    DiagnosticError, OperationalHistory, ProductionError, ProductionLimits,
    ProductionObservationSession, readback_revision,
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
    let limits = ProductionLimits::read(&limits_file).ok_or(StartupError::InvalidLimits)?;
    let mut history =
        OperationalHistory::open(&parsed.data_dir).map_err(StartupError::Diagnostic)?;
    let mut session = ProductionObservationSession::open(
        &parsed.data_dir.join("observations.sqlite3"),
        generation_limits(&limits)?,
        PublicationLimits::new(
            limits.max_publication_records,
            limits.max_publication_content_bytes,
        )
        .ok_or(StartupError::InvalidLimits)?,
        LifecycleLimits::new(
            limits.max_pending_bytes,
            limits.max_total_bytes,
            u64::try_from(limits.max_lifecycle_work).map_err(|_| StartupError::InvalidLimits)?,
            limits.max_delivery_attempts,
        )
        .ok_or(StartupError::InvalidLimits)?,
        limits.max_blockers,
    )
    .map_err(StartupError::Production)?;
    crate::production_runtime::run(&limits, &mut history, &mut session)
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

fn generation_limits(limits: &ProductionLimits) -> Result<GenerationLimits, StartupError> {
    let candidate = observation_ingest::CandidateLimits::new(
        limits.max_candidate_raw_bytes,
        limits.max_candidate_records,
        limits.max_candidate_batches,
        limits.max_candidate_work,
        u64::try_from(limits.max_message_age_millis).map_err(|_| StartupError::InvalidLimits)?,
        u64::try_from(limits.max_message_inactivity_millis)
            .map_err(|_| StartupError::InvalidLimits)?,
    )
    .ok_or(StartupError::InvalidLimits)?;
    let aggregate = observation_ingest::AggregateLimits::new(
        limits.max_candidates,
        limits.max_aggregate_bytes,
        limits.max_aggregate_records,
        limits.max_aggregate_batches,
        limits.max_aggregate_work,
    )
    .ok_or(StartupError::InvalidLimits)?;
    Ok(GenerationLimits::bounded(candidate, aggregate))
}
