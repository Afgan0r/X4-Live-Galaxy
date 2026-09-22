#[path = "production_open.rs"]
mod open;
#[path = "receiver_faction.rs"]
mod receiver_faction;
#[path = "production_reconciliation.rs"]
mod reconciliation;
#[path = "production_retention.rs"]
mod retention;
#[path = "production_selection.rs"]
mod selection;
#[path = "production_ship_scope.rs"]
mod ship_scope;
#[path = "production_ship_timing.rs"]
mod timing;

use observation_application::{
    LifecycleContext, LifecycleError, LifecycleInput, LifecycleLimits, LifecycleResult,
    ObservationLifecycle,
};
use observation_domain::{
    BatchId, SectionKey, SourceScopeId, SourceSessionIdentity, TransportEpoch,
};
use observation_ingest::{
    AcceptedProjection, DecisionEligibility, DecisionRevisionIndex, GenerationLimits,
    GenerationStager,
};
use observation_persistence::{ObservationRepository, SqliteObservationRepository};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductionError {
    InvalidLimits,
    RevisionExhausted,
    Storage,
    StaleShipParent,
    InvalidFactionCensus,
    Lifecycle(LifecycleError),
}

pub struct ProductionObservationSession<R = SqliteObservationRepository> {
    lifecycle: ObservationLifecycle<R>,
    last_received: Option<(BatchId, Vec<u8>, LifecycleContext)>,
    ship_scope: Option<SourceScopeId>,
    ship_scopes: Vec<SourceScopeId>,
    ship_scope_index: usize,
    heavy_limits: Option<crate::HeavyShipLimits>,
    ship_timing: timing::ShipTiming,
    faction_roster: Option<observation_domain::FactionObservationRoster>,
    faction_inventory: Vec<observation_domain::FactionOriginEvidence>,
}

impl<R: ObservationRepository> ProductionObservationSession<R> {
    pub fn from_repository(
        repository: R,
        generation_limits: GenerationLimits,
        lifecycle_limits: LifecycleLimits,
        blocker_limit: usize,
    ) -> Result<Self, ProductionError> {
        let index =
            DecisionRevisionIndex::new(blocker_limit).ok_or(ProductionError::InvalidLimits)?;
        let mut session = Self {
            lifecycle: ObservationLifecycle::new(
                GenerationStager::new(AcceptedProjection::empty(), generation_limits),
                index,
                repository,
                lifecycle_limits,
            ),
            last_received: None,
            ship_scope: None,
            ship_scopes: Vec::new(),
            ship_scope_index: 0,
            heavy_limits: None,
            ship_timing: timing::ShipTiming::default(),
            faction_roster: None,
            faction_inventory: Vec::new(),
        };
        session
            .restore_current_snapshot()
            .then_some(session)
            .ok_or(ProductionError::Storage)
    }

    pub fn submit(&mut self, input: LifecycleInput) -> Result<LifecycleResult, ProductionError> {
        self.lifecycle
            .submit(input)
            .map_err(ProductionError::Lifecycle)
    }

    pub fn submit_received(
        &mut self,
        epoch: TransportEpoch,
        identity: BatchId,
        bytes: Vec<u8>,
        work: usize,
        now: u64,
    ) -> Result<LifecycleResult, ProductionError> {
        let message = observation_ingest::decode_complete_message(
            &bytes,
            self.lifecycle.complete_message_limit(),
        )
        .map_err(|_| ProductionError::Lifecycle(LifecycleError::DecodeRejected))?;
        crate::receiver_ship::validate(&message, self.ship_scope.as_ref())?;
        self.validate_detail_dependency(&message, now)?;
        if crate::receiver_ship_replay::is_committed(&self.lifecycle, &message)? {
            return Ok(LifecycleResult::Disposition(
                observation_ingest::ReceiverDisposition::Committed,
            ));
        }
        let context = if let Some((prior_identity, prior_bytes, prior_context)) =
            &self.last_received
            && prior_identity == &identity
            && prior_bytes == &bytes
        {
            prior_context.clone()
        } else {
            self.receiver_context(&bytes)?
        };
        let input = LifecycleInput::new(
            epoch,
            identity.clone(),
            bytes.clone(),
            work,
            now,
            context.clone(),
        );
        let result = if let Some((scope, source_session)) =
            crate::receiver_context::source_boundary(&self.lifecycle, &message)
        {
            self.lifecycle
                .submit_source_boundary(input, scope, source_session)
        } else {
            self.lifecycle.submit(input)
        }
        .map_err(ProductionError::Lifecycle)?;
        self.last_received = Some((identity, bytes, context));
        if matches!(
            result,
            LifecycleResult::Disposition(observation_ingest::ReceiverDisposition::Committed)
        ) && matches!(&message, observation_domain::CompleteMessage::SectionCompletion(value) if value.section_key.as_str() == "faction_census")
        {
            self.accept_committed_faction_census()?;
        }
        Ok(result)
    }

    pub fn invalidate_source_scope(&mut self, scope: &SourceScopeId) {
        self.lifecycle.invalidate_source_scope(scope);
        self.ship_timing.clear();
        self.last_received = None;
    }

    pub fn mark_source_scope_uncertain(
        &mut self,
        scope: &SourceScopeId,
        session: SourceSessionIdentity,
    ) {
        self.lifecycle.mark_source_scope_uncertain(scope, session);
        self.ship_timing.clear();
        self.last_received = None;
    }

    pub fn expire_candidates(&mut self, now: u64) -> usize {
        let expired = self.lifecycle.expire_candidates(now);
        if expired > 0 {
            self.last_received = None;
        }
        expired
    }

    pub fn decision_eligibility(
        &self,
        required: &[SectionKey],
        now: u64,
        max_age: u64,
    ) -> DecisionEligibility {
        self.lifecycle.decision_eligibility(required, now, max_age)
    }

    fn receiver_context(&self, bytes: &[u8]) -> Result<LifecycleContext, ProductionError> {
        let message = observation_ingest::decode_complete_message(
            bytes,
            self.lifecycle.complete_message_limit(),
        )
        .map_err(|_| ProductionError::Lifecycle(LifecycleError::DecodeRejected))?;
        crate::receiver_context::assemble(&self.lifecycle, &message)
    }
}
