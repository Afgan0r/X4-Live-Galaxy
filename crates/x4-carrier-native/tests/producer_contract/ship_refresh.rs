use super::{
    ship_support::{completed_ship_section, take_current},
    support,
};
use observation_ingest::{CollectionIntentBody, ControlBody};
use x4_carrier_native::ProducerState;

#[test]
fn superseded_heavy_capture_accepts_only_fresh_core_and_keeps_refusals_terminal() {
    for (feedback, recoverable) in [
        ("timed_out_or_superseded", true),
        ("permanently_rejected", false),
        ("ambiguous_commit", false),
    ] {
        let (mut producer, source) = completed_ship_section(0);
        let _ = take_current(&mut producer, &source, feedback, 0);
        let intent = |key: &str| {
            support::control(
                &source,
                ControlBody::CollectionIntent(CollectionIntentBody {
                    section_key: key.into(),
                    next_revision: 2,
                    max_records: 4,
                    max_raw_bytes: 512,
                    max_work: 4,
                }),
            )
        };
        assert!(producer.apply_control(&intent("ship_crew:g0"), 1).is_err());
        assert_eq!(
            producer.apply_control(&intent("ship_core"), 1).is_ok(),
            recoverable
        );
        if recoverable {
            assert_eq!(producer.state(), ProducerState::Ready);
        }
    }
}
