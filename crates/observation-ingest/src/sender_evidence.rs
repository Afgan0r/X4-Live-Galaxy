use observation_domain::{
    CanonicalizationVersion, CaptureClock, CaptureWindow, DigestAlgorithmVersion,
    EnvelopeDecodeError, ObservationPolicyVersion, ObservationSchemaVersion, SectionAvailability,
    SectionCoverage, SectionFreshness, SectionQuality, SectionState, SenderEvidence,
    SourceBoundary, SourceConsistency, SourceEpochId, SourceEpochStatus,
};

use crate::wire::RawSenderEvidence;

pub fn decode_raw_sender_evidence(
    raw: RawSenderEvidence,
) -> Result<SenderEvidence, EnvelopeDecodeError> {
    let capture_clock = match raw.capture_clock.as_str() {
        "game_time_millis" => CaptureClock::GameTimeMillis,
        _ => return Err(EnvelopeDecodeError::InvalidShape),
    };
    let capture_window = CaptureWindow::new(raw.capture_start_millis, raw.capture_end_millis)
        .ok_or(EnvelopeDecodeError::InvalidShape)?;
    let freshness = match raw.freshness.as_str() {
        "fresh" => SectionFreshness::Fresh,
        "stale" => SectionFreshness::Stale,
        _ => return Err(EnvelopeDecodeError::InvalidShape),
    };
    let quality = quality(&raw.quality)?;
    let availability = match raw.availability.as_str() {
        "available" => SectionAvailability::Available,
        "unavailable" => SectionAvailability::Unavailable,
        _ => return Err(EnvelopeDecodeError::InvalidShape),
    };
    let coverage = coverage(&raw.coverage)?;
    let source_epoch = match raw.source_epoch {
        Some(value) => Some(SourceEpochId::new(value).ok_or(EnvelopeDecodeError::InvalidIdentity)?),
        None => None,
    };
    Ok(SenderEvidence {
        capture_clock,
        section_state: SectionState::with_evidence(
            capture_window,
            freshness,
            quality,
            availability,
            coverage,
        ),
        source_epoch,
        source_epoch_status: epoch_status(&raw.source_epoch_status)?,
        source_boundary: boundary(&raw.source_boundary)?,
        source_consistency: consistency(&raw.source_consistency)?,
        stable_identity: raw.stable_identity,
        schema_version: version(raw.schema_version)?,
        policy_version: ObservationPolicyVersion::new(raw.policy_version)
            .ok_or(EnvelopeDecodeError::InvalidVersion)?,
        canonicalization_version: CanonicalizationVersion::new(raw.canonicalization_version)
            .ok_or(EnvelopeDecodeError::InvalidVersion)?,
        digest_version: DigestAlgorithmVersion::new(raw.digest_version)
            .ok_or(EnvelopeDecodeError::InvalidVersion)?,
    })
}

pub fn decode_sender_evidence(
    bytes: &[u8],
    limit: usize,
) -> Result<SenderEvidence, EnvelopeDecodeError> {
    match crate::decode_complete_message(bytes, limit)? {
        observation_domain::CompleteMessage::SectionStart(start) => Ok(start.sender_evidence),
        _ => Err(EnvelopeDecodeError::InvalidShape),
    }
}

fn version(value: u64) -> Result<ObservationSchemaVersion, EnvelopeDecodeError> {
    ObservationSchemaVersion::new(value).ok_or(EnvelopeDecodeError::InvalidVersion)
}

fn quality(value: &str) -> Result<SectionQuality, EnvelopeDecodeError> {
    match value {
        "fresh" => Ok(SectionQuality::Fresh),
        "known_empty" => Ok(SectionQuality::KnownEmpty),
        "unknown" => Ok(SectionQuality::Unknown),
        "partial" => Ok(SectionQuality::Partial),
        "stale" => Ok(SectionQuality::Stale),
        "unsupported" => Ok(SectionQuality::Unsupported),
        _ => Err(EnvelopeDecodeError::InvalidShape),
    }
}

fn coverage(value: &str) -> Result<SectionCoverage, EnvelopeDecodeError> {
    match value {
        "complete" => Ok(SectionCoverage::Complete),
        "known_empty" => Ok(SectionCoverage::KnownEmpty),
        "point_measurement" => Ok(SectionCoverage::PointMeasurement),
        "unknown" => Ok(SectionCoverage::Unknown),
        "partial" => Ok(SectionCoverage::Partial),
        "unsupported" => Ok(SectionCoverage::Unsupported),
        _ => Err(EnvelopeDecodeError::InvalidShape),
    }
}

fn epoch_status(value: &str) -> Result<SourceEpochStatus, EnvelopeDecodeError> {
    match value {
        "known" => Ok(SourceEpochStatus::Known),
        "unknown" => Ok(SourceEpochStatus::Unknown),
        "boundary_uncertain" => Ok(SourceEpochStatus::BoundaryUncertain),
        _ => Err(EnvelopeDecodeError::InvalidShape),
    }
}

fn boundary(value: &str) -> Result<SourceBoundary, EnvelopeDecodeError> {
    match value {
        "runtime_start" => Ok(SourceBoundary::RuntimeStart),
        "game_loaded" => Ok(SourceBoundary::GameLoaded),
        "lua_reload" => Ok(SourceBoundary::LuaReload),
        "transport_reconnect" => Ok(SourceBoundary::TransportReconnect),
        "unknown" => Ok(SourceBoundary::Unknown),
        _ => Err(EnvelopeDecodeError::InvalidShape),
    }
}

fn consistency(value: &str) -> Result<SourceConsistency, EnvelopeDecodeError> {
    match value {
        "barrier" => Ok(SourceConsistency::Barrier),
        "versioned_manifest" => Ok(SourceConsistency::VersionedManifest),
        "event_interval" => Ok(SourceConsistency::EventInterval),
        "observed_count_fill_only" => Ok(SourceConsistency::ObservedCountFillOnly),
        "unknown" => Ok(SourceConsistency::Unknown),
        _ => Err(EnvelopeDecodeError::InvalidShape),
    }
}
