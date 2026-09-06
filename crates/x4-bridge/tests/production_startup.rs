#![expect(
    clippy::expect_used,
    reason = "invalid test setup must fail immediately"
)]

mod carrier_b_support;

use observation_application::{LifecycleContext, LifecycleResult};
use observation_ingest::ReceiverDisposition;
use x4_bridge::{DiagnosticError, StartupError, readback_revision, run_production};

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

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

    let output = readback_revision(database.path(), "economy.stations", 1);

    assert_eq!(
        output,
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

    std::fs::write(&limits, "max_records=16\n").expect("valid limits written");
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
fn accepted_journal_precedes_durable_startup_and_waiting_state() {
    let directory = TempDirectory::new("startup-ready");
    let limits = directory.path().join("limits.conf");
    std::fs::write(&limits, "max_records=16\n").expect("limits written");
    assert!(matches!(
        run_production([
            "--data-dir".into(),
            directory.path().as_os_str().into(),
            "--limits-file".into(),
            limits.as_os_str().into(),
        ]),
        Ok(None)
    ));
    let history = std::fs::read_to_string(directory.path().join("operational-history.jsonl"))
        .expect("history readable");
    assert!(history.contains("journal-accepted"));
    assert!(history.contains("peer-absent"));
    assert!(directory.path().join("observations.sqlite3").is_file());
}

struct TempDirectory(PathBuf);

impl TempDirectory {
    fn new(label: &str) -> Self {
        let ordinal = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "live-galaxy-production-{label}-{}-{ordinal}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("temporary directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
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
