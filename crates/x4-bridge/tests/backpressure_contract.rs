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
fn ingress_accepts_exact_frame_and_queue_limits_then_rejects_one_over() {
    let limits = FrameLimits::new(3, 2);
    let ingress = BoundedIngress::new(limits);
    let session = compatible_session();

    let (ingress, first) = ingress
        .submit(&session, SequenceNumber::new(1), "observation", "abc")
        .into_parts();
    let (ingress, second) = ingress
        .submit(&session, SequenceNumber::new(2), "observation", "abc")
        .into_parts();
    let (_, one_over_queue) = ingress
        .submit(&session, SequenceNumber::new(3), "observation", "abc")
        .into_parts();
    let (_, one_over_frame) = BoundedIngress::new(limits)
        .submit(&session, SequenceNumber::new(3), "observation", "abcd")
        .into_parts();

    assert_eq!(first, BackpressureOutcome::Accepted);
    assert_eq!(second, BackpressureOutcome::Accepted);
    assert_eq!(one_over_queue, BackpressureOutcome::QueueSaturated);
    assert_eq!(one_over_frame, BackpressureOutcome::FrameTooLarge);
}

#[test]
fn unsupported_kind_is_rejected_without_consuming_queue_capacity() {
    let session = compatible_session();
    let ingress = BoundedIngress::new(FrameLimits::new(3, 1));

    let (ingress, unsupported) = ingress
        .submit(&session, SequenceNumber::new(1), "effect", "abc")
        .into_parts();
    let (_, accepted) = ingress
        .submit(&session, SequenceNumber::new(1), "observation", "abc")
        .into_parts();

    assert_eq!(unsupported, BackpressureOutcome::UnsupportedFrameKind);
    assert_eq!(accepted, BackpressureOutcome::Accepted);
}

#[test]
fn terminal_or_stale_sessions_are_nonblocking_non_admissions() {
    let terminal = SessionState::new(1).admit_hello(SessionHello::new(
        2,
        "live-galaxy-x4-build-1",
        ["live-galaxy-observation-v1"],
    ));
    let ingress = BoundedIngress::new(FrameLimits::new(3, 1));

    let (ingress, terminal_outcome) = ingress
        .submit(&terminal, SequenceNumber::new(1), "observation", "abc")
        .into_parts();
    let (ingress, accepted) = ingress
        .submit(
            &compatible_session(),
            SequenceNumber::new(2),
            "observation",
            "abc",
        )
        .into_parts();
    let (_, stale_outcome) = ingress
        .submit(
            &compatible_session(),
            SequenceNumber::new(1),
            "observation",
            "abc",
        )
        .into_parts();

    assert_eq!(terminal_outcome, BackpressureOutcome::SessionNotCompatible);
    assert_eq!(accepted, BackpressureOutcome::Accepted);
    assert_eq!(stale_outcome, BackpressureOutcome::StaleSequence);
}

#[test]
fn ingress_owns_sequence_watermark_after_success_only() {
    let session = compatible_session();
    let ingress = BoundedIngress::new(FrameLimits::new(3, 2));

    let (ingress, rejected) = ingress
        .submit(&session, SequenceNumber::new(2), "observation", "toolong")
        .into_parts();
    let (ingress, accepted) = ingress
        .submit(&session, SequenceNumber::new(2), "observation", "abc")
        .into_parts();
    let (ingress, duplicate) = ingress
        .submit(&session, SequenceNumber::new(2), "observation", "abc")
        .into_parts();
    let (_, reordered) = ingress
        .submit(&session, SequenceNumber::new(1), "observation", "abc")
        .into_parts();

    assert_eq!(rejected, BackpressureOutcome::FrameTooLarge);
    assert_eq!(accepted, BackpressureOutcome::Accepted);
    assert_eq!(duplicate, BackpressureOutcome::StaleSequence);
    assert_eq!(reordered, BackpressureOutcome::StaleSequence);
}
