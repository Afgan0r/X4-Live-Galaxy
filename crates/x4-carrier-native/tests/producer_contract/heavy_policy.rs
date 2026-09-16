use super::{ship_support, support};
use observation_ingest::{CollectionIntentBody, ControlBody};
use x4_carrier_native::{ProducerAdmissionPolicy, ProducerError, SectionEvidence};

#[test]
fn native_heavy_policy_refuses_disagreed_intents_before_collection() {
    for case in 0..3 {
        let (mut producer, source) = ship_support::ready_ship(0);
        producer
            .configure_admission(ProducerAdmissionPolicy {
                max_inner_records: 1,
                max_attempts: 1,
                max_age_millis: 30_000,
            })
            .unwrap();
        producer.reset(source.clone(), 0).unwrap();
        producer.mark_local_handoff(0).unwrap();
        producer
            .apply_control(&support::control(&source, support::handshake()), 0)
            .unwrap();
        let intent = CollectionIntentBody {
            section_key: "ship_core".into(),
            next_revision: 1,
            max_records: if case == 0 { 5 } else { 4 },
            max_raw_bytes: if case == 1 { 513 } else { 512 },
            max_work: if case == 2 { 5 } else { 4 },
        };
        assert!(
            producer
                .apply_control(
                    &support::control(&source, ControlBody::CollectionIntent(intent)),
                    0
                )
                .is_err()
        );
        assert_eq!(
            producer.begin_ship_section(
                SectionEvidence::point_measurement("x4:faction:argon:ships"),
                1
            ),
            Err(ProducerError::InvalidInput)
        );
    }
}
#[test]
fn native_collection_age_and_backward_clock_are_independently_enforced() {
    let (mut producer, _) = ship_support::ready_ship(10);
    producer
        .configure_admission(ProducerAdmissionPolicy {
            max_inner_records: 1,
            max_attempts: 1,
            max_age_millis: 2,
        })
        .unwrap();
    assert_eq!(
        producer.progress(1, 9),
        Err(ProducerError::ClockUnavailable)
    );
    producer
        .begin_ship_section(
            SectionEvidence::point_measurement("x4:faction:argon:ships"),
            1,
        )
        .unwrap();
    producer.progress(1, 10).unwrap();
    assert_eq!(producer.progress(1, 13), Err(ProducerError::DataLimit));
    assert_eq!(producer.pending_bytes(), None);
}
