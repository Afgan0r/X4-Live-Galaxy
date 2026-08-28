---
phase: 03-faction-scoped-strategic-state
plan: 03
subsystem: strategic-state
tags: [rust, deterministic, shadow-primitives, replay]
requires: [03-01, 03-02]
provides: [bounded-shadow-primitive-contract, canonical-replay-fingerprint]
affects: [phase-04-persistence, phase-05-deliberation]
tech-stack:
  added: []
  patterns: [typed-allowlist, canonical-ordering, pure-replay-fingerprint]
key-files:
  created:
    - crates/strategic-state/src/primitive.rs
    - crates/strategic-state/src/fingerprint.rs
    - crates/strategic-state/tests/mutation_baseline.rs
  modified:
    - crates/strategic-state/src/fact.rs
    - crates/strategic-state/src/packet.rs
decisions:
  - Shadow planning accepts only four typed primitives; bilateral posture remains Executive-only.
  - Replay identity uses deterministic FNV-1a bytes over canonical packet and primitive inputs.
requirements-completed: [OBS-04, OBS-05, MIND-04]
status: complete
---

# Phase 3 Plan 3: Shadow Primitives and Replay Summary

Finite planning-only Shadow primitives and a canonical replay fingerprint provide bounded Executive inputs without any model, persistence, report, diplomacy institution, or X4 mutation route.

## Accomplishments

- Added four owned, bounded Shadow primitive variants with canonical evidence references.
- Added canonical packet/admission ordering and a deterministic replay fingerprint.
- Added behavioral baseline coverage for visibility, availability, capacity, and canonical order.

## Verification

- `cargo test -p strategic-state --test shadow_primitive_contract` — passed.
- `cargo test -p strategic-state --test packet_determinism` — passed.
- `cargo test -p strategic-state --test mutation_baseline` — passed.
- `cargo clippy -p strategic-state --all-targets -- -D warnings` — passed.
- `cargo run -p source-size-lint` and `cargo test --workspace` — passed.
- `cargo fmt --check` and `git diff --check` — passed.

## Mutation Baseline

`cargo mutants --version` was measured before Task 3 and failed with `error: no such command: mutants`. No package was installed and no mutation score was inferred. The scoped `cargo mutants -p strategic-state -- --test mutation_baseline` baseline is deferred to Phase 8 after the reviewed runner is made available. Counts and survivor classifications are therefore unavailable, not zero.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Tool availability] Recorded unavailable mutation runner**

- **Found during:** Task 3
- **Issue:** The reviewed `cargo-mutants` command is unavailable in this environment.
- **Fix:** Preserved the measured command failure and deferred the baseline rather than installing software.
- **Files modified:** This summary.

## Self-Check: PASSED

- Normal focused and workspace gates pass.
- The unavailable mutation runner is recorded without a fabricated baseline.
