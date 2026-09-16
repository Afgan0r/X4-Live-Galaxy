pub(super) use crate::producer_identity::{bootstrap_bytes, identity};

use crate::producer_message::SectionMessages;
use crate::producer_types::{PreparedRecord, ProducerProfile};
use crate::{ProducerError, ProducerLimits, ProducerSource, ProducerState, SectionEvidence};

pub(super) struct Pending {
    pub(super) bytes: Vec<u8>,
    pub(super) id: String,
    pub(super) handed_off: bool,
    pub(super) attempts: u8,
    pub(super) first_attempt_at: u64,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Readiness {
    Awaiting,
    Handshake,
    Intent,
    Ready,
}

pub struct Producer {
    pub(super) policy: crate::ProducerAdmissionPolicy,
    pub(super) raw_bytes: usize,
    pub(super) last_progress_at: u64,
    pub(super) section_started_at: Option<u64>,
    pub(super) limits: ProducerLimits,
    pub(super) source: ProducerSource,
    pub(super) state: ProducerState,
    pub(super) readiness: Readiness,
    pub(super) revision: u64,
    pub(super) evidence: Option<SectionEvidence>,
    pub(super) profile: ProducerProfile,
    pub(super) selected_key: String,
    pub(super) expected_records: usize,
    pub(super) records: Vec<PreparedRecord>,
    pub(super) finished: bool,
    pub(super) messages: Option<SectionMessages>,
    pub(super) next_batch_index: usize,
    pub(super) pending: Option<Pending>,
    pub(super) connection_generation: u64,
    pub(super) reconciliation: Option<crate::producer_recovery::CompletionOutcome>,
}

impl Producer {
    pub fn new(
        limits: ProducerLimits,
        source: ProducerSource,
        now_millis: u64,
    ) -> Result<Self, ProducerError> {
        if !limits.valid() {
            return Err(ProducerError::InvalidInput);
        }
        let bytes = bootstrap_bytes(&source, limits.control_message_bytes)?;
        Ok(Self {
            policy: crate::ProducerAdmissionPolicy {
                max_inner_records: limits.max_records,
                max_attempts: 2,
                max_age_millis: 5000,
            },
            raw_bytes: 0,
            last_progress_at: now_millis,
            section_started_at: None,
            limits,
            source,
            state: ProducerState::AwaitingCompatibility,
            readiness: Readiness::Awaiting,
            revision: 1,
            evidence: None,
            profile: ProducerProfile::Clock,
            selected_key: "carrier_b_realtime_sample".to_owned(),
            expected_records: 0,
            records: Vec::new(),
            finished: false,
            messages: None,
            next_batch_index: 0,
            pending: Some(Pending::new(bytes, "bootstrap", now_millis)),
            connection_generation: 0,
            reconciliation: None,
        })
    }

    #[must_use]
    pub const fn state(&self) -> ProducerState {
        self.state
    }

    pub(crate) const fn source(&self) -> &ProducerSource {
        &self.source
    }

    #[must_use]
    pub fn pending_bytes(&self) -> Option<&[u8]> {
        self.pending
            .as_ref()
            .filter(|value| !value.handed_off)
            .map(|value| value.bytes.as_slice())
    }

    #[must_use]
    pub(crate) fn collection_admitted(&self) -> bool {
        matches!(
            self.state,
            ProducerState::Ready | ProducerState::SectionReserved | ProducerState::Collecting
        ) && matches!(self.readiness, Readiness::Ready)
            && self.pending.is_none()
            && !self.finished
    }

    pub(crate) fn selection_status(&self) -> (&str, usize) {
        let remaining = self
            .limits
            .max_records
            .saturating_sub(self.next_batch_index + self.records.len());
        if matches!(self.readiness, Readiness::Ready) {
            (&self.selected_key, remaining)
        } else {
            ("none", remaining)
        }
    }

    pub fn mark_local_handoff(&mut self, now_millis: u64) -> Result<(), ProducerError> {
        let pending = self
            .pending
            .as_mut()
            .ok_or(ProducerError::InvalidTransition)?;
        if pending.handed_off {
            return Err(ProducerError::InvalidTransition);
        }
        if pending.attempts == 1 {
            pending.first_attempt_at = now_millis;
        }
        pending.handed_off = true;
        Ok(())
    }

    pub fn fail_section(&mut self) {
        self.discard_incomplete();
        self.state = ProducerState::PausedAfterFailure;
    }

    pub fn close(&mut self) {
        self.discard_incomplete();
        self.state = ProducerState::Closed;
    }

    pub fn reset(&mut self, source: ProducerSource, now_millis: u64) -> Result<(), ProducerError> {
        let mut replacement = Self::new(self.limits, source, now_millis)?;
        replacement.policy = self.policy;
        self.remember_completion();
        replacement.reconciliation = self.reconciliation.take();
        replacement.revision = self.revision;
        *self = replacement;
        Ok(())
    }

    pub(super) fn discard_incomplete(&mut self) {
        self.remember_completion();
        self.evidence = None;
        self.expected_records = 0;
        self.raw_bytes = 0;
        self.section_started_at = None;
        self.records.clear();
        self.finished = false;
        self.messages = None;
        self.next_batch_index = 0;
        self.pending = None;
    }
}

impl Pending {
    pub(super) fn new(bytes: Vec<u8>, id: impl Into<String>, now: u64) -> Self {
        Self {
            bytes,
            id: id.into(),
            handed_off: false,
            attempts: 1,
            first_attempt_at: now,
        }
    }
}
