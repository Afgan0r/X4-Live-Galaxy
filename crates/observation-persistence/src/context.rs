use std::collections::BTreeMap;

use observation_domain::{
    CanonicalizationVersion, CaptureWindow, DigestAlgorithmVersion, ObservationPolicyVersion,
    ObservationSchemaVersion, SectionKey, SectionRevisionId, SectionState,
};
use observation_ingest::{CandidateContext, ContractVersions};

use crate::context_codec::{
    availability_code, coverage_code, freshness_code, next_u64, parse_availability,
    parse_completion_counts, parse_coverage, parse_freshness, parse_quality, quality_code,
};

const FORMAT_VERSION: u64 = 2;

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedContext {
    format_version: u64,
    versions: ContractVersions,
    capture_window: CaptureWindow,
    state: SectionState,
    stable_identity: bool,
    batch_count: Option<usize>,
    raw_bytes: Option<usize>,
}

impl PersistedContext {
    pub const fn from_candidate(
        context: &CandidateContext,
        batch_count: usize,
        raw_bytes: usize,
    ) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            versions: context.versions(),
            capture_window: context.capture_window(),
            state: context.state(),
            stable_identity: context.stable_identity(),
            batch_count: Some(batch_count),
            raw_bytes: Some(raw_bytes),
        }
    }

    #[must_use]
    pub const fn completion_counts(&self) -> Option<(usize, usize)> {
        match (self.batch_count, self.raw_bytes) {
            (Some(batches), Some(bytes)) => Some((batches, bytes)),
            _ => None,
        }
    }

    pub const fn candidate(
        &self,
        dependencies: BTreeMap<SectionKey, SectionRevisionId>,
        expected_current: Option<SectionRevisionId>,
    ) -> CandidateContext {
        CandidateContext::new(
            self.versions,
            self.capture_window,
            self.state,
            dependencies,
            expected_current,
            self.stable_identity,
        )
    }

    #[must_use]
    pub fn canonical_payload(&self) -> String {
        if self.format_version == 1 {
            return format!(
                "1|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|0",
                self.versions.schema().get(),
                self.versions.policy().get(),
                self.versions.canonicalization().get(),
                self.versions.digest().get(),
                self.capture_window.start_millis(),
                self.capture_window.end_millis(),
                self.state.capture_window().start_millis(),
                self.state.capture_window().end_millis(),
                freshness_code(self.state.freshness()),
                quality_code(self.state.quality()),
                availability_code(self.state.availability()),
                coverage_code(self.state.coverage()),
                u8::from(self.stable_identity),
            );
        }
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            FORMAT_VERSION,
            self.versions.schema().get(),
            self.versions.policy().get(),
            self.versions.canonicalization().get(),
            self.versions.digest().get(),
            self.capture_window.start_millis(),
            self.capture_window.end_millis(),
            self.state.capture_window().start_millis(),
            self.state.capture_window().end_millis(),
            freshness_code(self.state.freshness()),
            quality_code(self.state.quality()),
            availability_code(self.state.availability()),
            coverage_code(self.state.coverage()),
            u8::from(self.stable_identity),
            self.batch_count.unwrap_or(0),
            self.raw_bytes.unwrap_or(0)
        )
    }

    #[must_use]
    pub fn parse(payload: &str) -> Option<Self> {
        let mut fields = payload.split('|');
        let format = next_u64(&mut fields)?;
        let schema = ObservationSchemaVersion::new(next_u64(&mut fields)?)?;
        let policy = ObservationPolicyVersion::new(next_u64(&mut fields)?)?;
        let canonicalization = CanonicalizationVersion::new(next_u64(&mut fields)?)?;
        let digest = DigestAlgorithmVersion::new(next_u64(&mut fields)?)?;
        let capture_window = CaptureWindow::new(next_u64(&mut fields)?, next_u64(&mut fields)?)?;
        let state_window = CaptureWindow::new(next_u64(&mut fields)?, next_u64(&mut fields)?)?;
        let freshness = parse_freshness(next_u64(&mut fields)?)?;
        let quality = parse_quality(next_u64(&mut fields)?)?;
        let availability = parse_availability(next_u64(&mut fields)?)?;
        let coverage = parse_coverage(next_u64(&mut fields)?)?;
        let stable_identity = match next_u64(&mut fields)? {
            0 => false,
            1 => true,
            _ => return None,
        };
        let (batch_count, raw_bytes) = parse_completion_counts(format, &mut fields)?;
        Some(Self {
            format_version: format,
            versions: ContractVersions::new(schema, policy, canonicalization, digest),
            capture_window,
            state: SectionState::with_evidence(
                state_window,
                freshness,
                quality,
                availability,
                coverage,
            ),
            stable_identity,
            batch_count,
            raw_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PersistedContext;

    #[test]
    fn legacy_context_round_trips_without_changing_integrity_material() {
        let payload = "1|1|2|3|1|10|20|10|20|1|1|1|1|1|0";
        let context = PersistedContext::parse(payload).expect("legacy context parses");
        assert_eq!(context.canonical_payload(), payload);
        assert_eq!(context.completion_counts(), None);
    }
}
