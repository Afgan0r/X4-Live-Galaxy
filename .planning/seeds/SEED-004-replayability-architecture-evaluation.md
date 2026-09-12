---
id: SEED-004
status: dormant
planted: 2026-09-12
planted_during: Phase 05.5 — Heavy Faction Ship Conformance
trigger_when: when relevant
scope: unknown
---

# SEED-004: Evaluate architectures for unique replayability

## Why This Matters

_To be filled in. Run `$gsd-capture --seed --enrich SEED-004` to add context._

## When to Surface

**Trigger:** when relevant

This seed will surface during `$gsd-new-milestone` when the milestone scope
matches.

## Scope Estimate

**Unknown** — run `$gsd-capture --seed --enrich SEED-004` to estimate effort.

## Breadcrumbs

- `.planning/PROJECT.md` — Live Galaxy replayability and Faction Mind goals.
- `.planning/phases/05-bounded-shadow-deliberation/05-AI-SPEC.md` — current
  bounded multi-role LLM hypothesis, context budgets, and cost evidence needs.
- `.planning/phases/08-evaluation-and-internal-prototype-gate/08-CONTEXT.md` —
  scenario-corpus and real-model evaluation boundary.
- `wing_x4_modding/decisions` —
  `drawer_wing_x4_modding_decisions_60d6dad8a0eb5c0777997ed2`, the confirmed
  2026-09-12 product and architecture clarification.
- `wing_x4_live_galaxy/decisions` —
  `drawer_wing_x4_live_galaxy_decisions_b623dbb53b71c0ba5b16efb3`, the
  historical three-layer evaluation-corpus decision.

## Notes

When this seed surfaces, evaluate the most effective architecture for making
each Live Galaxy playthrough causally distinct and meaningfully replayable.
Use several representative military, economic, political, and mixed scenarios,
including sourced historical cases and counterfactual X4 quest, mission, or
crisis situations where appropriate.

Do not pre-limit the comparison to deterministic logic, one LLM call, or a
multi-agent council. Consider any plausible deterministic, generative, or
hybrid design. Compare how reliably each candidate produces varied but
coherent state-responsive behavior, useful mistakes under incomplete
information, material consequences, player-facing news and missions, and
institutional depth. Include model and token cost, context requirements,
cacheability, testability, failure behavior, and operational complexity.

The outcome is evidence for a later architecture decision, not authorization
to start a spike, change the active milestone, or replace the current design.
