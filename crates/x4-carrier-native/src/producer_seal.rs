use crate::producer::Pending;
use crate::producer_feedback_codec::batch_identity;
use crate::{Producer, ProducerError, ProducerOutcome, ProducerState};

impl Producer {
    pub(super) fn seal_next(&mut self, now: u64) -> Result<ProducerOutcome, ProducerError> {
        if !self.records.is_empty() {
            return self.seal_batch(now);
        }
        if !self.finished {
            return Ok(ProducerOutcome::Accepted);
        }
        let evidence = self
            .evidence
            .as_ref()
            .ok_or(ProducerError::InvalidTransition)?;
        let scope = observation_domain::SourceScopeId::new(evidence.source_scope.clone())
            .ok_or(ProducerError::InvalidInput)?;
        let revision = observation_domain::SectionRevisionId::new(self.revision)
            .ok_or(ProducerError::InvalidInput)?;
        let messages = self
            .messages
            .as_mut()
            .ok_or(ProducerError::InvalidTransition)?;
        let envelope = crate::producer_message::completion(
            evidence,
            scope,
            observation_domain::ProducerIncarnationId::new(
                self.source.producer_incarnation.clone(),
            )
            .ok_or(ProducerError::InvalidInput)?,
            observation_domain::TransportEpoch::new(self.source.transport_epoch)
                .ok_or(ProducerError::StaleEpoch)?,
            observation_domain::SectionKey::new(messages.section_key.clone())
                .ok_or(ProducerError::InvalidInput)?,
            revision,
            &messages.certificate,
        )?;
        let bytes = observation_ingest::encode_complete_message(
            &observation_domain::CompleteMessage::SectionCompletion(envelope),
            self.limits.data_message_bytes,
        )
        .map_err(|_| ProducerError::DataLimit)?;
        self.pending = Some(Pending::new(
            bytes,
            format!(
                "message:complete:{}:{}",
                messages.section_key, self.revision
            ),
            now,
        ));
        self.state = ProducerState::PendingCompletion;
        Ok(ProducerOutcome::Progress)
    }

    fn seal_batch(&mut self, now: u64) -> Result<ProducerOutcome, ProducerError> {
        let evidence = self
            .evidence
            .as_ref()
            .ok_or(ProducerError::InvalidTransition)?;
        let scope = observation_domain::SourceScopeId::new(evidence.source_scope.clone())
            .ok_or(ProducerError::InvalidInput)?;
        let revision = observation_domain::SectionRevisionId::new(self.revision)
            .ok_or(ProducerError::InvalidInput)?;
        let messages = self
            .messages
            .as_mut()
            .ok_or(ProducerError::InvalidTransition)?;
        let record = self
            .records
            .first()
            .ok_or(ProducerError::InvalidTransition)?;
        let ordinal = self
            .next_batch_index
            .checked_add(1)
            .ok_or(ProducerError::DataLimit)?;
        let envelope = crate::producer_message::batch(
            &self.source,
            &scope,
            record,
            &messages.section_key,
            revision,
            ordinal,
        )?;
        let batch = observation_ingest::encode_complete_message(
            &observation_domain::CompleteMessage::ImmutableBatch(envelope.clone()),
            self.limits.data_message_bytes,
        )
        .map_err(|_| ProducerError::DataLimit)?;
        messages
            .certificate
            .append(&envelope)
            .ok_or(ProducerError::InvalidInput)?;
        let id = batch_identity(&batch, self.limits.data_message_bytes)?;
        self.records.remove(0);
        self.next_batch_index = ordinal;
        self.pending = Some(Pending::new(batch, id, now));
        self.state = ProducerState::PendingBatch;
        Ok(ProducerOutcome::Progress)
    }
}
