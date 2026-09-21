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

#[test]
fn per_ship_stale_marker_reaches_staging_without_rejecting_the_batch() {
    let database = carrier_b_support::database("cargo-stale-record-batch");
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
    assert!(
        submit(
            &mut receiver,
            detail_start(&messages[0], &key),
            "detail:start",
            4
        )
        .is_ok()
    );
    let content = CONTENT
        .replace("consistency=consistent", "consistency=possibly_stale")
        .replace(
            "consistency_reason=none",
            "consistency_reason=location_changed",
        );
    assert!(
        submit(
            &mut receiver,
            detail_batch(&messages[1], &key, &content, "ore|17", "ore|17"),
            "detail:batch",
            5,
        )
        .is_ok()
    );
}

#[test]
fn configured_full_parent_detail_batch_accepts_every_ordered_member() {
    let database = carrier_b_support::database("cargo-full-parent-batch");
    let mut receiver = ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("session");
    receiver.select_ship_core("argon").expect("selection");
    receiver
        .configure_heavy(
            x4_bridge::HeavyShipLimits::parse(include_str!(
                "../../../../config/heavy-ship-experiment.json"
            ))
            .expect("profile"),
        )
        .expect("configured");
    let messages = ship_support::messages(1, "argon");
    publish_core(&mut receiver, &messages);
    let key = SectionKey::new("ship_cargo:g0").expect("key");
    assert!(
        submit(
            &mut receiver,
            detail_start(&messages[0], &key),
            "detail:start",
            4
        )
        .is_ok()
    );

    let mut message = decode_complete_message(&messages[1], 4096).expect("decode");
    let CompleteMessage::ImmutableBatch(batch) = &mut message else {
        panic!("core fixture contains a batch");
    };
    batch.section_key = key;
    batch.section_revision = observation_domain::SectionRevisionId::new(2).expect("revision");
    for (index, record) in batch.records.iter_mut().enumerate() {
        let identity = record
            .entity_id
            .as_str()
            .strip_prefix("x4:ship:")
            .expect("identity");
        record.content = CONTENT.replace("9007199254740993", identity);
        record.record_id =
            observation_domain::RecordId::new(format!("carrier-b:2:{:020}", index + 1))
                .expect("record id");
        record.observation_version =
            observation_domain::ObservationVersion::new(2).expect("version");
    }
    let bytes = encode_complete_message(&message, 4096).expect("encode");
    assert!(submit(&mut receiver, bytes, "detail:batch", 5).is_ok());
}
