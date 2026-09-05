use std::collections::BTreeMap;

use observation_domain::{
    CanonicalizationVersion, CaptureWindow, DigestAlgorithmVersion, ObservationPolicyVersion,
    ObservationSchemaVersion, SectionAvailability, SectionCoverage, SectionFreshness, SectionKey,
    SectionQuality, SectionRevisionId, SectionState,
};
use observation_ingest::{CandidateContext, ContractVersions};

const FORMAT_VERSION: u64 = 1;

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedContext {
    versions: ContractVersions,
    capture_window: CaptureWindow,
    state: SectionState,
    stable_identity: bool,
}

impl PersistedContext {
    pub const fn from_candidate(context: &CandidateContext) -> Self {
        Self {
            versions: context.versions(),
            capture_window: context.capture_window(),
            state: context.state(),
            stable_identity: context.stable_identity(),
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
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
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
            0
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
        if format != FORMAT_VERSION || next_u64(&mut fields)? != 0 || fields.next().is_some() {
            return None;
        }
        Some(Self {
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
        })
    }
}

fn next_u64(fields: &mut std::str::Split<'_, char>) -> Option<u64> {
    fields.next()?.parse().ok()
}

const fn freshness_code(value: SectionFreshness) -> u8 {
    match value {
        SectionFreshness::Fresh => 1,
        SectionFreshness::Stale => 2,
    }
}
const fn parse_freshness(value: u64) -> Option<SectionFreshness> {
    match value {
        1 => Some(SectionFreshness::Fresh),
        2 => Some(SectionFreshness::Stale),
        _ => None,
    }
}
const fn availability_code(value: SectionAvailability) -> u8 {
    match value {
        SectionAvailability::Available => 1,
        SectionAvailability::Unavailable => 2,
    }
}
const fn parse_availability(value: u64) -> Option<SectionAvailability> {
    match value {
        1 => Some(SectionAvailability::Available),
        2 => Some(SectionAvailability::Unavailable),
        _ => None,
    }
}
const fn quality_code(value: SectionQuality) -> u8 {
    match value {
        SectionQuality::Fresh => 1,
        SectionQuality::KnownEmpty => 2,
        SectionQuality::Unknown => 3,
        SectionQuality::Partial => 4,
        SectionQuality::Stale => 5,
        SectionQuality::Unsupported => 6,
    }
}
const fn parse_quality(value: u64) -> Option<SectionQuality> {
    match value {
        1 => Some(SectionQuality::Fresh),
        2 => Some(SectionQuality::KnownEmpty),
        3 => Some(SectionQuality::Unknown),
        4 => Some(SectionQuality::Partial),
        5 => Some(SectionQuality::Stale),
        6 => Some(SectionQuality::Unsupported),
        _ => None,
    }
}
const fn coverage_code(value: SectionCoverage) -> u8 {
    match value {
        SectionCoverage::Complete => 1,
        SectionCoverage::KnownEmpty => 2,
        SectionCoverage::Unknown => 3,
        SectionCoverage::Partial => 4,
        SectionCoverage::Unsupported => 5,
    }
}
const fn parse_coverage(value: u64) -> Option<SectionCoverage> {
    match value {
        1 => Some(SectionCoverage::Complete),
        2 => Some(SectionCoverage::KnownEmpty),
        3 => Some(SectionCoverage::Unknown),
        4 => Some(SectionCoverage::Partial),
        5 => Some(SectionCoverage::Unsupported),
        _ => None,
    }
}
