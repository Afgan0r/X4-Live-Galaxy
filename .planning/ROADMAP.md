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
recognize observed KHK activity, and remain bounded and recoverable during
unattended X4 9.00 AFK/SETA operation without mutating the game.

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
4. Disposable X4 9.00 evidence shows observation and transport work is bounded and no mutation command exists or can be emitted.

**Research**: Required — confirm exact X4 9.00 observation, identity, scheduling, transport, embedded Lua, and Mission Director semantics before relying on them.
**Plans**: TBD

### Phase 2: Hostile-Faction Research Track

**Goal**: Future hostile-mind design has versioned XEN/KHK evidence without expanding or delaying the ZYA/ARG Shadow Director implementation.
**Depends on**: Nothing (parallel research track; Phase 3 does not depend on it)
**Requirements**: RES-01, RES-02, RES-03
**Success Criteria** (what must be TRUE):

1. A verifier can inspect one versioned artifact covering XEN and KHK state, events, identity, visibility, scheduling, and known control limitations.
2. Every claim is labeled documented, observed, inferred, or unknown and cites only materially influential provenance.
3. The ZYA/ARG critical path can proceed with unresolved hostile-faction questions explicitly bounded rather than promoted into implementation scope.

**Research**: Required — use current X4 9.00 vanilla and disposable runtime evidence; treat installed mods and third-party code as read-only sources.
**Plans**: TBD

### Phase 3: Faction-Scoped Strategic State

**Goal**: ZYA, ARG, and their primitive institutions receive deterministic, replayable strategic state grounded in authoritative observations and permitted faction information.
**Depends on**: Phase 1
**Requirements**: OBS-04, OBS-05, MIND-02, MIND-03, MIND-04, INST-01, INST-02
**Success Criteria** (what must be TRUE):

1. Frozen snapshots supply the supported economic, military, territorial, and threat facts needed by both factions, with XEN as shared primary pressure and KHK represented when observed.
2. A verifier can inspect each faction's permitted information and the exact visibility policy used to construct it.
3. Equivalent frozen snapshots and policies produce canonically ordered strategic facts, priorities, allowed shadow primitives, and identical admission inputs.
4. Each faction has a versioned primitive institution roster with fixed doctrine-conditioned priorities over the same authoritative faction-visible snapshot.
5. Missing, stale, faction-inaccessible, private-institution, or unsupported facts cannot silently enter a faction or institution strategic packet.

**Research**: Not normally required — use established typed-domain and deterministic-replay patterns; reopen research only for unresolved X4 fact semantics.
**Plans**: TBD

### Phase 4: Persistent Full Faction Minds

**Goal**: Full, distinct ZYA and ARG minds preserve coherent short- and long-term strategy plus one-owner institution initiatives across compaction, restart, retry, and schema transitions.
**Depends on**: Phase 3
**Requirements**: MIND-01, MIND-05, INST-03, INST-08, MODEL-05, STATE-01, STATE-02, STATE-03, STATE-04, STATE-05
**Success Criteria** (what must be TRUE):

1. An operator can inspect independent ZYA and ARG doctrine, motives, priorities, goals, short-term plans, and long-term plans that differ meaningfully on shared scenarios.
2. Mind history compacts within model-relative budgets into versioned typed-plus-narrative capsules while typed facts remain authoritative.
3. Every institution owns at most one active typed Shadow initiative with stable identity, objective, evidence, priority, lifecycle state, and owner.
4. Proposal, objection, disposition, validation, ownership, preemption, and terminal outcome records persist as replayable causal evidence.
5. Accepted snapshots, mind state, initiative state, replay inputs, admission state, and report intent recover transactionally without duplicating a plan, tick, initiative, or report.
6. Corrupt, partial, incompatible, duplicate, out-of-order, and version-transition fixtures fail closed or recover the last valid state with structured evidence.
7. The persistence boundary keeps compact authoritative runtime state under an X4-owned contract and never reads or modifies player save files.

**Research**: Targeted — standard Rust/SQLite recovery patterns apply, but the X4-owned persistence contract requires exact evidence before implementation.
**Plans**: TBD

### Phase 5: Bounded Shadow Deliberation

**Goal**: ZYA and ARG can request, arbitrate, validate, and admit typed Shadow plans and institution initiatives from interchangeable providers without trusting provider output or affecting X4 state.
**Depends on**: Phase 4
**Requirements**: MIND-06, MIND-07, INST-04, INST-05, INST-06, INST-07, MODEL-01, MODEL-02, MODEL-03, MODEL-04, MODEL-06, MODEL-07
**Success Criteria** (what must be TRUE):

1. A strategic tick, relevant event, or cooldown can request deduplicated per-faction deliberation within explicit queue, time, retry, payload, context, call, and history bounds.
2. Ollama or a deterministic fake can be exchanged behind the same typed provider boundary without changing strategic domain behavior.
3. Only proposals passing schema, semantic, information, safety, budget, and current-state validation become typed shadow plans with goals, priorities, horizons, supporting facts, trade-offs, and safe explanations.
4. The Executive may originate, assign, approve, revise, preempt, reject, or terminate initiatives but cannot execute them directly or bypass deterministic admission.
5. Aligned proposals proceed without dialogue; only material objection, mandate, preemption, or revision may open at most two full dialogue cycles before one final kernel-valid Executive disposition.
6. Every replacement preserves the prior initiative, trigger, suspend-or-cancel disposition, replacement proposal, Executive decision, and reason.
7. Rejected or timed-out work records a bounded reason and cannot partially admit a plan or initiative, alter authoritative persistence, or mutate X4 state.
8. Exact versioned cache keys and recorded fixtures make cache behavior and normal replay evaluation reproducible without a live provider.

**Research**: Required — benchmark current Ollama provider/model behavior and derive operating bounds from evidence rather than selecting them in advance.
**Plans**: TBD

### Phase 6: Correlated Reports and Diagnostics

**Goal**: Operators receive safe concise X4 reports and complete, bounded external evidence for every shadow decision and degraded path.
**Depends on**: Phase 5
**Requirements**: DIAG-01, DIAG-02, DIAG-03, DIAG-04, DIAG-05
**Success Criteria** (what must be TRUE):

1. An accepted shadow decision can emit one concise deduplicated Mail or Logbook report through an existing X4 surface.
2. Stable correlation IDs let an operator trace observation, snapshot, faction knowledge, institution proposal or objection, Executive disposition, provider request, cache result, validation, accepted plan and initiative, report intent, and acknowledgement end to end.
3. Bounded diagnostics expose health, failures, latency, usage, cost, queues, recovery, and state quality during unattended operation.
4. Player-visible and public output contains no credentials, machine-local paths, private prompts, hidden reasoning, or recipient-inaccessible information.
5. Captured snapshots and traces reproduce decisions offline while remaining explicitly non-authoritative.

**Research**: Required — verify exact Mail or Logbook emission, acknowledgement, deduplication, and recipient-information semantics in disposable X4 9.00 scenarios.
**Plans**: TBD

### Phase 7: X4 Operational Proof

**Goal**: Verifiers can distinguish local contracts from observed X4 behavior and show that Live Galaxy remains bounded through normal-speed, SETA, reconnect, and unattended runs.
**Depends on**: Phase 6
**Requirements**: VAL-01, VAL-02, VAL-03
**Success Criteria** (what must be TRUE):

1. Static, pure Lua where applicable, fake-adapter, Rust, and disposable in-game checks report their evidence levels separately.
2. Disposable normal-speed and SETA runs show bounded game-side work, bridge backlog, and reconnect behavior with no observable Live Galaxy-caused vanilla simulation stall.
3. A defined unattended AFK/SETA soak continues observation, deliberation, persistence, recovery, reporting, and diagnostics for the measured duration and workload.
4. Failures during the soak degrade safely and retain enough correlated evidence to identify the affected boundary without corrupting X4 state.

**Research**: Required — determine feasible X4 automation, SETA workload limits, reconnect semantics, and evidence capture from disposable scenarios rather than assuming headless support.
**Plans**: TBD

### Phase 8: Evaluation and Internal Prototype Gate

**Goal**: The owner can judge Shadow Director quality and reliability from measured evidence and package milestone 0.1 strictly as an internal prototype.
**Depends on**: Phase 2 and Phase 7
**Requirements**: MODEL-08, MODEL-09, VAL-04, VAL-05
**Success Criteria** (what must be TRUE):

1. A versioned scenario corpus scores grounding, continuity, information discipline, faction divergence, institution contribution, initiative causality, strategic consistency, schema reliability, latency, cache behavior, and model cost.
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
| 1. Read-Only Observation Spine | 0/TBD | Not started | - |
| 2. Hostile-Faction Research Track | 0/TBD | Not started | - |
| 3. Faction-Scoped Strategic State | 0/TBD | Not started | - |
| 4. Persistent Full Faction Minds | 0/TBD | Not started | - |
| 5. Bounded Shadow Deliberation | 0/TBD | Not started | - |
| 6. Correlated Reports and Diagnostics | 0/TBD | Not started | - |
| 7. X4 Operational Proof | 0/TBD | Not started | - |
| 8. Evaluation and Internal Prototype Gate | 0/TBD | Not started | - |

---

*Roadmap created: 2026-08-28 for milestone 0.1 — Shadow Director*
