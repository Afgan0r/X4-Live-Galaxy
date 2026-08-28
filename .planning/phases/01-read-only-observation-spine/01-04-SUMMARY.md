---
phase: 01-read-only-observation-spine
plan: "04"
subsystem: observation-ingest
tags: [rust, telemetry, atomic-admission, reconciliation, tdd]
requires:
  - phase: 01-03
    provides: typed duplicate and reconciliation policy
  - phase: 01-05
    provides: bounded telemetry session ordering contract
provides:
  - Atomic hostile-input rejection that preserves accepted snapshot content
  - Bounded rejection metadata without raw telemetry retention
  - Complete-scope delta reconciliation with idempotent replay
affects: [01-06, 01-07, 03]
tech-stack:
  added: []
  patterns: [validate-before-commit, immutable-snapshot-view, batch-delta-reconciliation]
key-files:
  created:
    - crates/observation-ingest/tests/atomic_rejection.rs
    - crates/observation-ingest/tests/batch_reconciliation.rs
    - tests/fixtures/malformed-envelope.json
    - tests/fixtures/oversized-envelope.json
    - tests/fixtures/reordered-duplicate-sequence.json
  modified:
    - crates/observation-ingest/src/lib.rs
key-decisions:
  - Rejection evidence is bounded metadata separate from the immutable accepted snapshot view.
  - Complete-scope reconciliation uses only records observed in the incoming batch, never inherited candidate records.
requirements-completed: [OBS-06, OBS-07]
actuals:
  tokens: 5833
  tasks: 2
  commits: 4
coverage:
  - id: D1
    description: Hostile telemetry preserves accepted snapshot content and records bounded rejection evidence.
    requirement: OBS-07
    verification:
      - kind: integration
        ref: crates/observation-ingest/tests/atomic_rejection.rs
        status: pass
    human_judgment: false
  - id: D2
    description: Complete runtime scopes reconcile only current-batch members and replay idempotently.
    requirement: OBS-06
    verification:
      - kind: integration
        ref: crates/observation-ingest/tests/batch_reconciliation.rs
        status: pass
    human_judgment: false
metrics:
  duration: 8 min
  completed: 2026-08-28
status: complete
---

# Phase 01 Plan 04: Atomic Observation Admission Summary

Validated telemetry batches now retain the last accepted snapshot on hostile input, keep bounded reason-only rejection evidence, and reconcile a completed runtime scope from its real batch delta.

## Accomplishments

- Added fixture-driven hostile-input coverage for malformed, oversized, stale, reordered, duplicate, and equal-version-conflict telemetry.
- Added atomic batch admission with bounded rejection evidence that contains no raw payload data.
- Reconciled only records observed in a successful complete scope, including correct tombstoning of absent prior members.

## Verification

- RED Task 1: `cargo test -p observation-ingest --test atomic_rejection` failed because the batch admission API did not exist.
- GREEN Task 1: `cargo test -p observation-ingest --test atomic_rejection` passed 4 tests.
- RED Task 2: `cargo test -p observation-ingest --test batch_reconciliation` failed because the snapshot contract could not expose reconciled members.
- GREEN Task 2: `cargo test -p observation-ingest --test batch_reconciliation` passed 3 tests.
- Final: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, both focused suites, and `cargo test --workspace` passed.

## TDD Gate Compliance

- RED commits: `4d5d152`, `370bf0a`.
- GREEN commits: `f809c13`, `cf280aa`.
- No refactor commit was necessary.

## Decisions Made

- Snapshot state and rejection evidence remain separate, so diagnostics can accumulate without changing the last accepted content.
- A completion marker reconciles only members observed in its batch; inherited candidate records cannot suppress tombstones.

## Deviations from Plan

### Auto-fixed Issues

1. [Rule 1 - Bug] Corrected complete-scope reconciliation to use the incoming batch delta.
   - Found during: Task 2 GREEN.
   - Issue: The cloned candidate snapshot included inherited members, so an alpha-only complete scope would retain prior beta.
   - Fix: Tracked observed member keys per scope during validation and passed those keys to domain reconciliation.
   - Files modified: `crates/observation-ingest/src/lib.rs`, `crates/observation-ingest/tests/batch_reconciliation.rs`.
   - Verification: focused reconciliation and full workspace tests passed.
   - Committed in: `cf280aa`.

---

**Total deviations:** 1 auto-fixed (Rule 1 bug).
**Impact on plan:** Required for OBS-06 correctness; no scope expansion.

## Issues Encountered

Git writes were denied to this executor's sandbox; the orchestrator created each atomic commit after the corresponding RED or GREEN evidence was supplied.

## Next Phase Readiness

The ingest boundary now exposes deterministic admission outcomes and replay-safe reconciliation for downstream consumers. Disposable X4 runtime evidence remains pending Plan 01-07.

## Self-Check: PASSED

- All six planned source, test, and fixture artifacts exist.
- All four RED and GREEN commits exist in the required order.
- No task commit deleted tracked files.

---

*Phase: 01-read-only-observation-spine*
*Completed: 2026-08-28*
