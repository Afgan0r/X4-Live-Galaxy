---
phase: 01-read-only-observation-spine
plan: "08"
subsystem: x4-telemetry-runtime
tags: [rust, lua, mission-director, named-pipe, telemetry]
requires: [01-04, 01-05, 01-06, 01-07]
provides: [bounded telemetry harness, Windows named-pipe listener, package guard]
affects: [01-09, 03]
tech-stack:
  added: [interprocess 2.4.3, recvmsg 1.0.0]
  patterns: [message-mode pipe, bounded admission, guarded native seam]
key-files:
  created: [crates/x4-bridge/src/server.rs, crates/x4-bridge/src/main.rs, extensions/live_galaxy/ui.xml, extensions/live_galaxy/lua/live_galaxy_runtime.lua, tests/x4-disposable/01-install-guard.ps1]
  modified: [Cargo.lock, crates/x4-bridge/Cargo.toml, crates/x4-bridge/src/lib.rs, extensions/live_galaxy/content.xml, extensions/live_galaxy/md/live_galaxy_observation.xml]
decisions:
  - The project-owned pipe endpoint is `\\.\pipe\live_galaxy`.
  - Windows message receive is capped at the existing 512-byte frame ceiling.
actuals:
  tokens: 56000
  tasks: 3
  commits: 1
status: complete
---

# Phase 01 Plan 08: Runnable Telemetry Harness Summary

The read-only telemetry path now registers a guarded Lua producer, raises an MD event, and admits bounded message-mode pipe frames through the Rust bridge.

## Accomplishments

- Added `interprocess 2.4.3` and `recvmsg 1.0.0` only for Windows; both are safe APIs with 0BSD-compatible licensing.
- Implemented the project-owned `live_galaxy` identity across extension registration, Lua adapter, pipe server, tests, and package guard.
- Kept the protocol telemetry-only: no command, report, acknowledgement, model, save, effect, or mutation vocabulary exists.
- Added a read-only package verifier that rejects missing graph links or bridge binary and refuses install while X4 runs.

## Verification

- `cargo test -p x4-bridge --test named_pipe_contract` — passed.
- `cargo test -p x4-bridge --test reconnect_idempotency` — passed.
- `cargo test -p x4-bridge --test backpressure_contract` — passed.
- `cargo build -p x4-bridge` — passed.
- `powershell -NoProfile -File tests/x4-disposable/01-install-guard.ps1 -VerifyPackageOnly` — passed.
- `cargo test --workspace` — passed.
- Strict workspace Clippy is green in the reviewed implementation; no runtime X4 claim was made.

## Evidence Classification

- **Verified locally:** Rust fake-boundary contracts, package graph, binary guard, and workspace tests.
- **Pending game smoke:** native Lua loading, MD delivery, support-API availability/return semantics, cadence, reconnect in a live X4 process, and SETA behavior.
- **Observed in X4:** none.

## Deviations from Plan

### Auto-fixed Issues

1. [Rule 3 - Blocking] Added safe Windows-only `interprocess = "=2.4.3"` and `recvmsg = "=1.0.0"` after explicit approval.
   - Reason: workspace forbids unsafe FFI, while a real message-mode named-pipe server needs safe platform bindings.
   - Provenance: locally inspected packaged metadata and APIs; `interprocess` is `0BSD OR Apache-2.0`, `recvmsg` is `0BSD`.

## Self-Check: PASSED

- Commit `b366d10` exists and includes the harness implementation.
- No X4 installation, extension, save file, or secret was accessed or changed.
