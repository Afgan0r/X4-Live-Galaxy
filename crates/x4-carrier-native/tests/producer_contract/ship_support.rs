use observation_domain::{
    CompleteMessage, SenderEvidence, ShipClass, ShipCoreRecord, ShipIdentity, ShipLocation,
    ShipOwner, ShipType, SourceScopeId,
};
use observation_ingest::{
    CollectionIntentBody, ControlBody, DemandBody, DispositionBody, decode_complete_message,
};
use x4_carrier_native::{
    Producer, ProducerLimits, ProducerOutcome, ProducerSource, SectionEvidence,
};

use super::support;

pub(super) fn ready_ship(now: u64) -> (Producer, ProducerSource) {
    let source = support::source(7);
    let limits = ProducerLimits {
        data_message_bytes: 4_096,
        control_message_bytes: 512,
        max_records: 4,
        max_raw_bytes: 512,
        max_batches: 4,
        max_work: 4,
        max_retry_age_millis: 5_000,
    };
    let mut producer = Producer::new(limits, source.clone(), now).expect("producer is valid");
    producer
        .mark_local_handoff(now)
        .expect("bootstrap handoff succeeds");
    for body in [
        support::handshake(),
        ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: "ship_core".to_owned(),
            next_revision: 1,
            max_records: 4,
            max_raw_bytes: 512,
            max_work: 4,
        }),
        ControlBody::Demand(DemandBody { credit: 1 }),
    ] {
        producer
            .apply_control(&support::control(&source, body), now)
            .expect("ship profile control is accepted");
    }
    (producer, source)
}

pub(super) fn ship(identity: &str) -> ShipCoreRecord {
    ShipCoreRecord::new(
        SourceScopeId::new("x4:faction:argon:ships").expect("scope is valid"),
        ShipIdentity::new(identity).expect("identity is canonical"),
        ShipOwner::new("argon").expect("owner is valid"),
        ShipType::new("ship_arg_l_destroyer_01_a_macro").expect("type is valid"),
        ShipClass::new("destroyer").expect("class is valid"),
        ShipLocation::new("sector:argon_prime").expect("location is valid"),
        SenderEvidence::legacy_default(),
    )
}

pub(super) fn completed_ship_section(now: u64) -> (Producer, ProducerSource) {
    let (mut producer, source) = ready_ship(now);
    producer
        .begin_ship_section(
            SectionEvidence::point_measurement("x4:faction:argon:ships"),
            2,
        )
        .expect("ship section starts");
    producer
        .push_ship_core(&ship("9007199254740993"))
        .expect("first ship accepted");
    producer
        .push_ship_core(&ship("9007199254740995"))
        .expect("second ship accepted");
    producer
        .finish_section(support::finish(now))
        .expect("section finishes");
    assert_eq!(producer.progress(2, now), Ok(ProducerOutcome::Progress));
    (producer, source)
}

pub(super) fn take_current(
    producer: &mut Producer,
    source: &ProducerSource,
    result: &str,
    now: u64,
) -> Vec<u8> {
    let bytes = producer.pending_bytes().expect("message pending").to_vec();
    producer.mark_local_handoff(now).expect("handoff succeeds");
    producer
        .apply_control(&disposition(source, &bytes, result), now)
        .expect("feedback accepted");
    bytes
}

pub(super) fn disposition(source: &ProducerSource, bytes: &[u8], result: &str) -> Vec<u8> {
    let message = decode_complete_message(bytes, 4_096).expect("message decodes");
    let (id, key, revision) = identity(&message);
    support::control(
        source,
        ControlBody::Disposition(DispositionBody {
            message_id: id,
            section_key: key,
            section_revision: revision,
            message_digest: support::digest(bytes),
            disposition: result.to_owned(),
        }),
    )
}

fn identity(message: &CompleteMessage) -> (String, String, u64) {
    match message {
        CompleteMessage::SectionStart(value) => (
            format!(
                "message:start:{}:{}",
                value.section_key.as_str(),
                value.section_revision.get()
            ),
            value.section_key.as_str().to_owned(),
            value.section_revision.get(),
        ),
        CompleteMessage::ImmutableBatch(value) => (
            value.batch_id.as_str().to_owned(),
            value.section_key.as_str().to_owned(),
            value.section_revision.get(),
        ),
        CompleteMessage::SectionCompletion(value) => (
            format!(
                "message:complete:{}:{}",
                value.section_key.as_str(),
                value.section_revision.get()
            ),
            value.section_key.as_str().to_owned(),
            value.section_revision.get(),
        ),
        CompleteMessage::Control(_) => panic!("data message expected"),
    }
}
