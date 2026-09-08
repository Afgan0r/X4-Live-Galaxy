use crate::producer::{Pending, Readiness, bootstrap_bytes};
use crate::{Producer, ProducerError, ProducerState};

impl Producer {
    pub fn observe_connection(
        &mut self,
        generation: u64,
        now_millis: u64,
    ) -> Result<(), ProducerError> {
        if generation == 0 || generation == self.connection_generation {
            return Ok(());
        }
        if self.connection_generation == 0 {
            self.connection_generation = generation;
            return Ok(());
        }
        self.connection_generation = generation;
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.id == "bootstrap")
        {
            self.pending
                .as_mut()
                .ok_or(ProducerError::InvalidTransition)?
                .handed_off = false;
            self.readiness = Readiness::Awaiting;
            self.state = ProducerState::AwaitingCompatibility;
            return Ok(());
        }
        self.recovery_state = Some(self.state);
        self.recovery_pending = self.pending.take();
        self.pending = Some(Pending::new(
            bootstrap_bytes(&self.source, self.limits.control_message_bytes)?,
            "bootstrap",
            now_millis,
        ));
        self.readiness = Readiness::Awaiting;
        self.state = ProducerState::AwaitingCompatibility;
        Ok(())
    }

    pub(super) fn restore_recovery(&mut self) {
        let Some(state) = self.recovery_state.take() else {
            return;
        };
        self.state = state;
        let Some(mut pending) = self.recovery_pending.take() else {
            return;
        };
        pending.handed_off = false;
        self.pending = Some(pending);
    }
}
