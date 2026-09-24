use super::support;
use observation_domain::CompleteMessage;
use observation_ingest::{
    CollectionIntentBody, ControlBody, DemandBody, DispositionBody, decode_complete_message,
};
use x4_carrier_native::{
    FactionCensusRecord, Producer, ProducerLimits, ProducerOutcome, ProducerSource, SectionEvidence,
};

fn receive(producer: &mut Producer, source: &ProducerSource) -> CompleteMessage {
    let bytes = producer.pending_bytes().unwrap().to_vec();
    let message = decode_complete_message(&bytes, 4_096).unwrap();
    let (message_id, section_revision) = match &message {
        CompleteMessage::SectionStart(value) => (
            format!(
                "message:start:faction_census:{}",
                value.section_revision.get()
            ),
            value.section_revision.get(),
        ),
        CompleteMessage::ImmutableBatch(value) => (
            value.batch_id.as_str().to_owned(),
            value.section_revision.get(),
        ),
        CompleteMessage::SectionCompletion(value) => (
            format!(
                "message:complete:faction_census:{}",
                value.section_revision.get()
            ),
            value.section_revision.get(),
        ),
        CompleteMessage::Control(_) => panic!("data message expected"),
    };
    producer.mark_local_handoff(0).unwrap();
    let result = if matches!(message, CompleteMessage::SectionCompletion(_)) {
        "committed"
    } else {
        "received"
    };
    producer
        .apply_control(
            &support::control(
                source,
                ControlBody::Disposition(DispositionBody {
                    message_id,
                    section_key: "faction_census".to_owned(),
                    section_revision,
                    message_digest: support::digest(&bytes),
                    disposition: result.to_owned(),
                }),
            ),
            0,
        )
        .unwrap();
    message
}

fn ready_census(count: usize) -> (Producer, ProducerSource) {
    let source = support::source(7);
    let limits = ProducerLimits {
        data_message_bytes: 4_096,
        control_message_bytes: 512,
        max_records: count,
        max_raw_bytes: count * 256,
        max_batches: count,
        max_work: 1,
        max_retry_age_millis: 5_000,
    };
    let mut producer = Producer::new(limits, source.clone(), 0).unwrap();
    producer.mark_local_handoff(0).unwrap();
    for body in [
        support::handshake(),
        ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: "faction_census".to_owned(),
            next_revision: 1,
            max_records: count,
            max_raw_bytes: count * 256,
            max_work: 1,
        }),
        ControlBody::Demand(DemandBody { credit: 1 }),
    ] {
        producer
            .apply_control(&support::control(&source, body), 0)
            .unwrap();
    }
    (producer, source)
}

#[test]
fn faction_census_stream_crosses_decimal_width_boundaries() {
    for count in [12, 101] {
        run_census(count);
    }
}

fn run_census(count: usize) {
    let (mut producer, source) = ready_census(count);
    producer
        .begin_faction_census(SectionEvidence::point_measurement("x4:factions"), count)
        .unwrap();
    assert_eq!(producer.progress(1, 0), Ok(ProducerOutcome::Progress));
    assert!(matches!(
        receive(&mut producer, &source),
        CompleteMessage::SectionStart(_)
    ));
    for ordinal in 1..=count {
        producer
            .push_faction_census(&FactionCensusRecord {
                faction_id: format!("faction{ordinal}"),
                discovery_revision: 1,
                disposition: "included".to_owned(),
                reason: "listed".to_owned(),
                origin: "vanilla".to_owned(),
                source_evidence: "GetAllFactions".to_owned(),
                mind_candidate: true,
            })
            .unwrap();
        assert_eq!(
            producer.progress(1, 0),
            Ok(ProducerOutcome::Progress),
            "ordinal {ordinal}"
        );
        let CompleteMessage::ImmutableBatch(batch) = receive(&mut producer, &source) else {
            panic!("batch expected")
        };
        assert_eq!(batch.section_ordinal, ordinal);
        assert_eq!(
            batch.records[0].record_id.as_str(),
            format!("carrier-b:1:{ordinal:020}")
        );
    }
    producer.finish_section(support::finish(0)).unwrap();
    assert_eq!(producer.progress(1, 0), Ok(ProducerOutcome::Progress));
    let CompleteMessage::SectionCompletion(completion) = receive(&mut producer, &source) else {
        panic!("completion expected")
    };
    assert_eq!(
        (completion.batch_count, completion.record_count),
        (count, count)
    );
}
