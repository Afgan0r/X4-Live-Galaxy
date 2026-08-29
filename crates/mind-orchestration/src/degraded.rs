use crate::{EvidenceClass, ProviderFailure, ProviderRequest};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DegradedDeliberation {
    request_identity: String,
    provider_id: String,
    model_id: String,
    evidence: EvidenceClass,
    failure: ProviderFailure,
    attempts: u8,
    paused_observation: u64,
}

impl DegradedDeliberation {
    pub(crate) fn from_failure(
        request: &ProviderRequest,
        evidence: EvidenceClass,
        failure: ProviderFailure,
    ) -> Self {
        Self {
            request_identity: request.identity().into(),
            provider_id: request.metadata().provider_id().into(),
            model_id: request.metadata().model_id().into(),
            evidence,
            failure,
            attempts: 1,
            paused_observation: request.observation_identity(),
        }
    }

    #[must_use]
    pub const fn evidence(&self) -> EvidenceClass {
        self.evidence
    }

    #[must_use]
    pub const fn failure(&self) -> ProviderFailure {
        self.failure
    }

    #[must_use]
    pub const fn attempts(&self) -> u8 {
        self.attempts
    }

    #[must_use]
    pub fn request_identity(&self) -> &str {
        &self.request_identity
    }

    #[must_use]
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    #[must_use]
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub const fn reconcile(&self, observation_identity: u64) -> Result<(), ProviderFailure> {
        if observation_identity > self.paused_observation {
            Ok(())
        } else {
            Err(ProviderFailure::Unavailable)
        }
    }
}
