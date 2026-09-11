---
id: SEED-003
status: dormant
planted: 2026-09-11
planted_during: Milestone 0.1 / Phase 05.5 discussion
trigger_when: before foreign ship observations feed public-alpha faction decisions
scope: owner discussion and source-backed research; effort unestimated
---

# SEED-003: Alpha Foreign Ship Visibility Before Full Reconnaissance

## Why This Matters

The owner asked how public-alpha factions obtain information about foreign
ships while the full reconnaissance system remains later work. Observation
coverage, recipient knowledge and the Faction Mind roster are distinct.
Including XEN and KHK in ship collection does not authorize global disclosure
of their fleet positions, cargo, crew or equipment to every faction.

The accepted information boundary permits authoritative own-state and static
resource-map facts, but foreign operational facts require an attributable
observation. The concrete alpha contact source and disclosure policy remain
unresolved. The current strategic filter admits threat-family facts without
itself verifying a game contact; that is not evidence of real sensor coverage.

## When to Surface

**Trigger:** before foreign ship observations feed public-alpha faction decisions.

Resolve the minimum alpha policy before admitting production foreign-ship
reasoning. Phase 05.5 remains an observation conformance proof; this seed does
not authorize implementing reconnaissance or broadening that phase.

## Scope Estimate

Owner discussion and source-backed research; effort is unestimated. Decide
contact acquisition, per-field disclosure, contact loss and stale observations,
then verify the exact X4 source capabilities and required runtime evidence.

## Breadcrumbs

- `.planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md` — D-01/D-03.
- `.planning/REQUIREMENTS.md` — MIND-02 and INST-02.
- `crates/strategic-state/src/derive.rs` — current `visible` filter.
- `.planning/phases/05.5-heavy-faction-ship-conformance/05.5-DISCUSS-CHECKPOINT.json`
  — originating discussion while in progress; final context replaces it.
- Historical personal memory
  `wing_x4_live_galaxy/decisions`, drawer
  `drawer_wing_x4_live_galaxy_decisions_067f488f83547d02a201fcbd` — accepted
  physical reconnaissance direction, without an alpha implementation contract.

## Notes

User-raised on 2026-09-11. A passive-contact alpha approach is a discussion
candidate, not an accepted implementation. Physical scouts and satellites are
the recorded broader direction; no X4 API, sensor range, contact timeout or
omniscient fallback is selected here.
