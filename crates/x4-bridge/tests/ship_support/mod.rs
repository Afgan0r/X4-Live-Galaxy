#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "contract fixtures fail immediately"
)]
use observation_domain::*;
use observation_ingest::{
    CarrierControl, CarrierIdentity, CollectionIntentBody, ControlBody, DemandBody,
    DispositionBody, HandshakeBody, complete_message_digest, decode_complete_message,
    encode_carrier_control,
};
use std::fmt::Write as _;
use x4_carrier_native::{
    Producer, ProducerLimits, ProducerSource, SectionEvidence, SectionFinishEvidence,
};

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

pub fn messages(revision: u64, owner: &str) -> Vec<Vec<u8>> {
    with_identities(revision, owner, ["9007199254740993", "9007199254740995"])
}

pub fn with_identities(revision: u64, owner: &str, identities: [&str; 2]) -> Vec<Vec<u8>> {
    let (mut producer, source) = ready(revision);
    let mut evidence = SectionEvidence::point_measurement(source.source_scope.clone());
    evidence.sender.section_state = SectionState::with_evidence(
        CaptureWindow::new(0, 0).expect("valid test fixture"),
        SectionFreshness::Fresh,
        SectionQuality::Unknown,
        SectionAvailability::Available,
        SectionCoverage::Partial,
    );
    evidence.sender.source_consistency = SourceConsistency::ObservedCountFillOnly;
    evidence.sender.stable_identity = true;
    producer
        .begin_ship_section(evidence.clone(), 2)
        .expect("valid test fixture");
    for id in identities {
        let record = ShipCoreRecord::new(
            SourceScopeId::new(source.source_scope.clone()).expect("valid test fixture"),
            ShipIdentity::new(id).expect("valid test fixture"),
            ShipOwner::new(owner).expect("valid test fixture"),
            ShipType::new("destroyer_macro").expect("valid test fixture"),
            ShipClass::new("destroyer").expect("valid test fixture"),
            ShipLocation::new("sector:2").expect("valid test fixture"),
            evidence.sender.clone(),
        );
        producer
            .push_ship_core(&record)
            .expect("valid test fixture");
    }
    producer
        .finish_section(SectionFinishEvidence {
            capture_end_millis: 0,
            succeeded: true,
            quality: SectionQuality::Unknown,
            availability: SectionAvailability::Available,
            coverage: SectionCoverage::Partial,
            consistency: SourceConsistency::ObservedCountFillOnly,
            stable_identity: true,
        })
        .expect("valid test fixture");
    producer.progress(1, 0).expect("valid test fixture");
    take_messages(producer, &source, revision)
}

fn ready(revision: u64) -> (Producer, ProducerSource) {
    let source = ProducerSource {
        session_id: "session:ship".into(),
        producer_incarnation: "producer:ship".into(),
        transport_epoch: 7,
        source_scope: "x4:faction:argon:ships".into(),
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
            section_key: "ship_core".into(),
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

fn take_messages(mut producer: Producer, source: &ProducerSource, revision: u64) -> Vec<Vec<u8>> {
    let mut messages = Vec::new();
    for ordinal in 0..4 {
        let bytes = producer
            .pending_bytes()
            .expect("valid test fixture")
            .to_vec();
        let message = decode_complete_message(&bytes, 4096).expect("valid test fixture");
        let id = match message {
            CompleteMessage::SectionStart(_) => format!("message:start:ship_core:{revision}"),
            CompleteMessage::ImmutableBatch(batch) => batch.batch_id.as_str().to_owned(),
            CompleteMessage::SectionCompletion(_) => {
                format!("message:complete:ship_core:{revision}")
            }
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
                        section_key: "ship_core".into(),
                        section_revision: revision,
                        message_digest: digest,
                        disposition: if ordinal == 3 {
                            "committed"
                        } else {
                            "received"
                        }
                        .into(),
                    }),
                ),
                0,
            )
            .expect("valid test fixture");
        messages.push(bytes);
    }
    messages
}
