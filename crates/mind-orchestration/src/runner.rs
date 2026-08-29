use crate::{DegradedDeliberation, EvidenceClass, ProviderRequest, ShadowProvider};
use mind_domain::{AdmissionDecision, DeliberationScheduler, MindAggregate, admit};
use strategic_state::Faction;

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

pub struct RunContext<'a> {
    pub current_snapshot_identity: &'a str,
    pub scheduler: &'a mut DeliberationScheduler,
    pub faction: Faction,
}

impl DeliberationRunner {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "context owns an exclusive scheduler borrow"
    )]
    pub fn run<P>(
        &mut self,
        provider: &mut P,
        request: &ProviderRequest,
        prior: &MindAggregate,
        context: RunContext<'_>,
    ) -> RunnerOutcome
    where
        P: ShadowProvider,
    {
        if request.request().snapshot_identity() != context.current_snapshot_identity
            || request.request().faction() != prior.faction()
        {
            let outcome = RunnerOutcome::Admitted {
                admission: AdmissionDecision::Rejected(
                    mind_domain::AdmissionRejection::CurrentState,
                ),
                evidence: provider.evidence(),
            };
            context.scheduler.complete(context.faction);
            return outcome;
        }
        let evidence = provider.evidence();
        let outcome = match provider.propose(request) {
            Ok(bytes) => RunnerOutcome::Admitted {
                admission: admit(
                    request.request(),
                    prior,
                    context.current_snapshot_identity,
                    &bytes,
                ),
                evidence,
            },
            Err(failure) => RunnerOutcome::Degraded(DegradedDeliberation::from_failure(
                request, evidence, failure,
            )),
        };
        match outcome {
            RunnerOutcome::Admitted { .. } => context.scheduler.complete(context.faction),
            RunnerOutcome::Degraded(_) => {
                let _ = context
                    .scheduler
                    .timeout(context.faction, request.observation_identity());
            }
        }
        outcome
    }

    #[must_use]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "context owns an exclusive scheduler borrow"
    )]
    pub fn run_cached(
        &mut self,
        request: &ProviderRequest,
        prior: &MindAggregate,
        candidate: &[u8],
        evidence: EvidenceClass,
        context: RunContext<'_>,
    ) -> RunnerOutcome {
        if request.request().snapshot_identity() != context.current_snapshot_identity
            || request.request().faction() != prior.faction()
        {
            context.scheduler.complete(context.faction);
            return RunnerOutcome::Admitted {
                admission: AdmissionDecision::Rejected(
                    mind_domain::AdmissionRejection::CurrentState,
                ),
                evidence,
            };
        }
        let outcome = RunnerOutcome::Admitted {
            admission: admit(
                request.request(),
                prior,
                context.current_snapshot_identity,
                candidate,
            ),
            evidence,
        };
        context.scheduler.complete(context.faction);
        outcome
    }
}
