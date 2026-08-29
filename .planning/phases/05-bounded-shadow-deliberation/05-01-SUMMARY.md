---
phase: 05
plan: 01
subsystem: bounded-shadow-admission
tags: [rust, validation, checkpoint, tdd]
dependency_graph:
  requires: [phase-04-mind-checkpoint]
  provides: [strict-shadow-admission, atomic-shadow-checkpoint]
  affects: [phase-05-scheduling, phase-05-provider]
tech_stack:
  added: []
  patterns: [serde-deny-unknown-fields, ordered-admission, checkpoint-cas]
key_files:
  created:
    - crates/mind-domain/src/deliberation.rs
    - crates/mind-domain/src/admission.rs
    - crates/mind-persistence/src/deliberation_checkpoint.rs
  modified:
    - crates/mind-domain/src/mind.rs
    - crates/mind-domain/tests/shadow_deliberation_evals.rs
    - crates/mind-persistence/tests/deliberation_checkpoint.rs
decisions:
  - Provider bytes remain untrusted until ordered deterministic admission succeeds.
  - Repeated accepted state reuses the durable checkpoint without a second CAS.
metrics:
  duration: measured-in-execution-session
  completed: 2026-08-29
status: complete
actuals:
  tokens: 7357
  tasks: 3
  commits: 3
---

# Phase 05 Plan 01: Bounded Shadow Deliberation Summary

Strict Shadow proposal admission now freezes faction-visible facts, rejects unsafe
or stale candidates before projection, and persists an accepted pending mind commit
through one checkpoint compare-and-set.

## Completed Work

- Added strict Serde proposal and frozen request types with bounded metadata.
- Added ordered size, decode, schema, semantic, visibility, safety, budget, and
  current-state admission gates.
- Added an atomic checkpoint projection with idempotent replay detection.
- Added deterministic SD-001 through SD-013 corpus metadata and focused contracts.

## Verification

- `cargo test -p mind-domain --test shadow_deliberation_evals --locked`
- `cargo test -p mind-persistence --test deliberation_checkpoint --locked`
- `cargo test --workspace --locked`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo run -p source-size-lint --locked -- crates`

All commands passed.

## Deviations from Plan

### Auto-fixed Issues

1. [Rule 2 - Missing critical functionality] Exposed `MindAggregate::faction`.
   Admission must compare the frozen request faction to current aggregate state
   before creating a pending commit. The existing aggregate lacked that safe getter.

2. [Rule 2 - Missing critical functionality] Promoted pinned `serde_json` to a
   production dependency of `mind-domain`. Direct strict byte decoding is required
   at the provider trust boundary.

## Known Stubs

None.

## Self-Check: PASSED

- Task commits exist: `bb8f3ba`, `d25256d`, and `7a28510`.
- New admission, deliberation, checkpoint, test, and manifest artifacts exist.
