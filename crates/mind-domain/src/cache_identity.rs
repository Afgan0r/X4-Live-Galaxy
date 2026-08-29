use crate::DeliberationRequest;

const CACHE_IDENTITY_VERSION: &str = "exact-cache-v1";
const MAX_BOUND: usize = 1_048_576;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactCacheKey(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestBounds {
    queue_depth: usize,
    request_bytes: usize,
    context_bytes: usize,
    output_bytes: usize,
    provider_calls: usize,
    retries: usize,
    timeout_millis: usize,
    retained_history: usize,
    dialogue_cycles: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundsError {
    MissingOrExcessive,
}

impl RequestBounds {
    #[expect(
        clippy::too_many_arguments,
        reason = "one explicit field per admission resource"
    )]
    pub fn new(
        queue_depth: usize,
        request_bytes: usize,
        context_bytes: usize,
        output_bytes: usize,
        provider_calls: usize,
        retries: usize,
        timeout_millis: usize,
        retained_history: usize,
        dialogue_cycles: usize,
    ) -> Result<Self, BoundsError> {
        let values = [
            queue_depth,
            request_bytes,
            context_bytes,
            output_bytes,
            provider_calls,
            retries,
            timeout_millis,
            retained_history,
            dialogue_cycles,
        ];
        if values.iter().any(|value| *value == 0 || *value > MAX_BOUND) {
            return Err(BoundsError::MissingOrExcessive);
        }
        Ok(Self {
            queue_depth,
            request_bytes,
            context_bytes,
            output_bytes,
            provider_calls,
            retries,
            timeout_millis,
            retained_history,
            dialogue_cycles,
        })
    }

    pub fn test_profile() -> Result<Self, BoundsError> {
        Self::new(1, 1024, 1024, 1024, 1, 1, 1_000, 8, 2)
    }

    fn canonical_values(&self) -> [String; 9] {
        [
            self.queue_depth.to_string(),
            self.request_bytes.to_string(),
            self.context_bytes.to_string(),
            self.output_bytes.to_string(),
            self.provider_calls.to_string(),
            self.retries.to_string(),
            self.timeout_millis.to_string(),
            self.retained_history.to_string(),
            self.dialogue_cycles.to_string(),
        ]
    }
}

impl ExactCacheKey {
    #[expect(
        clippy::too_many_arguments,
        reason = "the D-12 tuple is an explicit trust boundary"
    )]
    pub fn from_request<I, J, S>(
        request: &DeliberationRequest,
        bounds: &RequestBounds,
        schema_version: &str,
        provider: &str,
        model: &str,
        generation_settings: I,
        primitive_vocabulary: J,
        compaction_identity: &str,
    ) -> Result<Self, BoundsError>
    where
        I: IntoIterator<Item = S>,
        J: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let faction = match request.packet.faction() {
            strategic_state::Faction::Zya => "ZYA",
            strategic_state::Faction::Arg => "ARG",
        };
        Self::build(
            faction,
            &request.snapshot_identity,
            &request.policy_version,
            &request.prompt_package_hash,
            schema_version,
            provider,
            model,
            generation_settings,
            primitive_vocabulary,
            compaction_identity,
            bounds,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "test-only explicit D-12 mutation helper"
    )]
    pub fn for_test(
        faction: &str,
        snapshot: &str,
        policy: &str,
        prompt: &str,
        schema: &str,
        provider: &str,
        model: &str,
        generation: &str,
        primitives: &str,
        compaction: &str,
    ) -> Result<Self, BoundsError> {
        Self::build(
            faction,
            snapshot,
            policy,
            prompt,
            schema,
            provider,
            model,
            [generation],
            primitives.split(','),
            compaction,
            &RequestBounds::test_profile()?,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "canonical D-12 serialization has fixed components"
    )]
    fn build<I, J, S>(
        faction: &str,
        snapshot: &str,
        policy: &str,
        prompt: &str,
        schema: &str,
        provider: &str,
        model: &str,
        generation: I,
        primitives: J,
        compaction: &str,
        bounds: &RequestBounds,
    ) -> Result<Self, BoundsError>
    where
        I: IntoIterator<Item = S>,
        J: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut generation: Vec<String> = generation
            .into_iter()
            .map(|item| item.as_ref().into())
            .collect();
        let mut primitives: Vec<String> = primitives
            .into_iter()
            .map(|item| item.as_ref().into())
            .collect();
        generation.sort_unstable();
        primitives.sort_unstable();
        if [
            faction, snapshot, policy, prompt, schema, provider, model, compaction,
        ]
        .iter()
        .any(|item| item.is_empty())
            || generation.iter().any(String::is_empty)
            || primitives.iter().any(String::is_empty)
        {
            return Err(BoundsError::MissingOrExcessive);
        }
        let mut value = String::new();
        for item in std::iter::once(CACHE_IDENTITY_VERSION)
            .chain([
                faction, snapshot, policy, prompt, schema, provider, model, compaction,
            ])
            .chain(generation.iter().map(String::as_str))
            .chain(primitives.iter().map(String::as_str))
            .chain(bounds.canonical_values().iter().map(String::as_str))
        {
            frame(&mut value, item);
        }
        Ok(Self(value))
    }
}

fn frame(target: &mut String, value: &str) {
    target.push_str(&value.len().to_string());
    target.push(':');
    target.push_str(value);
}
