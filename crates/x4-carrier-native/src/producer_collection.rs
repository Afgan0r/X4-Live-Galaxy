use crate::producer::{Pending, Readiness};
use crate::producer_message::assemble;
use crate::{
    Producer, ProducerError, ProducerOutcome, ProducerState, SectionEvidence,
    SectionFinishEvidence, TypedFact,
};

impl Producer {
    pub fn begin_section(&mut self, evidence: SectionEvidence) -> Result<(), ProducerError> {
        if self.state != ProducerState::Ready || self.readiness != Readiness::Ready {
            return Err(ProducerError::InvalidTransition);
        }
        self.evidence = Some(evidence);
        self.state = ProducerState::SectionReserved;
        Ok(())
    }

    pub fn push_record(&mut self, fact: &TypedFact) -> Result<(), ProducerError> {
        if self.state != ProducerState::SectionReserved || self.fact.is_some() {
            return Err(ProducerError::InvalidTransition);
        }
        if invalid(fact, self.limits.max_raw_bytes) {
            return Err(ProducerError::DataLimit);
        }
        self.fact = Some(fact.clone());
        self.state = ProducerState::Collecting;
        Ok(())
    }

    pub fn finish_section(&mut self, finish: SectionFinishEvidence) -> Result<(), ProducerError> {
        if self.state != ProducerState::Collecting {
            return Err(ProducerError::InvalidTransition);
        }
        let evidence = self
            .evidence
            .as_mut()
            .ok_or(ProducerError::InvalidTransition)?;
        let state = evidence.sender.section_state;
        if !finish.succeeded
            || finish.capture_end_millis < state.capture_window().start_millis()
            || finish.quality != state.quality()
            || finish.availability != state.availability()
            || finish.coverage != state.coverage()
            || finish.consistency != evidence.sender.source_consistency
            || finish.stable_identity != evidence.sender.stable_identity
        {
            self.fail_section();
            return Err(ProducerError::InvalidInput);
        }
        let window = observation_domain::CaptureWindow::new(
            state.capture_window().start_millis(),
            finish.capture_end_millis,
        )
        .ok_or(ProducerError::InvalidInput)?;
        evidence.sender.section_state = observation_domain::SectionState::with_evidence(
            window,
            state.freshness(),
            state.quality(),
            state.availability(),
            state.coverage(),
        );
        self.finished = true;
        Ok(())
    }

    pub fn progress(
        &mut self,
        work_units: usize,
        now_millis: u64,
    ) -> Result<ProducerOutcome, ProducerError> {
        if self.pending.is_some() {
            return Ok(ProducerOutcome::CapacityUnavailable);
        }
        if self.state != ProducerState::Collecting
            || !self.finished
            || work_units == 0
            || work_units > self.limits.max_work
        {
            return Err(ProducerError::InvalidTransition);
        }
        let evidence = self
            .evidence
            .as_ref()
            .ok_or(ProducerError::InvalidTransition)?;
        let fact = self.fact.as_ref().ok_or(ProducerError::InvalidTransition)?;
        let messages = assemble(
            &self.source,
            evidence,
            fact,
            self.revision,
            self.limits.data_message_bytes,
        )?;
        self.pending = Some(Pending::new(
            messages.start.clone(),
            format!("message:start:carrier_b_realtime_sample:{}", self.revision),
            now_millis,
        ));
        self.messages = Some(messages);
        self.state = ProducerState::PendingStart;
        Ok(ProducerOutcome::Progress)
    }
}

fn invalid(fact: &TypedFact, max_raw_bytes: usize) -> bool {
    fact.raw_value.is_empty()
        || fact.raw_value.len() > max_raw_bytes
        || fact
            .raw_value
            .parse::<f64>()
            .map_or(true, |value| !value.is_finite())
        || fact.entity_id != "x4:runtime:realtime_clock"
        || fact.getter != "GetCurRealTime"
        || fact.semantics != "opaque_runtime_number"
        || fact.observation_version != 1
}
