pub use observation_ingest::{ControlEnvelope, TransportEpoch};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connected(TransportEpoch),
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FacadeError {
    StaleEpoch,
    MessageTooLarge,
    TransportUnavailable,
    InvalidControlLimit,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompleteMessageSendOutcome {
    LocalHandoff,
    CapacityUnavailable,
    Rejected(FacadeError),
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlPollOutcome {
    NoMessage,
    Message(ControlEnvelope),
    LimitReached,
    Rejected(FacadeError),
}

pub trait ObservationCarrierFacade {
    fn connection_state(&self) -> ConnectionState;
    fn try_send_complete(
        &mut self,
        epoch: TransportEpoch,
        complete_message: &[u8],
    ) -> CompleteMessageSendOutcome;
    fn poll_control(&mut self, max_messages: usize) -> ControlPollOutcome;
}
