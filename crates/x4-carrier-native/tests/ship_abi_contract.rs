#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "real ABI fixtures fail immediately when their contract is violated"
)]

use observation_domain::{CompleteMessage, SourceBoundary, SourceEpochStatus};
use observation_ingest::{ControlBody, decode_carrier_bootstrap};
#[path = "ship_abi_host.rs"]
mod host;
#[path = "ship_abi_wire.rs"]
mod wire;
use host::Host;
use wire::*;
#[test]
fn registered_dll_ship_operations_copy_strict_tables_and_preserve_boundary_evidence() {
    for (mode, boundary) in [
        ("runtime_start", SourceBoundary::RuntimeStart),
        ("game_loaded", SourceBoundary::GameLoaded),
        ("lua_reload", SourceBoundary::LuaReload),
        ("transport_reconnect", SourceBoundary::TransportReconnect),
    ] {
        let mut host = Host::start(mode);
        let mut peer = match std::panic::catch_unwind(connect) {
            Ok(peer) => peer,
            Err(_) => {
                host.finish();
                panic!("peer failed");
            }
        };
        let identity = qualify(&mut peer);
        let CompleteMessage::SectionStart(start) = receive(&mut peer, &identity, "received") else {
            panic!("start");
        };
        assert_eq!(start.sender_evidence.source_boundary, boundary);
        assert_eq!(
            start.sender_evidence.source_epoch_status,
            SourceEpochStatus::BoundaryUncertain
        );
        assert_eq!(start.source_scope.as_str(), "x4:faction:argon:ships");
        assert_eq!(start.section_revision.get(), 7);
        let CompleteMessage::ImmutableBatch(batch) = receive(&mut peer, &identity, "received")
        else {
            panic!("batch");
        };
        assert_eq!(
            batch.records[0].content,
            "profile=ship_core\nidentity=9007199254740993\nowner=argon\ntype=destroyer_macro\nclass=destroyer\nlocation=sector:1"
        );
        let CompleteMessage::SectionCompletion(done) = receive(&mut peer, &identity, "committed")
        else {
            panic!("completion");
        };
        assert_eq!(done.sender_evidence.source_boundary, boundary);
        assert_eq!(
            done.sender_evidence.source_epoch_status,
            SourceEpochStatus::BoundaryUncertain
        );
        assert_eq!(done.record_count, 1);
        host.finish();
    }
}

#[test]
fn incompatible_ship_selection_requires_restart_through_registered_operations() {
    let mut host = Host::start("incompatible");
    let mut peer = match std::panic::catch_unwind(connect) {
        Ok(peer) => peer,
        Err(_) => {
            host.finish();
            panic!("peer failed");
        }
    };
    let identity = decode_carrier_bootstrap(&peer.receive(512).unwrap(), 512).unwrap();
    send(&mut peer, &identity, super_handshake());
    send(&mut peer, &identity, intent("unsupported_ship_profile"));
    host.finish();
}
