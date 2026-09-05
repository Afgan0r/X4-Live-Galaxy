use crate::GenerationStager;
use observation_domain::SourceScopeId;
use std::num::{NonZeroU64, NonZeroUsize};
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CandidateLimits {
    pub(crate) raw_bytes: NonZeroUsize,
    pub(crate) decoded_bytes: NonZeroUsize,
    pub(crate) records: NonZeroUsize,
    pub(crate) batches: NonZeroUsize,
    pub(crate) work: NonZeroUsize,
    pub(crate) age_millis: NonZeroU64,
    pub(crate) inactivity_millis: NonZeroU64,
}
impl CandidateLimits {
    #[must_use]
    pub fn new(
        raw: usize,
        decoded: usize,
        records: usize,
        batches: usize,
        work: usize,
        age: u64,
        inactivity: u64,
    ) -> Option<Self> {
        Some(Self {
            raw_bytes: NonZeroUsize::new(raw)?,
            decoded_bytes: NonZeroUsize::new(decoded)?,
            records: NonZeroUsize::new(records)?,
            batches: NonZeroUsize::new(batches)?,
            work: NonZeroUsize::new(work)?,
            age_millis: NonZeroU64::new(age)?,
            inactivity_millis: NonZeroU64::new(inactivity)?,
        })
    }
}
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AggregateLimits {
    pub(crate) candidates: NonZeroUsize,
    pub(crate) raw_bytes: NonZeroUsize,
    pub(crate) decoded_bytes: NonZeroUsize,
    pub(crate) records: NonZeroUsize,
    pub(crate) batches: NonZeroUsize,
    pub(crate) work: NonZeroUsize,
}
impl AggregateLimits {
    #[must_use]
    pub fn new(
        candidates: usize,
        raw: usize,
        decoded: usize,
        records: usize,
        batches: usize,
        work: usize,
    ) -> Option<Self> {
        Some(Self {
            candidates: NonZeroUsize::new(candidates)?,
            raw_bytes: NonZeroUsize::new(raw)?,
            decoded_bytes: NonZeroUsize::new(decoded)?,
            records: NonZeroUsize::new(records)?,
            batches: NonZeroUsize::new(batches)?,
            work: NonZeroUsize::new(work)?,
        })
    }
}
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GenerationLimits {
    pub(crate) max_staged_bytes: usize,
    pub(crate) max_work_units: usize,
    pub(crate) candidate: CandidateLimits,
    pub(crate) aggregate: AggregateLimits,
}
impl GenerationLimits {
    pub const fn new(max_staged_bytes: usize, max_work_units: usize) -> Self {
        let raw = nonzero_usize(max_staged_bytes);
        let work = nonzero_usize(max_work_units);
        let candidate = CandidateLimits {
            raw_bytes: raw,
            decoded_bytes: raw,
            records: work,
            batches: work,
            work,
            age_millis: NonZeroU64::MAX,
            inactivity_millis: NonZeroU64::MAX,
        };
        let aggregate = AggregateLimits {
            candidates: work,
            raw_bytes: raw,
            decoded_bytes: raw,
            records: work,
            batches: work,
            work,
        };
        Self {
            max_staged_bytes,
            max_work_units,
            candidate,
            aggregate,
        }
    }

    pub const fn bounded(candidate: CandidateLimits, aggregate: AggregateLimits) -> Self {
        Self {
            max_staged_bytes: candidate.raw_bytes.get(),
            max_work_units: candidate.work.get(),
            candidate,
            aggregate,
        }
    }
}

const fn nonzero_usize(value: usize) -> NonZeroUsize {
    match NonZeroUsize::new(value) {
        Some(value) => value,
        None => NonZeroUsize::MIN,
    }
}
impl GenerationStager {
    pub fn invalidate_source_scope(&mut self, scope: &SourceScopeId) -> usize {
        let keys: Vec<_> = self
            .candidates
            .iter()
            .filter(|(_, candidate)| &candidate.start.source_scope == scope)
            .map(|(key, _)| key.clone())
            .collect();
        for key in &keys {
            self.drop_candidate(key);
        }
        keys.len()
    }

    pub fn expire_candidates(&mut self, now: u64) -> usize {
        let keys: Vec<_> = self
            .candidates
            .iter()
            .filter(|(_, candidate)| {
                now.saturating_sub(candidate.started_at) > self.limits.candidate.age_millis.get()
                    || now.saturating_sub(candidate.last_progress_at)
                        > self.limits.candidate.inactivity_millis.get()
            })
            .map(|(key, _)| key.clone())
            .collect();
        for key in &keys {
            self.drop_candidate(key);
        }
        keys.len()
    }
}
