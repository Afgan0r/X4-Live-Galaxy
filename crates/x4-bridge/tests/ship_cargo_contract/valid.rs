use super::*;

#[test]
fn valid_single_record_detail_batch_reaches_staging() {
    let database = carrier_b_support::database("cargo-valid-batch");
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
    assert_eq!(
        submit(
            &mut receiver,
            detail_batch(&messages[1], &key, CONTENT, "ore|17", "ore|17"),
            "detail:batch",
            5,
        ),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
}
