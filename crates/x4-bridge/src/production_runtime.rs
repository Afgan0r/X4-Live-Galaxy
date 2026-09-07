use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use observation_application::LifecycleResult;
use observation_ingest::{
    CarrierControl, CollectionIntentBody, ControlBody, DemandBody, DispositionBody,
    ReceiverDisposition, complete_message_digest, decode_carrier_bootstrap,
    decode_complete_message, encode_carrier_control,
};
use x4_carrier_native::{BridgePeer, TransportConfig};

use crate::production_runtime_message::{
    digest_hex, disposition_name, identity as message_identity,
};
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
            max_records: limits.max_candidate_records,
            max_raw_bytes: limits.max_candidate_raw_bytes,
            max_work: limits.max_candidate_work,
        }),
        ControlBody::Demand(DemandBody { credit: 1 }),
    ] {
        if send(peer, &identity, body, limits).is_err() {
            let _ = history.record("rejected", "control-response-loss");
            return None;
        }
    }
    let _ = history.record("collection", "compatible-session");
    let connected_at = Instant::now();
    let mut active_scope = None;
    while let Ok(bytes) = peer.receive(limits.complete_message_bytes) {
        if connected_at.elapsed() > Duration::from_millis(limits.max_message_age_millis as u64) {
            let _ = history.record("rejected", "message-expired");
            return active_scope;
        }
        let Ok((scope, disposition)) = admit(peer, &identity, &bytes, limits, session) else {
            let _ = history.record("rejected", "whole-message");
            return active_scope;
        };
        active_scope = Some(scope);
        let state = if disposition == ReceiverDisposition::Committed {
            "committed"
        } else {
            "collection"
        };
        let _ = history.record(state, disposition_name(disposition));
    }
    let _ = history.record("waiting", "peer-disconnected");
    active_scope
}

fn admit(
    peer: &mut BridgePeer,
    identity: &observation_ingest::CarrierIdentity,
    bytes: &[u8],
    limits: &ProductionLimits,
    session: &mut ProductionObservationSession,
) -> Result<(observation_domain::SourceScopeId, ReceiverDisposition), ()> {
    let decoded = decode_complete_message(bytes, limits.complete_message_bytes).map_err(|_| ())?;
    let (message_id, section_key, section_revision, scope) = message_identity(&decoded, identity)?;
    let result = session
        .submit_received(
            identity.epoch,
            message_id.clone(),
            bytes.to_owned(),
            bytes.len(),
            now(),
        )
        .map_err(|_| ())?;
    let LifecycleResult::Disposition(disposition) = result else {
        return Err(());
    };
    let body = ControlBody::Disposition(DispositionBody {
        message_id: message_id.as_str().to_owned(),
        section_key,
        section_revision,
        message_digest: digest_hex(complete_message_digest(bytes)),
        disposition: disposition_name(disposition).to_owned(),
    });
    send(peer, identity, body, limits)?;
    Ok((scope, disposition))
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

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |v| u64::try_from(v.as_millis()).unwrap_or(u64::MAX))
}
