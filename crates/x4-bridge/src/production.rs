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
    RevisionExhausted,
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
        let message = observation_ingest::decode_complete_message(
            &bytes,
            self.lifecycle.complete_message_limit(),
        )
        .map_err(|_| ProductionError::Lifecycle(LifecycleError::DecodeRejected))?;
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
        Ok(result)
    }

    pub fn invalidate_source_scope(&mut self, scope: &observation_domain::SourceScopeId) {
        self.lifecycle.invalidate_source_scope(scope);
        self.last_received = None;
    }

    pub fn mark_source_scope_uncertain(
        &mut self,
        scope: &observation_domain::SourceScopeId,
        session: observation_domain::SourceSessionIdentity,
    ) {
        self.lifecycle.mark_source_scope_uncertain(scope, session);
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
        required: &[observation_domain::SectionKey],
        now: u64,
        max_age: u64,
    ) -> observation_ingest::DecisionEligibility {
        self.lifecycle.decision_eligibility(required, now, max_age)
    }

    pub fn next_revision(
        &self,
        key: &observation_domain::SectionKey,
    ) -> Result<u64, ProductionError> {
        let current = self
            .lifecycle
            .current_revision(key)
            .map_err(|_| ProductionError::Storage)?;
        current.map_or(Ok(1), |value| {
            let next = value
                .revision()
                .revision
                .get()
                .checked_add(1)
                .ok_or(ProductionError::RevisionExhausted)?;
            (next <= observation_ingest::MAX_DURABLE_SECTION_REVISION)
                .then_some(next)
                .ok_or(ProductionError::RevisionExhausted)
        })
    }

    fn receiver_context(
        &self,
        bytes: &[u8],
    ) -> Result<observation_application::LifecycleContext, ProductionError> {
        let message = observation_ingest::decode_complete_message(
            bytes,
            self.lifecycle.complete_message_limit(),
        )
        .map_err(|_| ProductionError::Lifecycle(LifecycleError::DecodeRejected))?;
        crate::receiver_context::assemble(&self.lifecycle, &message)
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
