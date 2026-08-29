---
gsd_state_version: 1.0
current_phase: 1
current_phase_name: Read-Only Observation Spine
status: executing
stopped_at: Phase 4 verified locally; Phase 1 Plan 01-09 pending human X4 verification
last_updated: "2026-08-29T01:09:28.186Z"
last_activity: 2026-08-29
last_activity_desc: Phase 4 completed and verified locally
state_head: 421b36d
progress:
  total_phases: 8
  completed_phases: 3
  total_plans: 17
  completed_plans: 16
  percent: 38
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-08-28)

**Core value:** Factions pursue coherent, distinct, long-lived strategies while X4 remains authoritative and every proposed effect stays observable, recoverable, and bounded by deterministic validation.
**Current focus:** Phase 1 — Read-Only Observation Spine

## Current Position

Phase: 1 (Read-Only Observation Spine) — EXECUTING
Plan: 9 of 9
Status: Plan 01-09 pending human X4 verification
Last activity: 2026-08-29 — Phase 4 completed and verified locally

Progress: [████░░░░░░] 38%

## Performance Metrics

**Velocity:**

- Total plans completed: 16
- Average duration: 24.2 min across 6 recorded durations
- Total execution time: 2 hours 25 min across 6 recorded durations; Plan 01-07 duration not recorded

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
| --- | --- | --- | --- |
| 01 | 7 | 2 hours 25 min recorded | 24.2 min (6 recorded) |

**Recent Trend:**

- Last 5 plans: 01-03 (22 min), 01-05 (31 min), 01-04 (8 min), 01-06 (18 min), 01-07 (not recorded)
- Trend: Partial; one of seven completed-plan durations is not recorded

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 01 P01 | 31 min | 2 tasks | 5 files |
| Phase 01 P02 | 35 min | 2 tasks | 13 files |
| Phase 01 P03 | 22 min | 2 tasks | 3 files |
| Phase 01 P05 | 31 min | 2 tasks | 3 files |
| Phase 01 P04 | 8 min | 2 tasks | 6 files |
| Phase 01 P06 | 18 min | 2 tasks | 7 files |
| Phase 01 P07 | not recorded | 2 tasks | 2 files |
| Phase 02 P01 | 5 min | 3 tasks | 4 files |
| Phase 03-faction-scoped-strategic-state P01 | 40 min | 2 tasks | 10 files |
| Phase 03-faction-scoped-strategic-state P02 | 20min | 2 tasks | 6 files |
| Phase 04-persistent-full-faction-minds P04 | resumed execution | 2 tasks | 6 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Milestone 0.1]: Shadow Director is observation-only and internal; X4 remains authoritative and no mutation command is admitted.
- [Milestone 0.1]: Full ZYA and ARG minds share XEN pressure, recognize observed KHK, and are accepted through AFK/SETA evidence.
- [Milestone 0.1]: Primitive institutions share the authoritative faction-visible snapshot, apply fixed priorities, and own at most one active Shadow initiative under bounded Executive arbitration.
- [Workflow]: The 103 Bannerlord-derived ideas remain a reference catalogue; development selects one small visible milestone, verifies it in game, and only then discusses the next.
- [Milestone 0.1]: XEN/KHK research is an independent parallel track and cannot gate the ZYA/ARG implementation path.
- [Milestone 0.1]: The X4 adapter stays thin and stable; compatible Rust releases restart, update, and reconnect independently, while incompatible game-side protocol changes fail closed and require X4 restart.
- [Milestone 0.1]: Integration is asymmetric bidirectional: bounded telemetry flows X4-to-Rust, and only an allowlisted report-and-acknowledgement channel returns to X4.
- [Phase 1]: Milestone 0.1 uses exactly three institutions per faction: defense/military, economy/logistics, and territory/infrastructure. — This covers the observable strategic domains without multiplying low-value agents.
- [Phase 1]: The ZYA-ARG Shadow diplomatic posture belongs to the Executive Brain; 0.1 has no diplomacy institution or cross-faction negotiation. — The pair relationship remains testable without expanding the institution roster or mutation scope.
- [Phase 1]: Material cycle summaries go to Logbook; Mail is reserved for critical strategy changes and bridge/model degradation or recovery. — The X4 channel is verified without flooding unattended SETA runs.
- [Phase 1]: All real-model prototype evidence before 1.0.0 uses developer-controlled subscription tooling; deterministic fakes are test-only and public API runtime begins on the alpha path. — Pre-alpha iteration uses existing subscriptions while preserving a typed provider boundary.
- [Phase 1]: Initial observation contracts remain dependency-free and transport-free.
- [Phase 1]: Phase 1 bridge admission remains closed to bounded telemetry; incompatible capabilities require an X4 restart.
- [Phase 1]: Telemetry JSON decoding uses pinned serde contracts with strict unknown-field rejection.
- [Phase 1]: Known-empty requires a successful completion marker for the same runtime scope.
- [Phase 1]: Incomplete scopes preserve prior membership; over-limit scans reject without truncation.
- [Phase 1]: Compatible Rust reconnects increment bridge generation without demanding an X4 restart.
- [Phase 1]: Protocol major, capability, and game build mismatches remain terminal until X4 restarts.
- [Phase 1]: Queue and frame limits return explicit nonblocking outcomes before bridge admission.
- [Phase 1]: Rejection evidence is bounded metadata separate from immutable accepted snapshot content.
- [Phase 1]: Complete-scope reconciliation uses only incoming batch members, never inherited candidate records.
- [Phase 1]: Plan 01-06 keeps X4 globals behind injected adapters and makes every producer outcome explicit.
- [Phase 1]: Plan 01-06 defers embedded Lua and Mission Director runtime claims until a disposable X4 probe.
- [Phase 1]: The completed 01-07 procedure is local planning evidence only; it found no active MD cue, registered UI Lua, or runnable Rust named-pipe server, so OBS-X4-01 through OBS-X4-04 cannot yet run.
- [Phase 1]: Plan 01-08 must create the telemetry-only UI-Lua/MD/named-pipe harness before Plan 01-09 performs the sole human X4 gate.
- [Phase 2]: Phase 2 validates only structured static XEN/KHK evidence; runtime unknowns remain non-gating.
- [Phase 1]: Foreign changing facts stay inaccessible while static resource maps remain available under recorded policy.
- [Phase 3]: Institution views expose only capability and faction-visible snapshot identity; raw projections and private facts remain unavailable.
- [Phase 3]: Doctrine v1 labels and priorities are explicit Live Galaxy product policy, not official X4 numeric canon.
- [Phase 3]: Shadow planning is limited to four typed primitives; replay identity covers canonical policy, doctrine, fact, primitive, and evidence inputs.
- [Phase 4]: Checkpoint integrity binds authoritative identities, while durable advancement requires exact reread acknowledgement.
- [Phase 4]: Recovery retains the last acknowledged checkpoint and never exposes speculative state.
- [Phase 4]: Capsule compaction is provider/model budget-profile relative; narrative is non-authoritative.

### Pending Todos

None yet.

### Blockers/Concerns

- [Phase 1]: Exact X4 9.00 observation, transport, embedded Lua, Mission Director, identity, scheduling, protocol negotiation, degraded-mode, and restart-condition semantics require phase research and disposable evidence.
- [Phase 4]: The X4-owned compact persistence contract remains an evidence-dependent boundary decision; player save files are prohibited.
- [Phase 6]: The bounded Rust-to-X4 Mail/Logbook return channel and acknowledgement semantics require disposable evidence; topology and framing remain technical decisions.

## Deferred Items

Items acknowledged and deferred at milestone close, most recent first:

| Category | Item | Status | Deferred At | Milestone |
| --- | --- | --- | --- | --- |
| Scope | All mutation, autonomous XEN/KHK minds, player missions, custom interface, broad faction rollout, compatibility gates, and public release work | Deferred | Initialization | Later milestones |

## Session Continuity

Last session: 2026-08-29T01:09:28.186Z
Stopped at: Phase 4 verified locally; Phase 1 Plan 01-09 pending human X4 verification
Resume file: None
