use x4_bridge::{
    BackpressureOutcome, BoundedIngress, FrameLimits, SequenceNumber, SessionHello, SessionState,
};

fn compatible_session() -> SessionState {
    SessionState::new(1).admit_hello(SessionHello::new(
        1,
        "live-galaxy-x4-build-1",
        ["live-galaxy-observation-v1"],
    ))
}

#[test]
fn reconnect_resets_sequence_only_for_the_new_generation() {
    let generation_one = compatible_session();
    let generation_two = generation_one.reconnect();
    let ingress = BoundedIngress::new(FrameLimits::new(3, 2));

    let (ingress, first) = ingress
        .submit(
            &generation_one,
            SequenceNumber::new(2),
            "observation",
            "abc",
        )
        .into_parts();
    let (ingress, restarted) = ingress
        .submit(
            &generation_two,
            SequenceNumber::new(1),
            "observation",
            "abc",
        )
        .into_parts();
    let (_, stale_generation) = ingress
        .submit(
            &generation_one,
            SequenceNumber::new(3),
            "observation",
            "abc",
        )
        .into_parts();

    assert_eq!(first, BackpressureOutcome::Accepted);
    assert_eq!(restarted, BackpressureOutcome::Accepted);
    assert_eq!(stale_generation, BackpressureOutcome::StaleGeneration);
}

#[test]
fn rejected_frames_do_not_consume_the_active_generation_sequence() {
    let generation_one = compatible_session();
    let generation_two = generation_one.reconnect();
    let ingress = BoundedIngress::new(FrameLimits::new(3, 0));

    let (ingress, first_capacity_rejection) = ingress
        .submit(
            &generation_one,
            SequenceNumber::new(1),
            "observation",
            "abc",
        )
        .into_parts();
    let (ingress, repeated_capacity_rejection) = ingress
        .submit(
            &generation_one,
            SequenceNumber::new(1),
            "observation",
            "abc",
        )
        .into_parts();
    let (ingress, new_generation_rejection) = ingress
        .submit(
            &generation_two,
            SequenceNumber::new(1),
            "observation",
            "abc",
        )
        .into_parts();
    let (_, stale_generation) = ingress
        .submit(
            &generation_one,
            SequenceNumber::new(1),
            "observation",
            "abc",
        )
        .into_parts();

    assert_eq!(
        first_capacity_rejection,
        BackpressureOutcome::QueueSaturated
    );
    assert_eq!(
        repeated_capacity_rejection,
        BackpressureOutcome::QueueSaturated
    );
    assert_eq!(
        new_generation_rejection,
        BackpressureOutcome::QueueSaturated
    );
    assert_eq!(stale_generation, BackpressureOutcome::StaleGeneration);
}
