use std::path::{Path, PathBuf};

const MAX_FIELD: usize = 64;
const CASES: [(&str, &str, &str); 17] = [
    ("SD-001", "reject-information", "ci"),
    ("SD-002", "accept", "ci"),
    ("SD-003", "reject-decode", "ci"),
    ("SD-004", "reject-semantic", "ci"),
    ("SD-005", "reject-unbounded-request", "ci"),
    ("SD-006", "exact-cache-revalidate", "ci"),
    ("SD-007", "one-outstanding-coalesced", "ci"),
    ("SD-008", "zero-cycle", "ci"),
    ("SD-009", "two-cycle-cap", "ci"),
    ("SD-010-maintain", "accept-shadow-posture", "ci"),
    ("SD-010-de-escalate", "accept-shadow-posture", "ci"),
    ("SD-010-intensify", "accept-shadow-posture", "ci"),
    ("SD-010-coordinate", "accept-visible-threat-posture", "ci"),
    ("SD-010-reject", "reject-external-shadow-posture", "ci"),
    ("SD-011", "pause-reconcile-retained", "ci"),
    ("SD-012", "manual-benchmark", "benchmark"),
    ("SD-013", "reject-safety", "ci"),
];

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: String,
    schema_version: String,
    schema_path: String,
    schema_digest: String,
    cases: Vec<ManifestCase>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestCase {
    id: String,
    fixture_path: String,
    fixture_digest: String,
    expected: String,
    evidence_class: String,
}
#[derive(serde::Deserialize)]
struct Fixture {
    id: String,
    track: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRecord {
    request_identity: String,
    provider_id: String,
    model_id: String,
    redacted: bool,
}
impl EvidenceRecord {
    #[must_use]
    pub fn redacted(request_identity: &str, provider_id: &str, model_id: &str) -> Self {
        Self {
            request_identity: bounded(request_identity),
            provider_id: bounded(provider_id),
            model_id: bounded(model_id),
            redacted: true,
        }
    }
    #[must_use]
    pub const fn is_redacted(&self) -> bool {
        self.redacted
    }
    #[must_use]
    pub fn is_bounded(&self) -> bool {
        self.request_identity.len() <= MAX_FIELD
            && self.provider_id.len() <= MAX_FIELD
            && self.model_id.len() <= MAX_FIELD
    }
    #[must_use]
    pub fn validates_manifest(manifest: &str) -> bool {
        parse_manifest(manifest).is_some()
    }
}

#[must_use]
pub fn validate_corpus(root: &Path) -> bool {
    let Ok(bytes) = std::fs::read(root.join("manifest.json")) else {
        return false;
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return false;
    };
    let Some(manifest) = parse_manifest(text) else {
        return false;
    };
    artifact(root, &manifest.schema_path, &manifest.schema_digest)
        && manifest.cases.iter().all(|case| fixture(root, case))
}

#[expect(
    clippy::result_unit_err,
    reason = "corpus failures are intentionally redacted"
)]
pub fn benchmark_case_ids(root: &Path) -> Result<Vec<String>, ()> {
    let bytes = std::fs::read(root.join("manifest.json")).map_err(|_| ())?;
    let text = std::str::from_utf8(&bytes).map_err(|_| ())?;
    let manifest = parse_manifest(text).ok_or(())?;
    if !validate_corpus(root) {
        return Err(());
    }
    Ok(manifest
        .cases
        .into_iter()
        .filter(|case| case.evidence_class == "benchmark")
        .map(|case| case.id)
        .collect())
}

fn parse_manifest(value: &str) -> Option<Manifest> {
    let manifest = serde_json::from_str::<Manifest>(value).ok()?;
    if manifest.version != "v1"
        || manifest.schema_version != "schema-v1"
        || manifest.cases.len() != CASES.len()
    {
        return None;
    }
    let mut ids: Vec<_> = manifest.cases.iter().map(|case| case.id.as_str()).collect();
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return None;
    }
    let mut actual: Vec<_> = manifest
        .cases
        .iter()
        .map(|case| {
            (
                case.id.as_str(),
                case.expected.as_str(),
                case.evidence_class.as_str(),
            )
        })
        .collect();
    actual.sort_unstable();
    let mut expected = CASES.to_vec();
    expected.sort_unstable();
    (actual == expected).then_some(manifest)
}

fn fixture(root: &Path, case: &ManifestCase) -> bool {
    if !artifact(root, &case.fixture_path, &case.fixture_digest) {
        return false;
    }
    let Ok(bytes) = std::fs::read(root.join(&case.fixture_path)) else {
        return false;
    };
    let Ok(fixture) = serde_json::from_slice::<Fixture>(&bytes) else {
        return false;
    };
    fixture.id == case.id && fixture.track == case.evidence_class
}

fn artifact(root: &Path, relative: &str, digest: &str) -> bool {
    let Some(path) = confined(root, relative) else {
        return false;
    };
    std::fs::read(path).is_ok_and(|bytes| stable_digest(&bytes) == digest)
}

fn confined(root: &Path, relative: &str) -> Option<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path.components().count() == 0
        || path
            .components()
            .any(|item| matches!(item, std::path::Component::ParentDir))
    {
        return None;
    }
    Some(root.join(path))
}

fn stable_digest(bytes: &[u8]) -> String {
    format!(
        "bytes-v1:{:016x}",
        bytes
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |state, byte| (state
                ^ u64::from(*byte))
            .wrapping_mul(0x100_0000_01b3))
    )
}
fn bounded(value: &str) -> String {
    value.chars().take(MAX_FIELD).collect()
}
