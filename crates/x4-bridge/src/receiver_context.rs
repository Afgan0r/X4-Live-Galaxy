use std::collections::BTreeMap;

use observation_application::{LifecycleContext, ObservationLifecycle};
use observation_domain::CompleteMessage;
use observation_ingest::{CandidateContext, CompletionCurrent, ContractVersions};
use observation_persistence::ObservationRepository;

use crate::ProductionError;

pub fn assemble<R: ObservationRepository>(
    lifecycle: &mut ObservationLifecycle<R>,
    message: &CompleteMessage,
) -> Result<LifecycleContext, ProductionError> {
    match message {
        CompleteMessage::SectionStart(start) => {
            if matches!(
                start.sender_evidence.source_epoch_status,
                observation_domain::SourceEpochStatus::BoundaryUncertain
            ) || matches!(
                start.sender_evidence.source_boundary,
                observation_domain::SourceBoundary::GameLoaded
                    | observation_domain::SourceBoundary::LuaReload
                    | observation_domain::SourceBoundary::TransportReconnect
            ) {
                lifecycle.invalidate_source_scope(&start.source_scope);
            }
            let current = lifecycle
                .current_revision(&start.section_key)
                .map_err(|_| ProductionError::Storage)?
                .map(|value| value.receipt().revision);
            let evidence = &start.sender_evidence;
            Ok(LifecycleContext::Start(CandidateContext::new(
                ContractVersions::new(
                    evidence.schema_version,
                    evidence.policy_version,
                    evidence.canonicalization_version,
                    evidence.digest_version,
                ),
                evidence.section_state.capture_window(),
                evidence.section_state,
                BTreeMap::new(),
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
                BTreeMap::new(),
                current,
            )))
        }
        CompleteMessage::Control(_) => Err(ProductionError::Lifecycle(
            observation_application::LifecycleError::ContextMismatch,
        )),
    }
}
