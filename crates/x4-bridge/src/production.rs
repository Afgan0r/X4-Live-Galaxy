use std::path::Path;

use observation_application::{
    LifecycleError, LifecycleInput, LifecycleLimits, LifecycleResult, ObservationLifecycle,
    PublicationReconciler,
};
use observation_ingest::{
    AcceptedProjection, DecisionRevisionIndex, GenerationLimits, GenerationStager,
};
use observation_persistence::{
    ObservationRepository, PublicationLimits, SqliteObservationRepository,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductionError {
    InvalidLimits,
    Storage,
    Lifecycle(LifecycleError),
}

pub struct ProductionObservationSession<R = SqliteObservationRepository> {
    lifecycle: ObservationLifecycle<R>,
    last_received: Option<(
        observation_domain::BatchId,
        Vec<u8>,
        observation_application::LifecycleContext,
    )>,
}

impl ProductionObservationSession {
    pub fn open(
        path: &Path,
        generation_limits: GenerationLimits,
        publication_limits: PublicationLimits,
        lifecycle_limits: LifecycleLimits,
        blocker_limit: usize,
    ) -> Result<Self, ProductionError> {
        let repository = SqliteObservationRepository::open(path, publication_limits)
            .map_err(|_| ProductionError::Storage)?;
        Self::from_repository(
            repository,
            generation_limits,
            lifecycle_limits,
            blocker_limit,
        )
    }
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
        epoch: observation_domain::TransportEpoch,
        identity: observation_domain::BatchId,
        bytes: Vec<u8>,
        work: usize,
        now: u64,
    ) -> Result<LifecycleResult, ProductionError> {
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
        let result = self
            .lifecycle
            .submit(input)
            .map_err(ProductionError::Lifecycle)?;
        self.last_received = Some((identity, bytes, context));
        Ok(result)
    }

    pub fn invalidate_source_scope(&mut self, scope: &observation_domain::SourceScopeId) {
        self.lifecycle.invalidate_source_scope(scope);
        self.last_received = None;
    }

    fn receiver_context(
        &mut self,
        bytes: &[u8],
    ) -> Result<observation_application::LifecycleContext, ProductionError> {
        let message = observation_ingest::decode_complete_message(
            bytes,
            self.lifecycle.complete_message_limit(),
        )
        .map_err(|_| ProductionError::Lifecycle(LifecycleError::DecodeRejected))?;
        crate::receiver_context::assemble(&mut self.lifecycle, &message)
    }

    #[must_use]
    pub fn restore_current_snapshot(&mut self) -> bool {
        self.lifecycle.restore_current_snapshot()
    }
}

impl<R: ObservationRepository + PublicationReconciler> ProductionObservationSession<R> {
    pub fn reconcile_ambiguous(&mut self, now: u64) -> Result<LifecycleResult, ProductionError> {
        self.lifecycle
            .reconcile_ambiguous(now)
            .map_err(ProductionError::Lifecycle)
    }

    pub fn retry_proven_not_committed(&mut self) -> Result<LifecycleResult, ProductionError> {
        self.lifecycle
            .retry_proven_not_committed()
            .map_err(ProductionError::Lifecycle)
    }
}
