---
gsd_state_version: 1.0
current_phase: 05.3
current_phase_name: Generic Observation Contracts and Durable Publication
status: verifying
stopped_at: Completed 05.3-03-PLAN.md
last_updated: "2026-09-04T15:52:49.400Z"
last_activity: 2026-09-04
last_activity_desc: Phase 05.3 execution started
state_head: c2a8226c4178a6801b2ead91fa6bf48edf3e3e52
progress:
  total_phases: 14
  completed_phases: 5
  total_plans: 40
  completed_plans: 40
  percent: 36
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-08-28)

**Core value:** Factions pursue coherent, distinct, long-lived strategies while X4 remains authoritative and every proposed effect stays observable, recoverable, and bounded by deterministic validation.
**Current focus:** Phase 05.3 — Generic Observation Contracts and Durable Publication

## Current Position

Phase: 05.3 (Generic Observation Contracts and Durable Publication) — EXECUTING
Plan: 3 of 3
Status: Phase complete — ready for verification
Last activity: 2026-09-04 — Phase 05.3 execution started

Progress: [████░░░░░░] 36% (5 of 14 phases historically complete; Phase 05.3 has 3 pending plans)

## Performance Metrics

**Velocity:**

- Total plans completed: 22
- Average duration: 24.2 min across 6 recorded durations
- Total execution time: 2 hours 25 min across 6 recorded durations; Plan 01-07 duration not recorded

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
| --- | --- | --- | --- |
| 01 | 7 | 2 hours 25 min recorded | 24.2 min (6 recorded) |
| 05 | 5 | - | - |
| 05.2 | 10 | - | - |

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
| Phase 05 P01 | 0 | 3 tasks | 10 files |
| Phase 05 P02 | 25m | 3 tasks | 7 files |
| Phase 05 P03 | resumed execution | 3 tasks | 21 files |
| Phase 05 P04 | 4m | 3 tasks | 8 files |
| Phase 05 P05 | resumed execution | 3 tasks | 10 files |
| Phase 05.1 P01 | 28m | 3 tasks | 4 files |
| Phase 05.1 P02 | 24m | 2 tasks | 8 files |
| Phase 05.2 P01 | 19min | 3 tasks | 8 files |
| Phase 05.2 P02 | 14min | 2 tasks | 5 files |
| Phase 05.2 P03 | 15min | 2 tasks | 5 files |
| Phase 05.2 P04 | 14min | 2 tasks | 6 files |
| Phase 05.2 P05 | 31min | 2 tasks | 7 files |
| Phase 05.2 P06 | 27min | 3 tasks | 10 files |
| Phase 05.2 P07 | 49 min | 3 tasks | 9 files |
| Phase 05.2 P10 | 1h | 3 tasks | 9 files |
| Phase 05.1 P05 | 25m | 2 tasks | 6 files |
| Phase 05.1 P06 | 29m | 2 tasks | 6 files |
| Phase 05.1 P08 | 41m | 2 tasks | 5 files |
| Phase 05.3 P01 | 27min | 3 tasks | 13 files |
| Phase 05.3 P02 | 49min | 3 tasks | 19 files |
| Phase 05.3 P03 | 17h 17m | 3 tasks | 19 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Historical decisions follow. ADR-LG-001 through ADR-LG-024 and 05.3-CONTEXT.md override conflicting numerical, transport and retired research-platform policies:

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
- [Phase 05]: Cache entries supply bytes only; current admission revalidates every hit.
- [Phase 05]: Preemption records cross normal admission and must match restored checkpoint command history.
- [Phase 05]: Shadow posture remains frozen-fact evidence with no external effect projection.
- [Phase 05]: Provider adapters return bytes or typed failures; only shared admission and checkpoint persistence can affect authoritative state.
- [Phase 05]: Deterministic fixture evidence proves contracts and replay only; manual harness evidence is distinct from quality acceptance.
- [Phase 05]: The subscription route is manual-only and returns unavailable instead of using an API fallback.
- [Phase 05]: Benchmark evidence is bounded and redacted, never authoritative state or a quality threshold.
- [Phase 05.1]: Phase 05.1 uses a fixed two-path telemetry-only package allowlist before production component discovery.
- [Phase 05.1]: Station facts require a complete canonical envelope before accepted projection replacement.
- [Phase 05.1]: Failed component discovery emits health-only frames without a completion marker.
- [Phase 05.1]: Complete owner-scoped station facts validate before telemetry serialization.
- [Phase 05.2]: The complete dossier is a validation fixture and does not admit the runtime-pending Phase 05.1 seam.
- [Phase 05.2]: Registry growth requires evidence and executable coverage for every current row before dossier evaluation.
- [Phase 05.2]: Owner overrides bind one finding to the exact dossier digest, explicit decision, remaining risk, and a maximum 90-day expiry.
- [Phase 05.2]: Production-faithful package evidence requires the registered entrypoint, contained complete import graph, and one direct ffi.C acquisition site.
- [Phase 05.2]: Package conformance remains packaged-static and digest-bound; Plan 01 remains the independent admission authority.
- [Phase 05.2]: Bare, test-only, and alternate-binding-only loading is local-only and cannot satisfy AC-04.
- [Phase 05.2]: Every candidate emits exactly three ordered rows whose verdict progression is explicit and whose canonical payload is verified through an injected SHA-256 adapter.
- [Phase 05.2]: Candidate exceptions, timeouts, invalid results, digest failures, and effect mismatches emit closed privacy-safe reason codes; later candidates continue in sorted order.
- [Phase 05.2]: The source-derived fixture proves only a local developer contract and never claims observed-in-X4 evidence.
- [Phase 05.2]: Phase 05.2 Plan 04 partitions six identical read-only profiles together and isolates lifecycle registration instrumentation in one exclusive build.
- [Phase 05.2]: Generated candidate roots remain developer-only and prepared-not-executed, with per-group package conformance bound into each manifest.
- [Phase 05.2]: Source-resolvable Phase 05.1 design defects remain explicit exclusions rather than X4 runtime candidates.
- [Phase 05.2]: Retained runtime evidence remains private; Git receives only sanitized logical identities, verdicts, dispositions, and digests.
- [Phase 05.2]: Admission compares a completed sanitized ledger with the committed pending ledger and candidate matrix, never raw JSONL or private locator paths.
- [Phase 05.2]: Phase 05.1 runtime execution remains a later human-only disposable-campaign gate; Phase 05.2 makes no execution or admission claim.
- [Phase 05.2]: Production owner authority remains explicitly unconfigured until a separate owner-controlled reviewed ceremony updates both the fixed anchor contract and compiled pin.
- [Phase 05.2]: Owner overrides require a root-signed owner-override delegation bound to exact purpose, epoch, scope, policy, delegated SPKI, and matching non-exportable Windows CNG key.
- [Phase 05.2]: TEST-ONLY authority stays confined to explicit fixture helpers and is never accepted by production admission or signing entry points.
- [Phase 05.2]: Candidate declarations are exact inert data; callbacks, metatables, digests, watchdogs, and verdict authority are rejected.
- [Phase 05.2]: The pure runner owns completeness and expected-effect verdicts; only the trusted registry supplies local-contract adapters.
- [Phase 05.2]: Blocking candidate work uses one fixed no-shell worker with canonical bounded files, deadlines, and process-tree termination.
- [Phase 05.2]: Arm the descendant execution timeout only after a separate bounded readiness handshake succeeds.
- [Phase 05.2]: Use one canonical full-graph verifier for public and inert candidate packages with explicit native-binding policy.
- [Phase 05.2]: Gate every full-file read and digest from metadata before allocation, then revalidate identity after reading.
- [Phase 05.1]: The 1800-byte value is a canonical encoded frame ceiling only; a generation has no aggregate frame or byte cap.
- [Phase 05.1]: The Lua FIFO owns 16-message and 28800-byte concurrent capacities with independent watermarks and quotas.
- [Phase 05.1]: A successful named-pipe write is recorded only as local handoff, never as Rust acknowledgement or admission.
- [Phase 05.1]: Native discovery derives finite allocation and exact work limits from 129 members and the actual UniverseID size. — This replaces the superseded product count cap with source-owned measurable resource admission.
- [Phase 05.1]: The Lua package authority guard accepts exactly four named production files. — Fixed paths prevent unrelated production code from acquiring the phase telemetry vocabulary.
- [Phase 05.1]: Incremental generation candidates reconcile from their exact staged canonical membership rather than legacy aggregate ceilings. — This preserves strict atomic admission while allowing 129-plus streamed frames.
- [Phase 05.1]: Restart callers restore the last admitted generation explicitly. — Generation identity cannot be inferred safely from unrelated snapshot fields.
- [Phase 05.3]: ADR-LG-001 through ADR-LG-024 and 05.3-CONTEXT.md supersede conflicting historical Phase 01/05.1/05.2 entries: use source-agnostic sections, one immutable stop-and-wait contract slot, distinct received/committed, bounded keyed candidates and independent aggregate bounds; never resume the old remainder or retired research platform. — Owner approved GSD reconciliation on 2026-09-03; past implementation evidence is preserved without promoting obsolete policies.
- [Phases 05.3/05.4]: Docs MCP evidence demonstrated Carrier A's bounded-result gap. The owner selected Carrier B and required a separate phase: 05.3 owns generic Rust contracts/publication, while 05.4 owns the minimal native/Lua production path and runtime proof. Heavy ship and station proof move to 05.5/05.6. — Owner approved on 2026-09-04.
- [Phase 05.3]: Durable publication remains distinct from local handoff and volatile receipt; ambiguous publication never advances the sender.
- [Phase 05.3]: Game-time urgency is an ordering hint only; injected monotonic real time and finite non-zero limits own admission.
- [Phase 05.3]: The X4 facade exposes complete-message transport and bounded control vocabulary without any game-state mutation command.
- [Phase 05.3]: Candidate and aggregate limits remain injected non-zero policy; no production numeric default was introduced.
- [Phase 05.3]: Source uncertainty retires matching accepted revisions to history and drops matching volatile candidates without inventing authority.
- [Phase 05.3]: Deletion requires two qualifying same-scope absences; weak evidence preserves the core entity.
- [Phase 05.3]: Durable publication uses one immediate transaction for immutable content, terminal receipt, and conditional current-pointer advancement.
- [Phase 05.3]: Unknown commit results and post-commit response loss remain ambiguous until exact durable receipt and pointer reconciliation.
- [Phase 05.3]: Retention preserves all current, dependency-reachable, and decision-pinned revisions; unpinning only removes reachability.

### Pending Todos

None yet.

### Blockers/Concerns

- [Phase 1]: Exact X4 9.00 observation, transport, embedded Lua, Mission Director, identity, scheduling, protocol negotiation, degraded-mode, and restart-condition semantics require phase research and disposable evidence.
- [Phase 4]: The X4-owned compact persistence contract remains an evidence-dependent boundary decision; player save files are prohibited.
- [Phase 6]: The bounded Rust-to-X4 Mail/Logbook return channel and acknowledgement semantics require disposable evidence; topology and framing remain technical decisions.
- [Phase 05.4]: Carrier A cannot prove bounded nonblocking writes or typed short-write/error outcomes. Carrier B requires dedicated discussion, native-boundary research, security review and disposable X4 proof. Automatic save resume remains unsupported under current evidence.

### Quick Tasks Completed

| # | Description | Date | Commit | Status | Directory |
| --- | --- | --- | --- | --- | --- |
| 260831-3m9 | Reduce Phase 05.2 test iteration time | 2026-08-31 | 3f3d9b5 | | [260831-3m9-reduce-phase-05-2-test-iteration-time-by](./quick/260831-3m9-reduce-phase-05-2-test-iteration-time-by/) |
| 260903-06l | Retire research tooling; product all passes in 38.060 s | 2026-09-03 | 57db7ed | Verified | [260903-06l-remove-obsolete-x4-research-verification](./quick/260903-06l-remove-obsolete-x4-research-verification/) |
| 260903-32k | Replace custom Lua tooling with Busted; product all passes in 1.449 s | 2026-09-03 | 8783d31 | Verified | [260903-32k-replace-custom-lua-test-infrastructure-w](./quick/260903-32k-replace-custom-lua-test-infrastructure-w/) |

### Roadmap Evolution

- Phase 05.1 inserted after Phase 5: Bounded Real Component Discovery; conditional successor for actual component-level X4 telemetry, separated from the Phase 1 transport spine. (URGENT)
- Phase 05.2 inserted after Phase 5 and reordered before further Phase 05.1 X4 execution: it must build the admission gate and read-only candidate harness first; Phase 05.1 then owns one prepared X4 run over the remaining candidate matrix. (URGENT)
- Phase 05.3 inserted after Phase 5: Owner-approved ADR-LG-023 reconciliation originally grouped generic flow and production transport before heavy ship and station proof. Historical 01/05.1 gaps remain open under successor owners.
- Phase 05.4 inserted after Phase 05.3: Docs MCP evidence triggered an owned Carrier B. The owner split it from 05.3 on 2026-09-04; heavy ship proof moved to 05.5 and conditional station remainder to 05.6.

## Deferred Items

Items acknowledged and deferred at milestone close, most recent first:

| Category | Item | Status | Deferred At | Milestone |
| --- | --- | --- | --- | --- |
| Scope | All mutation, autonomous XEN/KHK minds, player missions, custom interface, broad faction rollout, compatibility gates, and public release work | Deferred | Initialization | Later milestones |

## Session Continuity

Last session: 2026-09-04T15:52:48.489Z
Stopped at: Completed 05.3-03-PLAN.md
Resume file: None
