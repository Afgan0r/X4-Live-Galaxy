---
id: SEED-005
status: dormant
planted: 2026-09-20
planted_during: Phase 05.5 — Heavy Faction Ship Conformance
trigger_when: when relevant
scope: unknown
---

# SEED-005: Consider shared cost-aware scheduling with fixed task frequencies

## Why This Matters

The owner questioned continuous data collection after reporting a 12 ms ship
collection measurement. That number is a user-reported observation, not a
verified end-to-end timing or a budget for the complete system.

The idea began as collection scheduling and expanded into a possible shared
mechanism for collection, summary recomputation, model calls, and evaluation
of previous decisions. Execution history would inform work placement and load
distribution while task frequencies remain explicitly configured.

## When to Surface

**Trigger:** when relevant

Surface when revisiting collection cadence or coordination between observation
and deliberation. Assess existing scheduling first; this is not a commitment
to build a second scheduler or merge distinct execution domains.

## Scope Estimate

**Unknown** — no implementation plan or architectural replacement is approved.

## Breadcrumbs

- `.planning/ROADMAP.md` — existing bounded deliberation scheduling in Phase 05
  and observation scheduling in Phase 05.3.
- `.planning/PROJECT.md` — performance, bounded work, and information discipline.
- `.planning/seeds/SEED-006-strategic-summaries-and-detail-requests.md` — an
  agent's request can create demand for a targeted data refresh.

## Notes

Owner-confirmed dialogue outcome on 2026-09-20:

- Fixed task frequencies are the working hypothesis. Measured execution cost
  informs scheduling; it does not decide whether the information is needed.
- An agent may request an out-of-cycle refresh of the specific data it needs.
- Automatically adapting frequencies to situations such as war remains a
  separate possible extension, not the selected baseline.
- Priority, freshness limits, urgent-request bounds, and relevant cost
  measurements remain open. A fast refresh is a desired capability, not a
  latency guarantee.

Status: future idea, not a change to the active phase or its accepted design.
