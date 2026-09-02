use super::fixtures::{HELLO, OBSERVATION};
use x4_bridge::{
    AcceptAttempt, AcceptDisposition, MAX_CONSECUTIVE_ACCEPT_FAILURES, PipeDisposition, PipeServer,
};

#[test]
fn transient_accept_error_retries_before_a_replacement_client() {
    let mut server = PipeServer::new();

    assert_eq!(server.admit_message(HELLO), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(OBSERVATION), PipeDisposition::Accepted);
    for _ in 1..MAX_CONSECUTIVE_ACCEPT_FAILURES {
        assert_eq!(
            server.record_accept(AcceptAttempt::TransientFailure),
            AcceptDisposition::RetryAccept
        );
    }
    assert_eq!(
        server.record_accept(AcceptAttempt::TransientFailure),
        AcceptDisposition::RetryAcceptDegraded { delay_millis: 100 }
    );
    assert_eq!(
        server.record_accept(AcceptAttempt::TransientFailure),
        AcceptDisposition::RetryAcceptDegraded { delay_millis: 200 }
    );
    for expected_delay in [400, 800, 1_000, 1_000] {
        assert_eq!(
            server.record_accept(AcceptAttempt::TransientFailure),
            AcceptDisposition::RetryAcceptDegraded {
                delay_millis: expected_delay
            }
        );
    }
    assert_eq!(
        server.record_accept(AcceptAttempt::ClientAccepted),
        AcceptDisposition::ServeClient
    );
    assert_eq!(server.snapshot().entity_ids(), Vec::<String>::new());
}
