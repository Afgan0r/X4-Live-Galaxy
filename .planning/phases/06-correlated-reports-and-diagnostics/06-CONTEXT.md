# Phase 6: Correlated Reports and Diagnostics - Context

**Gathered:** 2026-08-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver safe, bounded player-readable X4 reports and complete external
developer evidence for accepted Shadow decisions and degraded paths. This phase
does not add a custom interface or expose mutation commands.

</domain>

<decisions>
## Implementation Decisions

### X4 report routing

- **D-01:** A completed strategic cycle produces one faction Logbook summary
  only when the accepted plan changed materially.
- **D-02:** Mail is reserved for critical strategic changes and bridge or model
  degradation and recovery.
- **D-03:** Intermediate deliberation, unchanged plans, duplicates, routine
  health, and verbose traces remain external and do not create player messages.
- **D-04:** Both surfaces use the narrow allowlisted Rust-to-X4 return channel;
  no generic or game-state mutation command is admitted.

### Information and voice

- **D-05:** Reports explain the safe supporting facts and trade-offs from the
  faction's permitted information and doctrine-conditioned perspective.
- **D-06:** Player-visible output excludes credentials, machine-local paths,
  private prompts, hidden reasoning, exact internal weights, and information
  unavailable to the recipient.

### External diagnostics

- **D-07:** Stable correlation IDs connect observation, snapshot, faction view,
  institution proposal or objection, Executive disposition, provider request,
  cache result, validation, accepted plan, report intent, and acknowledgement.
- **D-08:** Diagnostics expose bounded health, failures, latency, usage, cost,
  queues, recovery, and state quality suitable for unattended analysis.
- **D-09:** Captured snapshots and traces support offline reproduction but are
  explicitly non-authoritative.

### the agent's Discretion

Exact materiality rules, cadence, deduplication window, Mail urgency threshold,
message layout, acknowledgement mechanics, trace schema, retention bounds, and
diagnostic presentation are technical choices validated against X4 evidence.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/PROJECT.md` — report-routing and observability decisions.
- `.planning/REQUIREMENTS.md` — DIAG-01 through DIAG-05.
- `.planning/ROADMAP.md` — Phase 6 goal and success criteria.
- `.planning/research/ARCHITECTURE.md` — correlation and return-channel design.
- `.planning/research/PITFALLS.md` — privacy, leakage, and diagnostic risks.
- `AGENTS.md` — X4 evidence and public-output safety boundaries.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- Phase 4 persists report intent and identity.
- Phase 5 supplies accepted typed plans, initiatives, and safe explanations.

### Established Patterns

- Detailed evidence stays external; X4 receives concise bounded output.
- Acknowledgement and retry must remain idempotent.

### Integration Points

- The Phase 1 session boundary is extended with an allowlisted return path.
- Phase 7 exercises report delivery, outage notices, and deduplication in X4.

</code_context>

<specifics>
## Specific Ideas

- In-game reports are an integration sanity check for 0.1; external evidence is
  the primary acceptance surface.

</specifics>

<deferred>
## Deferred Ideas

- Custom dossier, chronicle, institution UI, newspapers, and richer government
  statements are post-alpha work.

</deferred>

---

*Phase: 06-correlated-reports-and-diagnostics*
*Context gathered: 2026-08-28*
