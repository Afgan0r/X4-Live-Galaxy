use crate::{SectionEvidence, TypedFact};

pub(super) fn qualified_empty(evidence: &SectionEvidence) -> bool {
    use observation_domain::{SectionCoverage, SourceConsistency};
    evidence.sender.section_state.coverage() == SectionCoverage::KnownEmpty
        && matches!(
            evidence.sender.source_consistency,
            SourceConsistency::Barrier
                | SourceConsistency::VersionedManifest
                | SourceConsistency::EventInterval
        )
}

pub(super) fn invalid_clock(fact: &TypedFact, max_raw_bytes: usize) -> bool {
    fact.raw_value.is_empty()
        || fact.raw_value.len() > max_raw_bytes
        || fact
            .raw_value
            .parse::<f64>()
            .map_or(true, |value| !value.is_finite())
        || fact.entity_id != "x4:runtime:realtime_clock"
        || fact.getter != "GetCurRealTime"
        || fact.semantics != "opaque_runtime_number"
        || fact.observation_version != 1
}
