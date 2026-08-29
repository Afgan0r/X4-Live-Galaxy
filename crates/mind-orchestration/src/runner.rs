use crate::{DegradedDeliberation, EvidenceClass, ProviderRequest, ShadowProvider};
use mind_domain::{AdmissionDecision, MindAggregate, admit};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunnerOutcome {
    Admitted {
        admission: AdmissionDecision,
        evidence: EvidenceClass,
    },
    Degraded(DegradedDeliberation),
}

#[derive(Default)]
pub struct DeliberationRunner;

impl DeliberationRunner {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn run<P>(
        &mut self,
        provider: &mut P,
        request: &ProviderRequest,
        prior: &MindAggregate,
    ) -> RunnerOutcome
    where
        P: ShadowProvider,
    {
        let evidence = provider.evidence();
        match provider.propose(request) {
            Ok(bytes) => RunnerOutcome::Admitted {
                admission: admit(request.request(), prior, &bytes),
                evidence,
            },
            Err(failure) => RunnerOutcome::Degraded(DegradedDeliberation::from_failure(
                request, evidence, failure,
            )),
        }
    }
}
