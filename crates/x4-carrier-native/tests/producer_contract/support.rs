use observation_ingest::{
    CarrierControl, CarrierIdentity, CollectionIntentBody, ControlBody, DemandBody,
    DispositionBody, complete_message_digest, encode_carrier_control,
};
use x4_carrier_native::{
    Producer, ProducerLimits, ProducerSource, SectionEvidence, SectionFinishEvidence, TypedFact,
};

pub fn ready(now: u64) -> (Producer, ProducerSource) {
    let source = source(7);
    let mut producer = Producer::new(ProducerLimits::bring_up(), source.clone(), now).unwrap();
    producer.mark_local_handoff(now).unwrap();
    for body in [
        handshake(),
        intent(),
        ControlBody::Demand(DemandBody { credit: 1 }),
    ] {
        producer
            .apply_control(&control(&source, body), now)
            .unwrap();
    }
    (producer, source)
}

pub fn pending(now: u64) -> (Producer, ProducerSource) {
    let (mut producer, source) = ready(now);
    producer
        .begin_section(SectionEvidence::point_measurement(
            "x4:carrier_b_acceptance",
        ))
        .unwrap();
    producer.push_record(&sample("3")).unwrap();
    producer.finish_section(finish(now)).unwrap();
    producer.progress(1, now).unwrap();
    (producer, source)
}

pub fn take(
    producer: &mut Producer,
    source: &ProducerSource,
    result: &str,
    ordinal: usize,
) -> Vec<u8> {
    let bytes = producer.pending_bytes().unwrap().to_vec();
    producer.mark_local_handoff(0).unwrap();
    producer
        .apply_control(&disposition(source, &bytes, result, ordinal), 0)
        .unwrap();
    bytes
}

pub fn disposition(source: &ProducerSource, bytes: &[u8], result: &str, ordinal: usize) -> Vec<u8> {
    let id = match ordinal {
        1 => "message:start:carrier_b_realtime_sample:1".to_owned(),
        2 => "carrier-b:1:1".to_owned(),
        _ => "message:complete:carrier_b_realtime_sample:1".to_owned(),
    };
    control(
        source,
        ControlBody::Disposition(DispositionBody {
            message_id: id,
            section_key: "carrier_b_realtime_sample".to_owned(),
            section_revision: 1,
            message_digest: hex(complete_message_digest(bytes)),
            disposition: result.to_owned(),
        }),
    )
}

pub fn source(epoch: u64) -> ProducerSource {
    ProducerSource {
        session_id: "session:acceptance".to_owned(),
        producer_incarnation: "producer:acceptance".to_owned(),
        transport_epoch: epoch,
        source_scope: "x4:carrier_b_acceptance".to_owned(),
        source_epoch_status: observation_domain::SourceEpochStatus::Unknown,
        source_boundary: observation_domain::SourceBoundary::RuntimeStart,
    }
}

pub fn identity(source: &ProducerSource) -> CarrierIdentity {
    CarrierIdentity {
        session_id: source.session_id.clone(),
        producer_incarnation: source.producer_incarnation.clone(),
        epoch: observation_domain::TransportEpoch::new(source.transport_epoch).unwrap(),
    }
}

pub fn control(source: &ProducerSource, body: ControlBody) -> Vec<u8> {
    encode_carrier_control(
        &CarrierControl {
            identity: identity(source),
            body,
        },
        512,
    )
    .unwrap()
}

pub fn sample(value: &str) -> TypedFact {
    TypedFact {
        entity_id: "x4:runtime:realtime_clock".to_owned(),
        observation_version: 1,
        getter: "GetCurRealTime".to_owned(),
        semantics: "opaque_runtime_number".to_owned(),
        raw_value: value.to_owned(),
    }
}

pub const fn finish(end: u64) -> SectionFinishEvidence {
    SectionFinishEvidence {
        capture_end_millis: end,
        succeeded: true,
        quality: observation_domain::SectionQuality::Unknown,
        availability: observation_domain::SectionAvailability::Available,
        coverage: observation_domain::SectionCoverage::PointMeasurement,
        consistency: observation_domain::SourceConsistency::Unknown,
        stable_identity: false,
    }
}

fn handshake() -> ControlBody {
    ControlBody::Handshake(observation_ingest::HandshakeBody {
        native_abi: 2,
        envelope_contract: 2,
        schema_version: 1,
        policy_version: 2,
        canonicalization_version: 3,
        digest_version: 1,
    })
}

fn intent() -> ControlBody {
    ControlBody::CollectionIntent(CollectionIntentBody {
        section_key: "carrier_b_realtime_sample".to_owned(),
        max_records: 1,
        max_raw_bytes: 96,
        max_work: 2_048,
    })
}

fn hex(bytes: [u8; 32]) -> String {
    use core::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ignored = write!(text, "{byte:02x}");
        text
    })
}

pub fn digest(bytes: &[u8]) -> String {
    hex(complete_message_digest(bytes))
}
