---
phase: 04-persistent-full-faction-minds
plan: "01"
subsystem: mind-domain
tags: [rust, deterministic, initiative-lifecycle, causal-ledger]
requires: [03-03]
provides: [independent-faction-minds, one-owner-initiative-lifecycle]
affects: [04-03, 05-bounded-shadow-deliberation, 06-reports]
tech-stack:
  added: [mind-domain workspace crate]
  patterns: [pure aggregate transition, bounded causal ledger, exact command idempotency]
key-files:
  created: [crates/mind-domain/src/initiative.rs, crates/mind-domain/src/ledger.rs, crates/mind-domain/tests/initiative_lifecycle.rs]
  modified: [crates/mind-domain/src/causal.rs, crates/mind-domain/src/mind.rs, crates/mind-domain/src/lib.rs]
decisions:
  - "Each faction owns exactly three capability-indexed optional initiative slots."
  - "Exact repeated lifecycle commands replay their retained event set, while same-ID changed commands fail closed."
actuals:
  tokens: 5611
  tasks: 2
  commits: 2
status: complete
---

# Phase 04 Plan 01: Persistent Full Faction Minds Summary

Pure ZYA and ARG mind aggregates now preserve doctrine-driven state and bounded, replayable one-owner initiative causality without any provider, transport, persistence, report, or X4 mutation path.

## Accomplishments

- Added the `mind-domain` aggregate, seeded only from frozen `StrategicPacket` inputs, with doctrine, motives, priorities, goals, short- and long-term plans, and Executive posture.
- Added exactly three faction-local capability slots, immutable event ledger entries for every INST-08 category, terminal outcomes, predecessor-preserving preemption, and bounded histories.
- Added exact-command idempotency and same-ID/different-content rejection.

## Task Commits

1. **Task 1: Transition independent ZYA and ARG mind aggregates through one replayable path** — `1695f3e`
2. **Task 2: Preserve one-owner initiative and preemption causality in the aggregate** — `7590900`

## Verification

- `cargo test -p mind-domain --test initiative_lifecycle` — passed (2 tests).
- `cargo test -p mind-domain --test mind_tracer` — passed.
- `cargo run -p source-size-lint` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo test --workspace` — passed.
- `cargo fmt --all` and `git diff --check` — passed.

## Decisions Made

- The aggregate stores one optional active initiative for each of the fixed three `Capability` values, never a dynamically sized institution collection.
- Causal events are typed and bounded; no conversation prose or external service decides lifecycle state.

## Deviations from Plan

### Auto-fixed Issues

1. [Rule 3 - Blocking] Split lifecycle logic into cohesive bounded modules.
   - **Found during:** Task 2
   - **Issue:** A single lifecycle implementation exceeded the 200-line Rust source-file limit.
   - **Fix:** Separated command/value types from ledger transition logic.
   - **Files modified:** `crates/mind-domain/src/initiative.rs`, `crates/mind-domain/src/ledger.rs`.
   - **Verification:** source-size lint passes.

## Known Stubs

None.

## Self-Check: PASSED

- All six Task 2 source and test files exist.
- Task commits `1695f3e` and `7590900` exist.
