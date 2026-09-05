use super::*;

fn record(record: &str, version: u64, content: &str) -> EnvelopeRecord {
    EnvelopeRecord {
        record_id: id(record, RecordId::new),
        entity_id: id("entity:ships", EntityId::new),
        observation_version: ObservationVersion::new(version).expect("version is positive"),
        content: content.to_owned(),
    }
}

fn candidate() -> GenerationStager {
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
    assert_eq!(
        stager.start_section(start("ships", 1, 2), 1),
        ReceiverDisposition::Received
    );
    stager
}

fn candidate_batch(
    identity: &str,
    ordinal: usize,
    records: Vec<EnvelopeRecord>,
) -> ImmutableBatchEnvelope {
    let mut value = batch("ships", 1, identity, 1);
    value.section_ordinal = ordinal;
    value.records = records;
    value
}

#[test]
fn same_batch_rejects_lower_and_equal_conflicting_versions() {
    for second in [
        record("record:2", 1, "older"),
        record("record:2", 3, "changed"),
    ] {
        let mut stager = candidate();
        let records = vec![record("record:1", 3, "new"), second];
        assert_eq!(
            stager.stage_section_batch(candidate_batch("batch:1", 1, records), 1, 2),
            ReceiverDisposition::PermanentlyRejected
        );
        assert_eq!(stager.candidate_count(), 0);
    }
}

#[test]
fn cross_batch_rejects_lower_and_equal_conflicting_versions() {
    for second in [
        record("record:2", 1, "older"),
        record("record:2", 3, "changed"),
    ] {
        let mut stager = candidate();
        assert_eq!(
            stager.stage_section_batch(
                candidate_batch("batch:1", 1, vec![record("record:1", 3, "new")]),
                1,
                2,
            ),
            ReceiverDisposition::Received
        );
        assert_eq!(
            stager.stage_section_batch(candidate_batch("batch:2", 2, vec![second]), 1, 3),
            ReceiverDisposition::PermanentlyRejected
        );
        assert_eq!(stager.candidate_count(), 0);
    }
}
