use observation_ingest::{ControlBody, DemandBody, decode_carrier_bootstrap};
use x4_carrier_native::{
    Producer, ProducerError, ProducerLimits, ProducerOutcome, ProducerSource, ProducerState,
    SectionEvidence,
};

use super::support::{control, disposition, identity, ready, sample, source};

fn pending(now: u64) -> (Producer, ProducerSource) {
    let (mut producer, source) = ready(now);
    producer
        .begin_section(SectionEvidence::point_measurement(
            "x4:carrier_b_acceptance",
        ))
        .unwrap();
    producer.push_record(&sample("3")).unwrap();
    producer
        .finish_section(super::support::finish(now))
        .unwrap();
    producer.progress(1, now).unwrap();
    (producer, source)
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
                elapsed,
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
    let mut stale = source;
    stale.transport_epoch += 1;
    assert_eq!(
        producer.apply_control(
            &control(&stale, ControlBody::Demand(DemandBody { credit: 1 })),
            0,
        ),
        Err(ProducerError::StaleEpoch)
    );
}

#[test]
fn exact_limits_conflicting_feedback_and_reset_fail_closed() {
    let (mut producer, _) = ready(10);
    producer
        .begin_section(SectionEvidence::point_measurement(
            "x4:carrier_b_acceptance",
        ))
        .unwrap();
    assert_eq!(
        producer.push_record(&sample("")),
        Err(ProducerError::DataLimit)
    );
    producer
        .push_record(&sample(&format!("1.{}", "0".repeat(94))))
        .unwrap();
    producer.finish_section(super::support::finish(10)).unwrap();
    producer.progress(1, 10).unwrap();
    let original = source(7);
    let bytes = producer.pending_bytes().unwrap().to_vec();
    producer.mark_local_handoff(10).unwrap();
    assert_eq!(
        producer.apply_control(&disposition(&original, &bytes, "received", 2), 10),
        Err(ProducerError::InvalidInput)
    );
    assert_eq!(
        producer.apply_control(
            &disposition(&original, &bytes, "permanently_rejected", 1),
            10,
        ),
        Ok(ProducerOutcome::PermanentlyRejected)
    );
    assert_eq!(producer.state(), ProducerState::PausedAfterFailure);
    assert_eq!(producer.pending_bytes(), None);

    let fresh = source(8);
    producer.reset(fresh.clone(), 11).unwrap();
    assert_eq!(producer.state(), ProducerState::AwaitingCompatibility);
    assert_eq!(
        decode_carrier_bootstrap(producer.pending_bytes().unwrap(), 512),
        Ok(identity(&fresh))
    );
}
