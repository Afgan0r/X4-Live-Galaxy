use observation_domain::SenderEvidence;

pub fn is_ship_section(key: &str) -> bool {
    key == "ship_core"
        || ["ship_cargo:g", "ship_crew:g", "ship_loadout:g"]
            .iter()
            .any(|prefix| {
                key.strip_prefix(prefix)
                    .is_some_and(|v| v.parse::<u16>().is_ok_and(|n| n.to_string() == v))
            })
}

pub fn ship_order_matches(
    start: &observation_domain::SectionStartEnvelope,
    records: &[observation_domain::EnvelopeRecord],
) -> bool {
    let key = &start.section_key;
    if !is_ship_section(key.as_str()) {
        return true;
    }
    if key.as_str() != "ship_core" {
        return records
            .windows(2)
            .all(|pair| pair[0].entity_id.as_str() < pair[1].entity_id.as_str());
    }
    let prefix = format!("carrier-b:{}:", start.section_revision.get());
    let mut previous = 0_u64;
    for (index, record) in records.iter().enumerate() {
        let Some(identity) = record
            .entity_id
            .as_str()
            .strip_prefix("x4:ship:")
            .and_then(|value| value.parse::<u64>().ok())
        else {
            return false;
        };
        let Some(ordinal) = record
            .record_id
            .as_str()
            .strip_prefix(&prefix)
            .filter(|value| value.len() == 20 && value.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|value| value.parse::<usize>().ok())
        else {
            return false;
        };
        if identity <= previous || ordinal != index + 1 {
            return false;
        }
        previous = identity;
    }
    true
}

// A streamed start freezes all source claims. Finish may only close its capture window.
pub fn finish_matches(start: &SenderEvidence, finish: &SenderEvidence) -> bool {
    let start_window = start.section_state.capture_window();
    let finish_window = finish.section_state.capture_window();
    if finish_window.start_millis() != start_window.start_millis()
        || finish_window.end_millis() < start_window.end_millis()
    {
        return false;
    }
    let mut closed = start.clone();
    let state = start.section_state;
    closed.section_state = observation_domain::SectionState::with_evidence(
        finish_window,
        state.freshness(),
        state.quality(),
        state.availability(),
        state.coverage(),
    );
    closed == *finish
}

#[cfg(test)]
mod tests {
    use super::{finish_matches, ship_order_matches};
    use observation_domain::{
        CaptureWindow, EntityId, EnvelopeRecord, ObservationVersion, RecordId, SectionKey,
        SectionRevisionId, SectionState, SenderEvidence,
    };

    #[test]
    fn closing_capture_preserves_frozen_source_claims() {
        let start = SenderEvidence::legacy_default();
        let mut finish = start.clone();
        let state = start.section_state;
        let window = CaptureWindow::new(0, 1).expect("valid capture window fixture");
        finish.section_state = SectionState::with_evidence(
            window,
            state.freshness(),
            state.quality(),
            state.availability(),
            state.coverage(),
        );
        assert!(finish_matches(&start, &finish));
        finish.stable_identity = !start.stable_identity;
        assert!(!finish_matches(&start, &finish));
    }

    #[test]
    fn ship_core_requires_global_record_ordinals_from_one() {
        let key = SectionKey::new("ship_core").expect("section key");
        let revision = SectionRevisionId::new(7).expect("revision");
        let record = |ordinal: usize, identity: u64| EnvelopeRecord {
            record_id: RecordId::new(format!("carrier-b:7:{ordinal:020}"))
                .expect("record identity"),
            entity_id: EntityId::new(format!("x4:ship:{identity}")).expect("entity identity"),
            observation_version: ObservationVersion::new(7).expect("observation version"),
            content: String::new(),
        };
        let start = observation_domain::SectionStartEnvelope {
            source_scope: observation_domain::SourceScopeId::new("scope:x4").expect("scope"),
            producer_incarnation: observation_domain::ProducerIncarnationId::new("producer:1")
                .expect("producer"),
            transport_epoch: observation_domain::TransportEpoch::new(1).expect("epoch"),
            section_key: key,
            section_revision: revision,
            expected_records: 2,
            sender_evidence: SenderEvidence::legacy_default(),
        };
        assert!(ship_order_matches(&start, &[record(1, 10), record(2, 20)]));
        assert!(!ship_order_matches(&start, &[record(2, 10), record(3, 20)]));
        assert!(!ship_order_matches(&start, &[record(1, 10), record(3, 20)]));
    }
}
