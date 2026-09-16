#![expect(clippy::expect_used, reason = "contract fixtures fail immediately")]
mod carrier_b_support;
mod ship_support;
use observation_application::LifecycleResult;
use observation_domain::{BatchId, CompleteMessage, SectionKey, TransportEpoch};
use observation_ingest::{ReceiverDisposition, decode_complete_message, encode_complete_message};
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use x4_bridge::ProductionObservationSession;

const CONTENT: &str = "profile=ship_cargo\nidentity=9007199254740993\nowner=argon\ncore_revision=1\nmember_revision=1\npolicy=2\ncapture_start=10\ncapture_end=11\nsource=x4-9.00-steam-23660954-ship-detail-source-v1\nwares_outcome=value\nstorage_outcome=value\nreservation_policy=excluded\nware=ore|17\nstorage=solid|1000|170";

#[test]
fn cargo_dependency_and_nested_failures_preserve_accepted_core() {
    for (index, (from, to)) in [
        ("core_revision=1", "core_revision=9"),
        ("member_revision=1", "member_revision=9"),
        ("owner=argon", "owner=teladi"),
        ("policy=2", "policy=3"),
        ("capture_end=11", "capture_end=9"),
        ("9007199254740993", "9007199254740994"),
        ("ore|17", "ore|NaN"),
        ("ore|17", "ore|9007199254740992"),
        (
            "source=x4-9.00-steam-23660954-ship-detail-source-v1",
            "source=unverified",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        run_case(index, from, to);
    }
}
fn run_case(index: usize, from: &str, to: &str) {
    let database = carrier_b_support::database(&format!("cargo-negative-{index}"));
    let mut receiver = ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("session");
    receiver.select_ship_core("argon").expect("selection");
    let messages = ship_support::messages(1, "argon");
    publish_core(&mut receiver, &messages);
    let key = SectionKey::new("ship_cargo:g0").expect("key");
    assert_eq!(
        submit(
            &mut receiver,
            detail_start(&messages[0], &key),
            "detail:start",
            4
        ),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert!(
        submit(
            &mut receiver,
            detail_batch(&messages[1], &key, from, to),
            "detail:batch",
            5
        )
        .is_err(),
        "reject {to}"
    );
    drop(receiver);
    let repository =
        SqliteObservationRepository::open(database.path(), carrier_b_support::publication_limits())
            .expect("reopen");
    assert!(repository.current(&key).expect("readback").is_none());
    assert_eq!(
        repository
            .current(&SectionKey::new("ship_core").expect("key"))
            .expect("readback")
            .expect("core")
            .receipt()
            .revision
            .get(),
        1
    );
}
fn publish_core(receiver: &mut ProductionObservationSession, messages: &[Vec<u8>]) {
    for (ordinal, bytes) in messages.iter().enumerate() {
        let expected = if ordinal == 3 {
            ReceiverDisposition::Committed
        } else {
            ReceiverDisposition::Received
        };
        assert_eq!(
            submit(
                receiver,
                bytes.clone(),
                &format!("core:{ordinal}"),
                ordinal as u64
            ),
            Ok(LifecycleResult::Disposition(expected))
        );
    }
}
fn submit(
    receiver: &mut ProductionObservationSession,
    bytes: Vec<u8>,
    id: &str,
    now: u64,
) -> Result<LifecycleResult, x4_bridge::ProductionError> {
    receiver.submit_received(
        TransportEpoch::new(7).expect("epoch"),
        BatchId::new(id).expect("id"),
        bytes,
        1,
        now,
    )
}
fn detail_start(bytes: &[u8], key: &SectionKey) -> Vec<u8> {
    let mut message = decode_complete_message(bytes, 4096).expect("decode");
    if let CompleteMessage::SectionStart(value) = &mut message {
        value.section_key = key.clone();
        value.section_revision = observation_domain::SectionRevisionId::new(2).expect("revision");
        value.expected_records = 2;
    }
    encode_complete_message(&message, 4096).expect("encode")
}
fn detail_batch(bytes: &[u8], key: &SectionKey, from: &str, to: &str) -> Vec<u8> {
    let mut message = decode_complete_message(bytes, 4096).expect("decode");
    if let CompleteMessage::ImmutableBatch(value) = &mut message {
        value.section_key = key.clone();
        value.section_revision = observation_domain::SectionRevisionId::new(2).expect("revision");
        value.records[0].content = CONTENT.replace(from, to);
        value.records[0].record_id =
            observation_domain::RecordId::new("carrier-b:2:00000000000000000001").expect("id");
        value.records[0].observation_version =
            observation_domain::ObservationVersion::new(2).expect("version");
    }
    encode_complete_message(&message, 4096).expect("encode")
}
