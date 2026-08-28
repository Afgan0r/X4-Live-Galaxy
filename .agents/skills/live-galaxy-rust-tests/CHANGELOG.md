# Changelog

## 2026-08-28

- Created the initial deterministic and recovery-focused Rust test strategy.
- Added a phase/release mutation-testing gate with measured thresholds and
  explicit survivor disposition.
- Selected `cargo-mutants` as the Rust mutation runner, with a pinned version
  once Cargo development tooling exists.
