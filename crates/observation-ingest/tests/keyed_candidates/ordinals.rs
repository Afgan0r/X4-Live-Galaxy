use super::*;

#[test]
fn section_ordinals_reject_reversed_missing_duplicate_and_changed_input() {
    for ordinals in [[2, 1], [1, 3], [1, 1]] {
        let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
        assert_eq!(
            stager.start_section(start("ships", 1, 2), 1),
            ReceiverDisposition::Received
        );
        assert_ordinal_sequence(&mut stager, ordinals);
    }
    changed_ordinal_conflicts_with_staged_identity();
}

fn assert_ordinal_sequence(stager: &mut GenerationStager, ordinals: [usize; 2]) {
    for (index, ordinal) in ordinals.into_iter().enumerate() {
        let mut item = batch("ships", 1, &format!("batch:{index}"), 1);
        item.section_ordinal = ordinal;
        let outcome = stager.stage_section_batch(item, 1, 2 + index as u64);
        let expected = if index == 0 && ordinal == 1 {
            ReceiverDisposition::Received
        } else {
            ReceiverDisposition::PermanentlyRejected
        };
        assert_eq!(outcome, expected);
        if expected == ReceiverDisposition::PermanentlyRejected {
            break;
        }
    }
}

fn changed_ordinal_conflicts_with_staged_identity() {
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
    assert_eq!(
        stager.start_section(start("ships", 1, 1), 1),
        ReceiverDisposition::Received
    );
    let original = batch("ships", 1, "batch:same", 1);
    assert_eq!(
        stager.stage_section_batch(original.clone(), 1, 2),
        ReceiverDisposition::Received
    );
    let mut changed = original;
    changed.section_ordinal = 2;
    assert_eq!(
        stager.stage_section_batch(changed, 1, 3),
        ReceiverDisposition::PermanentlyRejected
    );
}
