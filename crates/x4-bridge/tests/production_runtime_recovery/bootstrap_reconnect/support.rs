use std::time::Duration;

use observation_domain::{SourceBoundary, SourceEpochStatus, TransportEpoch};
use observation_ingest::CarrierIdentity;
use x4_carrier_native::{
    HandleToken, NativeTransport, Producer, ProducerOutcome, ProducerSource, SectionEvidence,
    SectionFinishEvidence, TransportSendOutcome, TypedFact,
};

use super::startup_support;

pub fn producer_source(label: &str) -> ProducerSource {
    ProducerSource {
        session_id: format!("session-{label}"),
        producer_incarnation: "producer:1".to_owned(),
        transport_epoch: 1,
        source_scope: "x4:carrier_b_acceptance".to_owned(),
        source_epoch_status: SourceEpochStatus::Unknown,
        source_boundary: SourceBoundary::RuntimeStart,
    }
}

pub fn identity(source: &ProducerSource, epoch: u64) -> CarrierIdentity {
    CarrierIdentity {
        session_id: source.session_id.clone(),
        producer_incarnation: source.producer_incarnation.clone(),
        epoch: TransportEpoch::new(epoch).expect("epoch"),
    }
}

pub fn negotiate(
    producer: &mut Producer,
    transport: &NativeTransport,
    token: HandleToken,
    epoch: u64,
) {
    let snapshot = transport.snapshot();
    producer
        .observe_connection(
            snapshot.connection_generation,
            snapshot.monotonic_millis.expect("clock"),
        )
        .expect("observe connection");
    negotiate_observed(producer, transport, token, epoch);
}

pub fn negotiate_observed(
    producer: &mut Producer,
    transport: &NativeTransport,
    token: HandleToken,
    epoch: u64,
) {
    let bootstrap = producer.pending_bytes().expect("bootstrap");
    assert_eq!(decode_bootstrap_epoch(bootstrap), epoch);
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

fn decode_bootstrap_epoch(bytes: &[u8]) -> u64 {
    observation_ingest::decode_carrier_bootstrap(bytes, 2_048)
        .expect("bootstrap")
        .epoch
        .get()
}

pub fn collect(producer: &mut Producer, source: &ProducerSource, now: u64, value: &str) {
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
            raw_value: value.to_owned(),
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

pub fn publish_pending(producer: &mut Producer, transport: &NativeTransport, token: HandleToken) {
    for expected in [
        ProducerOutcome::Received,
        ProducerOutcome::Received,
        ProducerOutcome::Committed,
    ] {
        send_and_apply(producer, transport, token, expected);
    }
}

pub fn send_and_apply(
    producer: &mut Producer,
    transport: &NativeTransport,
    token: HandleToken,
    expected: ProducerOutcome,
) {
    send_pending(producer, transport, token);
    let bytes = startup_support::await_control(transport, token);
    let now = transport.snapshot().monotonic_millis.expect("clock");
    assert_eq!(producer.apply_control(&bytes, now), Ok(expected));
}

pub fn send_and_drop_disposition(
    producer: &mut Producer,
    transport: &NativeTransport,
    token: HandleToken,
) {
    send_pending(producer, transport, token);
    let _lost = startup_support::await_control(transport, token);
}

pub fn send_pending(producer: &mut Producer, transport: &NativeTransport, token: HandleToken) {
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

pub fn wait_for_generation(transport: &NativeTransport, previous: u64) {
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while transport.snapshot().connection_generation <= previous
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(transport.snapshot().connection_generation > previous);
}
