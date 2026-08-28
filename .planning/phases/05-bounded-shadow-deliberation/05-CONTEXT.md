# Phase 5: Bounded Shadow Deliberation - Context

**Gathered:** 2026-08-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Run bounded real-model Shadow deliberation for ZYA, ARG, their three
institutions, and Executive arbitration, then admit only typed kernel-valid
plans, initiatives, and diplomatic postures. No accepted result mutates X4.

</domain>

<decisions>
## Implementation Decisions

### Pre-alpha model access

- **D-01:** All real-model prototype and benchmark runs before `1.0.0` use a
  developer-controlled subscription harness.
- **D-02:** Public runtime API integration is not part of milestone 0.1 and
  begins on the public-alpha release path.
- **D-03:** Deterministic fakes are required for contracts, failure tests, and
  replay, but fake trajectories cannot satisfy strategic-quality acceptance.
- **D-04:** The subscription harness and fake remain behind the same typed
  provider-neutral domain boundary.

### Deliberation and admission

- **D-05:** Provider output remains untrusted until schema, semantic,
  information, safety, budget, and current-state validation all pass.
- **D-06:** Scheduling combines strategic ticks, relevant events, and cooldowns
  while remaining bounded and deduplicated per faction.
- **D-07:** Each institution proposes at most one active initiative. The
  Executive may originate, assign, approve, revise, preempt, reject, or
  terminate it but cannot bypass admission.
- **D-08:** Agreement does not open dialogue. Material objection, mandate,
  revision, or preemption may open at most two complete Executive–institution
  dialogue cycles before a final kernel-valid disposition.

### Diplomatic posture

- **D-09:** The Executive may maintain, de-escalate, intensify, or seek limited
  threat-driven coordination in the typed ZYA–ARG Shadow posture.
- **D-10:** There is no diplomacy institution, cross-faction model negotiation,
  or X4 relationship mutation in 0.1.

### Failure behavior

- **D-11:** Provider outage or timeout pauses new strategic decisions, records
  bounded degraded evidence, preserves accepted state, and reconciles current
  observations before replanning after recovery.
- **D-12:** Exact versioned cache identity includes faction, snapshot, policy,
  prompt package, schema, provider, model, and relevant generation settings.

### the agent's Discretion

The planner owns harness process mechanics, concrete subscription models,
model-role routing, invocation relevance policy, prompt/schema design, queue
limits, retry/backoff, cache storage, and benchmark-derived budgets. These
choices may not introduce an API runtime dependency before alpha.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/PROJECT.md` — model, institution, diplomacy, and trust boundaries.
- `.planning/REQUIREMENTS.md` — MIND-06, MIND-07, INST-04 through INST-07,
  MODEL-01 through MODEL-04, MODEL-06, and MODEL-07.
- `.planning/ROADMAP.md` — Phase 5 goal and acceptance criteria.
- `.planning/research/ARCHITECTURE.md` — orchestration and validation boundary.
- `.planning/research/PITFALLS.md` — model, caching, and unbounded-loop risks.
- `AGENTS.md` — model-output trust boundary and technical ownership rules.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- Phase 3 supplies deterministic strategic packets and allowed primitives.
- Phase 4 supplies persistent mind, initiative, and replay state.

### Established Patterns

- Models propose; the deterministic kernel admits or rejects.
- Typed state, not model conversation prose, remains authoritative.

### Integration Points

- Phase 6 consumes accepted plan and report intents.
- Phase 8 scores complete subscription-backed trajectories.

</code_context>

<specifics>
## Specific Ideas

- Exact model choice is an empirical result, not a product commitment.
- Subscription usage is a development harness, not a supported public mod
  runtime.

</specifics>

<deferred>
## Deferred Ideas

- OpenAI API and other public runtime adapters begin on the alpha path.
- Local-model public support remains outside milestone 0.1.
- Inter-faction negotiations and executable diplomacy are later work.

</deferred>

---

*Phase: 05-bounded-shadow-deliberation*
*Context gathered: 2026-08-28*
