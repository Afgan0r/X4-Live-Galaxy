---
phase: 01-read-only-observation-spine
fixed_at: 2026-08-29T04:30:00+07:00
review_path: .planning/phases/01-read-only-observation-spine/01-REVIEW.md
iteration: 5
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 01: Code Review Fix Report

**Fixed at:** 2026-08-29T04:30:00+07:00
**Source review:** `.planning/phases/01-read-only-observation-spine/01-REVIEW.md`
**Iteration:** 5

## Summary

- Findings in scope: 2
- Fixed: 2
- Skipped: 0
- Verification location: main checkout; no commit was requested.

## Fixed Issues

### CR-01: One-frame pipe delivery separated observations from their marker

**Files modified:** `extensions/live_galaxy/lua/live_galaxy_runtime.lua`,
`crates/observation-ingest/src/wire.rs`,
`crates/observation-ingest/src/batch.rs`,
`crates/x4-bridge/src/server.rs`,
`crates/x4-bridge/src/listener.rs`,
`crates/x4-bridge/tests/named_pipe_contract.rs`

**Applied fix:** The pipe server now buffers at most 64 observations and a
bounded byte budget for one scope/version. It atomically admits observations
and their matching marker only at completion; disconnect, malformed input, or
scope/version mismatch discards pending state without changing the projection.

### CR-02: Degraded accept retry had no operational effect

**Files modified:** `crates/x4-bridge/src/server.rs`,
`crates/x4-bridge/src/lib.rs`,
`crates/x4-bridge/tests/named_pipe_contract.rs`

**Applied fix:** The listener uses bounded exponential delays after three
consecutive accept failures, from 100 ms through a deterministic 1,000 ms cap.
Later success resets the health/backoff policy without altering accepted state.

## Verification

- `cargo test -p x4-bridge --test named_pipe_contract` — passed (4 tests).
- `cargo lint` — passed.
- `cargo build -p x4-bridge` — passed.
- `powershell -NoProfile -File tests/x4-disposable/01-install-guard.ps1 -VerifyPackageOnly` — passed.
- `cargo test --workspace` — passed.
- `cargo fmt --check`, `git diff --check`, and `git diff --cached --check` — passed.

## Residual Runtime Evidence

The contracts are fake-boundary/local evidence only. Native Lua execution, MD
dispatch, and support-API named-pipe semantics remain pending the disposable X4
human gate.

---

_Fixed: 2026-08-29T04:30:00+07:00_
_Fixer: gsd-code-fixer_
_Iteration: 5_
