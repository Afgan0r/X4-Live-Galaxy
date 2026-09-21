use crate::producer::{Pending, Readiness};
use crate::producer_message::assemble;
use crate::producer_types::{PreparedRecord, ProducerProfile};
use crate::producer_validation::{invalid_clock, qualified_empty};
use crate::{
    Producer, ProducerError, ProducerOutcome, ProducerState, SectionEvidence,
    SectionFinishEvidence, TypedFact,
};

impl Producer {
    pub fn begin_ship_section(
        &mut self,
        evidence: SectionEvidence,
        expected_records: usize,
    ) -> Result<(), ProducerError> {
        if self.profile == ProducerProfile::Clock
            || expected_records > self.limits.max_records
            || (expected_records == 0 && !qualified_empty(&evidence))
        {
            return Err(ProducerError::InvalidInput);
        }
        self.begin(evidence, expected_records)
    }
    pub fn push_ship_core(
        &mut self,
        record: &observation_domain::ShipCoreRecord,
    ) -> Result<(), ProducerError> {
        if self.profile != ProducerProfile::ShipCore {
            return Err(ProducerError::InvalidTransition);
        }
        let content = format!(
            "profile=ship_core\nidentity={}\nowner={}\ntype={}\nclass={}\nlocation={}\nconsistency={}\nconsistency_reason={}",
            record.identity().as_str(),
            record.owner().as_str(),
            record.ship_type().as_str(),
            record.class().as_str(),
            record.location().as_str(),
            record.consistency().fields().0,
            record.consistency().fields().1
        );
        let expected_scope = self
            .evidence
            .as_ref()
            .map(|value| value.source_scope.as_str())
            .ok_or(ProducerError::InvalidTransition)?;
        if record.source_scope().as_str() != expected_scope
            || content.len() > self.limits.max_raw_bytes
        {
            return Err(ProducerError::DataLimit);
        }
        self.push(PreparedRecord {
            entity_id: format!("x4:ship:{}", record.identity().as_str()),
            content,
        })
    }
    pub fn begin_section(&mut self, evidence: SectionEvidence) -> Result<(), ProducerError> {
        if self.profile != ProducerProfile::Clock {
            return Err(ProducerError::InvalidTransition);
        }
        self.begin(evidence, 1)
    }
    pub(super) fn begin(
        &mut self,
        evidence: SectionEvidence,
        expected_records: usize,
    ) -> Result<(), ProducerError> {
        if self.state != ProducerState::Ready || self.readiness != Readiness::Ready {
            return Err(ProducerError::InvalidTransition);
        }
        self.evidence = Some(evidence);
        self.expected_records = expected_records;
        self.state = ProducerState::SectionReserved;
        Ok(())
    }
    pub fn push_record(&mut self, fact: &TypedFact) -> Result<(), ProducerError> {
        if self.profile != ProducerProfile::Clock {
            return Err(ProducerError::InvalidTransition);
        }
        if invalid_clock(fact, self.limits.max_raw_bytes) {
            return Err(ProducerError::DataLimit);
        }
        self.push(PreparedRecord {
            entity_id: fact.entity_id.clone(),
            content: format!(
                "getter={}\nraw_value={}\nsemantics={}",
                fact.getter, fact.raw_value, fact.semantics
            ),
        })
    }
    pub(super) fn push(&mut self, record: PreparedRecord) -> Result<(), ProducerError> {
        if !matches!(
            self.state,
            ProducerState::SectionReserved | ProducerState::Collecting
        ) || self.emitted_records + self.records.len() >= self.expected_records
        {
            return Err(ProducerError::InvalidTransition);
        }
        let bytes = self
            .raw_bytes
            .checked_add(record.content.len())
            .ok_or(ProducerError::DataLimit)?;
        if self.profile != ProducerProfile::Clock && bytes > self.limits.max_raw_bytes {
            return Err(ProducerError::DataLimit);
        }
        self.records.push(record);
        self.raw_bytes = bytes;
        self.state = ProducerState::Collecting;
        Ok(())
    }

    pub fn finish_section(&mut self, finish: SectionFinishEvidence) -> Result<(), ProducerError> {
        if !matches!(
            self.state,
            ProducerState::SectionReserved | ProducerState::Collecting
        ) || self.emitted_records + self.records.len() != self.expected_records
        {
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
        self.observe_progress(now_millis)?;
        if self.pending.is_some() {
            return Ok(ProducerOutcome::CapacityUnavailable);
        }
        if !matches!(
            self.state,
            ProducerState::Collecting | ProducerState::SectionReserved
        ) || work_units == 0
            || work_units > self.limits.max_work
        {
            return Err(ProducerError::InvalidTransition);
        }
        if self.messages.is_some() {
            return self.seal_next(now_millis);
        }
        let evidence = self
            .evidence
            .as_ref()
            .ok_or(ProducerError::InvalidTransition)?;
        let messages = assemble(
            &self.source,
            evidence,
            &self.selected_key,
            self.expected_records,
            self.revision,
            self.limits.data_message_bytes,
        )?;
        self.pending = Some(Pending::new(
            messages,
            format!("message:start:{}:{}", self.selected_key, self.revision),
            now_millis,
        ));
        self.messages = Some(crate::producer_message::SectionMessages {
            section_key: self.selected_key.clone(),
            certificate: observation_ingest::ProducerCertificateStream::default(),
        });
        self.next_batch_index = 0;
        self.emitted_records = 0;
        self.state = ProducerState::PendingStart;
        Ok(ProducerOutcome::Progress)
    }
}
