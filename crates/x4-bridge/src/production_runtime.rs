use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use observation_ingest::{
    CarrierControl, CollectionIntentBody, ControlBody, DemandBody, ReceiverDisposition,
    decode_carrier_bootstrap, encode_carrier_control,
};
use x4_carrier_native::{BridgePeer, TransportConfig};

use crate::production_runtime_idle::await_progress;
use crate::production_runtime_message::disposition_name;
use crate::{OperationalHistory, PIPE_ENDPOINT, ProductionLimits, ProductionObservationSession};

pub fn run(
    limits: &ProductionLimits,
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
) -> ! {
    let config = TransportConfig {
        pipe_name: PIPE_ENDPOINT.to_owned(),
        max_data_message_bytes: limits.complete_message_bytes,
        max_control_message_bytes: limits.control_message_bytes,
    };
    loop {
        history.bind_session("", 0);
        let _ = history.record("waiting", "peer-absent");
        let scope = connect(&config, limits)
            .and_then(|mut peer| serve(&mut peer, limits, history, session));
        if let Some(scope) = scope {
            session.invalidate_source_scope(&scope);
        }
        std::thread::sleep(Duration::from_millis(
            limits.availability_interval_millis as u64,
        ));
    }
}

fn connect(config: &TransportConfig, limits: &ProductionLimits) -> Option<BridgePeer> {
    let timeout = Duration::from_millis(limits.reconnect_delay_millis as u64);
    (0..limits.reconnect_attempts).find_map(|_| BridgePeer::connect(config, timeout).ok())
}

fn serve(
    peer: &mut BridgePeer,
    limits: &ProductionLimits,
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
) -> Option<observation_domain::SourceScopeId> {
    let Ok(bootstrap) = peer.receive(limits.control_message_bytes) else {
        let _ = history.record("rejected", "bootstrap-read");
        return None;
    };
    let Ok(identity) = decode_carrier_bootstrap(&bootstrap, limits.control_message_bytes) else {
        let _ = history.record("rejected", "bootstrap-incompatible");
        return None;
    };
    let next_revision = revision_floor(session, history)?;
    history.bind_session(&identity.session_id, identity.epoch.get());
    if send_initial_controls(peer, &identity, next_revision, limits).is_err() {
        let _ = history.record("rejected", "control-response-loss");
        return None;
    }
    let _ = history.record("collection", "compatible-session");
    let mut last_progress = Instant::now();
    let mut active_scope = None;
    while let Ok(bytes) = await_progress(peer, limits, history, session, last_progress) {
        last_progress = Instant::now();
        history.bind_message("", "", 0);
        let (scope, disposition) = match crate::production_admission::admit(
            &identity,
            &bytes,
            limits.complete_message_bytes,
            history,
            session,
            |body| send(peer, &identity, body, limits),
        ) {
            Ok(value) => value,
            Err(error) => {
                return crate::production_admission::finish_error(&error, history, active_scope);
            }
        };
        active_scope = Some(scope);
        let state = if disposition == ReceiverDisposition::Committed {
            "committed"
        } else {
            "collection"
        };
        let _ = history.record(state, disposition_name(disposition));
        if disposition != ReceiverDisposition::Committed {
            continue;
        }
        if let Err(reason) = send_next_demand(peer, &identity, limits) {
            let _ = history.record("rejected", reason);
            return active_scope;
        }
        last_progress = Instant::now();
    }
    let _ = history.record("waiting", "peer-disconnected");
    active_scope
}

fn send_next_demand(
    peer: &mut BridgePeer,
    identity: &observation_ingest::CarrierIdentity,
    limits: &ProductionLimits,
) -> Result<(), &'static str> {
    let interval = Duration::from_millis(limits.availability_interval_millis as u64);
    match peer.receive_timeout(limits.complete_message_bytes, interval) {
        Ok(None) => {}
        Ok(Some(_)) => return Err("message-before-demand"),
        Err(_) => return Err("peer-disconnected"),
    }
    send(
        peer,
        identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
        limits,
    )
    .map_err(|()| "control-response-loss")
}

fn send_initial_controls(
    peer: &mut BridgePeer,
    identity: &observation_ingest::CarrierIdentity,
    next_revision: u64,
    limits: &ProductionLimits,
) -> Result<(), ()> {
    for body in [
        ControlBody::Handshake(observation_ingest::HandshakeBody {
            native_abi: 2,
            envelope_contract: 2,
            schema_version: 1,
            policy_version: 2,
            canonicalization_version: 3,
            digest_version: 1,
        }),
        ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: "carrier_b_realtime_sample".to_owned(),
            next_revision,
            max_records: limits.max_candidate_records,
            max_raw_bytes: limits.max_candidate_raw_bytes,
            max_work: limits.max_candidate_work,
        }),
        ControlBody::Demand(DemandBody { credit: 1 }),
    ] {
        send(peer, identity, body, limits)?;
    }
    Ok(())
}

fn revision_floor(
    session: &ProductionObservationSession,
    history: &mut OperationalHistory,
) -> Option<u64> {
    let key = observation_domain::SectionKey::new("carrier_b_realtime_sample")?;
    session.next_revision(&key).map_or_else(
        |_| {
            let _ = history.record("rejected", "revision-floor-unavailable");
            None
        },
        Some,
    )
}

fn send(
    peer: &mut BridgePeer,
    identity: &observation_ingest::CarrierIdentity,
    body: ControlBody,
    limits: &ProductionLimits,
) -> Result<(), ()> {
    let bytes = encode_carrier_control(
        &CarrierControl {
            identity: identity.clone(),
            body,
        },
        limits.control_message_bytes,
    )
    .map_err(|_| ())?;
    peer.send_control(&bytes).map_err(|_| ())
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |v| u64::try_from(v.as_millis()).unwrap_or(u64::MAX))
}
