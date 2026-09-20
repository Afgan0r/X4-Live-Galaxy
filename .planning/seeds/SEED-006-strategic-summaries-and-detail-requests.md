---
id: SEED-006
status: dormant
planted: 2026-09-20
planted_during: Phase 05.5 — Heavy Faction Ship Conformance
trigger_when: when relevant
scope: unknown
---

# SEED-006: Evaluate strategic summaries with bounded detail requests

## Why This Matters

The owner's main concern about LLM-driven faction strategy was the size of a
context containing every ship position and every station's details. Evaluate
compact strategic summaries as the default input: force composition and
distribution, sector shortages and surpluses, losses, and economic changes.

The confirmed working hypothesis allows the model to request limited detail
through tools when a specific strategic problem requires it. The bridge would
return only faction-accessible information. An out-of-cycle refresh may be
needed when the available observation is insufficiently current.

## When to Surface

**Trigger:** when relevant

Surface when evaluating model context, strategic quality, or observation demand.
This seed is a focused candidate for the architecture comparison in SEED-004.

## Scope Estimate

**Unknown** — summary fields, tool contracts, and evaluation cases are unresolved.

## Breadcrumbs

- `.planning/PROJECT.md` — faction-visible strategic state and bounded proposals.
- `.planning/ROADMAP.md` — Phase 03 strategic packets, Phase 05 deliberation,
  and Phase 08 real-model quality evaluation.
- `.planning/seeds/SEED-004-replayability-architecture-evaluation.md`
- `.planning/seeds/SEED-005-cost-aware-work-scheduling.md`
- [OpenAI function calling](https://developers.openai.com/api/docs/guides/function-calling#the-tool-calling-flow)
  — documented application-side tool execution and model continuation.

## Notes

Owner-confirmed dialogue outcome on 2026-09-20:

- Start with summaries and permit bounded, task-specific detail requests.
- Consider own and enemy losses, economic condition, and known enemy military
  and economic state when evaluating outcomes. Intelligence remains future
  scope; this does not grant factions access to hidden world state.
- Keep the number of models and separate institutional agents unresolved.
- Test whether summaries preserve decision-relevant distinctions and whether
  extra requests improve decisions enough to justify latency and token cost.
- API support for tool calling does not prove that a particular X4 data field
  is available or that the project's current provider path implements it.

Status: future working hypothesis, not implementation authorization or a change
to the active milestone. No strategic-quality improvement has been demonstrated.
