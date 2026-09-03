use observation_domain::EntityId;
use std::collections::BTreeSet;

use crate::candidate_limits::{AggregateLimits, CandidateLimits};
use crate::model::AdmissionError;

use super::batch::{
    MAX_BATCH_BYTES, MAX_BATCH_FRAMES, MAX_BATCH_MARKERS, MAX_BATCH_OBSERVATIONS, MAX_BATCH_SCOPES,
};

#[must_use]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CandidateUsage {
    pub raw_bytes: usize,
    pub decoded_bytes: usize,
    pub records: usize,
    pub batches: usize,
    pub work: usize,
}

impl CandidateUsage {
    pub(crate) fn charged(
        self,
        raw: usize,
        decoded: usize,
        records: usize,
        work: usize,
    ) -> Option<Self> {
        Some(Self {
            raw_bytes: self.raw_bytes.checked_add(raw)?,
            decoded_bytes: self.decoded_bytes.checked_add(decoded)?,
            records: self.records.checked_add(records)?,
            batches: self.batches.checked_add(1)?,
            work: self.work.checked_add(work)?,
        })
    }
    pub(crate) const fn within(self, limits: CandidateLimits) -> bool {
        self.raw_bytes <= limits.raw_bytes.get()
            && self.decoded_bytes <= limits.decoded_bytes.get()
            && self.records <= limits.records.get()
            && self.batches <= limits.batches.get()
            && self.work <= limits.work.get()
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AggregateUsage {
    pub candidate_count: usize,
    pub raw_bytes: usize,
    pub decoded_bytes: usize,
    pub records: usize,
    pub batches: usize,
    pub work: usize,
}

impl AggregateUsage {
    pub(crate) const ZERO: Self = Self {
        candidate_count: 0,
        raw_bytes: 0,
        decoded_bytes: 0,
        records: 0,
        batches: 0,
        work: 0,
    };

    pub(crate) fn add_candidate(self) -> Option<Self> {
        Some(Self {
            candidate_count: self.candidate_count.checked_add(1)?,
            ..self
        })
    }
    pub(crate) fn add(self, charge: CandidateUsage) -> Option<Self> {
        Some(Self {
            candidate_count: self.candidate_count,
            raw_bytes: self.raw_bytes.checked_add(charge.raw_bytes)?,
            decoded_bytes: self.decoded_bytes.checked_add(charge.decoded_bytes)?,
            records: self.records.checked_add(charge.records)?,
            batches: self.batches.checked_add(charge.batches)?,
            work: self.work.checked_add(charge.work)?,
        })
    }
    pub(crate) fn release(self, charge: CandidateUsage) -> Option<Self> {
        Some(Self {
            candidate_count: self.candidate_count.checked_sub(1)?,
            raw_bytes: self.raw_bytes.checked_sub(charge.raw_bytes)?,
            decoded_bytes: self.decoded_bytes.checked_sub(charge.decoded_bytes)?,
            records: self.records.checked_sub(charge.records)?,
            batches: self.batches.checked_sub(charge.batches)?,
            work: self.work.checked_sub(charge.work)?,
        })
    }
    pub(crate) const fn within(self, limits: AggregateLimits) -> bool {
        self.candidate_count <= limits.candidates.get()
            && self.raw_bytes <= limits.raw_bytes.get()
            && self.decoded_bytes <= limits.decoded_bytes.get()
            && self.records <= limits.records.get()
            && self.batches <= limits.batches.get()
            && self.work <= limits.work.get()
    }
}

pub struct BatchBudget {
    aggregate_bytes: usize,
    observations: usize,
    markers: usize,
    scopes: BTreeSet<EntityId>,
}

impl BatchBudget {
    pub const fn new(frame_count: usize) -> Result<Self, AdmissionError> {
        if frame_count > MAX_BATCH_FRAMES {
            return Err(AdmissionError::CollectionLimitExceeded);
        }
        Ok(Self {
            aggregate_bytes: 0,
            observations: 0,
            markers: 0,
            scopes: BTreeSet::new(),
        })
    }
    pub fn record_frame(&mut self, bytes: usize) -> Result<(), AdmissionError> {
        self.aggregate_bytes = self
            .aggregate_bytes
            .checked_add(bytes)
            .ok_or(AdmissionError::CollectionLimitExceeded)?;
        (self.aggregate_bytes <= MAX_BATCH_BYTES)
            .then_some(())
            .ok_or(AdmissionError::CollectionLimitExceeded)
    }
    pub const fn record_observation(&mut self) -> Result<(), AdmissionError> {
        self.observations += 1;
        if self.observations > MAX_BATCH_OBSERVATIONS {
            return Err(AdmissionError::CollectionLimitExceeded);
        }
        Ok(())
    }
    pub const fn record_marker(&mut self) -> Result<(), AdmissionError> {
        if self.markers == MAX_BATCH_MARKERS {
            return Err(AdmissionError::CollectionLimitExceeded);
        }
        self.markers += 1;
        Ok(())
    }
    pub fn register_scope(&mut self, scope: &EntityId) -> Result<(), AdmissionError> {
        if !self.scopes.contains(scope) && self.scopes.len() == MAX_BATCH_SCOPES {
            return Err(AdmissionError::CollectionLimitExceeded);
        }
        self.scopes.insert(scope.clone());
        Ok(())
    }
}
