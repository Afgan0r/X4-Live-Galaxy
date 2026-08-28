# Phase 3: Faction-Scoped Strategic State - Context

**Gathered:** 2026-08-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Transform frozen observations into deterministic, replayable ZYA and ARG
strategic packets, including the three locked institution capability views and
the inputs needed for an Executive-owned Shadow diplomatic posture. This phase
does not invoke models or persist full minds.

</domain>

<decisions>
## Implementation Decisions

### Faction knowledge

- **D-01:** Each faction receives authoritative own-state facts plus only the
  operational information available to that faction under a recorded
  visibility policy.
- **D-02:** Complete static resource-map potential may be known, but changing
  foreign production, fleet, movement, and station facts are not omniscient.
- **D-03:** Missing, stale, inaccessible, or unsupported facts remain explicit
  and cannot be filled by model inference.

### Institution roster

- **D-04:** Each faction has exactly three primitive institutions mapped to:
  defense and military strategy; economy and logistics; territorial
  development and infrastructure.
- **D-05:** ZYA and ARG use canon-grounded institution identities and different
  doctrine-conditioned fixed priorities while sharing those three engine
  capability contracts.
- **D-06:** All institutions see the same authoritative faction-visible
  snapshot. Private institutional knowledge and false beliefs are excluded.

### Executive diplomatic posture

- **D-07:** Diplomacy is not a fourth institution. The strategic packet exposes
  the typed facts and allowed dispositions needed for the Executive to preserve
  relations, de-escalate, increase pressure, or seek limited coordination
  against a shared threat.
- **D-08:** This posture concerns only Shadow planning; it cannot negotiate with
  another faction or alter X4 relations.

### Determinism

- **D-09:** Equivalent frozen snapshots and policies yield canonically ordered
  facts, priorities, allowed primitives, and admission inputs.

### the agent's Discretion

Exact schemas, scoring formulas, priority weights, and canon-grounded
institution names are owned by research and planning. Names and profiles must
be supported by X4 evidence rather than invented LLM stereotypes.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/PROJECT.md` — faction, institution, doctrine, and diplomacy scope.
- `.planning/REQUIREMENTS.md` — OBS-04, OBS-05, MIND-02 through MIND-04,
  INST-01, and INST-02.
- `.planning/ROADMAP.md` — Phase 3 goal and success criteria.
- `.planning/research/ARCHITECTURE.md` — normalized state and deterministic
  kernel boundaries.
- `.planning/research/FEATURES.md` — faction-mind and institution background.
- `AGENTS.md` — X4 evidence routing and deterministic-domain invariants.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- Phase 1 will provide the normalized frozen snapshot boundary.

### Established Patterns

- Typed facts remain authoritative; prose and model output never repair missing
  state.

### Integration Points

- Phase 4 persists these packets and mind state.
- Phase 5 consumes the allowed primitives and validates proposals.

</code_context>

<specifics>
## Specific Ideas

- ZYA and ARG should face shared scenarios so doctrine differences are
  measurable rather than cosmetic.

</specifics>

<deferred>
## Deferred Ideas

- Private institutional knowledge, mutable influence, sabotage, and internal
  political simulation are post-alpha work.
- A separate diplomacy institution and inter-faction negotiation are outside
  milestone 0.1.

</deferred>

---

*Phase: 03-faction-scoped-strategic-state*
*Context gathered: 2026-08-28*
