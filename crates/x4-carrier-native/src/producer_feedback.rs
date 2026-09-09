use observation_domain::CompleteMessage;
use observation_ingest::{
    ControlBody, complete_message_digest, decode_carrier_control, decode_complete_message,
};

use crate::producer::{Pending, Readiness, identity};
use crate::{Producer, ProducerError, ProducerFeedback, ProducerOutcome, ProducerState};

impl Producer {
    pub fn apply_control(
        &mut self,
        bytes: &[u8],
        now_millis: u64,
    ) -> Result<ProducerOutcome, ProducerError> {
        let control = decode_carrier_control(
            bytes,
            &identity(&self.source)?,
            self.limits.control_message_bytes,
        )
        .map_err(|error| match error {
            observation_ingest::CarrierCodecError::StaleEpoch => ProducerError::StaleEpoch,
            observation_ingest::CarrierCodecError::RestartRequired => ProducerError::Incompatible,
            _ => ProducerError::InvalidInput,
        })?;
        match control.body {
            ControlBody::Handshake(_)
                if self.state == ProducerState::AwaitingCompatibility
                    && self
                        .pending
                        .as_ref()
                        .is_some_and(|pending| pending.id == "bootstrap" && pending.handed_off) =>
            {
                self.readiness = Readiness::Handshake;
                self.pending = None;
                self.state = ProducerState::Ready;
                Ok(ProducerOutcome::Accepted)
            }
            ControlBody::CollectionIntent(value)
                if self.readiness == Readiness::Handshake
                    && value.section_key == "carrier_b_realtime_sample"
                    && value.next_revision > 0
                    && value.max_records >= 1
                    && value.max_raw_bytes >= self.limits.max_raw_bytes
                    && value.max_work >= 1 =>
            {
                self.revision = self.revision.max(value.next_revision);
                self.readiness = Readiness::Intent;
                Ok(ProducerOutcome::Accepted)
            }
            ControlBody::Demand(value)
                if self.readiness == Readiness::Intent && value.credit == 1 =>
            {
                self.readiness = Readiness::Ready;
                Ok(ProducerOutcome::Accepted)
            }
            ControlBody::Disposition(value) => self.apply_disposition(&value, now_millis),
            ControlBody::Reset(_) => Ok(ProducerOutcome::Disconnected),
            _ => Err(ProducerError::InvalidTransition),
        }
    }

    fn apply_disposition(
        &mut self,
        value: &observation_ingest::DispositionBody,
        now: u64,
    ) -> Result<ProducerOutcome, ProducerError> {
        let pending = self
            .pending
            .as_mut()
            .ok_or(ProducerError::InvalidTransition)?;
        if !pending.handed_off
            || value.message_id != pending.id
            || value.section_key != "carrier_b_realtime_sample"
            || value.section_revision != self.revision
            || value.message_digest != hex(complete_message_digest(&pending.bytes))
        {
            return Err(ProducerError::InvalidInput);
        }
        let feedback = disposition(&value.disposition)?;
        if feedback == ProducerFeedback::CapacityUnavailable {
            return Ok(self.retry(now));
        }
        if feedback == ProducerFeedback::Ambiguous {
            self.state = ProducerState::PausedAfterFailure;
            return Ok(ProducerOutcome::Ambiguous);
        }
        if matches!(
            feedback,
            ProducerFeedback::PermanentlyRejected
                | ProducerFeedback::StaleEpoch
                | ProducerFeedback::TimedOutOrSuperseded
        ) {
            self.discard_incomplete();
            self.state = ProducerState::PausedAfterFailure;
            return Ok(ProducerOutcome::PermanentlyRejected);
        }
        self.advance_message(feedback, now)
    }

    fn advance_message(
        &mut self,
        feedback: ProducerFeedback,
        now: u64,
    ) -> Result<ProducerOutcome, ProducerError> {
        let messages = self
            .messages
            .as_ref()
            .ok_or(ProducerError::InvalidTransition)?;
        let next = match self.state {
            ProducerState::PendingStart if feedback == ProducerFeedback::Received => Some((
                messages.batch.clone(),
                batch_identity(&messages.batch, self.limits.data_message_bytes)?,
                ProducerState::PendingBatch,
            )),
            ProducerState::PendingBatch if feedback == ProducerFeedback::Received => Some((
                messages.completion.clone(),
                format!(
                    "message:complete:carrier_b_realtime_sample:{}",
                    self.revision
                ),
                ProducerState::PendingCompletion,
            )),
            ProducerState::PendingCompletion if feedback == ProducerFeedback::Committed => None,
            _ => return Err(ProducerError::InvalidTransition),
        };
        if let Some((bytes, id, state)) = next {
            self.pending = Some(Pending::new(bytes, id, now));
            self.state = state;
            return Ok(ProducerOutcome::Received);
        }
        self.discard_incomplete();
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or(ProducerError::InvalidTransition)?;
        self.state = ProducerState::Ready;
        self.readiness = Readiness::Intent;
        Ok(ProducerOutcome::Committed)
    }

    fn retry(&mut self, now: u64) -> ProducerOutcome {
        let Some(pending) = self.pending.as_mut() else {
            return ProducerOutcome::PausedAfterFailure;
        };
        if pending.attempts >= 2
            || now.saturating_sub(pending.first_attempt_at) > self.limits.max_retry_age_millis
        {
            self.state = ProducerState::PausedAfterFailure;
            return ProducerOutcome::PausedAfterFailure;
        }
        pending.attempts += 1;
        pending.handed_off = false;
        ProducerOutcome::CapacityUnavailable
    }
}

fn batch_identity(bytes: &[u8], limit: usize) -> Result<String, ProducerError> {
    let CompleteMessage::ImmutableBatch(batch) =
        decode_complete_message(bytes, limit).map_err(|_| ProducerError::InvalidInput)?
    else {
        return Err(ProducerError::InvalidInput);
    };
    Ok(batch.batch_id.as_str().to_owned())
}

fn disposition(value: &str) -> Result<ProducerFeedback, ProducerError> {
    match value {
        "capacity_unavailable" => Ok(ProducerFeedback::CapacityUnavailable),
        "received" => Ok(ProducerFeedback::Received),
        "committed" => Ok(ProducerFeedback::Committed),
        "permanently_rejected" => Ok(ProducerFeedback::PermanentlyRejected),
        "ambiguous_commit" => Ok(ProducerFeedback::Ambiguous),
        "stale_epoch" => Ok(ProducerFeedback::StaleEpoch),
        "timed_out_or_superseded" => Ok(ProducerFeedback::TimedOutOrSuperseded),
        _ => Err(ProducerError::InvalidInput),
    }
}

fn hex(bytes: [u8; 32]) -> String {
    use core::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ignored = write!(text, "{byte:02x}");
            text
        })
}
