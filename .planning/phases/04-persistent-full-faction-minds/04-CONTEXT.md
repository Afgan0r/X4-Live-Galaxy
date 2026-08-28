# Phase 4: Persistent Full Faction Minds - Context

**Gathered:** 2026-08-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Persist independent ZYA and ARG minds, their Executive diplomatic postures,
three institution initiative lifecycles, causal history, and replay state across
compaction, crash, retry, schema transition, and compatible Rust restart. This
phase does not yet admit live model proposals.

</domain>

<decisions>
## Implementation Decisions

### Mind and institution continuity

- **D-01:** Each faction preserves doctrine, motives, priorities, short-term
  plans, long-term plans, and one Executive-owned typed diplomatic posture.
- **D-02:** Each of the three institutions owns at most one active typed Shadow
  initiative. Preemption preserves the previous initiative and its disposition.
- **D-03:** Executive and institution conversations may retain separate
  continuity, but the bridge-owned typed ledger is authoritative; conversation
  prose is not.

### Compaction

- **D-04:** History compacts by provider/model-relative token budgets into
  versioned typed-plus-narrative capsules. Typed facts and commitments remain
  authoritative.
- **D-05:** Exact thresholds and safety headroom are benchmark-derived rather
  than fixed by event count or elapsed game time.

### Persistence authority

- **D-06:** Compact runtime state uses an X4-owned persistence contract.
  External databases, caches, diagnostics, and prose are non-authoritative.
- **D-07:** The implementation never reads or modifies player save files.
- **D-08:** Accepted snapshot, mind, initiative, replay, admission, and report
  intent state is transactional and idempotent.

### Recovery and restart

- **D-09:** Corrupt, partial, incompatible, duplicate, out-of-order, and
  version-transition inputs fail closed or recover the last valid state with
  structured evidence.
- **D-10:** A compatible Rust process can restart, update, and reconnect to the
  same running X4 process without duplicating accepted state or report identity.
- **D-11:** An incompatible game-side protocol revision fails closed and names
  the required X4 restart.

### the agent's Discretion

Storage engine, transaction layout, schema versions, migration mechanics,
capsule encoding, and crash points are technical decisions. They must satisfy
the locked authority and recovery contract.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/PROJECT.md` — persistence, restart, and mind-continuity contract.
- `.planning/REQUIREMENTS.md` — MIND-01, MIND-05, INST-03, INST-08, MODEL-05,
  and STATE-01 through STATE-06.
- `.planning/ROADMAP.md` — Phase 4 goal and recovery criteria.
- `.planning/research/ARCHITECTURE.md` — proposed state ownership and recovery
  boundaries.
- `.planning/research/PITFALLS.md` — persistence and partial-failure risks.
- `AGENTS.md` — save-file prohibition and Rust engineering invariants.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- Phase 3 supplies canonical faction packets and typed institution inputs.

### Established Patterns

- Accepted state is idempotent; retry never implies duplicate admission.
- External diagnostics may reproduce decisions but never become campaign truth.

### Integration Points

- Phase 5 writes admitted plans and dispositions through this boundary.
- Phase 6 persists report intent and acknowledgement identity.
- Phase 7 exercises compatible restart and recovery in X4.

</code_context>

<specifics>
## Specific Ideas

- Recovery must preserve causal evidence for proposals, objections,
  dispositions, preemptions, and terminal outcomes.

</specifics>

<deferred>
## Deferred Ideas

- Migration of public API credentials or public runtime settings is outside
  milestone 0.1.
- Mutable institutional power and multiple simultaneous initiatives per
  institution remain later work.

</deferred>

---

*Phase: 04-persistent-full-faction-minds*
*Context gathered: 2026-08-28*
