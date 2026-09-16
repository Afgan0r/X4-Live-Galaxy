use observation_domain::SenderEvidence;

pub fn ship_order_matches(
    key: &observation_domain::SectionKey,
    records: &[observation_domain::EnvelopeRecord],
) -> bool {
    if key.as_str() != "ship_core" {
        return true;
    }
    let mut previous = 0;
    for record in records {
        let Some(identity) = record
            .entity_id
            .as_str()
            .strip_prefix("x4:ship:")
            .and_then(|value| value.parse::<u64>().ok())
        else {
            return false;
        };
        if identity <= previous {
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
    use super::finish_matches;
    use observation_domain::{CaptureWindow, SectionState, SenderEvidence};

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
}
