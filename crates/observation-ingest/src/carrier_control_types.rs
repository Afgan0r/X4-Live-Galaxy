use observation_domain::TransportEpoch;
use serde::{Deserialize, Serialize};

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
