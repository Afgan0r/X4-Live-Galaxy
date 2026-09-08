#![expect(
    clippy::panic,
    reason = "bounded integration watchdog fails immediately"
)]

use observation_ingest::{
    CollectionIntentBody, ControlBody, DemandBody, DispositionBody, HandshakeBody,
    complete_message_digest,
};
use x4_carrier_native::{
    BridgePeer, HandleToken, NativeTransport, Producer, ProducerLimits, ProducerOutcome,
    ProducerSource, ProducerState, SectionEvidence, SectionFinishEvidence, TransportConfig,
    TransportSendOutcome, TypedFact,
};

use super::startup_support::await_control;

#[path = "producer_matrix/support.rs"]
mod support;

pub fn run() {
    for (label, disposition, expected) in [
        (
            "permanent",
            "permanently_rejected",
            ProducerOutcome::PermanentlyRejected,
        ),
        ("ambiguous", "ambiguous_commit", ProducerOutcome::Ambiguous),
    ] {
        let (mut producer, transport, mut peer, token, source, bytes) = pending(label);
        send_disposition(&mut peer, &source, &bytes, disposition);
        let control = await_control(&transport, token);
        assert_eq!(producer.apply_control(&control, 2), Ok(expected));
        assert_eq!(producer.state(), ProducerState::PausedAfterFailure);
        support::close(&transport, token);
    }
    retry_exhaustion();
}

fn retry_exhaustion() {
    let (mut producer, transport, mut peer, token, source, bytes) = pending("retry-exhaustion");
    send_disposition(&mut peer, &source, &bytes, "capacity_unavailable");
    let control = await_control(&transport, token);
    assert_eq!(
        producer.apply_control(&control, 2),
        Ok(ProducerOutcome::CapacityUnavailable)
    );
    assert_eq!(producer.pending_bytes(), Some(bytes.as_slice()));
    send_pending(&mut producer, &transport, token);
    assert_eq!(peer.receive(8_192), Ok(bytes.clone()));
    send_disposition(&mut peer, &source, &bytes, "capacity_unavailable");
    let control = await_control(&transport, token);
    assert_eq!(
        producer.apply_control(&control, 3),
        Ok(ProducerOutcome::PausedAfterFailure)
    );
    assert_eq!(producer.state(), ProducerState::PausedAfterFailure);
    support::close(&transport, token);
}

fn pending(
    label: &str,
) -> (
    Producer,
    NativeTransport,
    BridgePeer,
    HandleToken,
    ProducerSource,
    Vec<u8>,
) {
    let config = TransportConfig {
        pipe_name: format!(
            r"\\.\pipe\live-galaxy-matrix-{}-{label}",
            std::process::id()
        ),
        max_data_message_bytes: 8_192,
        max_control_message_bytes: 2_048,
    };
    let token = HandleToken {
        generation: 1,
        slot: 1,
    };
    let transport = NativeTransport::start(config.clone(), token).expect("transport");
    let mut peer = BridgePeer::connect(&config, std::time::Duration::from_secs(2)).expect("peer");
    let source = source(label);
    let mut producer =
        Producer::new(ProducerLimits::bring_up(), source.clone(), 1).expect("producer");
    producer
        .observe_connection(transport.snapshot().connection_generation, 1)
        .expect("generation");
    send_pending(&mut producer, &transport, token);
    let _bootstrap = peer.receive(8_192).expect("bootstrap");
    negotiate(&mut producer, &transport, &mut peer, token, &source);
    collect(&mut producer, &source);
    let bytes = producer.pending_bytes().expect("pending").to_vec();
    send_pending(&mut producer, &transport, token);
    assert_eq!(peer.receive(8_192), Ok(bytes.clone()));
    (producer, transport, peer, token, source, bytes)
}

fn source(label: &str) -> ProducerSource {
    ProducerSource {
        session_id: format!("session-{label}"),
        producer_incarnation: "producer:matrix".to_owned(),
        transport_epoch: 1,
        source_scope: "x4:carrier_b_acceptance".to_owned(),
        source_epoch_status: observation_domain::SourceEpochStatus::Unknown,
        source_boundary: observation_domain::SourceBoundary::RuntimeStart,
    }
}

fn negotiate(
    producer: &mut Producer,
    transport: &NativeTransport,
    peer: &mut BridgePeer,
    token: HandleToken,
    source: &ProducerSource,
) {
    for body in [
        ControlBody::Handshake(HandshakeBody {
            native_abi: 2,
            envelope_contract: 2,
            schema_version: 1,
            policy_version: 2,
            canonicalization_version: 3,
            digest_version: 1,
        }),
        ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: "carrier_b_realtime_sample".to_owned(),
            max_records: 1,
            max_raw_bytes: 96,
            max_work: 1,
        }),
        ControlBody::Demand(DemandBody { credit: 1 }),
    ] {
        peer.send_control(&support::encode(source, body))
            .expect("control");
        let control = await_control(transport, token);
        assert_eq!(
            producer.apply_control(&control, 1),
            Ok(ProducerOutcome::Accepted)
        );
    }
}

fn collect(producer: &mut Producer, source: &ProducerSource) {
    producer
        .begin_section(SectionEvidence::point_measurement(
            source.source_scope.clone(),
        ))
        .expect("section");
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
    producer.progress(1, 1).expect("progress");
}

fn send_pending(producer: &mut Producer, transport: &NativeTransport, token: HandleToken) {
    let bytes = producer.pending_bytes().expect("sendable pending");
    loop {
        match transport.try_send(token, bytes) {
            TransportSendOutcome::LocalHandoff => break,
            TransportSendOutcome::CapacityUnavailable => std::thread::yield_now(),
            outcome @ TransportSendOutcome::Rejected(_) => panic!("send rejected: {outcome:?}"),
        }
    }
    producer.mark_local_handoff(1).expect("handoff");
}

fn send_disposition(peer: &mut BridgePeer, source: &ProducerSource, bytes: &[u8], value: &str) {
    peer.send_control(&support::encode(
        source,
        ControlBody::Disposition(DispositionBody {
            message_id: "message:start:carrier_b_realtime_sample:1".to_owned(),
            section_key: "carrier_b_realtime_sample".to_owned(),
            section_revision: 1,
            message_digest: support::hex(complete_message_digest(bytes)),
            disposition: value.to_owned(),
        }),
    ))
    .expect("disposition");
}
