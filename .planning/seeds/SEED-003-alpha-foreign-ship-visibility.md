---
id: SEED-003
status: dormant
planted: 2026-09-11
planted_during: Milestone 0.1 / Phase 05.5 discussion
trigger_when: before foreign ship observations feed public-alpha faction decisions
scope: accepted passive-observation direction; source-backed design pending
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
observation. The owner accepted passive observation for alpha: existing faction
objects provide contacts; detection does not automatically disclose cargo,
crew or full equipment; contact loss retains only the last observation and its
age, without refreshing hidden positions. Active scout deployment and building
an intelligence network remain later work. The concrete contact source,
per-field disclosure and aging thresholds still require design and research.
The current strategic filter admits threat-family facts without
itself verifying a game contact; that is not evidence of real sensor coverage.

## When to Surface

**Trigger:** before foreign ship observations feed public-alpha faction decisions.

Implement the accepted direction through a checked alpha policy before admitting production foreign-ship
reasoning. Phase 05.5 remains an observation conformance proof; this seed does
not authorize implementing reconnaissance or broadening that phase.

## Scope Estimate

Source-backed design and any remaining owner decisions; effort is unestimated.
Resolve the contact source, per-field disclosure and observation aging within
the accepted direction, then verify exact X4 capabilities and runtime behavior.

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

User-raised and passive-observation direction approved on 2026-09-11. This is
product-direction approval, not permission to implement it in Phase 05.5.
Physical scouts and satellites remain the recorded broader direction. No X4
API, sensor range or contact timeout is selected; no omniscient fallback is
permitted. The seed remains dormant until its delivery scope is admitted.
