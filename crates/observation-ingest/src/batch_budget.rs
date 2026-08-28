use std::collections::BTreeSet;

use observation_domain::EntityId;

use crate::model::AdmissionError;

use super::batch::{
    MAX_BATCH_BYTES, MAX_BATCH_FRAMES, MAX_BATCH_MARKERS, MAX_BATCH_OBSERVATIONS, MAX_BATCH_SCOPES,
};

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

    pub fn record_frame(&mut self, frame_bytes: usize) -> Result<(), AdmissionError> {
        self.aggregate_bytes = self
            .aggregate_bytes
            .checked_add(frame_bytes)
            .ok_or(AdmissionError::CollectionLimitExceeded)?;
        if self.aggregate_bytes > MAX_BATCH_BYTES {
            return Err(AdmissionError::CollectionLimitExceeded);
        }
        Ok(())
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
