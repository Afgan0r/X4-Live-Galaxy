use observation_ingest::{ControlEnvelope, TransportEpoch};
use x4_carrier_native::{
    HandleToken, NativeTransport, TransportConfig, TransportError, TransportPoll,
    TransportSendOutcome,
};

use crate::carrier_b_codec::validate_identity;
pub use crate::carrier_b_codec::{
    CarrierCodecError, CarrierIdentity, decode_carrier_control, encode_carrier_control,
};
use crate::{ConnectionState, ControlPollOutcome, ObservationCarrierFacade};

#[derive(Debug)]
pub struct CarrierBFacade {
    transport: NativeTransport,
    token: HandleToken,
    identity: CarrierIdentity,
    control_limit: usize,
    negotiated: bool,
}

impl CarrierBFacade {
    pub fn start(
        config: TransportConfig,
        identity: CarrierIdentity,
    ) -> Result<Self, CarrierCodecError> {
        validate_identity(&identity)?;
        let generation =
            u32::try_from(identity.epoch.get()).map_err(|_| CarrierCodecError::InvalidIdentity)?;
        let token = HandleToken {
            generation,
            slot: 1,
        };
        let control_limit = config.max_control_message_bytes;
        let transport =
            NativeTransport::start(config, token).map_err(|_| CarrierCodecError::InvalidShape)?;
        Ok(Self {
            transport,
            token,
            identity,
            control_limit,
            negotiated: false,
        })
    }
}

impl ObservationCarrierFacade for CarrierBFacade {
    fn connection_state(&self) -> ConnectionState {
        if self.negotiated && self.transport.snapshot().connected {
            ConnectionState::Connected(self.identity.epoch)
        } else {
            ConnectionState::Disconnected
        }
    }

    fn try_send_complete(
        &mut self,
        epoch: TransportEpoch,
        complete_message: &[u8],
    ) -> crate::CompleteMessageSendOutcome {
        if epoch != self.identity.epoch {
            return crate::CompleteMessageSendOutcome::Rejected(crate::FacadeError::StaleEpoch);
        }
        if self.connection_state() == ConnectionState::Disconnected {
            return crate::CompleteMessageSendOutcome::Rejected(
                crate::FacadeError::TransportUnavailable,
            );
        }
        match self.transport.try_send(self.token, complete_message) {
            TransportSendOutcome::LocalHandoff => crate::CompleteMessageSendOutcome::LocalHandoff,
            TransportSendOutcome::CapacityUnavailable => {
                crate::CompleteMessageSendOutcome::CapacityUnavailable
            }
            TransportSendOutcome::Rejected(TransportError::MessageTooLarge) => {
                crate::CompleteMessageSendOutcome::Rejected(crate::FacadeError::MessageTooLarge)
            }
            TransportSendOutcome::Rejected(_) => crate::CompleteMessageSendOutcome::Rejected(
                crate::FacadeError::TransportUnavailable,
            ),
        }
    }

    fn poll_control(&mut self, max_messages: usize) -> ControlPollOutcome {
        if max_messages == 0 {
            return ControlPollOutcome::LimitReached;
        }
        match self.transport.poll_control(self.token, self.control_limit) {
            TransportPoll::NoMessage => ControlPollOutcome::NoMessage,
            TransportPoll::Message(bytes) => self.admit_control(&bytes),
            TransportPoll::Closed | TransportPoll::Rejected(_) => {
                ControlPollOutcome::Rejected(crate::FacadeError::TransportUnavailable)
            }
        }
    }
}

impl CarrierBFacade {
    fn admit_control(&mut self, bytes: &[u8]) -> ControlPollOutcome {
        match decode_carrier_control(bytes, &self.identity, self.control_limit) {
            Ok(kind) if self.negotiated || kind == ControlEnvelope::Handshake => {
                self.negotiated = true;
                ControlPollOutcome::Message(kind)
            }
            Err(CarrierCodecError::StaleEpoch) => {
                ControlPollOutcome::Rejected(crate::FacadeError::StaleEpoch)
            }
            Err(CarrierCodecError::MessageTooLarge) => {
                ControlPollOutcome::Rejected(crate::FacadeError::MessageTooLarge)
            }
            Ok(_) | Err(_) => {
                ControlPollOutcome::Rejected(crate::FacadeError::TransportUnavailable)
            }
        }
    }
}
