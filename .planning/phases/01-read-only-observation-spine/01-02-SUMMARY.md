---
phase: 01-read-only-observation-spine
plan: "02"
subsystem: x4-observation-tracer
tags: [rust, x4, telemetry, tracer, tdd]
requires: [01-01]
provides:
  - Bounded telemetry-only capability admission
  - Atomic single-frame observation snapshot admission
  - Static X4 extension shell with deterministic fake-adapter evidence
affects: [01-03, 01-04, 01-07]
tech-stack:
  added: [serde 1.0.229, serde_json 1.0.151]
  patterns: [closed telemetry vocabulary, fail-closed capability decision, bounded schema decoding]
key-files:
  created:
    - Cargo.lock
    - crates/observation-ingest/Cargo.toml
    - crates/observation-ingest/src/lib.rs
    - crates/observation-ingest/tests/tracer_ingest.rs
    - crates/x4-bridge/Cargo.toml
    - crates/x4-bridge/src/lib.rs
    - crates/x4-bridge/tests/protocol_contract.rs
    - extensions/live_galaxy/content.xml
    - extensions/live_galaxy/lua/live_galaxy_telemetry.lua
    - extensions/live_galaxy/md/live_galaxy_observation.xml
    - tests/fixtures/tracer-observation.json
  modified:
    - Cargo.toml
    - crates/observation-domain/src/lib.rs
decisions:
  - Keep the bridge protocol closed to telemetry and make incompatible capabilities require an X4 restart.
  - Decode bounded telemetry through pinned serde contracts rather than ad-hoc JSON parsing.
metrics:
  duration: 35 min
  completed: 2026-08-29
  tasks: 2
  files: 13
status: complete
actuals:
  tokens: 3861
  tasks: 2
  commits: 5
---

# Phase 01 Plan 02: Telemetry Tracer Summary

One bounded X4 telemetry fixture now crosses a fail-closed bridge into an atomically admitted typed snapshot, with no game-state-effect vocabulary.

## Accomplishments

- Added `observation-ingest` and `x4-bridge` crates to the workspace.
- Added a closed `TelemetryFrame` family, explicit capability decision, 512-byte frame ceiling, and first-snapshot admission path.
- Added strict, size-bounded JSON decoding with pinned `serde` and `serde_json`; unknown fields and oversized input reject before snapshot creation.
- Added a thin injected-adapter Lua producer, static extension/MD XML shell, and deterministic fake-adapter contract coverage.

## Verification

- RED: `cargo test -p x4-bridge protocol_contract` and `cargo test -p observation-ingest tracer_ingest` failed on absent public contracts before implementation.
- Tracer feedback gate: the two focused tracer commands passed again after the GREEN commit.
- RED: `cargo test -p x4-bridge fake_x4_adapter_contract` failed because the X4 producer shell and explicit quality API were absent.
- GREEN: `cargo test -p x4-bridge fake_x4_adapter_contract` passed, and both extension XML files parsed successfully.
- Final: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, both focused suites, the XML parse command, and `cargo test --workspace` passed.
- Verified locally: Rust tracer, fake-adapter contract, and XML parsing.
- Pending game smoke test: embedded Lua and Mission Director runtime syntax, cadence, and native API behavior in a disposable X4 campaign.

## TDD Gate Compliance

- RED commits: `1e67e06`, `e33f943`.
- GREEN commits: `7ec10cf`, `c1dece4`.
- No refactor commit was necessary.

## Deviations from Plan

### Auto-fixed Issues

1. [Rule 3 - Blocking workspace integration] Added the two planned crates to the workspace and exposed typed domain accessors.
   - Found during: Task 1 RED setup.
   - Fix: updated `Cargo.toml` membership and added minimal `EntityId`/`ObservationVersion` readers needed by the public tracer contract.
   - Files modified: `Cargo.toml`, `crates/observation-domain/src/lib.rs`.
   - Commit: `1e67e06`, `7ec10cf`.

2. [Rule 2 - Security] Replaced ad-hoc JSON field scanning at the telemetry trust boundary.
   - Found during: final contract review.
   - Fix: added strict, pinned `serde`/`serde_json` decoding with unknown-field and size rejection tests.
   - Files modified: `Cargo.lock`, `crates/observation-ingest/Cargo.toml`, `crates/observation-ingest/src/lib.rs`, `crates/observation-ingest/tests/tracer_ingest.rs`.
   - Commit: `78d135d`.

## Known Stubs

| File | Line | Reason |
| --- | --- | --- |
| `extensions/live_galaxy/md/live_galaxy_observation.xml` | 3-4 | The MD scheduler is intentionally an empty shell until a disposable X4 runtime probe proves the exact event syntax and cadence. |

## Self-Check: PASSED

- All planned source, test, fixture, and X4 shell files exist.
- RED and GREEN commits exist in the required order.
- No task commit deleted tracked files.
