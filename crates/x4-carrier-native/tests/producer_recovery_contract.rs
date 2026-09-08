#![expect(
    clippy::unwrap_used,
    reason = "contract fixtures fail immediately when construction assumptions break"
)]
#![expect(
    dead_code,
    reason = "shared producer fixture exposes helpers used by its sibling contract"
)]

use observation_ingest::{ControlBody, ResetBody};
use x4_carrier_native::{ProducerError, ProducerOutcome, ProducerState, SectionEvidence};

#[path = "producer_recovery_contract/helpers.rs"]
mod recovery_helpers;
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
    recovery_helpers::retry_then_receive(&mut producer, &source, 2, now + 2);
    recovery_helpers::retry_then_receive(&mut producer, &source, 3, now + 4);
    assert_eq!(producer.state(), ProducerState::Ready);
}

#[test]
fn peer_reset_waits_for_reconnect_and_preserves_session_identity() {
    let (mut producer, source) = support::ready(20_000);
    producer.observe_connection(1, 20_000).unwrap();
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
    assert_eq!(producer.state(), ProducerState::Ready);
    producer.observe_connection(2, 20_002).unwrap();
    assert_eq!(producer.state(), ProducerState::AwaitingCompatibility);
    let identity =
        observation_ingest::decode_carrier_bootstrap(producer.pending_bytes().unwrap(), 512)
            .unwrap();
    assert_eq!(identity.epoch.get(), source.transport_epoch);
    assert_eq!(identity.producer_incarnation, source.producer_incarnation);
    assert_eq!(identity.session_id, source.session_id);
}

#[test]
fn observing_the_same_connection_generation_is_idempotent() {
    let (mut producer, _) = support::ready(22_000);
    producer.observe_connection(1, 22_000).unwrap();
    let state = producer.state();
    let pending = producer.pending_bytes().map(<[u8]>::to_vec);
    producer.observe_connection(1, 22_001).unwrap();
    assert_eq!(producer.state(), state);
    assert_eq!(producer.pending_bytes(), pending.as_deref());
}

#[test]
fn reconnect_bootstrap_fences_and_restores_exact_pending_bytes() {
    let now = 25_000;
    let (mut producer, source) = pending_section(now);
    producer.observe_connection(1, now).unwrap();
    let pending = producer.pending_bytes().unwrap().to_vec();
    producer.mark_local_handoff(now).unwrap();

    producer.observe_connection(2, now + 1).unwrap();
    let bootstrap = producer.pending_bytes().unwrap().to_vec();
    assert_ne!(bootstrap, pending);
    producer.mark_local_handoff(now + 1).unwrap();
    for body in [
        support::handshake(),
        support::intent(),
        ControlBody::Demand(observation_ingest::DemandBody { credit: 1 }),
    ] {
        producer
            .apply_control(&support::control(&source, body), now + 2)
            .unwrap();
    }
    assert_eq!(producer.state(), ProducerState::PendingStart);
    assert_eq!(producer.pending_bytes(), Some(pending.as_slice()));
}

#[test]
fn each_message_stage_rejects_a_later_stage_disposition() {
    assert_wrong_stage_rejected(ProducerState::PendingStart, "committed");
    assert_wrong_stage_rejected(ProducerState::PendingBatch, "committed");
    assert_wrong_stage_rejected(ProducerState::PendingCompletion, "received");
}

#[test]
fn retry_budget_allows_exactly_one_retry() {
    let now = 30_000;
    let (mut producer, source) = pending_section(now);
    let bytes = producer.pending_bytes().unwrap().to_vec();
    producer.mark_local_handoff(now).unwrap();
    assert_eq!(
        producer.apply_control(
            &support::disposition(&source, &bytes, "capacity_unavailable", 1),
            now + 1,
        ),
        Ok(ProducerOutcome::CapacityUnavailable)
    );
    producer.mark_local_handoff(now + 1).unwrap();
    assert_eq!(
        producer.apply_control(
            &support::disposition(&source, &bytes, "capacity_unavailable", 1),
            now + 2,
        ),
        Ok(ProducerOutcome::PausedAfterFailure)
    );
    assert_eq!(producer.state(), ProducerState::PausedAfterFailure);
}

#[test]
fn first_handoff_uses_actual_high_epoch_time() {
    let (mut producer, source) = pending_section(0);
    let bytes = producer.pending_bytes().unwrap().to_vec();
    let handoff = 9_000_000_000;
    producer.mark_local_handoff(handoff).unwrap();
    assert_eq!(
        producer.apply_control(
            &support::disposition(&source, &bytes, "capacity_unavailable", 1),
            handoff + 1,
        ),
        Ok(ProducerOutcome::CapacityUnavailable)
    );
    assert_eq!(producer.pending_bytes(), Some(bytes.as_slice()));
}

fn assert_wrong_stage_rejected(state: ProducerState, disposition: &str) {
    let now = 40_000;
    let (mut producer, source) = pending_section(now);
    let stages = match state {
        ProducerState::PendingStart => 0,
        ProducerState::PendingBatch => 1,
        ProducerState::PendingCompletion => 2,
        _ => unreachable!("test selects pending states only"),
    };
    for ordinal in 1..=stages {
        let bytes = producer.pending_bytes().unwrap().to_vec();
        producer.mark_local_handoff(now).unwrap();
        producer
            .apply_control(
                &support::disposition(&source, &bytes, "received", ordinal),
                now,
            )
            .unwrap();
    }
    let ordinal = stages + 1;
    let bytes = producer.pending_bytes().unwrap().to_vec();
    producer.mark_local_handoff(now).unwrap();
    assert_eq!(
        producer.apply_control(
            &support::disposition(&source, &bytes, disposition, ordinal),
            now,
        ),
        Err(ProducerError::InvalidTransition)
    );
    assert_eq!(producer.state(), state);
}

fn pending_section(
    now: u64,
) -> (
    x4_carrier_native::Producer,
    x4_carrier_native::ProducerSource,
) {
    let (mut producer, source) = support::ready(now);
    producer
        .begin_section(SectionEvidence::point_measurement(
            "x4:carrier_b_acceptance",
        ))
        .unwrap();
    producer.push_record(&support::sample("3")).unwrap();
    producer.finish_section(support::finish(now)).unwrap();
    producer.progress(1, now).unwrap();
    (producer, source)
}
