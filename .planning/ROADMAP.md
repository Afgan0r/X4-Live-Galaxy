# Roadmap: Live Galaxy

## Overview

Milestone 0.1 — Shadow Director is an internal observation-only prototype. The
roadmap first proves a bounded, read-only X4 observation path, then builds
replayable faction knowledge, persistent ZYA and ARG minds, primitive
fixed-priority institutions, provider-neutral Shadow deliberation, correlated
reporting, and unattended AFK/SETA evidence.
XEN and KHK research proceeds as an independent parallel track and does not
gate the ZYA/ARG implementation path. X4 remains authoritative throughout, and
no phase introduces game-state mutation or public-ready claims.

## Milestone 0.1 — Shadow Director

**Milestone goal:** Demonstrate that full ZYA and ARG Faction Minds can maintain
distinct, persistent, explainable Shadow strategies under shared XEN pressure,
coordinate primitive institutions through bounded Executive arbitration,
maintain an Executive-owned ZYA–ARG Shadow diplomatic posture, recognize
observed KHK activity, and remain bounded and recoverable during unattended X4
9.00 AFK/SETA operation without mutating the game.

## Phases

- [ ] **Phase 1: Read-Only Observation Spine** - Produce trustworthy, bounded, replayable X4 observations without mutation or game-thread stalls.
- [ ] **Phase 2: Hostile-Faction Research Track** - Establish versioned XEN/KHK evidence independently of the ZYA/ARG critical path.
- [ ] **Phase 3: Faction-Scoped Strategic State** - Turn frozen observations into deterministic faction and fixed-priority institution inputs for ZYA and ARG.
- [ ] **Phase 4: Persistent Full Faction Minds** - Preserve distinct ZYA/ARG goals, institution initiatives, causal history, and recovery state across interruption boundaries.
- [ ] **Phase 5: Bounded Shadow Deliberation** - Arbitrate and admit provider proposals and institution initiatives only through deterministic validation, budgets, and exact replay contracts.
- [ ] **Phase 6: Correlated Reports and Diagnostics** - Emit safe concise X4 reports while retaining complete external decision evidence.
- [ ] **Phase 7: X4 Operational Proof** - Demonstrate bounded normal-speed, SETA, reconnect, recovery, and unattended behavior in disposable X4 runs.
- [ ] **Phase 8: Evaluation and Internal Prototype Gate** - Measure strategic quality and reliability and classify the 0.1 evidence without public-ready claims.

## Phase Details

### Phase 1: Read-Only Observation Spine

**Goal**: Verifiers can obtain trustworthy, bounded, versioned X4 9.00 world-state observations without blocking or mutating the game.
**Depends on**: Nothing (first phase)
**Requirements**: OBS-01, OBS-02, OBS-03, OBS-06, OBS-07, OBS-08, VAL-06
**Success Criteria** (what must be TRUE):

1. A verifier can capture a versioned observation whose entities and events carry stable typed identities, source, time, and monotonic versions.
2. Captured sections distinguish fresh, stale, partial, unknown, and unsupported data while discovering sectors, assets, capacity, and ownership from runtime state.
3. Malformed, oversized, duplicate, stale, and out-of-order input leaves the last accepted snapshot intact and produces bounded rejection or reconciliation evidence.
4. The X4 adapter and Rust bridge negotiate protocol and capability identities before accepting traffic, and unsupported combinations enter a bounded fail-closed degraded state with an explicit restart condition.
5. Disposable X4 9.00 evidence shows observation and transport work is bounded and no mutation command exists or can be emitted.

**Research**: Required — confirm exact X4 9.00 observation, identity, scheduling, transport, embedded Lua, Mission Director, compatibility-negotiation, degraded-mode, and restart-condition semantics before relying on them.
**Plans**: 9 plans

- [x] 01-01-PLAN.md
- [x] 01-02-PLAN.md
- [x] 01-03-PLAN.md
- [x] 01-04-PLAN.md
- [x] 01-05-PLAN.md
- [x] 01-06-PLAN.md
- [x] 01-07-PLAN.md
- [x] 01-08-PLAN.md
- [ ] 01-09-PLAN.md

### Phase 2: Hostile-Faction Research Track

**Goal**: Future hostile-mind design has versioned XEN/KHK evidence without expanding or delaying the ZYA/ARG Shadow Director implementation.
**Depends on**: Nothing (parallel research track; Phase 3 does not depend on it)
**Requirements**: RES-01, RES-02, RES-03
**Success Criteria** (what must be TRUE):

1. A verifier can inspect one versioned artifact covering XEN and KHK state, events, identity, visibility, scheduling, and known control limitations.
2. Every claim is labeled documented, observed, inferred, or unknown and cites only materially influential provenance.
3. The ZYA/ARG critical path can proceed with unresolved hostile-faction questions explicitly bounded rather than promoted into implementation scope.

**Research**: Required — produce installed X4 9.00 static vanilla evidence now; Phase 1 and Phase 7 gather disposable runtime evidence through their scheduled X4 gates, which remains versioned non-gating input until observed. Treat installed mods and third-party code as read-only sources; no separate Phase 2 human gate.
**Plans**: 1/1 plans executed

Plans:

- [x] 02-01-PLAN.md — Produce and audit the versioned XEN/KHK static-evidence record.

### Phase 3: Faction-Scoped Strategic State

**Goal**: ZYA, ARG, and their primitive institutions receive deterministic, replayable strategic state grounded in authoritative observations and permitted faction information.
**Depends on**: Phase 1
**Requirements**: OBS-04, OBS-05, MIND-02, MIND-03, MIND-04, INST-01, INST-02
**Success Criteria** (what must be TRUE):

1. Frozen snapshots supply the supported economic, military, territorial, and threat facts needed by both factions, with XEN as shared primary pressure and KHK represented when observed.
2. A verifier can inspect each faction's permitted information and the exact visibility policy used to construct it.
3. Equivalent frozen snapshots and policies produce canonically ordered strategic facts, priorities, allowed shadow primitives, and identical admission inputs.
4. Each faction has exactly three versioned primitive institutions mapped to defense and military strategy, economy and logistics, and territorial development and infrastructure, with canon-grounded identities and fixed doctrine-conditioned priorities over the same authoritative faction-visible snapshot.
5. Missing, stale, faction-inaccessible, private-institution, or unsupported facts cannot silently enter a faction or institution strategic packet.

**Research**: Targeted — use established typed-domain and deterministic-replay patterns, but verify canon-grounded ZYA/ARG institution identities and reopen X4 fact research only for unresolved semantics.
**Plans**: 3/3 plans executed

Plans:

- [x] 03-01-PLAN.md — Compile the pure availability-aware faction packet tracer.
- [x] 03-02-PLAN.md — Add exact shared institution capabilities and doctrine profiles.
- [x] 03-03-PLAN.md — Add finite Shadow primitives, Executive diplomacy inputs, replay identity, and mutation evidence.

### Phase 4: Persistent Full Faction Minds

**Goal**: Full, distinct ZYA and ARG minds preserve coherent short- and long-term strategy plus one-owner institution initiatives across compaction, restart, retry, and schema transitions.
**Depends on**: Phase 3
**Requirements**: MIND-01, MIND-05, INST-03, INST-08, MODEL-05, STATE-01, STATE-02, STATE-03, STATE-04, STATE-05, STATE-06
**Success Criteria** (what must be TRUE):

1. An operator can inspect independent ZYA and ARG doctrine, motives, priorities, goals, short-term plans, long-term plans, and Executive-owned typed diplomatic postures that differ meaningfully on shared scenarios.
2. Mind history compacts within model-relative budgets into versioned typed-plus-narrative capsules while typed facts remain authoritative.
3. Every institution owns at most one active typed Shadow initiative with stable identity, objective, evidence, priority, lifecycle state, and owner.
4. Proposal, objection, disposition, validation, ownership, preemption, and terminal outcome records persist as replayable causal evidence.
5. Accepted snapshots, mind state, initiative state, replay inputs, admission state, and report intent recover transactionally without duplicating a plan, tick, initiative, or report.
6. Corrupt, partial, incompatible, duplicate, out-of-order, and version-transition fixtures fail closed or recover the last valid state with structured evidence.
7. The persistence boundary keeps compact authoritative runtime state under an X4-owned contract and never reads or modifies player save files.
8. A protocol-compatible Rust release can restart, update, and reconnect while X4 remains running, preserving accepted state and report identity; an incompatible game-side protocol revision fails closed and names the X4-restart requirement.

**Research**: Targeted — standard Rust recovery patterns apply; documented Mission Director save-state semantics support the X4-owned checkpoint contract, while payload and interruption behavior remain pending the existing Phase 7 runtime gate.
**Plans**: 4/4 plans executed

Plans:

- [x] 04-01-PLAN.md — Build deterministic independent mind aggregates and causal initiative lifecycle.
- [x] 04-02-PLAN.md — Declare and statically validate the X4-owned MD checkpoint contract.
- [x] 04-03-PLAN.md — Add canonical checkpoint codec and acknowledged fake-port recovery contract.
- [x] 04-04-PLAN.md — Add fail-closed recovery, schema migration, and typed-authoritative compaction.

### Phase 5: Bounded Shadow Deliberation

**Goal**: ZYA and ARG can request, arbitrate, validate, and admit typed Shadow plans and institution initiatives from interchangeable providers without trusting provider output or affecting X4 state.
**Depends on**: Phase 4
**Requirements**: MIND-06, MIND-07, INST-04, INST-05, INST-06, INST-07, MODEL-01, MODEL-02, MODEL-03, MODEL-04, MODEL-06, MODEL-07
**Success Criteria** (what must be TRUE):

1. A strategic tick, relevant event, or cooldown can request deduplicated per-faction deliberation within explicit queue, time, retry, payload, context, call, and history bounds.
2. A developer-controlled subscription harness or a deterministic fake can be exchanged behind the same typed provider boundary without changing strategic domain behavior; the fake is contract-test evidence only.
3. Only proposals passing schema, semantic, information, safety, budget, and current-state validation become typed shadow plans with goals, priorities, horizons, supporting facts, trade-offs, and safe explanations.
4. The Executive may originate, assign, approve, revise, preempt, reject, or terminate initiatives but cannot execute them directly or bypass deterministic admission.
5. The Executive may maintain, de-escalate, intensify, or seek limited threat-driven coordination in its typed ZYA–ARG Shadow diplomatic posture, but no inter-faction negotiation or X4 relationship mutation occurs.
6. Aligned proposals proceed without dialogue; only material objection, mandate, preemption, or revision may open at most two full dialogue cycles before one final kernel-valid Executive disposition.
7. Every replacement preserves the prior initiative, trigger, suspend-or-cancel disposition, replacement proposal, Executive decision, and reason.
8. Rejected or timed-out work records a bounded reason and cannot partially admit a plan or initiative, alter authoritative persistence, or mutate X4 state.
9. Exact versioned cache keys and recorded fixtures make cache behavior and normal replay evaluation reproducible without a live provider.

**Research**: Required — benchmark available subscription-backed models and harness behavior, then derive operating bounds from evidence rather than selecting a public runtime API or local provider in advance.
**Plans**: TBD

### Phase 6: Correlated Reports and Diagnostics

**Goal**: Operators receive safe concise X4 reports and complete, bounded external evidence for every shadow decision and degraded path.
**Depends on**: Phase 5
**Requirements**: DIAG-01, DIAG-02, DIAG-03, DIAG-04, DIAG-05
**Success Criteria** (what must be TRUE):

1. A materially changed completed strategic cycle emits one concise deduplicated faction Logbook summary; Mail is reserved for critical strategic changes and bridge or model degradation and recovery. Both use the bounded allowlisted return channel, and no mutation command is admitted.
2. Stable correlation IDs let an operator trace observation, snapshot, faction knowledge, institution proposal or objection, Executive disposition, provider request, cache result, validation, accepted plan and initiative, report intent, and acknowledgement end to end.
3. Bounded diagnostics expose health, failures, latency, usage, cost, queues, recovery, and state quality during unattended operation.
4. Player-visible and public output contains no credentials, machine-local paths, private prompts, hidden reasoning, or recipient-inaccessible information.
5. Captured snapshots and traces reproduce decisions offline while remaining explicitly non-authoritative.

**Research**: Required — verify the exact bounded report return channel, Mail or Logbook emission, acknowledgement, deduplication, and recipient-information semantics in disposable X4 9.00 scenarios.
**Plans**: TBD

### Phase 7: X4 Operational Proof

**Goal**: Verifiers can distinguish local contracts from observed X4 behavior and show that Live Galaxy remains bounded through normal-speed, SETA, reconnect, and unattended runs.
**Depends on**: Phase 6
**Requirements**: VAL-01, VAL-02, VAL-03
**Success Criteria** (what must be TRUE):

1. Static, pure Lua where applicable, fake-adapter, Rust, and disposable in-game checks report their evidence levels separately.
2. Disposable normal-speed and SETA runs show bounded game-side work, bridge backlog, and reconnect behavior with no observable Live Galaxy-caused vanilla simulation stall.
3. A compatible Rust process restart and update reconnects to the same running X4 process without loss or duplication of accepted state, strategic ticks, or reports; an incompatible protocol fails closed under the defined X4-restart condition.
4. A defined unattended AFK/SETA soak continues observation, deliberation, persistence, recovery, reporting, and diagnostics for the measured duration and workload.
5. Failures during the soak degrade safely and retain enough correlated evidence to identify the affected boundary without corrupting X4 state.

**Research**: Required — determine feasible X4 automation, SETA workload limits, compatible Rust restart and reconnect semantics, exact incompatible-protocol restart conditions, and evidence capture from disposable scenarios rather than assuming headless support.
**Plans**: TBD

### Phase 8: Evaluation and Internal Prototype Gate

**Goal**: The owner can judge Shadow Director quality and reliability from measured evidence and package milestone 0.1 strictly as an internal prototype.
**Depends on**: Phase 2 and Phase 7
**Requirements**: MODEL-08, MODEL-09, VAL-04, VAL-05
**Success Criteria** (what must be TRUE):

1. A versioned scenario corpus scores complete subscription-backed real-model trajectories for grounding, continuity, information discipline, faction divergence, institution contribution, initiative causality, strategic consistency, schema reliability, latency, cache behavior, and model cost; fake trajectories remain test-only.
2. Strategic-quality acceptance is blocked unless an independently measured reliability floor passes, with all thresholds derived from recorded baselines.
3. Representative pure high-risk Rust and Lua logic has measured mutation-tool baselines, reviewed survivors, and evidence-based operator and threshold decisions.
4. The milestone package and evidence inventory separates implemented, locally verified, pending game smoke, and observed-in-X4 claims and never describes 0.1 as playable or public-ready.
5. The final inventory includes the bounded XEN/KHK research artifact without making autonomous hostile minds part of milestone 0.1.

**Research**: Targeted — validate mutation-tool compatibility and corpus adequacy only where measured evidence leaves a concrete gap.
**Plans**: TBD

## Progress

**Execution order:** Phase 1 starts the critical path. Phase 2 is an independent
research track and may run in parallel; Phase 3 depends only on Phase 1. The
critical implementation path continues through Phases 3–7. Phase 8 requires
both the research artifact from Phase 2 and operational evidence from Phase 7.

| Phase | Plans Complete | Status | Completed |
| --- | --- | --- | --- |
| 1. Read-Only Observation Spine | 8/9 | In Progress |  |
| 2. Hostile-Faction Research Track | 1/1 | Complete | 2026-08-29 |
| 3. Faction-Scoped Strategic State | 3/3 | Complete | 2026-08-29 |
| 4. Persistent Full Faction Minds | 4/4 | In Progress |  |
| 5. Bounded Shadow Deliberation | 0/TBD | Not started | - |
| 6. Correlated Reports and Diagnostics | 0/TBD | Not started | - |
| 7. X4 Operational Proof | 0/TBD | Not started | - |
| 8. Evaluation and Internal Prototype Gate | 0/TBD | Not started | - |

---

*Roadmap created: 2026-08-28 for milestone 0.1 — Shadow Director*
