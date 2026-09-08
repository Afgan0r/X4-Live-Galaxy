#![expect(
    clippy::unwrap_used,
    reason = "contract fixtures fail immediately when construction assumptions break"
)]
#![expect(
    dead_code,
    reason = "shared producer fixture exposes helpers used by its sibling contract"
)]

use observation_ingest::{ControlBody, ResetBody};
use x4_carrier_native::{ProducerOutcome, ProducerState, SectionEvidence};

#[path = "producer_contract/support.rs"]
mod support;

#[test]
fn high_epoch_retry_budget_covers_batch_and_completion_handoffs() {
    let now = 10_000;
    let (mut producer, source) = support::ready(now);
    producer
        .begin_section(SectionEvidence::point_measurement(
            "x4:carrier_b_acceptance",
        ))
        .unwrap();
    producer.push_record(&support::sample("3")).unwrap();
    producer.finish_section(support::finish(now)).unwrap();
    producer.progress(1, now).unwrap();

    let start = producer.pending_bytes().unwrap().to_vec();
    producer.mark_local_handoff(now).unwrap();
    producer
        .apply_control(
            &support::disposition(&source, &start, "received", 1),
            now + 1,
        )
        .unwrap();
    retry_then_receive(&mut producer, &source, 2, now + 2);
    retry_then_receive(&mut producer, &source, 3, now + 4);
    assert_eq!(producer.state(), ProducerState::Ready);
}

#[test]
fn peer_reset_creates_a_fresh_bootstrap_identity() {
    let (mut producer, source) = support::ready(20_000);
    let reset = support::control(
        &source,
        ControlBody::Reset(ResetBody {
            reason: "peer-restarted".to_owned(),
        }),
    );
    assert_eq!(
        producer.apply_control(&reset, 20_001),
        Ok(ProducerOutcome::Disconnected)
    );
    assert_eq!(producer.state(), ProducerState::AwaitingCompatibility);
    let identity =
        observation_ingest::decode_carrier_bootstrap(producer.pending_bytes().unwrap(), 512)
            .unwrap();
    assert_ne!(identity.epoch.get(), source.transport_epoch);
    assert_ne!(identity.producer_incarnation, source.producer_incarnation);
}

fn retry_then_receive(
    producer: &mut x4_carrier_native::Producer,
    source: &x4_carrier_native::ProducerSource,
    ordinal: usize,
    now: u64,
) {
    let bytes = producer.pending_bytes().unwrap().to_vec();
    producer.mark_local_handoff(now).unwrap();
    assert_eq!(
        producer.apply_control(
            &support::disposition(source, &bytes, "capacity_unavailable", ordinal),
            now + 1,
        ),
        Ok(ProducerOutcome::CapacityUnavailable)
    );
    assert_eq!(producer.pending_bytes(), Some(bytes.as_slice()));
    producer.mark_local_handoff(now + 1).unwrap();
    producer
        .apply_control(
            &support::disposition(
                source,
                &bytes,
                if ordinal == 3 {
                    "committed"
                } else {
                    "received"
                },
                ordinal,
            ),
            now + 2,
        )
        .unwrap();
}
