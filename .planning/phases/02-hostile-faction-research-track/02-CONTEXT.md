# Phase 2: Hostile-Faction Research Track - Context

**Gathered:** 2026-08-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Produce a versioned XEN/KHK evidence artifact for future hostile-faction design.
This is a read-only, non-gating research track and does not implement hostile
minds, change the ZYA/ARG critical path, or select a hostile architecture.

</domain>

<decisions>
## Implementation Decisions

### Research scope

- **D-01:** Cover XEN and KHK state, events, identity, visibility, scheduling,
  economy or spawning ownership, and known control limitations in X4 9.00.
- **D-02:** XEN is the primary observed hostile pressure for 0.1; KHK is
  recognized only when authoritative observations contain it.
- **D-03:** Do not assign ordinary government institutions, motives, or
  diplomacy to XEN or KHK by analogy.

### Evidence discipline

- **D-04:** Label every claim documented, observed, inferred, or unknown.
- **D-05:** Use vanilla files and disposable runtime observations as primary
  evidence. Installed mods and third-party code are read-only precedents.
- **D-06:** Record only materially influential provenance; do not copy a raw
  foreign corpus into the repository.

### Independence

- **D-07:** Unresolved hostile-faction questions cannot delay Phases 1 or 3–7
  and cannot silently expand milestone 0.1.
- **D-08:** Phase 8 inventories the research result without treating it as an
  autonomous hostile-mind implementation.

### the agent's Discretion

The researcher owns evidence collection order, artifact organization, and the
exact disposable scenarios, provided the source hierarchy and claim labels are
preserved.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/PROJECT.md` — hostile-faction and prototype scope.
- `.planning/REQUIREMENTS.md` — RES-01 through RES-03 and OBS-05.
- `.planning/ROADMAP.md` — Phase 2 non-gating contract.
- `.planning/research/FEATURES.md` — product and research background.
- `.planning/research/PITFALLS.md` — evidence and scope risks.
- `AGENTS.md` — installed X4, installed-mod, provenance, and reference-research
  rules.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- No hostile-faction implementation exists; the output is a GSD-owned research
  artifact.

### Established Patterns

- The repository distinguishes documented, observed, inferred, and unknown
  evidence and keeps external sources read-only.

### Integration Points

- Phase 8 consumes the final evidence inventory.
- Later milestones may use the artifact before choosing hostile-mind
  architecture or write primitives.

</code_context>

<specifics>
## Specific Ideas

- Research should surface missing telemetry and primitive gaps while the world
  model is still cheap to change.

</specifics>

<deferred>
## Deferred Ideas

- Autonomous XEN/KHK minds, replacement behavior, and hostile write primitives
  are later-milestone work.

</deferred>

---

*Phase: 02-hostile-faction-research-track*
*Context gathered: 2026-08-28*
