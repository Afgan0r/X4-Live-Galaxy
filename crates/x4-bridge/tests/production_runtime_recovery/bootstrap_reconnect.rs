#![expect(
    clippy::panic,
    reason = "bounded integration watchdog fails immediately"
)]

use observation_ingest::{ControlBody, decode_carrier_bootstrap, decode_carrier_control};
use x4_carrier_native::{Producer, ProducerLimits, ProducerOutcome, ProducerState};

use super::{Harness, TEST_LOCK, startup_support};

#[path = "bootstrap_reconnect/durable.rs"]
mod durable;
#[path = "bootstrap_reconnect/support.rs"]
mod support;

#[derive(Clone, Copy, Debug)]
enum DisconnectCut {
    PendingStart,
    PendingBatch,
    PendingCompletion,
    CommitBeforeDisposition,
}

#[test]
fn replacement_bridge_fences_every_pending_stage_and_uncertain_commit() {
    let _guard = TEST_LOCK.lock().expect("test lock");
    for cut in [
        DisconnectCut::PendingStart,
        DisconnectCut::PendingBatch,
        DisconnectCut::PendingCompletion,
        DisconnectCut::CommitBeforeDisposition,
    ] {
        run_replacement(cut);
    }
}

fn run_replacement(cut: DisconnectCut) {
    let label = format!("producer-reconnect-{cut:?}");
    let mut harness = Harness::start(&label, startup_support::valid_limits());
    let source = support::producer_source(&label);
    let now = harness
        .transport
        .snapshot()
        .monotonic_millis
        .expect("clock");
    let mut producer =
        Producer::new(ProducerLimits::bring_up(), source.clone(), now).expect("producer");
    support::negotiate(&mut producer, &harness.transport, harness.token, 1);
    support::collect(&mut producer, &source, now, "123.0");
    advance_to_cut(&mut producer, &harness, &source, cut);

    let old_generation = harness.transport.snapshot().connection_generation;
    harness.child.kill().expect("old bridge killed");
    harness.child.wait().expect("old bridge reaped");
    harness.child = startup_support::spawn_bridge(
        harness.directory.path(),
        &harness.directory.path().join("limits.json"),
    );
    support::wait_for_generation(&harness.transport, old_generation);

    let snapshot = harness.transport.snapshot();
    producer
        .observe_connection(
            snapshot.connection_generation,
            snapshot.monotonic_millis.expect("clock"),
        )
        .expect("replacement observed");
    assert_eq!(producer.state(), ProducerState::AwaitingCompatibility);
    let bootstrap = producer.pending_bytes().expect("replacement bootstrap");
    assert_eq!(
        decode_carrier_bootstrap(bootstrap, 2_048).expect("bootstrap"),
        support::identity(&source, 2)
    );
    support::negotiate_observed(&mut producer, &harness.transport, harness.token, 2);
    assert_eq!(producer.state(), ProducerState::Ready);
    assert!(producer.pending_bytes().is_none());

    support::collect(&mut producer, &source, now, "123.0");
    support::publish_pending(&mut producer, &harness.transport, harness.token);
    assert_eq!(producer.state(), ProducerState::Ready);
    durable::assert_state(harness.directory.path(), cut);
    harness.stop();
}

fn advance_to_cut(
    producer: &mut Producer,
    harness: &Harness,
    source: &x4_carrier_native::ProducerSource,
    cut: DisconnectCut,
) {
    if matches!(cut, DisconnectCut::PendingStart) {
        support::send_and_drop_disposition(producer, &harness.transport, harness.token);
        return;
    }
    support::send_and_apply(
        producer,
        &harness.transport,
        harness.token,
        ProducerOutcome::Received,
    );
    if matches!(cut, DisconnectCut::PendingBatch) {
        support::send_and_drop_disposition(producer, &harness.transport, harness.token);
        return;
    }
    support::send_and_apply(
        producer,
        &harness.transport,
        harness.token,
        ProducerOutcome::Received,
    );
    if matches!(cut, DisconnectCut::PendingCompletion) {
        return;
    }
    support::send_pending(producer, &harness.transport, harness.token);
    let bytes = startup_support::await_control(&harness.transport, harness.token);
    let identity = support::identity(source, 1);
    let control = decode_carrier_control(&bytes, &identity, 2_048).expect("committed disposition");
    assert!(matches!(
        control.body,
        ControlBody::Disposition(value) if value.disposition == "committed"
    ));
}
