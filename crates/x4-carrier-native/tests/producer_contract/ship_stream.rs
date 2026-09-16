use observation_domain::{
    CaptureWindow, CompleteMessage, SectionAvailability, SectionCoverage, SectionFreshness,
    SectionQuality, SectionState, SourceConsistency,
};
use observation_ingest::decode_complete_message;
use x4_carrier_native::{ProducerError, ProducerOutcome, ProducerState, SectionEvidence};

use super::{
    ship_support::{completed_ship_section, disposition, ready_ship, ship, take_current},
    support,
};

#[test]
fn receipts_reopen_collection_before_the_section_finishes() {
    let (mut producer, source) = ready_ship(0);
    producer
        .begin_ship_section(
            SectionEvidence::point_measurement("x4:faction:argon:ships"),
            2,
        )
        .expect("reserve");
    producer.progress(1, 0).expect("start without records");
    let _start = take_current(&mut producer, &source, "received", 0);
    assert_eq!(producer.pending_bytes(), None);
    producer
        .push_ship_core(&ship("9007199254740993"))
        .expect("first");
    producer.progress(1, 1).expect("first batch before finish");
    let first = take_current(&mut producer, &source, "received", 1);
    assert!(
        matches!(decode_complete_message(&first, 4096).expect("batch"),
        CompleteMessage::ImmutableBatch(value) if value.section_ordinal == 1)
    );
    assert_eq!(producer.pending_bytes(), None);
    producer
        .push_ship_core(&ship("9007199254740995"))
        .expect("second after receipt");
    producer.progress(1, 2).expect("second batch");
    let _second = take_current(&mut producer, &source, "received", 2);
    assert_eq!(producer.pending_bytes(), None);
    producer
        .finish_section(support::finish(3))
        .expect("finish drained builder");
    producer.progress(1, 3).expect("completion");
    let done = take_current(&mut producer, &source, "committed", 3);
    assert!(
        matches!(decode_complete_message(&done, 4096).expect("completion"),
        CompleteMessage::SectionCompletion(value) if value.record_count == 2)
    );
}

#[test]
fn ship_section_accepts_multiple_complete_records() {
    let (mut producer, _) = ready_ship(0);
    let evidence = SectionEvidence::point_measurement("x4:faction:argon:ships");
    assert_eq!(producer.begin_ship_section(evidence, 2), Ok(()));
    assert_eq!(producer.push_ship_core(&ship("9007199254740993")), Ok(()));
    assert_eq!(producer.push_ship_core(&ship("9007199254740995")), Ok(()));
}

#[test]
fn ship_stream_emits_ordered_complete_record_batches_and_completion() {
    let (mut producer, source) = completed_ship_section(0);
    let start = take_current(&mut producer, &source, "received", 0);
    let first = take_current(&mut producer, &source, "received", 0);
    let second = take_current(&mut producer, &source, "received", 0);
    let completion = take_current(&mut producer, &source, "committed", 0);

    assert!(matches!(
        decode_complete_message(&start, 4_096).expect("start decodes"),
        CompleteMessage::SectionStart(value) if value.expected_records == 2
    ));
    for (bytes, ordinal, identity) in [
        (&first, 1, "x4:ship:9007199254740993"),
        (&second, 2, "x4:ship:9007199254740995"),
    ] {
        let CompleteMessage::ImmutableBatch(batch) =
            decode_complete_message(bytes, 4_096).expect("batch decodes")
        else {
            panic!("batch expected");
        };
        assert_eq!(batch.section_ordinal, ordinal);
        assert_eq!(batch.records.len(), 1);
        assert_eq!(batch.records[0].entity_id.as_str(), identity);
    }
    assert!(matches!(
        decode_complete_message(&completion, 4_096).expect("completion decodes"),
        CompleteMessage::SectionCompletion(value)
            if value.batch_count == 2 && value.record_count == 2
    ));
    assert_eq!(producer.state(), ProducerState::Ready);
}

#[test]
fn retry_keeps_exact_bytes_but_reconnect_discards_the_attempt() {
    let (mut producer, source) = completed_ship_section(10);
    let _start = take_current(&mut producer, &source, "received", 10);
    let bytes = producer
        .pending_bytes()
        .expect("first batch is pending")
        .to_vec();
    producer.mark_local_handoff(10).expect("handoff succeeds");
    assert_eq!(
        producer.apply_control(&disposition(&source, &bytes, "capacity_unavailable"), 11),
        Ok(ProducerOutcome::CapacityUnavailable)
    );
    assert_eq!(producer.pending_bytes(), Some(bytes.as_slice()));

    producer
        .observe_connection(1, 11)
        .expect("baseline generation");
    producer
        .observe_connection(2, 12)
        .expect("reconnect is accepted");
    assert_eq!(producer.state(), ProducerState::AwaitingCompatibility);
    assert_ne!(producer.pending_bytes(), Some(bytes.as_slice()));
}

#[test]
fn source_resets_discard_copied_records_for_load_reload_and_uncertainty() {
    use observation_domain::{SourceBoundary, SourceEpochStatus};
    for boundary in [SourceBoundary::GameLoaded, SourceBoundary::LuaReload] {
        let (mut producer, mut source) = completed_ship_section(0);
        let old = producer.pending_bytes().expect("old start").to_vec();
        source.transport_epoch += 1;
        source.source_boundary = boundary;
        source.source_epoch_status = SourceEpochStatus::BoundaryUncertain;
        producer.reset(source.clone(), 1).expect("source reset");
        assert_eq!(producer.state(), ProducerState::AwaitingCompatibility);
        assert_ne!(producer.pending_bytes(), Some(old.as_slice()));
        assert_eq!(
            producer.push_ship_core(&ship("9007199254740997")),
            Err(ProducerError::InvalidTransition)
        );
        assert_eq!(
            producer.finish_section(support::finish(2)),
            Err(ProducerError::InvalidTransition)
        );
    }
}

#[test]
fn exact_durable_reconciliation_never_republishes_completion() {
    let (mut producer, source) = completed_ship_section(20);
    let _start = take_current(&mut producer, &source, "received", 20);
    let _first = take_current(&mut producer, &source, "received", 20);
    let _second = take_current(&mut producer, &source, "received", 20);
    let completion = producer
        .pending_bytes()
        .expect("completion pending")
        .to_vec();
    let CompleteMessage::SectionCompletion(done) =
        decode_complete_message(&completion, 4_096).expect("completion decodes")
    else {
        panic!("completion expected");
    };
    let id = format!("message:complete:ship_core:{}", done.section_revision.get());
    let digest = support::digest(&completion);
    producer.mark_local_handoff(20).expect("handoff succeeds");
    assert_eq!(
        producer.apply_control(&disposition(&source, &completion, "ambiguous_commit"), 21),
        Ok(ProducerOutcome::Ambiguous)
    );
    assert_eq!(
        producer.reconcile_committed(&id, &digest),
        Ok(ProducerOutcome::Committed)
    );
    assert_eq!(producer.pending_bytes(), None);
    assert_eq!(
        producer.reconcile_committed(&id, &digest),
        Err(ProducerError::InvalidTransition)
    );
}

#[test]
fn zero_records_require_authoritative_empty_evidence() {
    let (mut producer, _) = ready_ship(0);
    assert_eq!(
        producer.begin_ship_section(
            SectionEvidence::point_measurement("x4:faction:argon:ships"),
            0
        ),
        Err(ProducerError::InvalidInput)
    );
    let window = CaptureWindow::new(0, 0).expect("window is ordered");
    let mut evidence = SectionEvidence::point_measurement("x4:faction:argon:ships");
    evidence.sender.section_state = SectionState::with_evidence(
        window,
        SectionFreshness::Fresh,
        SectionQuality::KnownEmpty,
        SectionAvailability::Available,
        SectionCoverage::KnownEmpty,
    );
    evidence.sender.source_consistency = SourceConsistency::Barrier;
    assert_eq!(producer.begin_ship_section(evidence, 0), Ok(()));
}
