use observation_ingest::{ControlEnvelope, TransportEpoch};
use x4_carrier_native::TransportConfig;

use crate::{ConnectionState, ControlPollOutcome, ObservationCarrierFacade};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CarrierIdentity {
    pub session_id: String,
    pub producer_incarnation: String,
    pub epoch: TransportEpoch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CarrierCodecError {
    MessageTooLarge,
    InvalidShape,
    UnknownKind,
    UnknownField,
    InvalidIdentity,
    StaleEpoch,
    MutationForbidden,
}

pub fn encode_carrier_control(
    _kind: ControlEnvelope,
    _identity: &CarrierIdentity,
    _limit: usize,
) -> Result<Vec<u8>, CarrierCodecError> {
    Err(CarrierCodecError::InvalidShape)
}

pub fn decode_carrier_control(
    _bytes: &[u8],
    _expected: &CarrierIdentity,
    _limit: usize,
) -> Result<ControlEnvelope, CarrierCodecError> {
    Err(CarrierCodecError::InvalidShape)
}

#[derive(Debug)]
pub struct CarrierBFacade;

impl CarrierBFacade {
    pub fn start(
        _config: TransportConfig,
        _identity: CarrierIdentity,
    ) -> Result<Self, CarrierCodecError> {
        Err(CarrierCodecError::InvalidShape)
    }
}

impl ObservationCarrierFacade for CarrierBFacade {
    fn connection_state(&self) -> ConnectionState {
        ConnectionState::Disconnected
    }

    fn try_send_complete(
        &mut self,
        _epoch: TransportEpoch,
        _complete_message: &[u8],
    ) -> crate::CompleteMessageSendOutcome {
        crate::CompleteMessageSendOutcome::Rejected(crate::FacadeError::TransportUnavailable)
    }

    fn poll_control(&mut self, _max_messages: usize) -> ControlPollOutcome {
        ControlPollOutcome::NoMessage
    }
}
