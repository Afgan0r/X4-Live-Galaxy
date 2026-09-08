#![expect(
    clippy::panic,
    reason = "bounded integration watchdog fails immediately"
)]

use std::time::Duration;

use x4_carrier_native::{
    HandleToken, NativeTransport, Producer, ProducerLimits, ProducerOutcome, ProducerSource,
    ProducerState, SectionEvidence, SectionFinishEvidence, TransportSendOutcome, TypedFact,
};

use super::{Harness, TEST_LOCK, startup_support};

#[test]
fn replacement_bridge_requires_bootstrap_before_exact_pending_replay() {
    let _guard = TEST_LOCK.lock().expect("test lock");
    let mut harness = Harness::start("producer-reconnect", startup_support::valid_limits());
    let source = producer_source("producer-reconnect");
    let now = harness
        .transport
        .snapshot()
        .monotonic_millis
        .expect("clock");
    let mut producer =
        Producer::new(ProducerLimits::bring_up(), source.clone(), now).expect("producer");
    negotiate(&mut producer, &harness.transport, harness.token);
    collect(&mut producer, &source, now);
    let immutable = producer.pending_bytes().expect("pending start").to_vec();
    send_pending(&mut producer, &harness.transport, harness.token);

    let old_generation = harness.transport.snapshot().connection_generation;
    harness.child.kill().expect("old bridge killed");
    harness.child.wait().expect("old bridge reaped");
    harness.child = startup_support::spawn_bridge(
        harness.directory.path(),
        &harness.directory.path().join("limits.json"),
    );
    wait_for_generation(&harness.transport, old_generation);
    negotiate(&mut producer, &harness.transport, harness.token);
    assert_eq!(producer.state(), ProducerState::PendingStart);
    assert_eq!(producer.pending_bytes(), Some(immutable.as_slice()));

    for expected in [
        ProducerOutcome::Received,
        ProducerOutcome::Received,
        ProducerOutcome::Committed,
    ] {
        send_pending(&mut producer, &harness.transport, harness.token);
        let bytes = startup_support::await_control(&harness.transport, harness.token);
        let current = harness
            .transport
            .snapshot()
            .monotonic_millis
            .expect("clock");
        assert_eq!(producer.apply_control(&bytes, current), Ok(expected));
    }
    assert_eq!(producer.state(), ProducerState::Ready);
    assert!(producer.pending_bytes().is_none());
    assert_actual_readback(harness.directory.path());
    let history =
        std::fs::read_to_string(harness.directory.path().join("operational-history.jsonl"))
            .expect("history");
    assert!(history.contains("\"session\":\"session-producer-reconnect\""));
    assert!(history.contains("\"epoch\":1"));
    harness.stop();
}

fn collect(producer: &mut Producer, source: &ProducerSource, now: u64) {
    producer
        .begin_section(SectionEvidence::point_measurement(
            source.source_scope.clone(),
        ))
        .expect("reserve");
    producer
        .push_record(&TypedFact {
            entity_id: "x4:runtime:realtime_clock".to_owned(),
            observation_version: 1,
            getter: "GetCurRealTime".to_owned(),
            semantics: "opaque_runtime_number".to_owned(),
            raw_value: "123.0".to_owned(),
        })
        .expect("fact");
    producer
        .finish_section(SectionFinishEvidence {
            capture_end_millis: 0,
            succeeded: true,
            quality: observation_domain::SectionQuality::Unknown,
            availability: observation_domain::SectionAvailability::Available,
            coverage: observation_domain::SectionCoverage::PointMeasurement,
            consistency: observation_domain::SourceConsistency::Unknown,
            stable_identity: false,
        })
        .expect("finish");
    producer.progress(1, now).expect("assemble");
}

fn producer_source(label: &str) -> ProducerSource {
    ProducerSource {
        session_id: format!("session-{label}"),
        producer_incarnation: "producer:1".to_owned(),
        transport_epoch: 1,
        source_scope: "x4:carrier_b_acceptance".to_owned(),
        source_epoch_status: observation_domain::SourceEpochStatus::Unknown,
        source_boundary: observation_domain::SourceBoundary::RuntimeStart,
    }
}

fn negotiate(producer: &mut Producer, transport: &NativeTransport, token: HandleToken) {
    let snapshot = transport.snapshot();
    producer
        .observe_connection(
            snapshot.connection_generation,
            snapshot.monotonic_millis.expect("clock"),
        )
        .expect("observe connection");
    send_pending(producer, transport, token);
    for _ in 0..3 {
        let bytes = startup_support::await_control(transport, token);
        let now = transport.snapshot().monotonic_millis.expect("clock");
        assert_eq!(
            producer.apply_control(&bytes, now),
            Ok(ProducerOutcome::Accepted)
        );
    }
}

fn send_pending(producer: &mut Producer, transport: &NativeTransport, token: HandleToken) {
    let bytes = producer.pending_bytes().expect("sendable pending");
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    loop {
        match transport.try_send(token, bytes) {
            TransportSendOutcome::LocalHandoff => break,
            TransportSendOutcome::CapacityUnavailable if std::time::Instant::now() < deadline => {
                std::thread::yield_now();
            }
            TransportSendOutcome::CapacityUnavailable => panic!("send watchdog expired"),
            outcome @ TransportSendOutcome::Rejected(_) => panic!("send failed: {outcome:?}"),
        }
    }
    let now = transport.snapshot().monotonic_millis.expect("clock");
    producer.mark_local_handoff(now).expect("handoff");
}

fn wait_for_generation(transport: &NativeTransport, previous: u64) {
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while transport.snapshot().connection_generation <= previous
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(transport.snapshot().connection_generation > previous);
}

fn assert_actual_readback(directory: &std::path::Path) {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_x4-bridge"))
        .args([
            "--readback",
            "--data-dir",
            directory.to_str().expect("path"),
        ])
        .args([
            "--section-key",
            "carrier_b_realtime_sample",
            "--section-revision",
            "1",
        ])
        .output()
        .expect("readback");
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("typed JSON");
    assert_eq!(value["section_key"], "carrier_b_realtime_sample");
    assert_eq!(value["section_revision"], 1);
    assert_eq!(value["records"].as_array().map(Vec::len), Some(1));
    assert_eq!(value["records"][0]["record_id"], "carrier-b:1:1");
    assert_eq!(
        value["records"][0]["entity_id"],
        "x4:runtime:realtime_clock"
    );
    assert_eq!(value["records"][0]["observation_version"], 1);
    assert_eq!(
        value["records"][0]["content"],
        "getter=GetCurRealTime\nraw_value=123.0\nsemantics=opaque_runtime_number"
    );
    assert_eq!(value["receipt"]["ordinal"], 1);
    assert!(value["receipt"]["accepted_at"].as_u64().is_some());
}
