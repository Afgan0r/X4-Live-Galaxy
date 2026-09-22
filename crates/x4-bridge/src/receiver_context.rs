use observation_application::{LifecycleContext, ObservationLifecycle};
use observation_domain::{
    CompleteMessage, SourceBoundary, SourceEpochStatus, SourceSessionIdentity,
};
use observation_ingest::{CandidateContext, CompletionCurrent, ContractVersions};
use observation_persistence::ObservationRepository;

use crate::ProductionError;

pub fn assemble<R: ObservationRepository>(
    lifecycle: &ObservationLifecycle<R>,
    message: &CompleteMessage,
) -> Result<LifecycleContext, ProductionError> {
    match message {
        CompleteMessage::SectionStart(start) => {
            let current = lifecycle
                .current_revision(&start.section_key)
                .map_err(|_| ProductionError::Storage)?
                .map(|value| value.receipt().revision);
            let evidence = &start.sender_evidence;
            let versions =
                if observation_domain::ShipSectionIdentity::parse(start.section_key.as_str())
                    .is_some()
                {
                    crate::receiver_ship::validate_evidence(evidence)?;
                    observation_domain::SenderEvidence::legacy_default()
                } else {
                    evidence.clone()
                };
            Ok(LifecycleContext::Start(CandidateContext::new(
                ContractVersions::new(
                    versions.schema_version,
                    versions.policy_version,
                    versions.canonicalization_version,
                    versions.digest_version,
                ),
                evidence.section_state.capture_window(),
                evidence.section_state,
                crate::receiver_ship_detail::dependencies(lifecycle, start.section_key.as_str())?,
                current,
                evidence.stable_identity,
            )))
        }
        CompleteMessage::ImmutableBatch(_) => Ok(LifecycleContext::Batch),
        CompleteMessage::SectionCompletion(done) => {
            let current = lifecycle
                .current_revision(&done.section_key)
                .map_err(|_| ProductionError::Storage)?
                .map(|value| value.receipt().revision);
            Ok(LifecycleContext::Completion(CompletionCurrent::new(
                crate::receiver_ship_detail::dependencies(lifecycle, done.section_key.as_str())?,
                current,
            )))
        }
        CompleteMessage::Control(_) => Err(ProductionError::Lifecycle(
            observation_application::LifecycleError::ContextMismatch,
        )),
    }
}

pub fn source_boundary<R: ObservationRepository>(
    lifecycle: &ObservationLifecycle<R>,
    message: &CompleteMessage,
) -> Option<(observation_domain::SourceScopeId, SourceSessionIdentity)> {
    let CompleteMessage::SectionStart(start) = message else {
        return None;
    };
    let explicit_boundary = matches!(
        start.sender_evidence.source_epoch_status,
        SourceEpochStatus::BoundaryUncertain
    ) || matches!(
        start.sender_evidence.source_boundary,
        SourceBoundary::GameLoaded | SourceBoundary::LuaReload
    );
    let producer_reincarnated = lifecycle
        .authoritative_source_session(&start.source_scope)
        .is_some_and(|current| current.producer_incarnation() != &start.producer_incarnation);
    (explicit_boundary || producer_reincarnated).then(|| {
        (
            start.source_scope.clone(),
            SourceSessionIdentity::new(start.producer_incarnation.clone(), start.transport_epoch),
        )
    })
}
