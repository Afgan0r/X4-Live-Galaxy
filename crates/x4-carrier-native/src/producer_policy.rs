use crate::{Producer, ProducerError};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProducerAdmissionPolicy {
    pub max_inner_records: usize,
    pub max_attempts: u8,
    pub max_age_millis: u64,
}
impl Producer {
    pub(crate) const fn intent_limits_match(
        &self,
        value: &observation_ingest::CollectionIntentBody,
    ) -> bool {
        self.policy.max_attempts != 1
            || (value.max_records == self.limits.max_records
                && value.max_raw_bytes == self.limits.max_raw_bytes
                && value.max_work == self.limits.max_work)
    }
    pub(crate) fn observe_progress(&mut self, now: u64) -> Result<(), ProducerError> {
        if now < self.last_progress_at {
            return Err(ProducerError::ClockUnavailable);
        }
        self.last_progress_at = now;
        if self
            .section_started_at
            .is_some_and(|started| now.saturating_sub(started) > self.policy.max_age_millis)
        {
            self.fail_section();
            return Err(ProducerError::DataLimit);
        }
        if matches!(
            self.state,
            crate::ProducerState::SectionReserved | crate::ProducerState::Collecting
        ) {
            self.section_started_at.get_or_insert(now);
        }
        Ok(())
    }
    pub const fn configure_admission(
        &mut self,
        policy: ProducerAdmissionPolicy,
    ) -> Result<(), ProducerError> {
        if policy.max_inner_records == 0
            || policy.max_inner_records > self.limits.max_records
            || policy.max_attempts == 0
            || policy.max_attempts > 2
            || policy.max_age_millis == 0
        {
            return Err(ProducerError::InvalidInput);
        }
        self.policy = policy;
        Ok(())
    }
    pub(crate) const fn inner_limit(&self) -> usize {
        self.policy.max_inner_records
    }
    pub(crate) const fn collection_revision(&self) -> u64 {
        self.revision
    }
}
