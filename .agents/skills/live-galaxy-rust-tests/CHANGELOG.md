# Changelog

## 2026-09-05

- Moved shared scenario, oracle, fixture, double, persistence, diagnostics, and
  execution evidence rules to `live-galaxy-tests`.
- Added Rust-specific coverage IDs for deterministic admission, atomic
  rejection, recovery receipts, bounds, and adapter seams.

## 2026-08-28

- Created the initial deterministic and recovery-focused Rust test strategy.
- Added a phase/release mutation-testing gate with measured thresholds and
  explicit survivor disposition.
- Selected `cargo-mutants` as the Rust mutation runner, with a pinned version
  once Cargo development tooling exists.
