use mind_domain::{ArbitrationError, DialogueState};

#[test]
fn sd_008_direct_agreement_finishes_without_a_dialogue_cycle() {
    let final_state = DialogueState::DirectAgreement.finalize();
    assert_eq!(final_state, Ok(DialogueState::FinalDisposition));
    assert_eq!(DialogueState::DirectAgreement.cycles(), 0);
}

#[test]
fn sd_009_material_objection_has_two_cycles_then_one_final_disposition() {
    let first = DialogueState::MaterialObjection { cycles: 0 }.advance();
    assert_eq!(first, Ok(DialogueState::MaterialObjection { cycles: 1 }));
    let Ok(second) = first else { return };
    let second = second.advance();
    assert_eq!(second, Ok(DialogueState::MaterialObjection { cycles: 2 }));
    let Ok(final_cycle) = second else { return };
    assert_eq!(final_cycle.advance(), Err(ArbitrationError::CycleCap));
    assert_eq!(final_cycle.finalize(), Ok(DialogueState::FinalDisposition));
}
