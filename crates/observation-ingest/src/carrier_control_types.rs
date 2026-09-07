use observation_domain::{ControlEnvelope, TransportEpoch};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    RestartRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CarrierControl {
    pub identity: CarrierIdentity,
    pub body: ControlBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControlBody {
    Handshake(HandshakeBody),
    Demand(DemandBody),
    Disposition(DispositionBody),
    CollectionIntent(CollectionIntentBody),
    Health(HealthBody),
    Reset(ResetBody),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandshakeBody {
    pub native_abi: u16,
    pub envelope_contract: u16,
    pub schema_version: u16,
    pub policy_version: u16,
    pub canonicalization_version: u16,
    pub digest_version: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DemandBody {
    pub credit: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionBody {
    pub message_id: String,
    pub section_key: String,
    pub section_revision: u64,
    pub message_digest: String,
    pub disposition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionIntentBody {
    pub section_key: String,
    pub max_records: usize,
    pub max_raw_bytes: usize,
    pub max_work: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealthBody {
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResetBody {
    pub reason: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BootstrapControl {
    control_version: u16,
    kind: String,
    session: String,
    incarnation: String,
    epoch: u64,
    body: HandshakeBody,
}

pub fn decode_carrier_bootstrap(
    bytes: &[u8],
    limit: usize,
) -> Result<CarrierIdentity, CarrierCodecError> {
    if bytes.len() > limit {
        return Err(CarrierCodecError::MessageTooLarge);
    }
    let raw: BootstrapControl = serde_json::from_slice(bytes).map_err(|error| {
        if error.to_string().contains("unknown field") {
            CarrierCodecError::UnknownField
        } else {
            CarrierCodecError::InvalidShape
        }
    })?;
    if raw.control_version != 2 || raw.body != crate::carrier_control::exact_handshake() {
        return Err(CarrierCodecError::RestartRequired);
    }
    if raw.kind != "handshake" {
        return Err(CarrierCodecError::UnknownKind);
    }
    let identity = CarrierIdentity {
        session_id: raw.session,
        producer_incarnation: raw.incarnation,
        epoch: TransportEpoch::new(raw.epoch).ok_or(CarrierCodecError::InvalidIdentity)?,
    };
    validate_carrier_identity(&identity)?;
    Ok(identity)
}

#[must_use]
pub fn complete_message_digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

pub fn validate_carrier_identity(identity: &CarrierIdentity) -> Result<(), CarrierCodecError> {
    let invalid = identity.session_id.trim().is_empty()
        || identity.producer_incarnation.trim().is_empty()
        || identity.session_id.len() > 128
        || identity.producer_incarnation.len() > 128;
    (!invalid)
        .then_some(())
        .ok_or(CarrierCodecError::InvalidIdentity)
}

pub const fn carrier_control_kind(body: &ControlBody) -> ControlEnvelope {
    match body {
        ControlBody::Handshake(_) => ControlEnvelope::Handshake,
        ControlBody::Demand(_) => ControlEnvelope::Demand,
        ControlBody::Disposition(_) => ControlEnvelope::Disposition,
        ControlBody::CollectionIntent(_) => ControlEnvelope::CollectionIntent,
        ControlBody::Health(_) => ControlEnvelope::Health,
        ControlBody::Reset(_) => ControlEnvelope::Reset,
    }
}
