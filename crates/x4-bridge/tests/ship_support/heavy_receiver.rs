use super::{
    CONTENT, carrier_b_support, detail_batch, detail_start, publish_core, ship_support, submit,
};
use observation_domain::{CompleteMessage, SectionKey};
use observation_ingest::{decode_complete_message, encode_complete_message};
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use x4_bridge::{HeavyShipLimits, ProductionObservationSession};

#[test]
fn configured_receiver_rejects_stale_backward_duplicate_and_wrong_group_without_publication() {
    for case in 0..4 {
        run_case(case);
    }
}
fn run_case(case: usize) {
    let database = carrier_b_support::database(&format!("heavy-independent-{case}"));
    let mut receiver = ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("session");
    receiver.select_ship_core("argon").expect("selection");
    let limits = HeavyShipLimits::parse(include_str!(
        "../../../../config/heavy-ship-experiment.json"
    ))
    .expect("profile");
    receiver.configure_heavy(limits).expect("configured");
    let messages = ship_support::messages(1, "argon");
    publish_core(&mut receiver, &messages);
    let key = SectionKey::new("ship_cargo:g0").expect("key");
    let mut start =
        decode_complete_message(&detail_start(&messages[0], &key), 4096).expect("decode");
    if let CompleteMessage::SectionStart(value) = &mut start {
        value.expected_records = if case == 3 { 2 } else { 1 };
    }
    let now = match case {
        0 => 30004,
        1 => 2,
        _ => 4,
    };
    let result = submit(
        &mut receiver,
        encode_complete_message(&start, 4096).expect("encode"),
        "detail:start",
        now,
    );
    if case == 2 {
        assert!(result.is_ok(), "fresh exact single-member group");
        let bytes = detail_batch(
            &messages[1],
            &key,
            CONTENT,
            "ware=ore|17",
            "ware=ore|17\nware=ore|1",
        );
        assert!(
            submit(&mut receiver, bytes, "detail:batch", 5).is_err(),
            "receiver rejects duplicate nested ware identities independently of DLL"
        );
    } else {
        assert!(result.is_err(), "configured refusal {case}");
    }
    drop(receiver);
    assert_unchanged(database.path(), &key);
}
fn assert_unchanged(path: &std::path::Path, key: &SectionKey) {
    let repository =
        SqliteObservationRepository::open(path, carrier_b_support::publication_limits())
            .expect("reopen");
    assert!(repository.current(key).expect("detail read").is_none());
    assert_eq!(
        repository
            .current(&SectionKey::new("ship_core").expect("key"))
            .expect("core read")
            .expect("core")
            .receipt()
            .revision
            .get(),
        1
    );
}
