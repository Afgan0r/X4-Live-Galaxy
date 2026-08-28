---
phase: 04-persistent-full-faction-minds
reviewed: 2026-08-29T00:00:00Z
depth: deep
files_reviewed: 17
files_reviewed_list:
  - crates/mind-domain/src/causal.rs
  - crates/mind-domain/src/initiative.rs
  - crates/mind-domain/src/ledger.rs
  - crates/mind-domain/src/lib.rs
  - crates/mind-domain/src/mind.rs
  - crates/mind-persistence/src/capsule.rs
  - crates/mind-persistence/src/checkpoint.rs
  - crates/mind-persistence/src/fake_port.rs
  - crates/mind-persistence/src/lib.rs
  - crates/mind-persistence/src/port.rs
  - crates/mind-persistence/src/recovery.rs
  - crates/mind-persistence/tests/capsule_contract.rs
  - crates/mind-persistence/tests/checkpoint_tracer.rs
  - crates/mind-persistence/tests/fake_port_contract.rs
  - crates/mind-persistence/tests/recovery_contract.rs
  - extensions/live_galaxy/checkpoint_schema.json
  - extensions/live_galaxy/md/live_galaxy_persistence.xml
findings:
  critical: 2
  warning: 0
  info: 0
  total: 2
status: issues_found
---

# Phase 04: Code Review Report

**Reviewed:** 2026-08-29T00:00:00Z
**Depth:** deep
**Files Reviewed:** 17
**Status:** issues_found

## Summary

The pure domain, checkpoint bounds, fake-port idempotency, and XML/manifest
static contract are locally coherent. Two persistence-contract assertions are
not true in the implementation: provider headroom eligibility is inverted, and
the claimed legacy migration merely relabels the already-current checkpoint.

## Critical Issues

### CR-01: Provider-relative context-budget eligibility is inverted

**File:** `crates/mind-persistence/src/capsule.rs:47-49, 91-101`

**Issue:** `BudgetProfile::eligible()` returns true only when
`measured_tokens + headroom_tokens >= context_limit`. A capsule with ample safe
space, e.g. limit 100, measured 40, headroom 10, is classified ineligible;
one exactly at or above the limit is eligible. This reverses the safety budget
and permits compaction/application at a provider's context boundary while
discarding valid capsules below it.

**Fix:** Make eligibility require a checked/saturating sum **at most** the
provider context limit, and add boundary tests below, exactly at, and above the
limit (including overflow-sized values).

### CR-02: Legacy migration never consumes or migrates legacy data

**File:** `crates/mind-persistence/src/recovery.rs:68-80, 115-121`; `crates/mind-persistence/tests/recovery_contract.rs:104-116`

**Issue:** `RecoveryInput::Migration` carries only an already-decoded current
`CheckpointEnvelope` plus two version strings. For the allowed
`mind-checkpoint-v0` path, `migrate()` simply revalidates and returns that
current envelope; it cannot decode a v0 payload, validate its fields, map them
to v1, or reject malformed v0 data. The test passes the current v1 envelope,
so it proves relabelling rather than a schema migration. A real pre-v1 save is
therefore unrecoverable despite the claimed ordered legacy migration.

**Fix:** Define a typed v0 wire envelope and make migration accept raw legacy
bytes, decode/validate them under the v0 schema, explicitly construct a v1
checkpoint, and test successful v0 conversion plus malformed, partial, stale,
and content-collision v0 failures. If migration is intentionally deferred,
remove the v0 compatibility claim and fail closed as unsupported.

## Evidence Run

- `cargo test -p mind-domain -p mind-persistence` — passed.
- `cargo clippy -p mind-domain -p mind-persistence --all-targets -- -D warnings` — passed.
- `cargo run -p source-size-lint -- --max-lines 200` — passed.
- `extensions/live_galaxy/tests/persistence_schema_contract.ps1` — passed.

## Residual Verification Risks

- X4 save/load, interruption, payload capacity, and reconnect behavior remain
  correctly documented as pending Phase 7 runtime evidence; this review makes
  no observed-in-X4 claim. `cargo-mutants` remains unavailable and is a
  residual mutation-evidence gap.

---

_Reviewed: 2026-08-29T00:00:00Z_
_Reviewer: gsd-code-reviewer_
_Depth: deep_
