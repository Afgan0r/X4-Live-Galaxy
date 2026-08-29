const MAX_BOUND: usize = 1_048_576;

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

    pub(crate) fn canonical_values(&self) -> [String; 9] {
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
