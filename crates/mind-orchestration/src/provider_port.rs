use mind_domain::DeliberationRequest;

const MAX_ID: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceClass {
    DeterministicFixture,
    ManualHarness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderFailure {
    Timeout,
    Oversized,
    Transport,
    Stream,
    DrainIncomplete,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderMetadata {
    provider_id: String,
    model_id: String,
}

impl ProviderMetadata {
    pub fn new(provider_id: &str, model_id: &str) -> Result<Self, ProviderFailure> {
        if !valid(provider_id) || !valid(model_id) {
            return Err(ProviderFailure::Unavailable);
        }
        Ok(Self {
            provider_id: provider_id.into(),
            model_id: model_id.into(),
        })
    }

    #[must_use]
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    #[must_use]
    pub fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRequest {
    identity: String,
    observation_identity: u64,
    request: DeliberationRequest,
    metadata: ProviderMetadata,
}

impl ProviderRequest {
    pub fn new(
        identity: &str,
        observation_identity: u64,
        request: DeliberationRequest,
        metadata: ProviderMetadata,
    ) -> Result<Self, ProviderFailure> {
        if !valid(identity) || observation_identity == 0 {
            return Err(ProviderFailure::Unavailable);
        }
        Ok(Self {
            identity: identity.into(),
            observation_identity,
            request,
            metadata,
        })
    }

    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    #[must_use]
    pub const fn observation_identity(&self) -> u64 {
        self.observation_identity
    }

    #[must_use]
    pub const fn request(&self) -> &DeliberationRequest {
        &self.request
    }

    #[must_use]
    pub const fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }
}

pub trait ShadowProvider {
    fn propose(&mut self, request: &ProviderRequest) -> Result<Vec<u8>, ProviderFailure>;
    fn evidence(&self) -> EvidenceClass;
}

const fn valid(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_ID
}
