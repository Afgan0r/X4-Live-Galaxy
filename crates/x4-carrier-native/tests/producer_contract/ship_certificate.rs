use super::{
    ship_support::{ship, take_current},
    support,
};
use observation_domain::CompleteMessage;
use observation_ingest::{
    CollectionIntentBody, ContractVersions, ControlBody, DemandBody, bind_completion_certificate,
    decode_complete_message,
};
use x4_carrier_native::{Producer, ProducerError, ProducerLimits, SectionEvidence};

#[test]
fn large_incremental_ship_certificate_equals_the_receiver_batch_certificate() {
    let source = support::source(7);
    let limits = ProducerLimits {
        max_records: 129,
        max_batches: 129,
        max_work: 129,
        max_raw_bytes: 512,
        data_message_bytes: 4096,
        ..ProducerLimits::bring_up()
    };
    let mut producer = Producer::new(limits, source.clone(), 0).unwrap();
    producer.mark_local_handoff(0).unwrap();
    for body in [
        support::handshake(),
        ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: "ship_core".to_owned(),
            next_revision: 1,
            max_records: 129,
            max_raw_bytes: 512,
            max_work: 129,
        }),
        ControlBody::Demand(DemandBody { credit: 1 }),
    ] {
        producer
            .apply_control(&support::control(&source, body), 0)
            .unwrap();
    }
    assert_eq!(
        producer.begin_ship_section(
            SectionEvidence::point_measurement("x4:faction:argon:ships"),
            130
        ),
        Err(ProducerError::InvalidInput)
    );
    producer
        .begin_ship_section(
            SectionEvidence::point_measurement("x4:faction:argon:ships"),
            129,
        )
        .unwrap();
    producer.progress(1, 0).unwrap();
    take_current(&mut producer, &source, "received", 0);
    let mut batches = Vec::new();
    for ordinal in 1..=129 {
        let identity = (9_007_199_254_740_992_u64 + ordinal).to_string();
        producer.push_ship_core(&ship(&identity)).unwrap();
        producer.progress(1, ordinal).unwrap();
        let bytes = take_current(&mut producer, &source, "received", ordinal);
        assert!(bytes.len() <= limits.data_message_bytes);
        let CompleteMessage::ImmutableBatch(batch) = decode_complete_message(&bytes, 4096).unwrap()
        else {
            panic!("batch");
        };
        assert_eq!(batch.section_ordinal, usize::try_from(ordinal).unwrap());
        assert_eq!(batch.records.len(), 1);
        assert_eq!(
            batch.records[0].entity_id.as_str(),
            format!("x4:ship:{identity}")
        );
        batches.push(batch);
        assert_eq!(producer.pending_bytes(), None);
    }
    assert_eq!(
        producer.push_ship_core(&ship("1")),
        Err(ProducerError::InvalidTransition)
    );
    producer.finish_section(support::finish(130)).unwrap();
    producer.progress(1, 130).unwrap();
    let bytes = take_current(&mut producer, &source, "committed", 130);
    let CompleteMessage::SectionCompletion(done) = decode_complete_message(&bytes, 4096).unwrap()
    else {
        panic!("completion");
    };
    let versions = ContractVersions::new(
        done.schema_version,
        done.policy_version,
        done.canonicalization_version,
        done.digest_version,
    );
    let expected = bind_completion_certificate(done.clone(), &batches, versions).unwrap();
    assert_eq!(
        done, expected,
        "streaming hashes/counters equal canonical sorted batch evidence across ordinals 9/10/99/100"
    );
    assert_eq!((done.batch_count, done.record_count), (129, 129));
    assert!(
        done.raw_bytes > limits.max_raw_bytes,
        "builder bound is independent of total certificate bytes"
    );
}
