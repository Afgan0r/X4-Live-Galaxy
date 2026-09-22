use observation_domain::*;
use observation_ingest::{
    CarrierControl, CarrierIdentity, CollectionIntentBody, ControlBody, DemandBody,
    DispositionBody, HandshakeBody, complete_message_digest, decode_complete_message,
    encode_carrier_control,
};
use std::fmt::Write as _;
use x4_carrier_native::{Producer, ProducerLimits, ProducerSource};

fn control(source: &ProducerSource, body: ControlBody) -> Vec<u8> {
    encode_carrier_control(
        &CarrierControl {
            identity: CarrierIdentity {
                session_id: source.session_id.clone(),
                producer_incarnation: source.producer_incarnation.clone(),
                epoch: TransportEpoch::new(7).expect("valid test fixture"),
            },
            body,
        },
        512,
    )
    .expect("valid test fixture")
}

pub fn ready(revision: u64, faction: &str, key: &str) -> (Producer, ProducerSource) {
    let source = ProducerSource {
        session_id: "session:ship".into(),
        producer_incarnation: "producer:ship".into(),
        transport_epoch: 7,
        source_scope: format!("x4:faction:{faction}:ships"),
        source_epoch_status: SourceEpochStatus::Unknown,
        source_boundary: SourceBoundary::RuntimeStart,
    };
    let mut producer = Producer::new(
        ProducerLimits {
            data_message_bytes: 4096,
            control_message_bytes: 512,
            max_records: 4,
            max_raw_bytes: 512,
            max_batches: 4,
            max_work: 4,
            max_retry_age_millis: 5000,
        },
        source.clone(),
        0,
    )
    .expect("valid test fixture");
    producer.mark_local_handoff(0).expect("valid test fixture");
    for body in [
        ControlBody::Handshake(HandshakeBody {
            native_abi: 2,
            envelope_contract: 2,
            schema_version: 1,
            policy_version: 2,
            canonicalization_version: 3,
            digest_version: 1,
        }),
        ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: key.into(),
            next_revision: revision,
            max_records: 4,
            max_raw_bytes: 512,
            max_work: 4,
        }),
        ControlBody::Demand(DemandBody { credit: 1 }),
    ] {
        producer
            .apply_control(&control(&source, body), 0)
            .expect("valid test fixture");
    }
    (producer, source)
}

pub fn take_messages(
    mut producer: Producer,
    source: &ProducerSource,
    revision: u64,
    key: &str,
) -> Vec<Vec<u8>> {
    let mut messages = Vec::new();
    loop {
        let bytes = producer
            .pending_bytes()
            .expect("valid test fixture")
            .to_vec();
        let message = decode_complete_message(&bytes, 4096).expect("valid test fixture");
        let (id, disposition, complete) = match message {
            CompleteMessage::SectionStart(_) => {
                (format!("message:start:{key}:{revision}"), "received", false)
            }
            CompleteMessage::ImmutableBatch(batch) => {
                (batch.batch_id.as_str().to_owned(), "received", false)
            }
            CompleteMessage::SectionCompletion(_) => (
                format!("message:complete:{key}:{revision}"),
                "committed",
                true,
            ),
            CompleteMessage::Control(_) => panic!("unexpected control"),
        };
        let mut digest = String::new();
        for byte in complete_message_digest(&bytes) {
            write!(&mut digest, "{byte:02x}").expect("digest writes");
        }
        producer.mark_local_handoff(0).expect("valid test fixture");
        producer
            .apply_control(
                &control(
                    source,
                    ControlBody::Disposition(DispositionBody {
                        message_id: id,
                        section_key: key.into(),
                        section_revision: revision,
                        message_digest: digest,
                        disposition: disposition.into(),
                    }),
                ),
                0,
            )
            .expect("valid test fixture");
        messages.push(bytes);
        if complete {
            break;
        }
    }
    messages
}
