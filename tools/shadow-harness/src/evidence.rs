const MAX_FIELD: usize = 64;

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
        (1..=13).all(|number| {
            let id = format!("SD-{number:03}");
            manifest.contains(&id) && manifest.contains("fixture_hash")
        }) && manifest.contains("\"evidence_class\":\"benchmark\"")
    }
}

fn bounded(value: &str) -> String {
    value.chars().take(MAX_FIELD).collect()
}
