use crate::producer::{Pending, Readiness, bootstrap_bytes};
use crate::{Producer, ProducerError, ProducerOutcome, ProducerState};

impl Producer {
    pub fn reconcile_committed(
        &mut self,
        message_id: &str,
        message_digest: &str,
    ) -> Result<ProducerOutcome, ProducerError> {
        let pending = self
            .pending
            .as_ref()
            .ok_or(ProducerError::InvalidTransition)?;
        if !pending.handed_off
            || pending.id != message_id
            || message_digest != digest(&pending.bytes)
            || !matches!(
                self.state,
                ProducerState::PendingCompletion | ProducerState::PausedAfterFailure
            )
        {
            return Err(ProducerError::InvalidInput);
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

    pub fn observe_connection(
        &mut self,
        generation: u64,
        now_millis: u64,
    ) -> Result<(), ProducerError> {
        if generation == 0 || generation <= self.connection_generation {
            return Ok(());
        }
        if self.connection_generation == 0 {
            self.connection_generation = generation;
            return Ok(());
        }

        let transport_epoch = self
            .source
            .transport_epoch
            .checked_add(1)
            .ok_or(ProducerError::StaleEpoch)?;
        let revision = if self.has_incomplete_attempt() {
            self.revision
                .checked_add(1)
                .ok_or(ProducerError::InvalidTransition)?
        } else {
            self.revision
        };
        let mut source = self.source.clone();
        source.transport_epoch = transport_epoch;
        let bootstrap = bootstrap_bytes(&source, self.limits.control_message_bytes)?;

        self.discard_incomplete();
        self.source = source;
        self.revision = revision;
        self.connection_generation = generation;
        self.pending = Some(Pending::new(bootstrap, "bootstrap", now_millis));
        self.readiness = Readiness::Awaiting;
        self.state = ProducerState::AwaitingCompatibility;
        Ok(())
    }

    fn has_incomplete_attempt(&self) -> bool {
        !matches!(
            self.state,
            ProducerState::AwaitingCompatibility | ProducerState::Ready
        )
    }
}

fn digest(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    observation_ingest::complete_message_digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ignored = write!(text, "{byte:02x}");
            text
        })
}
