---
phase: 04-persistent-full-faction-minds
plan: "04"
requirements: [MODEL-05, STATE-04, STATE-05]
subsystem: checkpoint-persistence
tags: [rust, recovery, migration, compaction, capsule]
requires:
  - phase: 04-03
    provides: acknowledged checkpoint envelope and fake-port contract
provides: [fail-closed recovery, typed-authoritative capsules, mutation baseline tests]
affects: [05-bounded-shadow-deliberation, 06-reports, 07-x4-runtime-proof]
tech-stack:
  added: []
  patterns: [last-acknowledged recovery, provider-relative budget profiles, narrative non-authority]
key-files:
  created: [crates/mind-persistence/src/capsule.rs, crates/mind-persistence/tests/capsule_contract.rs]
  modified: [crates/mind-persistence/src/recovery.rs, crates/mind-persistence/src/lib.rs]
decisions:
  - "Recovery retains the supplied acknowledged checkpoint and requests no port write for crash, invalid, stale, collision, or out-of-order candidates."
  - "Capsule eligibility is provider/model budget-profile relative; typed commitments are independent of optional narrative."
actuals:
  tokens: 5051
  tasks: 2
  commits: 2
coverage:
  - id: D1
    description: Fail-closed checkpoint recovery, crash-point handling, and copied migration policy
    requirement: STATE-04
    verification:
      - kind: integration
        ref: crates/mind-persistence/tests/recovery_contract.rs
        status: pass
    human_judgment: false
  - id: D2
    description: Provider-relative typed-authoritative compaction capsules
    requirement: MODEL-05
    verification:
      - kind: integration
        ref: crates/mind-persistence/tests/capsule_contract.rs
        status: pass
    human_judgment: false
duration: resumed execution
completed: 2026-08-29
status: complete
---

# Phase 04 Plan 04: Recovery and Compaction Summary

**Acknowledged checkpoint recovery now fails closed across deterministic crash and migration fixtures, while provider-relative capsules preserve typed commitments independently of bounded narrative.**

## Accomplishments

- Added last-acknowledged recovery policy for corrupt, partial, duplicate-content, stale, out-of-order, unsupported-migration, and three crash-point inputs without a port write.
- Added bounded raw legacy-to-current conversion that validates and rehashes a complete target envelope before requesting its durable write; unsupported or invalid input retains the fallback without a write.
- Added provider/model budget-profile capsules with source range/hash identity, measured headroom, bounded narrative, and typed commitment authority.

## Task Commits

1. **Task 1: Recover only the last valid acknowledged checkpoint through schema transitions** — `d0474e2`
2. **Task 2: Compact history with typed authority and measure recovery-policy mutants** — `638d2eb`

## Verification

- `cargo test -p mind-persistence --test recovery_contract` — passed (5 tests).
- `cargo test -p mind-persistence --test capsule_contract` — passed (3 tests).
- `cargo test -p mind-persistence --test mutation_baseline` — passed (2 tests).
- `cargo run -p source-size-lint`, strict workspace Clippy, `cargo test --workspace`, formatter, and `git diff --check` — passed.
- `cargo mutants -p mind-persistence -- --test mutation_baseline` — unavailable: Cargo reported `no such command: mutants`; no runner was installed, substituted, scored, or assigned survivor dispositions.

## Decisions Made

- The acknowledged X4 checkpoint remains the sole crash-recovery candidate; speculative work is invisible after every modeled crash point. A valid acknowledged v0 record may be copied into a fully validated v1 target.
- Capsule compaction has no event-count or elapsed-time trigger: it depends exclusively on its recorded provider/model budget profile.

## Deviations from Plan

### Auto-fixed Issues

1. [Rule 1 - Bug] Distinguished stale and out-of-order candidates from exact retries.
   - **Found during:** Task 2.
   - **Fix:** Added deterministic cursor classification and fixtures without changing the port boundary.
   - **Verification:** recovery contract passed.

**Total deviations:** 1 auto-fixed (1 Rule 1 bug).

## Mutation Evidence

`cargo-mutants` is not installed in this environment. The exact scoped command was attempted after normal tests passed and returned Cargo's `no such command: mutants` error. No mutation score, killed/surviving/invalid/equivalent counts, or survivor dispositions are inferred.

## Issues Encountered

The repository intentionally ignores `.planning/**` for Markdownlint. An attempted standard runner invocation also failed with an EPERM error opening the machine-local npm cache, so no alternate lint configuration was forced onto the planning artifact.

## Known Stubs

None.

## Next Phase Readiness

Phase 5 can consume typed commitments and provider-relative capsule identities. All payload, interruption, save/load, and reconnect runtime observations remain pending-X4 under Phase 7.

## Self-Check: PASSED

- Task commits `d0474e2` and `638d2eb` exist.
- Recovery, capsule, and mutation-baseline sources and tests exist.
