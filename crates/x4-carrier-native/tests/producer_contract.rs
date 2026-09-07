#![expect(
    clippy::unwrap_used,
    reason = "contract fixtures fail immediately when construction assumptions break"
)]

use observation_domain::{CompleteMessage, SourceBoundary, SourceConsistency, SourceEpochStatus};
use observation_ingest::{
    ControlBody, DemandBody, decode_carrier_bootstrap, decode_complete_message,
};
use x4_carrier_native::{
    Producer, ProducerError, ProducerLimits, ProducerOutcome, ProducerState, SectionEvidence,
};

#[path = "producer_contract/support.rs"]
mod support;
use support::{control, disposition, identity, pending, ready, sample, source, take};

#[test]
fn typed_fact_produces_v2_start_batch_and_completion() {
    let (mut producer, source) = ready(0);
    let mut evidence = SectionEvidence::point_measurement("x4:carrier_b_acceptance");
    evidence.sender.source_epoch_status = SourceEpochStatus::Unknown;
    evidence.sender.source_boundary = SourceBoundary::RuntimeStart;
    evidence.sender.source_consistency = SourceConsistency::Unknown;
    producer.begin_section(evidence.clone()).unwrap();
    let mut fact = sample("123.5");
    producer.push_record(&fact).unwrap();
    fact.raw_value = "mutated-after-admission".to_owned();
    producer.finish_section().unwrap();
    assert_eq!(producer.progress(1, 0), Ok(ProducerOutcome::Progress));
    let start = take(&mut producer, &source, "received", 1);
    let batch = take(&mut producer, &source, "received", 2);
    let completion = take(&mut producer, &source, "committed", 3);
    assert_eq!(
        support::digest(&start),
        "288c197bcb1afe2d1589dfb207b11cdbfe8281a0a32e5935f5de695a4d86328c"
    );
    assert_eq!(
        support::digest(&batch),
        "17aae5b571e64200337d9675477d4e402de13f3ca37712d6aea3c06bfa21eae0"
    );
    assert_eq!(
        support::digest(&completion),
        "f223cbfd8a1e83bb7d891b829244b89bc90c83c8abd249bf6e6db6233e89f64d"
    );
    let CompleteMessage::SectionStart(start) = decode_complete_message(&start, 2_048).unwrap()
    else {
        panic!("start expected")
    };
    assert_eq!(start.sender_evidence, evidence.sender);
    let CompleteMessage::ImmutableBatch(batch) = decode_complete_message(&batch, 2_048).unwrap()
    else {
        panic!("batch expected")
    };
    assert!(batch.records[0].content.contains("raw_value=123.5"));
    assert!(!batch.records[0].content.contains("mutated"));
    let CompleteMessage::SectionCompletion(done) =
        decode_complete_message(&completion, 2_048).unwrap()
    else {
        panic!("completion expected")
    };
    assert_eq!(done.sender_evidence, evidence.sender);
    assert_eq!((done.batch_count, done.record_count), (1, 1));
    let marker = b"\"ordered_batch_manifest_digest\":\"";
    let offset = completion
        .windows(marker.len())
        .position(|window| window == marker)
        .unwrap()
        + marker.len();
    let mut tampered = completion.clone();
    tampered[offset] = b'z';
    assert!(decode_complete_message(&tampered, 2_048).is_err());
    assert_eq!(producer.state(), ProducerState::Ready);
}

#[test]
fn bootstrap_retry_edges_and_ambiguity_are_fail_closed() {
    let source = source(7);
    let producer = Producer::new(ProducerLimits::bring_up(), source.clone(), 0).unwrap();
    assert_eq!(
        decode_carrier_bootstrap(producer.pending_bytes().unwrap(), 512),
        Ok(identity(&source))
    );
    for (elapsed, expected) in [
        (4_999, ProducerOutcome::CapacityUnavailable),
        (5_000, ProducerOutcome::CapacityUnavailable),
        (5_001, ProducerOutcome::PausedAfterFailure),
    ] {
        let (mut producer, source) = pending(0);
        let bytes = producer.pending_bytes().unwrap().to_vec();
        producer.mark_local_handoff(0).unwrap();
        assert_eq!(
            producer.apply_control(
                &disposition(&source, &bytes, "capacity_unavailable", 1),
                elapsed
            ),
            Ok(expected)
        );
        if elapsed <= 5_000 {
            assert_eq!(producer.pending_bytes(), Some(bytes.as_slice()));
        }
    }
    let (mut producer, source) = pending(0);
    let bytes = producer.pending_bytes().unwrap().to_vec();
    producer.mark_local_handoff(0).unwrap();
    assert_eq!(
        producer.apply_control(&disposition(&source, &bytes, "ambiguous_commit", 1), 1),
        Ok(ProducerOutcome::Ambiguous)
    );
    assert_eq!(producer.pending_bytes(), None);
}

#[test]
fn invalid_capacity_failure_and_stale_epoch_never_complete() {
    let (mut producer, source) = ready(0);
    producer
        .begin_section(SectionEvidence::point_measurement(
            "x4:carrier_b_acceptance",
        ))
        .unwrap();
    assert_eq!(
        producer.push_record(&sample(&"x".repeat(97))),
        Err(ProducerError::DataLimit)
    );
    assert_eq!(producer.state(), ProducerState::SectionReserved);
    producer.fail_section();
    assert_eq!(producer.pending_bytes(), None);
    let (mut producer, _) = ready(0);
    let mut stale = source.clone();
    stale.transport_epoch += 1;
    assert_eq!(
        producer.apply_control(
            &control(&stale, ControlBody::Demand(DemandBody { credit: 1 })),
            0
        ),
        Err(ProducerError::StaleEpoch)
    );
}
