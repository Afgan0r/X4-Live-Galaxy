use crate::{ProducerError, ProducerSource};
use observation_ingest::{CarrierControl, CarrierIdentity, ControlBody, encode_carrier_control};

pub(super) fn identity(source: &ProducerSource) -> Result<CarrierIdentity, ProducerError> {
    let identity = CarrierIdentity {
        session_id: source.session_id.clone(),
        producer_incarnation: source.producer_incarnation.clone(),
        epoch: observation_domain::TransportEpoch::new(source.transport_epoch)
            .ok_or(ProducerError::StaleEpoch)?,
    };
    observation_ingest::validate_carrier_identity(&identity)
        .map_err(|_| ProducerError::InvalidInput)?;
    Ok(identity)
}

pub(super) fn bootstrap_bytes(
    source: &ProducerSource,
    limit: usize,
) -> Result<Vec<u8>, ProducerError> {
    encode_carrier_control(
        &CarrierControl {
            identity: identity(source)?,
            body: ControlBody::Handshake(observation_ingest::HandshakeBody {
                native_abi: 2,
                envelope_contract: 2,
                schema_version: 1,
                policy_version: 2,
                canonicalization_version: 3,
                digest_version: 1,
            }),
        },
        limit,
    )
    .map_err(|_| ProducerError::ControlLimit)
}
