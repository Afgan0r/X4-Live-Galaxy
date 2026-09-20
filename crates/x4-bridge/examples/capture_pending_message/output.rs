use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use super::{cleanup_result, debug};

pub fn receive_and_persist(
    output: &PathBuf,
    demand: Result<(), String>,
    receive: impl FnOnce() -> Result<Option<Vec<u8>>, String>,
) -> Result<Option<Vec<u8>>, String> {
    demand?;
    let Some(bytes) = receive()? else {
        return Ok(None);
    };
    persist_new(output, &bytes)?;
    Ok(Some(bytes))
}

fn persist_new(output: &PathBuf, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(debug)?;
    let persisted = file
        .write_all(bytes)
        .map_err(debug)
        .and_then(|()| file.sync_all().map_err(debug));
    drop(file);
    if let Err(primary) = persisted {
        return match std::fs::remove_file(output).map_err(debug) {
            Ok(()) => Err(primary),
            Err(cleanup) => Err(format!(
                "{primary}; partial output cleanup failed: {cleanup}"
            )),
        };
    }
    Ok(())
}

pub fn after_persist<T>(
    persisted: Result<(), String>,
    decode: impl FnOnce() -> Result<T, String>,
    cleanup: impl FnOnce() -> Result<(), String>,
) -> Result<T, String> {
    persisted?;
    cleanup_result(decode(), cleanup())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{after_persist, receive_and_persist};
    use crate::{DATA_BYTES, cleanup_result, read_bounded, timeout_error};

    fn output_path(case: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "live-galaxy-capture-{case}-{}-{nonce}.bin",
            std::process::id()
        ))
    }

    #[test]
    fn diagnostic_input_and_cleanup_failures_remain_bounded_and_visible() {
        let mut exact = std::io::Cursor::new(vec![0; DATA_BYTES]);
        assert_eq!(
            read_bounded(&mut exact).expect("exact input").len(),
            DATA_BYTES
        );
        let mut over = std::io::Cursor::new(vec![0; DATA_BYTES + 1]);
        assert!(read_bounded(&mut over).is_err());
        assert_eq!(timeout_error(Ok(())), "pending message timed out");
        assert_eq!(
            timeout_error(Err("closed".to_owned())),
            "pending message timed out; reset failed: closed"
        );
        assert_eq!(
            cleanup_result::<()>(Err("decode failed".to_owned()), Err("closed".to_owned())),
            Err("decode failed; reset failed: closed".to_owned())
        );
    }

    #[test]
    fn diagnostic_capture_resets_only_after_durable_persistence() {
        let decoded = Cell::new(false);
        let reset = Cell::new(false);
        assert_eq!(
            after_persist::<()>(
                Err("sync failed".to_owned()),
                || {
                    decoded.set(true);
                    Ok(())
                },
                || {
                    reset.set(true);
                    Ok(())
                },
            ),
            Err("sync failed".to_owned())
        );
        assert!(!decoded.get());
        assert!(!reset.get());

        let reset = Cell::new(false);
        assert_eq!(
            after_persist::<()>(
                Ok(()),
                || Err("decode failed".to_owned()),
                || {
                    reset.set(true);
                    Ok(())
                },
            ),
            Err("decode failed".to_owned())
        );
        assert!(reset.get());
    }

    #[test]
    fn failed_pre_receipt_attempts_do_not_publish_output() {
        for (case, demand, received) in [
            ("demand", Err("send failed".to_owned()), Ok(Some(vec![1]))),
            ("receive", Ok(()), Err("receive failed".to_owned())),
            ("timeout", Ok(()), Ok(None)),
        ] {
            let output = output_path(case);
            let result = receive_and_persist(&output, demand, || received);
            assert!(result.is_err() || result == Ok(None));
            assert!(!output.exists(), "{case} published an output artifact");
        }
    }
}
