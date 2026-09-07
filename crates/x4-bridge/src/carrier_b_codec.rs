use observation_ingest::{
    CarrierControl, CollectionIntentBody, ControlBody, DemandBody, DispositionBody, HandshakeBody,
    HealthBody, ResetBody,
};

pub use observation_ingest::{CarrierCodecError, CarrierIdentity};
use observation_ingest::{ControlEnvelope, decode_carrier_control as decode_typed};

pub fn encode_carrier_control(
    kind: ControlEnvelope,
    identity: &CarrierIdentity,
    limit: usize,
) -> Result<Vec<u8>, CarrierCodecError> {
    let body = match kind {
        ControlEnvelope::Handshake => ControlBody::Handshake(HandshakeBody {
            native_abi: 2,
            envelope_contract: 2,
            schema_version: 1,
            policy_version: 2,
            canonicalization_version: 3,
            digest_version: 1,
        }),
        ControlEnvelope::Demand => ControlBody::Demand(DemandBody { credit: 1 }),
        ControlEnvelope::Disposition => ControlBody::Disposition(DispositionBody {
            message_id: "message:compatibility".to_owned(),
            section_key: "acceptance".to_owned(),
            section_revision: 1,
            message_digest: "0".repeat(64),
            disposition: "received".to_owned(),
        }),
        ControlEnvelope::CollectionIntent => ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: "acceptance".to_owned(),
            max_records: 1,
            max_raw_bytes: 1,
            max_work: 1,
        }),
        ControlEnvelope::Health => ControlBody::Health(HealthBody {
            status: "available".to_owned(),
        }),
        ControlEnvelope::Reset => ControlBody::Reset(ResetBody {
            reason: "requested".to_owned(),
        }),
    };
    observation_ingest::encode_carrier_control(
        &CarrierControl {
            identity: identity.clone(),
            body,
        },
        limit,
    )
}

pub fn decode_carrier_control(
    bytes: &[u8],
    expected: &CarrierIdentity,
    limit: usize,
) -> Result<ControlEnvelope, CarrierCodecError> {
    decode_typed(bytes, expected, limit)
        .map(|control| observation_ingest::carrier_control_kind(&control.body))
}

pub fn validate_identity(identity: &CarrierIdentity) -> Result<(), CarrierCodecError> {
    observation_ingest::validate_carrier_identity(identity)
}
