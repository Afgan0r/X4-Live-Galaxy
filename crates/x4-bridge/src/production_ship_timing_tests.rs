fn completion() -> observation_domain::CompleteMessage {
    use observation_domain::*;
    let mut sender = SenderEvidence::legacy_default();
    sender.section_state = observation_domain::SectionState::with_evidence(
        observation_domain::CaptureWindow::new(0, 30_000).expect("accelerated game interval"),
        observation_domain::SectionFreshness::Fresh,
        observation_domain::SectionQuality::Unknown,
        observation_domain::SectionAvailability::Available,
        observation_domain::SectionCoverage::Partial,
    );
    CompleteMessage::SectionCompletion(SectionCompletionEnvelope {
        source_scope: scope(),
        producer_incarnation: ProducerIncarnationId::new("producer:ship").expect("producer"),
        transport_epoch: identity().epoch,
        section_key: SectionKey::new("ship_cargo:g0").expect("key"),
        section_revision: SectionRevisionId::new(1).expect("revision"),
        batch_count: 1,
        record_count: 1,
        raw_bytes: 1,
        ordered_batch_manifest_digest: [0; 32],
        canonical_content_digest: [0; 32],
        schema_version: sender.schema_version,
        policy_version: sender.policy_version,
        canonicalization_version: sender.canonicalization_version,
        digest_version: sender.digest_version,
        coverage: CompletionCoverage::Partial,
        sender_evidence: sender,
    })
}

fn scoped_completion(kind: &str) -> observation_domain::CompleteMessage {
    let mut message = completion();
    let observation_domain::CompleteMessage::SectionCompletion(value) = &mut message else {
        panic!("completion")
    };
    value.section_key =
        observation_domain::SectionKey::new(format!("{kind}:argon:g0")).expect("scoped detail key");
    message
}

fn identity() -> observation_ingest::CarrierIdentity {
    observation_ingest::CarrierIdentity {
        session_id: "session:ship".into(),
        producer_incarnation: "producer:ship".into(),
        epoch: observation_domain::TransportEpoch::new(7).expect("epoch"),
    }
}

fn scope() -> observation_domain::SourceScopeId {
    observation_domain::SourceScopeId::new("x4:faction:argon:ships").expect("scope")
}

#[test]
fn accelerated_game_capture_does_not_inflate_receiver_wall_cost() {
    let receiver_start = 10_000;
    let receiver_commit = 15_000;
    let mut timing = super::super::timing::ShipTiming::default();
    timing.issued(&identity(), "ship_cargo:g0", 1, receiver_start, &scope());
    timing.completed(&completion(), receiver_commit);
    let observed = timing.observed("ship_cargo:g0");
    assert!(
        !crate::production_ship_cursor::refresh_required(receiver_commit, 0, 30_000, observed, 25),
        "five receiver-wall seconds must not become thirty game seconds and force core refresh"
    );
    assert_eq!(observed, receiver_commit - receiver_start);
}

#[test]
fn faction_scoped_detail_timings_preserve_each_family_cost() {
    let mut timing = super::super::timing::ShipTiming::default();
    for (family, duration) in [
        ("ship_cargo", 4_000),
        ("ship_crew", 5_000),
        ("ship_loadout", 6_000),
    ] {
        let key = format!("{family}:argon:g0");
        timing.issued(&identity(), &key, 1, 10_000, &scope());
        timing.completed(&scoped_completion(family), 10_000 + duration);
        assert_eq!(timing.observed(&key), duration);
    }
}

#[test]
fn wrong_receipt_and_backward_clock_cannot_publish_cost() {
    let mut timing = super::super::timing::ShipTiming::default();
    timing.issued(&identity(), "ship_cargo:g0", 2, 10_000, &scope());
    timing.completed(&completion(), 15_000);
    assert_eq!(timing.observed("ship_cargo:g0"), 0);
    timing.issued(&identity(), "ship_cargo:g0", 1, 10_000, &scope());
    timing.completed(&completion(), 9000);
    assert_eq!(timing.observed("ship_cargo:g0"), 0);
}

#[test]
fn scope_incarnation_epoch_and_reset_bound_cost_identity() {
    let mut timing = super::super::timing::ShipTiming::default();
    for field in 0..3 {
        timing.issued(&identity(), "ship_cargo:g0", 1, 10_000, &scope());
        let mut message = completion();
        let observation_domain::CompleteMessage::SectionCompletion(value) = &mut message else {
            panic!("completion")
        };
        match field {
            0 => {
                value.source_scope =
                    observation_domain::SourceScopeId::new("x4:faction:teladi:ships")
                        .expect("scope");
            }
            1 => {
                value.producer_incarnation =
                    observation_domain::ProducerIncarnationId::new("producer:other")
                        .expect("producer");
            }
            _ => value.transport_epoch = observation_domain::TransportEpoch::new(8).expect("epoch"),
        }
        timing.completed(&message, 15_000);
        assert_eq!(timing.observed("ship_cargo:g0"), 0);
    }
    timing.clear();
    timing.completed(&completion(), 15_000);
    assert_eq!(timing.observed("ship_cargo:g0"), 0);
}
