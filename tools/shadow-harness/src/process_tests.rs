use super::{
    Drained, ProcessControl, cleanup_both, complete, drain, receive, start_drain, wait_for_exit,
};
use mind_orchestration::ProviderFailure;
use std::{
    io::{Cursor, Read},
    sync::mpsc,
    time::{Duration, Instant},
};

struct Pending;
impl ProcessControl for Pending {
    fn poll(&mut self) -> Result<Option<bool>, ProviderFailure> {
        Ok(None)
    }
}

struct Broken;
impl Read for Broken {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("broken"))
    }
}

#[test]
fn process_success_joins_bounded_drains() {
    let out_rx = start_drain(Cursor::new(b"{}".to_vec()));
    let err_rx = start_drain(Cursor::new(Vec::new()));
    assert_eq!(
        complete(
            true,
            out_rx,
            err_rx,
            Instant::now() + Duration::from_secs(1)
        ),
        Ok(b"{}".to_vec())
    );
}

#[test]
fn deadline_expiry_is_timeout_without_waiting_for_exit() {
    assert_eq!(wait_for_exit(&mut Pending, Instant::now()), Ok(None));
}

#[test]
fn oversized_and_read_failure_are_typed() {
    let out_rx = start_drain(Cursor::new(vec![0; super::MAX_OUTPUT_BYTES + 1]));
    let err_rx = start_drain(Cursor::new(Vec::new()));
    assert_eq!(
        complete(
            true,
            out_rx,
            err_rx,
            Instant::now() + Duration::from_secs(1)
        ),
        Err(ProviderFailure::Oversized)
    );
    let (tx, rx) = mpsc::channel();
    drain(Broken, tx);
    assert_eq!(
        receive(rx, Instant::now() + Duration::from_secs(1)).map(|value: Drained| value.failed),
        Ok(true)
    );
}

#[test]
fn incomplete_drain_is_bounded() {
    let (_tx, rx) = mpsc::channel::<Drained>();
    assert!(matches!(
        receive(rx, Instant::now()),
        Err(ProviderFailure::DrainIncomplete)
    ));
}

#[test]
fn timeout_cleanup_attempts_and_reconciles_both_workers_after_first_failure() {
    let attempts = std::cell::Cell::new(0);
    let result = cleanup_both(
        || {
            attempts.set(attempts.get() + 1);
            Err(ProviderFailure::DrainIncomplete)
        },
        || {
            attempts.set(attempts.get() + 1);
            Ok(Drained {
                bytes: Vec::new(),
                oversized: false,
                failed: false,
            })
        },
    );
    assert_eq!(result, Err(ProviderFailure::DrainIncomplete));
    assert_eq!(attempts.get(), 2);
}
