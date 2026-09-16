use observation_domain::{SourceBoundary, SourceEpochStatus};
use observation_ingest::{CollectionIntentBody, ControlBody, DemandBody};
use x4_carrier_native::{ProducerError, ProducerOutcome, ProducerState, SectionEvidence};

use super::{ship_support::*, support};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "recovery trace keeps discard, reconciliation and new baseline assertions in causal order"
)]
fn discarded_completion_reconciles_without_replaying_capture_or_skipping_handshake() {
    for boundary in [
        SourceBoundary::GameLoaded,
        SourceBoundary::LuaReload,
        SourceBoundary::TransportReconnect,
    ] {
        let (mut producer, mut source) = completed_ship_section(20);
        for _ in 0..3 {
            take_current(&mut producer, &source, "received", 20);
        }
        let bytes = producer.pending_bytes().expect("completion").to_vec();
        let id = "message:complete:ship_core:1";
        let digest = support::digest(&bytes);
        producer.mark_local_handoff(20).unwrap();
        producer.fail_section(); // source uncertainty after completion handoff
        assert_eq!(producer.pending_bytes(), None);
        assert!(producer.reconcile_committed(id, "wrong").is_err());
        source.transport_epoch += 1;
        source.source_boundary = boundary;
        source.source_epoch_status = SourceEpochStatus::BoundaryUncertain;
        producer.reset(source.clone(), 21).unwrap();
        let bootstrap = producer.pending_bytes().unwrap().to_vec();
        let mut stale = source.clone();
        stale.transport_epoch -= 1;
        assert_eq!(
            producer.apply_control(&disposition(&stale, &bytes, "committed"), 21),
            Err(ProducerError::StaleEpoch)
        );
        assert_eq!(producer.pending_bytes(), Some(bootstrap.as_slice()));
        assert_eq!(
            producer.reconcile_committed(id, &digest),
            Ok(ProducerOutcome::Committed),
            "discarded handoff must retain exact outcome identity only"
        );
        assert_eq!(producer.pending_bytes(), Some(bootstrap.as_slice()));
        assert_eq!(producer.state(), ProducerState::AwaitingCompatibility);
        assert_eq!(
            producer.begin_ship_section(
                SectionEvidence::point_measurement("x4:faction:argon:ships"),
                1
            ),
            Err(ProducerError::InvalidInput)
        );
        assert_eq!(
            producer.push_ship_core(&ship("9007199254740997")),
            Err(ProducerError::InvalidTransition)
        );
        producer.mark_local_handoff(22).unwrap();
        for body in [
            support::handshake(),
            ControlBody::CollectionIntent(CollectionIntentBody {
                section_key: "ship_core".to_owned(),
                next_revision: 2,
                max_records: 4,
                max_raw_bytes: 512,
                max_work: 4,
            }),
            ControlBody::Demand(DemandBody { credit: 1 }),
        ] {
            producer
                .apply_control(&support::control(&source, body), 22)
                .unwrap();
        }
        producer
            .begin_ship_section(
                SectionEvidence::point_measurement("x4:faction:argon:ships"),
                1,
            )
            .unwrap();
        producer.push_ship_core(&ship("9007199254740997")).unwrap();
        producer.finish_section(support::finish(23)).unwrap();
        producer.progress(1, 23).unwrap();
        let start = take_current(&mut producer, &source, "received", 23);
        let batch = take_current(&mut producer, &source, "received", 23);
        assert_ne!(start, bytes);
        assert!(
            String::from_utf8(batch)
                .unwrap()
                .contains("9007199254740997")
        );
        take_current(&mut producer, &source, "committed", 23);
        assert_eq!(producer.pending_bytes(), None);
        assert!(producer.reconcile_committed(id, &digest).is_err());
    }
}

#[test]
fn unhanded_completion_and_incomplete_batches_cannot_reconcile_as_committed() {
    let (mut producer, source) = completed_ship_section(0);
    let start = producer.pending_bytes().unwrap().to_vec();
    producer.mark_local_handoff(0).unwrap();
    producer.fail_section();
    assert!(
        producer
            .reconcile_committed("message:start:ship_core:1", &support::digest(&start))
            .is_err()
    );
    producer.reset(source, 1).unwrap();
    assert_eq!(producer.state(), ProducerState::AwaitingCompatibility);
}
