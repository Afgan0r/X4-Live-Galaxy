use observation_ingest::{ControlBody, ResetBody};

use super::*;

#[test]
fn peer_reset_waits_for_reconnect_and_rotates_transport_epoch() {
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
    assert_eq!(identity.epoch.get(), source.transport_epoch + 1);
    assert_eq!(identity.producer_incarnation, source.producer_incarnation);
    assert_eq!(identity.session_id, source.session_id);
}

#[test]
fn observing_the_same_connection_generation_is_idempotent() {
    let (mut producer, source) = support::ready(22_000);
    producer.observe_connection(1, 21_999).unwrap();
    producer.observe_connection(2, 22_000).unwrap();
    let state = producer.state();
    let pending = producer.pending_bytes().map(<[u8]>::to_vec);
    let epoch = observation_ingest::decode_carrier_bootstrap(
        pending.as_deref().expect("replacement bootstrap"),
        512,
    )
    .expect("bootstrap decodes")
    .epoch;
    producer.observe_connection(2, 22_001).unwrap();
    assert_eq!(producer.state(), state);
    assert_eq!(producer.pending_bytes(), pending.as_deref());
    producer.observe_connection(1, 22_002).unwrap();
    assert_eq!(producer.state(), state);
    assert_eq!(producer.pending_bytes(), pending.as_deref());
    assert_eq!(
        observation_ingest::decode_carrier_bootstrap(
            producer.pending_bytes().expect("bootstrap retained"),
            512,
        )
        .expect("bootstrap decodes")
        .epoch,
        epoch
    );
    let reconnect_source = x4_carrier_native::ProducerSource {
        transport_epoch: source.transport_epoch + 1,
        ..source
    };
    finish_bootstrap(&mut producer, &reconnect_source, 22_002);
    collect(&mut producer, 22_002);
    assert_start(&producer, 1, reconnect_source.transport_epoch);
}

#[test]
fn reconnect_without_an_attempt_preserves_the_next_revision() {
    let now = 23_000;
    let (mut producer, source) = support::ready(now);
    producer.observe_connection(1, now).unwrap();
    producer.observe_connection(2, now + 1).unwrap();
    let reconnect_source = x4_carrier_native::ProducerSource {
        transport_epoch: source.transport_epoch + 1,
        ..source
    };
    finish_bootstrap(&mut producer, &reconnect_source, now);
    collect(&mut producer, now);
    assert_start(&producer, 1, reconnect_source.transport_epoch);
}

#[test]
fn reconnect_bootstrap_fences_and_discards_incomplete_attempt() {
    let now = 25_000;
    let (mut producer, source) = pending_section(now);
    producer.observe_connection(1, now).unwrap();
    let pending = producer.pending_bytes().unwrap().to_vec();
    producer.mark_local_handoff(now).unwrap();

    producer.observe_connection(2, now + 1).unwrap();
    assert_ne!(producer.pending_bytes().unwrap(), pending);
    let reconnect_source = x4_carrier_native::ProducerSource {
        transport_epoch: source.transport_epoch + 1,
        ..source
    };
    finish_bootstrap(&mut producer, &reconnect_source, now);
    assert_eq!(producer.state(), ProducerState::Ready);
    assert!(producer.pending_bytes().is_none());
    collect(&mut producer, now);
    assert_start(&producer, 2, reconnect_source.transport_epoch);
}

fn finish_bootstrap(
    producer: &mut x4_carrier_native::Producer,
    source: &x4_carrier_native::ProducerSource,
    now: u64,
) {
    producer.mark_local_handoff(now + 1).unwrap();
    for body in [
        support::handshake(),
        support::intent(),
        ControlBody::Demand(observation_ingest::DemandBody { credit: 1 }),
    ] {
        producer
            .apply_control(&support::control(source, body), now + 2)
            .unwrap();
    }
}

fn collect(producer: &mut x4_carrier_native::Producer, now: u64) {
    producer
        .begin_section(SectionEvidence::point_measurement(
            "x4:carrier_b_acceptance",
        ))
        .unwrap();
    producer.push_record(&support::sample("3")).unwrap();
    producer.finish_section(support::finish(now + 3)).unwrap();
    producer.progress(1, now + 3).unwrap();
}

fn assert_start(producer: &x4_carrier_native::Producer, revision: u64, epoch: u64) {
    let start =
        observation_ingest::decode_complete_message(producer.pending_bytes().unwrap(), 8_192)
            .unwrap();
    assert!(matches!(
        start,
        observation_domain::CompleteMessage::SectionStart(value)
            if value.section_revision.get() == revision && value.transport_epoch.get() == epoch
    ));
}
