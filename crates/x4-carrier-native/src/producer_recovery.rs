use crate::producer::{Pending, Readiness, bootstrap_bytes};
use crate::{Producer, ProducerError, ProducerState};

impl Producer {
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
