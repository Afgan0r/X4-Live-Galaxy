use mind_orchestration::{EvidenceClass, ProviderFailure, ProviderRequest, ShadowProvider};
use std::process::Command;

const MAX_OUTPUT_BYTES: usize = 65_536;
const TIMEOUT_MILLIS: u64 = 30_000;

#[derive(Debug)]
pub struct SubscriptionAdapter {
    available: bool,
}

impl SubscriptionAdapter {
    #[must_use]
    pub const fn unavailable() -> Self {
        Self { available: false }
    }

    pub const fn preflight(&self) -> Result<(), ProviderFailure> {
        if self.available {
            Ok(())
        } else {
            Err(ProviderFailure::Unavailable)
        }
    }

    pub fn explicit_benchmark(&mut self, request: &ProviderRequest) -> Result<Vec<u8>, ProviderFailure> {
        self.preflight()?;
        let output = Command::new("codex")
            .args(["exec", "--json", "--output-schema", "schema.json", request.identity()])
            .output()
            .map_err(|_| ProviderFailure::Unavailable)?;
        if !output.status.success() {
            return Err(ProviderFailure::Transport);
        }
        if output.stdout.len() > MAX_OUTPUT_BYTES || TIMEOUT_MILLIS == 0 {
            return Err(ProviderFailure::Timeout);
        }
        Ok(output.stdout)
    }
}

impl ShadowProvider for SubscriptionAdapter {
    fn propose(&mut self, request: &ProviderRequest) -> Result<Vec<u8>, ProviderFailure> {
        self.explicit_benchmark(request)
    }

    fn evidence(&self) -> EvidenceClass {
        EvidenceClass::ManualHarness
    }
}
