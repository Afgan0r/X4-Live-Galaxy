---
phase: 03-faction-scoped-strategic-state
plan: "02"
subsystem: strategic-state
tags: [rust, faction-profiles, institution-views, deterministic-policy]
requires:
  - phase: 03-faction-scoped-strategic-state
    provides: bounded paired ZYA and ARG strategic packets
provides:
  - exhaustive shared institution capability views
  - versioned ZYA and ARG doctrine profiles
  - locked internal labels and deterministic priority policy
affects: [03-03, 04-persistent-full-faction-minds, 05-bounded-shadow-deliberation]
actuals:
  tokens: 4003
  tasks: 2
  commits: 2
tech-stack:
  added: []
  patterns: [immutable packet views, profile-versioned doctrine, fixed capability order]
key-files:
  created: [crates/strategic-state/src/packet.rs, crates/strategic-state/tests/capability_contract.rs, crates/strategic-state/tests/doctrine_priority.rs]
  modified: [crates/strategic-state/src/faction.rs, crates/strategic-state/src/derive.rs, crates/strategic-state/src/lib.rs]
key-decisions:
  - "Institution views carry only a fixed capability and faction-visible snapshot ID, never a raw projection or private facts."
  - "The six labels and priority order are versioned Live Galaxy product policy, not official X4 institution names or numeric canon."
requirements-completed: [MIND-03, INST-01, INST-02]
coverage:
  - id: D1
    description: Each faction packet exposes exactly three shared capabilities backed by one faction-visible snapshot identity.
    requirement: INST-01
    verification:
      - kind: integration
        ref: crates/strategic-state/tests/capability_contract.rs#exposes_only_the_three_shared_capabilities_from_one_visible_snapshot
        status: pass
    human_judgment: false
  - id: D2
    description: Versioned ZYA and ARG profiles retain their locked internal labels and differentiated deterministic priorities.
    requirement: INST-02
    verification:
      - kind: integration
        ref: crates/strategic-state/tests/doctrine_priority.rs#versioned_profiles_keep_locked_live_galaxy_labels_and_priority_policy
        status: pass
    human_judgment: false
duration: 20min
completed: 2026-08-28
status: complete
---

# Phase 03 Plan 02: Faction-Scoped Strategic State Summary

Shared three-capability institution views now reference one faction-visible snapshot while immutable ZYA and ARG profiles supply locked labels and deterministic doctrine priorities.

## Accomplishments

- Added an exhaustive three-item capability contract for defense/military, economy/logistics, and territorial/infrastructure views.
- Kept every institution view bound to its packet's one immutable faction-visible snapshot ID without raw projection or private-fact access.
- Added versioned, explicitly non-official Live Galaxy profile labels and shared-scenario priorities: ZYA defense/territorial/economy and ARG economy/defense/territorial.

## Task Commits

1. **Task 1: Define the exhaustive shared capability and shared-snapshot contract** — `b047fe9` (feat)
2. **Task 2: Add evidence-labeled ZYA and ARG doctrine profiles with fixed priorities** — `6cfe407` (feat)

## Files Created/Modified

- `crates/strategic-state/src/faction.rs` — shared capability enum and immutable faction profiles.
- `crates/strategic-state/src/packet.rs` — private faction-visible snapshots and institution views.
- `crates/strategic-state/src/derive.rs` — packet construction through the bounded view model.
- `crates/strategic-state/src/lib.rs` — public strategic-state exports.
- `crates/strategic-state/tests/capability_contract.rs` — exhaustive capability and snapshot-boundary contract.
- `crates/strategic-state/tests/doctrine_priority.rs` — labels, evidence classification, and doctrine-priority fixture.

## Decisions Made

- Institution views expose a capability and snapshot identity only; the packet keeps the compiled facts private to the faction-visible boundary.
- Doctrine profiles use a `doctrine-v1` replay input. Their labels and priority orders are inferred Live Galaxy product policy, not official X4 statistics or institution names.

## Deviations from Plan

None - plan executed exactly as written.

## Verification

- `cargo test -p strategic-state --test doctrine_priority` — passed.
- `cargo test -p strategic-state --test capability_contract` — passed.
- `cargo run -p source-size-lint` — passed.
- `cargo clippy -p strategic-state --all-targets -- -D warnings` — passed.
- `cargo test --workspace` — passed.
- `cargo fmt --check` and `git diff --check` — passed.

## Known Stubs

None.

## Next Phase Readiness

Plan 03-03 can add planning-only primitives and a replay fingerprint that includes the visibility-policy and doctrine-profile versions.

## Self-Check: PASSED

- Task commits `b047fe9` and `6cfe407` exist.
- All six planned strategic-state source and contract-test files exist.
