---
phase: 03-faction-scoped-strategic-state
plan: "01"
subsystem: strategic-state
tags: [rust, deterministic, visibility, faction-packets]
requires:
  - phase: 01-read-only-observation-spine
    provides: accepted ProjectionSnapshot and SectionQuality contracts
provides:
  - bounded paired ZYA and ARG strategic packets
  - explicit fact availability and versioned visibility policy
affects: [03-02, 03-03, 04-persistent-full-faction-minds, 05-bounded-shadow-deliberation]
actuals:
  tokens: 4394
  tasks: 2
  commits: 4
tech-stack:
  added: [strategic-state workspace crate]
  patterns: [pure projection compiler, policy-first visibility, explicit availability]
key-files:
  created: [crates/strategic-state/src/derive.rs, crates/strategic-state/src/fact.rs, crates/strategic-state/src/faction.rs, crates/strategic-state/src/policy.rs, crates/strategic-state/tests/tracer_packet.rs, crates/strategic-state/tests/visibility_contract.rs]
  modified: [Cargo.toml, Cargo.lock]
key-decisions:
  - "Foreign changing facts are retained only as inaccessible availability, while static resource-map potential stays available."
  - "XEN and observed KHK are typed threat subjects without hostile minds or action vocabulary."
patterns-established:
  - "Strategic derivation accepts only an immutable ProjectionSnapshot and uses no I/O, model, persistence, reporting, or X4 mutation dependency."
requirements-completed: [OBS-04, OBS-05, MIND-02, MIND-03]
coverage:
  - id: D1
    description: Paired ZYA and ARG four-family packets preserve explicit quality and threat availability.
    requirement: OBS-04
    verification:
      - kind: integration
        ref: crates/strategic-state/tests/tracer_packet.rs#derives_paired_packets_with_explicit_four_family_availability
        status: pass
    human_judgment: false
  - id: D2
    description: Visibility policy excludes foreign changing operations while admitting static resource-map potential.
    requirement: MIND-02
    verification:
      - kind: integration
        ref: crates/strategic-state/tests/visibility_contract.rs#policies_filter_before_derivation_and_preserve_phase_one_quality_assumption
        status: pass
    human_judgment: false
duration: 40min
completed: 2026-08-29
status: complete
---

# Phase 03 Plan 01: Faction-Scoped Strategic State Summary

Paired deterministic ZYA and ARG packets now compile from one accepted projection with explicit availability, bounded capacity, and versioned visibility.

## Accomplishments

- Added the pure workspace-local `strategic-state` crate with no transport, model, storage, reporting, or X4 mutation dependency.
- Preserved `Available`, `Unknown`, `Stale`, `Inaccessible`, and `Unsupported` availability through four fact families.
- Proved shared XEN pressure, observed KHK semantics, capacity refusal, and policy-first foreign operational exclusion.

## Task Commits

1. **Task 1: Compile one accepted projection into paired bounded faction packets** — `ad57053`, `1f4909f`
2. **Task 2: Enforce pre-derivation faction visibility and Phase 1 compatibility** — `d538d53`, `e932eaa`

## Decisions Made

- The Phase 1 `SectionQuality` mapping is a local runtime-semantics assumption and remains explicit in the visibility contract.
- Static resource maps take a separate visibility path from changing foreign operations.

## Deviations from Plan

### Auto-fixed Issues

1. [Rule 3 - Blocking] Split strategic-state types into cohesive modules.
   - **Found during:** Task 1
   - **Issue:** A single compiler module exceeded the mandatory 200-line Rust source limit after formatting.
   - **Fix:** Moved fact and faction domain types into bounded modules while retaining the pure compiler boundary.
   - **Files modified:** `crates/strategic-state/src/fact.rs`, `crates/strategic-state/src/faction.rs`, `crates/strategic-state/src/lib.rs`
   - **Verification:** source-size lint and strict workspace Clippy pass.
   - **Committed in:** `1f4909f`

**Total deviations:** 1 auto-fixed (Rule 3).

## Verification

- `cargo test -p strategic-state --test tracer_packet` — passed.
- `cargo test -p strategic-state --test visibility_contract` — passed.
- `cargo run -p source-size-lint` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo test --workspace` — passed.
- `cargo fmt --check` and `git diff --check` — passed.

## Known Stubs

None.

## Next Phase Readiness

Phase 03-02 can add the three shared institution capability views above the immutable faction-visible packets. Phase 1 runtime-semantics remain a local compatibility assumption pending X4 evidence.

## Self-Check: PASSED

- All ten crate, manifest, and test files exist.
- Task commits `ad57053`, `1f4909f`, `d538d53`, and `e932eaa` exist.
