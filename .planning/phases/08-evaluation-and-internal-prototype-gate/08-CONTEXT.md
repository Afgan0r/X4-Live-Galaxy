# Phase 8: Evaluation and Internal Prototype Gate - Context

**Gathered:** 2026-08-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Evaluate real Shadow Director trajectories, apply the independent reliability
floor, inventory all evidence, and classify milestone 0.1 as an internal
prototype. This phase does not make a playable or public-release claim.

</domain>

<decisions>
## Implementation Decisions

### Required evidence

- **D-01:** Fake trajectories test contracts but cannot satisfy the milestone
  quality gate. Complete real-model trajectories must come from the
  subscription-backed development harness.
- **D-02:** The corpus progresses from synthetic microcases to bounded
  historical causal cases and recorded X4 crisis snapshots or trajectories.
- **D-03:** Evaluation covers grounding, continuity, information discipline,
  faction divergence, institution contribution, initiative causality,
  strategic consistency, schema reliability, latency, cache behavior, and
  model usage or cost evidence.

### Judging

- **D-04:** Deterministic hard gates cover schemas, allowed primitives, budgets,
  prerequisites, information boundaries, and other mechanically verifiable
  constraints.
- **D-05:** Qualitative outputs are anonymized and scored by two independent
  LLM judges. The owner audits judge disagreements and a sample of agreements;
  one judge cannot promote a candidate alone.
- **D-06:** Each benchmark cell begins with three independent runs and may
  expand adaptively up to ten when results are close, unstable, disputed, or
  failure-prone. The stopping reason is recorded.

### Acceptance and classification

- **D-07:** Strategic quality cannot compensate for a failed independent
  reliability floor.
- **D-08:** Quality, reliability, latency, token, cost, cache, and mutation
  thresholds are derived from recorded baselines rather than invented upfront.
- **D-09:** The final inventory separates implemented, locally verified,
  pending game smoke, and observed-in-X4 claims.
- **D-10:** The package includes the bounded XEN/KHK research artifact but no
  autonomous hostile-mind claim.
- **D-11:** Every `0.x` version is an internal prototype. Only `1.0.0` may be
  called the first public alpha after its separate gates pass.

### the agent's Discretion

Case counts, exact rubrics, judge selection and rotation, statistical stability
rules, mutation operators, evidence-based thresholds, and package layout are
owned by evaluation planning within the locked hybrid method.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/PROJECT.md` — milestone purpose, maturity, and model-access
  boundary.
- `.planning/REQUIREMENTS.md` — MODEL-08, MODEL-09, VAL-04, and VAL-05.
- `.planning/ROADMAP.md` — Phase 8 gate and evidence criteria.
- `.planning/research/SUMMARY.md` — baseline research conclusions.
- `.planning/research/PITFALLS.md` — evaluation, leakage, and false-confidence
  risks.
- `AGENTS.md` — verification levels, release wording, and mutation policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- Phase 2 supplies the hostile-faction research artifact.
- Phase 7 supplies live trajectories and operational evidence.

### Established Patterns

- Reliability and strategic quality are independent gates.
- Multiple valid trajectories may pass a causal scenario; evaluation should not
  reward only one scripted answer.

### Integration Points

- The completed evidence inventory is the input to milestone audit and closure,
  not a public release pipeline.

</code_context>

<specifics>
## Specific Ideas

- Historical cases are evaluated ex ante with identities or quantities masked
  or transformed to reduce answer leakage.

</specifics>

<deferred>
## Deferred Ideas

- API runtime integration, local-model public support, compatibility gates,
  licensing, packaging, and normal-play validation belong to the alpha path.
- The 103 Bannerlord-derived candidates remain reference material for future
  milestone brainstorms, not an active backlog for 0.1.

</deferred>

---

*Phase: 08-evaluation-and-internal-prototype-gate*
*Context gathered: 2026-08-28*
