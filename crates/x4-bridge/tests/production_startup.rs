#![expect(
    clippy::expect_used,
    reason = "invalid test setup must fail immediately"
)]

#[path = "production_startup/actual_pipe.rs"]
mod actual_pipe;
mod carrier_b_support;
#[path = "production_startup/support.rs"]
mod startup_support;

use observation_application::{LifecycleContext, LifecycleResult};
use observation_ingest::ReceiverDisposition;
use startup_support::{TempDirectory, valid_limits};
use x4_bridge::{DiagnosticError, StartupError, readback_revision, run_production};

#[test]
fn offline_readback_returns_exact_durable_revision_and_receipt() {
    let database = carrier_b_support::database("production-readback");
    let mut session = x4_bridge::ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("production session opens");
    let context = carrier_b_support::context(observation_domain::SectionCoverage::KnownEmpty);
    let _ = session
        .submit(carrier_b_support::input(
            "batch:start:readback",
            carrier_b_support::start_bytes("economy.stations"),
            LifecycleContext::Start(context),
            100,
        ))
        .expect("start accepted");
    let result = session
        .submit(carrier_b_support::input(
            "batch:complete:readback",
            carrier_b_support::completion_bytes("economy.stations"),
            LifecycleContext::Completion(carrier_b_support::current()),
            101,
        ))
        .expect("completion accepted");
    assert_eq!(
        result,
        LifecycleResult::Disposition(ReceiverDisposition::Committed)
    );
    assert_eq!(
        readback_revision(database.path(), "economy.stations", 1),
        Ok(String::from(
            "{\"section_key\":\"economy.stations\",\"section_revision\":1,\"records\":[],\"receipt\":{\"ordinal\":1,\"accepted_at\":101}}"
        ))
    );
}

#[test]
fn startup_requires_explicit_owned_paths_and_rejects_unknown_parameters() {
    assert!(matches!(
        run_production(Vec::<std::ffi::OsString>::new()),
        Err(StartupError::MissingArgument)
    ));
    assert!(matches!(
        run_production(["--data-dir".into(), "data".into(), "--surprise".into()]),
        Err(StartupError::UnknownArgument)
    ));
}

#[test]
fn invalid_limits_and_unwritable_journal_block_startup() {
    let directory = TempDirectory::new("startup-blocks");
    let limits = directory.path().join("limits.conf");
    std::fs::write(&limits, "max_records=0\n").expect("invalid limits written");
    assert!(matches!(
        run_production([
            "--data-dir".into(),
            directory.path().as_os_str().into(),
            "--limits-file".into(),
            limits.as_os_str().into(),
        ]),
        Err(StartupError::InvalidLimits)
    ));
    std::fs::write(&limits, valid_limits()).expect("valid limits written");
    let occupied = directory.path().join("occupied");
    std::fs::write(&occupied, "not a directory").expect("occupied path written");
    assert!(matches!(
        run_production([
            "--data-dir".into(),
            occupied.as_os_str().into(),
            "--limits-file".into(),
            limits.as_os_str().into(),
        ]),
        Err(StartupError::Diagnostic(DiagnosticError::Storage))
    ));
}

#[test]
fn production_limits_require_exact_bounded_json_shape() {
    let directory = TempDirectory::new("strict-limits");
    let limits = directory.path().join("limits.json");
    for invalid in [
        "max_records=16\n",
        r#"{"complete_message_bytes":8192}"#,
        r#"{"complete_message_bytes":8192,"complete_message_bytes":8192}"#,
        &valid_limits().replacen('}', ",\"extra\":1}", 1),
        &valid_limits().replacen(
            "\"max_pending_bytes\":8192",
            "\"max_pending_bytes\":4096",
            1,
        ),
    ] {
        std::fs::write(&limits, invalid).expect("invalid limits written");
        assert!(matches!(
            run_production([
                "--data-dir".into(),
                directory.path().as_os_str().into(),
                "--limits-file".into(),
                limits.as_os_str().into(),
            ]),
            Err(StartupError::InvalidLimits)
        ));
    }
}

#[test]
fn offline_readback_rejects_unknown_and_mismatched_identity() {
    let database = carrier_b_support::database("production-readback-missing");
    assert_eq!(
        readback_revision(database.path(), "", 1),
        Err(DiagnosticError::InvalidIdentity)
    );
    assert_eq!(
        readback_revision(database.path(), "economy.stations", 1),
        Err(DiagnosticError::MissingRevision)
    );
}
