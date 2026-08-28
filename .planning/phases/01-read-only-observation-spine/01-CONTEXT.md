# Phase 1: Read-Only Observation Spine - Context

**Gathered:** 2026-08-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver a bounded, versioned, read-only X4 9.00 observation path and its
protocol/capability handshake. This phase proves trustworthy telemetry and
fail-closed compatibility behavior; it does not build faction strategy,
persistence, reporting, or any game-state mutation path.

</domain>

<decisions>
## Implementation Decisions

### Authority and transport boundary

- **D-01:** X4 remains authoritative. The game-facing adapter is thin and may
  only emit bounded observations during this phase.
- **D-02:** The long-term integration is asymmetric and bidirectional, but
  Phase 1 implements only telemetry plus the minimum session/capability
  negotiation needed to accept it. The report return path belongs to Phase 6.
- **D-03:** No fleet, economy, diplomacy, institution, or generic mutation
  command may exist in the 0.1 protocol vocabulary.

### Compatibility and restart behavior

- **D-04:** Compatible Rust bridge releases must be able to restart, update,
  and reconnect without restarting X4.
- **D-05:** A game-facing code change or incompatible protocol combination
  fails closed and explicitly identifies that X4 must restart.
- **D-06:** Unsupported, malformed, stale, duplicate, oversized, or
  out-of-order traffic cannot replace the last accepted snapshot.

### Observation semantics

- **D-07:** Every strategic entity and event carries stable typed identity,
  source, observation time, and monotonic state or event version.
- **D-08:** Snapshot sections represent freshness, coverage, quality, unknown,
  partial, stale, and unsupported states explicitly.
- **D-09:** Runtime sectors, assets, capacity, and ownership are discovered
  from X4 rather than assumed from a vanilla map or fixed job count.

### the agent's Discretion

Transport topology, framing, buffering, acknowledgement mechanics, handshake
schema, observation cadence, and section partitioning are technical decisions.
They must preserve the locked bounds, restart behavior, and game-thread safety.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/PROJECT.md` — authority, restart, trust, and milestone boundaries.
- `.planning/REQUIREMENTS.md` — OBS-01 through OBS-03, OBS-06 through OBS-08,
  and VAL-06.
- `.planning/ROADMAP.md` — Phase 1 goal and acceptance criteria.
- `.planning/research/ARCHITECTURE.md` — proposed X4/Rust boundaries and data
  flow.
- `.planning/research/SUMMARY.md` — initialization research conclusions.
- `AGENTS.md` — X4 evidence routing, safety rules, and reference repositories.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- No Live Galaxy runtime code exists yet; this phase establishes the first
  production boundary.

### Established Patterns

- Planning research already specifies typed envelopes, bounded queues,
  deterministic rejection, and explicit evidence levels.

### Integration Points

- Phase 3 consumes frozen normalized snapshots.
- Phase 4 adds recovery across compatible bridge restarts.
- Phase 6 adds the narrow report-and-acknowledgement return path.

</code_context>

<specifics>
## Specific Ideas

- Target the installed X4 9.00 behavior, not remembered APIs.
- Use disposable X4 scenarios for claims that cannot be proven statically.

</specifics>

<deferred>
## Deferred Ideas

- Report delivery and acknowledgements are deferred to Phase 6.
- Faction reasoning is deferred to Phases 3–5.
- Every game-state mutation path is deferred beyond milestone 0.1.

</deferred>

---

*Phase: 01-read-only-observation-spine*
*Context gathered: 2026-08-28*
