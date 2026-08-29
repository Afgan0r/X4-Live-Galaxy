use crate::process::BenchmarkProcess;
use mind_orchestration::{EvidenceClass, ProviderFailure, ProviderRequest, ShadowProvider};

#[derive(Debug)]
pub struct SubscriptionAdapter<P> {
    process: Option<P>,
}

impl<P> SubscriptionAdapter<P> {
    #[must_use]
    pub const fn unavailable() -> Self {
        Self { process: None }
    }

    #[must_use]
    pub const fn for_explicit_benchmark(process: P) -> Self {
        Self {
            process: Some(process),
        }
    }

    pub const fn preflight(&self) -> Result<(), ProviderFailure> {
        if self.process.is_some() {
            Ok(())
        } else {
            Err(ProviderFailure::Unavailable)
        }
    }
}

impl<P: BenchmarkProcess> SubscriptionAdapter<P> {
    pub fn explicit_benchmark(
        &mut self,
        request: &ProviderRequest,
    ) -> Result<Vec<u8>, ProviderFailure> {
        let Some(process) = self.process.as_mut() else {
            return Err(ProviderFailure::Unavailable);
        };
        process.invoke(request.identity())
    }
}

impl<P: BenchmarkProcess> ShadowProvider for SubscriptionAdapter<P> {
    fn propose(&mut self, request: &ProviderRequest) -> Result<Vec<u8>, ProviderFailure> {
        self.explicit_benchmark(request)
    }

    fn evidence(&self) -> EvidenceClass {
        EvidenceClass::ManualHarness
    }
}
