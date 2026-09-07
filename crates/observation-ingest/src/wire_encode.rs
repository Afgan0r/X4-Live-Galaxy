use observation_domain::{
    CompleteMessage, CompletionCoverage, ControlEnvelope, SectionCoverage, SectionFreshness,
    SectionQuality, SourceBoundary, SourceConsistency, SourceEpochStatus,
};
use serde_json::{Value, json};
use std::fmt::Write;

use crate::EnvelopeDecodeError;

pub fn encode_complete_message(
    message: &CompleteMessage,
    limit: usize,
) -> Result<Vec<u8>, EnvelopeDecodeError> {
    let value = match message {
        CompleteMessage::SectionStart(start) => json!({
            "type": "section_start", "contract_version": 2,
            "source_scope": start.source_scope.as_str(),
            "producer_incarnation": start.producer_incarnation.as_str(),
            "transport_epoch": start.transport_epoch.get(),
            "section_key": start.section_key.as_str(),
            "section_revision": start.section_revision.get(),
            "expected_records": start.expected_records,
            "sender_evidence": evidence(&start.sender_evidence),
        }),
        CompleteMessage::ImmutableBatch(batch) => json!({
            "type": "immutable_batch", "contract_version": 2,
            "source_scope": batch.source_scope.as_str(),
            "producer_incarnation": batch.producer_incarnation.as_str(),
            "transport_epoch": batch.transport_epoch.get(),
            "section_key": batch.section_key.as_str(),
            "section_revision": batch.section_revision.get(),
            "batch_id": batch.batch_id.as_str(),
            "section_ordinal": batch.section_ordinal,
            "records": batch.records.iter().map(|record| json!({
                "record_id": record.record_id.as_str(),
                "entity_id": record.entity_id.as_str(),
                "observation_version": record.observation_version.get(),
                "content": record.content,
            })).collect::<Vec<_>>(),
            "optional_detail": batch.optional_detail,
        }),
        CompleteMessage::SectionCompletion(done) => json!({
            "type": "section_completion", "contract_version": 2,
            "source_scope": done.source_scope.as_str(),
            "producer_incarnation": done.producer_incarnation.as_str(),
            "transport_epoch": done.transport_epoch.get(),
            "section_key": done.section_key.as_str(),
            "section_revision": done.section_revision.get(),
            "batch_count": done.batch_count, "record_count": done.record_count,
            "raw_bytes": done.raw_bytes,
            "ordered_batch_manifest_digest": hex(done.ordered_batch_manifest_digest),
            "canonical_content_digest": hex(done.canonical_content_digest),
            "schema_version": done.schema_version.get(),
            "policy_version": done.policy_version.get(),
            "canonicalization_version": done.canonicalization_version.get(),
            "digest_version": done.digest_version.get(),
            "coverage": completion_coverage(done.coverage),
            "sender_evidence": evidence(&done.sender_evidence),
        }),
        CompleteMessage::Control(control) => json!({
            "type": control_name(*control), "contract_version": 2,
        }),
    };
    let bytes = serde_json::to_vec(&value).map_err(|_| EnvelopeDecodeError::InvalidShape)?;
    if bytes.len() > limit {
        return Err(EnvelopeDecodeError::MessageTooLarge);
    }
    Ok(bytes)
}

fn evidence(value: &observation_domain::SenderEvidence) -> Value {
    let state = value.section_state;
    json!({
        "capture_clock": "game_time_millis",
        "capture_start_millis": state.capture_window().start_millis(),
        "capture_end_millis": state.capture_window().end_millis(),
        "freshness": match state.freshness() { SectionFreshness::Fresh => "fresh", SectionFreshness::Stale => "stale" },
        "quality": quality(state.quality()), "availability": match state.availability() { observation_domain::SectionAvailability::Available => "available", observation_domain::SectionAvailability::Unavailable => "unavailable" },
        "coverage": coverage(state.coverage()),
        "source_epoch": value.source_epoch.as_ref().map(observation_domain::SourceEpochId::as_str),
        "source_epoch_status": epoch_status(value.source_epoch_status),
        "source_boundary": boundary(value.source_boundary),
        "source_consistency": consistency(value.source_consistency),
        "stable_identity": value.stable_identity,
        "schema_version": value.schema_version.get(), "policy_version": value.policy_version.get(),
        "canonicalization_version": value.canonicalization_version.get(), "digest_version": value.digest_version.get(),
    })
}

fn hex(value: [u8; 32]) -> String {
    value
        .iter()
        .fold(String::with_capacity(64), |mut encoded, byte| {
            let _ = write!(encoded, "{byte:02x}");
            encoded
        })
}
const fn control_name(value: ControlEnvelope) -> &'static str {
    match value {
        ControlEnvelope::Handshake => "handshake",
        ControlEnvelope::Demand => "demand",
        ControlEnvelope::Disposition => "disposition",
        ControlEnvelope::CollectionIntent => "collection_intent",
        ControlEnvelope::Health => "health",
        ControlEnvelope::Reset => "reset",
    }
}
const fn quality(value: SectionQuality) -> &'static str {
    match value {
        SectionQuality::Fresh => "fresh",
        SectionQuality::KnownEmpty => "known_empty",
        SectionQuality::Unknown => "unknown",
        SectionQuality::Partial => "partial",
        SectionQuality::Stale => "stale",
        SectionQuality::Unsupported => "unsupported",
    }
}
const fn coverage(value: SectionCoverage) -> &'static str {
    match value {
        SectionCoverage::Complete => "complete",
        SectionCoverage::KnownEmpty => "known_empty",
        SectionCoverage::PointMeasurement => "point_measurement",
        SectionCoverage::Unknown => "unknown",
        SectionCoverage::Partial => "partial",
        SectionCoverage::Unsupported => "unsupported",
    }
}
const fn completion_coverage(value: CompletionCoverage) -> &'static str {
    match value {
        CompletionCoverage::Complete => "complete",
        CompletionCoverage::KnownEmpty => "known_empty",
        CompletionCoverage::PointMeasurement => "point_measurement",
        CompletionCoverage::Unknown => "unknown",
        CompletionCoverage::Partial => "partial",
        CompletionCoverage::Unsupported => "unsupported",
    }
}
const fn epoch_status(value: SourceEpochStatus) -> &'static str {
    match value {
        SourceEpochStatus::Known => "known",
        SourceEpochStatus::Unknown => "unknown",
        SourceEpochStatus::BoundaryUncertain => "boundary_uncertain",
    }
}
const fn boundary(value: SourceBoundary) -> &'static str {
    match value {
        SourceBoundary::RuntimeStart => "runtime_start",
        SourceBoundary::GameLoaded => "game_loaded",
        SourceBoundary::LuaReload => "lua_reload",
        SourceBoundary::TransportReconnect => "transport_reconnect",
        SourceBoundary::Unknown => "unknown",
    }
}
const fn consistency(value: SourceConsistency) -> &'static str {
    match value {
        SourceConsistency::Barrier => "barrier",
        SourceConsistency::VersionedManifest => "versioned_manifest",
        SourceConsistency::EventInterval => "event_interval",
        SourceConsistency::ObservedCountFillOnly => "observed_count_fill_only",
        SourceConsistency::Unknown => "unknown",
    }
}
