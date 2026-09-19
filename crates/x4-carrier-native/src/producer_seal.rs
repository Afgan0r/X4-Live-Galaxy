use crate::producer::Pending;
use crate::producer_feedback_codec::batch_identity;
use crate::{Producer, ProducerError, ProducerOutcome, ProducerState};
use observation_domain::{ImmutableBatchEnvelope, SectionRevisionId, SourceScopeId};

type EncodedBatch = (ImmutableBatchEnvelope, Vec<u8>, usize);

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
            .as_ref()
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
        let section_key = self
            .messages
            .as_ref()
            .map(|value| value.section_key.clone())
            .ok_or(ProducerError::InvalidTransition)?;
        let ordinal = self
            .next_batch_index
            .checked_add(1)
            .ok_or(ProducerError::DataLimit)?;
        if ordinal > self.limits.max_batches {
            return Err(ProducerError::DataLimit);
        }
        let first_record = self
            .emitted_records
            .checked_add(1)
            .ok_or(ProducerError::DataLimit)?;
        let (envelope, batch, count) =
            largest_batch(self, &scope, &section_key, revision, ordinal, first_record)?;
        self.messages
            .as_mut()
            .ok_or(ProducerError::InvalidTransition)?
            .certificate
            .append(&envelope)
            .ok_or(ProducerError::InvalidInput)?;
        let id = batch_identity(&batch, self.limits.data_message_bytes)?;
        self.records.drain(..count);
        self.next_batch_index = ordinal;
        self.emitted_records = self
            .emitted_records
            .checked_add(count)
            .ok_or(ProducerError::DataLimit)?;
        self.pending = Some(Pending::new(batch, id, now));
        self.state = ProducerState::PendingBatch;
        Ok(ProducerOutcome::Progress)
    }
}

fn largest_batch(
    producer: &Producer,
    scope: &SourceScopeId,
    section_key: &str,
    revision: SectionRevisionId,
    ordinal: usize,
    first_record: usize,
) -> Result<EncodedBatch, ProducerError> {
    let mut low = 1;
    let mut high = producer.records.len();
    if let Some(full) = encoded_prefix(
        producer,
        scope,
        section_key,
        revision,
        ordinal,
        first_record,
        high,
    )? {
        return Ok(full);
    }
    high = high.saturating_sub(1);
    let mut selected = None;
    while low <= high {
        let count = low + (high - low) / 2;
        let Some(candidate) = encoded_prefix(
            producer,
            scope,
            section_key,
            revision,
            ordinal,
            first_record,
            count,
        )?
        else {
            high = count.saturating_sub(1);
            continue;
        };
        selected = Some(candidate);
        low = count.saturating_add(1);
    }
    selected.ok_or(ProducerError::DataLimit)
}

fn encoded_prefix(
    producer: &Producer,
    scope: &SourceScopeId,
    section_key: &str,
    revision: SectionRevisionId,
    ordinal: usize,
    first_record: usize,
    count: usize,
) -> Result<Option<EncodedBatch>, ProducerError> {
    let envelope = crate::producer_message::batch(
        &producer.source,
        scope,
        &producer.records[..count],
        section_key,
        revision,
        ordinal,
        first_record,
    )?;
    let encoded = observation_ingest::encode_complete_message(
        &observation_domain::CompleteMessage::ImmutableBatch(envelope.clone()),
        producer.limits.data_message_bytes,
    );
    Ok(encoded.ok().map(|bytes| (envelope, bytes, count)))
}
