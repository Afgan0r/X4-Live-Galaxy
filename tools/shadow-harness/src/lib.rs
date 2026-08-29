#![forbid(unsafe_code)]

mod benchmark_fixture;
pub mod evidence;
pub mod process;
mod process_schema;
pub mod subscription_adapter;

pub use benchmark::run_cli;
pub use evidence::{EvidenceRecord, benchmark_case_ids, validate_corpus};
pub use process::{BenchmarkProcess, CodexProcess};
pub use subscription_adapter::SubscriptionAdapter;
pub mod benchmark;
