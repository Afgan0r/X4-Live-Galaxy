# Phase 7: X4 Operational Proof - Context

**Gathered:** 2026-08-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Prove the complete Shadow Director path in disposable X4 9.00 normal-speed and
SETA runs, including bounded work, subscription-backed deliberation, reconnect,
recovery, reporting, and unattended operation. This is an engineering test
stand, not normal gameplay or a playable-release gate.

</domain>

<decisions>
## Implementation Decisions

### Evidence layers

- **D-01:** Static, pure Lua where applicable, fake-adapter, Rust, and
  disposable in-game evidence are reported separately; lower layers cannot be
  described as observed-in-X4 proof.
- **D-02:** The phase does not assume X4 has a headless automation path. The
  feasible test harness must be established from current evidence.

### Operational scenarios

- **D-03:** Exercise both normal speed and SETA with measured game-side work,
  bridge backlog, model latency, queue behavior, and vanilla responsiveness.
- **D-04:** Restart and update a compatible Rust process while the same X4
  process remains running, then prove no accepted state, strategic tick,
  initiative, or report is lost or duplicated.
- **D-05:** Exercise an incompatible protocol path that fails closed and
  clearly identifies the X4 restart requirement.
- **D-06:** Exercise model/bridge degradation and recovery, including Mail
  notification, preserved state, bounded retry, and observation reconciliation
  before replanning.

### Unattended acceptance

- **D-07:** The primary 0.1 acceptance environment is an unattended AFK/SETA
  test stand; the owner does not need to resume normal play for this milestone.
- **D-08:** The soak uses real subscription-backed deliberation. Fakes may
  isolate failures but cannot replace the end-to-end run.
- **D-09:** Duration, workload, and safe bounds are derived from measured
  baselines rather than invented in the milestone contract.

### the agent's Discretion

Automation method, scenario setup, sampling, exact soak duration, SETA factor,
fault-injection schedule, evidence capture, and measured thresholds are owned by
research and planning. Human-only X4 validation remains an explicit pause gate.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/PROJECT.md` — AFK/SETA, restart, and maturity boundaries.
- `.planning/REQUIREMENTS.md` — VAL-01 through VAL-03.
- `.planning/ROADMAP.md` — Phase 7 goal and operational criteria.
- `.planning/research/ARCHITECTURE.md` — end-to-end runtime boundaries.
- `.planning/research/PITFALLS.md` — X4 testing and long-session risks.
- `AGENTS.md` — disposable-campaign, save-file, and evidence rules.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- Phases 1 and 3–6 provide the complete path under test.

### Established Patterns

- Accepted work and reports are idempotent across retry and restart.
- X4 remains authoritative and continues vanilla simulation during degradation.

### Integration Points

- Phase 8 consumes the captured trajectories, reliability data, and evidence
  classification.

</code_context>

<specifics>
## Specific Ideas

- Capture enough correlated evidence to identify whether a failure originated
  in X4 observation, transport, normalization, persistence, model work,
  validation, report delivery, or recovery.

</specifics>

<deferred>
## Deferred Ideas

- Normal-play campaign validation is required before public alpha, not for
  milestone 0.1.
- Mutation, missions, and compatibility certification are later milestones.

</deferred>

---

*Phase: 07-x4-operational-proof*
*Context gathered: 2026-08-28*
