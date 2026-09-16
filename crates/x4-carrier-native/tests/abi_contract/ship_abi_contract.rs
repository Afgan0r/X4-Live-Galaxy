use observation_domain::{CompleteMessage, SourceBoundary, SourceEpochStatus};
use observation_ingest::decode_carrier_bootstrap;
#[path = "ship_abi_host.rs"]
mod host;
#[path = "ship_abi_wire.rs"]
mod wire;
use host::Host;
use wire::*;
static PIPE_FIXTURE: std::sync::Mutex<()> = std::sync::Mutex::new(());
#[expect(
    clippy::too_many_lines,
    reason = "ordered DLL lifecycle trace remains one acceptance scenario"
)]
#[test]
fn registered_dll_ship_operations_copy_strict_tables_and_preserve_boundary_evidence() {
    let _fixture = PIPE_FIXTURE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for (mode, boundary) in [
        ("runtime_start", SourceBoundary::RuntimeStart),
        ("game_loaded", SourceBoundary::GameLoaded),
        ("lua_reload", SourceBoundary::LuaReload),
        ("transport_reconnect", SourceBoundary::TransportReconnect),
    ] {
        let mut host = Host::start(mode);
        let mut peer = connect();
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
        send(
            &mut peer,
            &identity,
            observation_ingest::ControlBody::Demand(observation_ingest::DemandBody { credit: 1 }),
        );
        host.wait_replacement();
        drop(peer);
        host.release_peer();
        let mut replacement = connect();
        let fresh_identity = qualify(&mut replacement);
        assert_ne!(fresh_identity, identity);
        let CompleteMessage::SectionStart(fresh_start) =
            receive(&mut replacement, &fresh_identity, "received")
        else {
            panic!("fresh start");
        };
        assert_eq!(fresh_start.source_scope, start.source_scope);
        assert_eq!(fresh_start.sender_evidence.source_boundary, boundary);
        let CompleteMessage::ImmutableBatch(fresh_batch) =
            receive(&mut replacement, &fresh_identity, "received")
        else {
            panic!("fresh batch");
        };
        assert!(
            fresh_batch.records[0]
                .content
                .contains("identity=9007199254740995")
        );
        assert!(fresh_batch.records[0].content.contains("location=sector:2"));
        let CompleteMessage::SectionCompletion(fresh_done) =
            receive(&mut replacement, &fresh_identity, "committed")
        else {
            panic!("fresh completion");
        };
        assert_eq!(fresh_done.record_count, 1);
        assert_eq!(
            fresh_done.sender_evidence.source_epoch_status,
            SourceEpochStatus::BoundaryUncertain
        );
        host.finish();
    }
}

#[test]
fn incompatible_ship_selection_requires_restart_through_registered_operations() {
    let _fixture = PIPE_FIXTURE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut host = Host::start("incompatible");
    let mut peer = connect();
    let identity = decode_carrier_bootstrap(&peer.receive(512).unwrap(), 512).unwrap();
    send(&mut peer, &identity, super_handshake());
    send(&mut peer, &identity, intent("unsupported_ship_profile"));
    host.finish();
}
