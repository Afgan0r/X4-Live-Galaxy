# Requirements: Live Galaxy Milestone 0.1

**Defined:** 2026-08-28
**Core value:** Factions pursue coherent, distinct, long-lived strategies while X4 remains authoritative and every proposed effect stays observable, recoverable, and bounded by deterministic validation.

## Milestone 0.1 Requirements

### Observation and Normalization

- [x] **OBS-01**: The system can ingest bounded, versioned observation envelopes from X4 without blocking the game thread on bridge or model work, and the X4 adapter and Rust bridge negotiate explicit transport/session capabilities before accepting traffic.
- [x] **OBS-02**: Every observed entity and event used by strategy has a stable typed identity, source, observation time, and monotonic state or event version.
- [x] **OBS-03**: Normalized world-state sections preserve freshness, coverage, quality, and explicit unknown or unsupported states instead of fabricating missing facts.
- [x] **OBS-04**: The observation model provides the supported economic, military, territorial, and threat facts required by the ZYA and ARG minds.
- [x] **OBS-05**: XEN is represented as the primary hostile pressure shared by ZYA and ARG, and KHK is recognized when authoritative observations contain it.
- [x] **OBS-06**: Runtime sectors, assets, capacity, and ownership are discovered from observed state rather than assumed from a fixed vanilla map or job count.
- [x] **OBS-07**: Malformed, oversized, duplicate, stale, and out-of-order observation input is rejected or reconciled without corrupting the last accepted snapshot.
- [x] **OBS-08**: Milestone 0.1 X4 integration exposes no fleet, economy, diplomacy, institution, or other game-state mutation command.

### Faction Minds and Deterministic Strategy

- [x] **MIND-01**: ZYA and ARG each have an independent full Faction Mind with explicit doctrine, motives, priorities, goals, short-term plans, long-term plans, and an Executive-owned typed Shadow diplomatic posture for their mutual relationship.
- [x] **MIND-02**: Each Faction Mind receives only authoritative truth and information available to that faction, with the applied visibility policy recorded for replay.
- [x] **MIND-03**: The deterministic kernel derives bounded strategic facts, priorities, and allowed shadow primitives from a frozen normalized snapshot.
- [x] **MIND-04**: Equivalent replay inputs produce canonically ordered, reproducible deterministic inputs and admission results.
- [x] **MIND-05**: ZYA and ARG exhibit measurably distinct strategic responses to shared scenarios rather than differing only by faction labels.
- [x] **MIND-06**: Accepted shadow plans and Executive diplomatic postures carry typed goals or dispositions, priorities, horizon, supporting facts, expected trade-offs, and safe player-facing explanations without opening inter-faction negotiation.
- [ ] **MIND-07**: A strategic tick, relevant event trigger, or cooldown can request deliberation while per-faction scheduling remains bounded and deduplicated.

### Primitive Institutions

- [x] **INST-01**: ZYA and ARG each have exactly three primitive institutions mapped to defense and military strategy, economy and logistics, and territorial development and infrastructure; their canon-grounded identities and fixed priorities are versioned and conditioned by faction doctrine.
- [x] **INST-02**: Every 0.1 institution reasons from the same authoritative faction-visible snapshot and cannot introduce private institutional knowledge or unsupported facts.
- [x] **INST-03**: Each institution owns at most one active typed Shadow initiative with stable identity, objective, supporting evidence, priority, lifecycle state, and owner.
- [ ] **INST-04**: The Executive Brain may originate, assign, approve, revise, preempt, reject, or terminate an institution initiative but cannot execute it directly or bypass deterministic admission.
- [ ] **INST-05**: Replacing active work requires an explicit preemption request containing the trigger, previous initiative state, suspend-or-cancel disposition, replacement, Executive decision, and preserved reason.
- [ ] **INST-06**: Direct Executive–institution agreement proceeds without dialogue; only a material objection, forced mandate, preemption, or revision can open negotiation.
- [ ] **INST-07**: Exceptional Executive–institution negotiation is capped at two full dialogue cycles and ends in one final kernel-valid Executive disposition without political refusal or sabotage.
- [x] **INST-08**: Proposal, objection, disposition, validation, ownership, preemption, and terminal outcome records persist as replayable causal evidence without mutating X4.

### Model Orchestration and Evaluation

- [ ] **MODEL-01**: Real-model prototype deliberation is isolated behind a provider-neutral typed interface and uses a developer-controlled subscription harness before `1.0.0`; public runtime API integration is not required by milestone 0.1.
- [x] **MODEL-02**: Provider output remains untrusted until schema, semantic, information, safety, budget, and current-state validation all succeed.
- [x] **MODEL-03**: Rejected or timed-out provider work records a bounded reason and cannot partially admit a plan or alter authoritative X4 state.
- [ ] **MODEL-04**: Cache entries use exact versioned keys covering faction, snapshot, policy, prompt package, schema, provider, model, and relevant generation settings.
- [x] **MODEL-05**: Context and history compaction uses model-relative token budgets and produces versioned typed-plus-narrative capsules whose typed facts remain authoritative.
- [ ] **MODEL-06**: Provider calls, retries, time, payload size, context size, queue depth, and retained history have explicit enforceable bounds.
- [ ] **MODEL-07**: Recorded fixtures and deterministic fakes allow normal tests and replay evaluation to run without a live model, but fake output cannot satisfy real-model strategic-quality acceptance.
- [ ] **MODEL-08**: A scenario corpus scores factual grounding, continuity, information discipline, faction divergence, strategic consistency, schema reliability, latency, cache behavior, and model cost.
- [ ] **MODEL-09**: Strategic-quality acceptance uses complete subscription-backed real-model trajectories, is gated by an independently measured reliability floor, and derives thresholds from recorded baselines rather than invented targets.

### Persistence and Recovery

- [x] **STATE-01**: Compact authoritative runtime state has a versioned X4-owned persistence contract, while external cache, diagnostics, and prose remain explicitly non-authoritative.
- [x] **STATE-02**: The Rust bridge persists accepted snapshots, mind state, replay inputs, admission state, and report intent transactionally with schema-version metadata.
- [x] **STATE-03**: Restart recovery is idempotent: replay, reconnect, and retry cannot duplicate an accepted plan, strategic tick, or report.
- [x] **STATE-04**: A corrupt, incompatible, or partial persisted record fails closed with structured diagnostics while preserving the last recoverable state.
- [x] **STATE-05**: Recovery and migration behavior is executable against crash-point, duplicate, out-of-order, and version-transition fixtures without reading or modifying player save files.
- [x] **STATE-06**: A protocol-compatible Rust bridge release can restart, update, and reconnect without restarting X4 or losing or duplicating accepted state; an incompatible game-side protocol revision fails closed with an explicit X4-restart requirement.

### Reports and Diagnostics

- [ ] **DIAG-01**: A materially changed completed strategic cycle emits one concise deduplicated faction Logbook summary, while Mail is reserved for critical strategic changes and bridge or model degradation and recovery; both use a bounded allowlisted Rust-to-X4 return channel with no mutation command.
- [ ] **DIAG-02**: External diagnostics correlate observation, snapshot, faction view, model request, cache result, validation, accepted plan, report intent, and acknowledgement by stable IDs.
- [ ] **DIAG-03**: Diagnostics expose bounded health, failure, latency, usage, cost, queue, recovery, and state-quality evidence suitable for unattended runs.
- [ ] **DIAG-04**: Public or player-visible output excludes credentials, machine-local paths, private prompts, raw hidden reasoning, and information unavailable to the recipient.
- [ ] **DIAG-05**: Captured snapshots and decision traces contain enough versioned inputs to reproduce a strategic decision offline without making external evidence authoritative.

### X4 and Milestone Validation

- [ ] **VAL-01**: X4 integration has layered static, pure Lua where applicable, fake-adapter contract, Rust, and disposable in-game tests with evidence levels reported separately.
- [ ] **VAL-02**: Disposable normal-speed and SETA runs demonstrate bounded game-side work, bounded bridge backlog, reconnect behavior, and no observable vanilla simulation stall caused by Live Galaxy.
- [ ] **VAL-03**: An unattended AFK/SETA soak demonstrates continuing observation, deliberation, persistence, recovery, reporting, and diagnostics for the defined duration and measured workload.
- [ ] **VAL-04**: Mutation testing is applied to representative pure high-risk Rust and Lua logic only after measured baselines establish useful operators and thresholds.
- [ ] **VAL-05**: The milestone package and evidence inventory distinguishes implemented, locally verified, pending game smoke, and observed-in-X4 claims and does not describe 0.1 as playable or public-ready.
- [ ] **VAL-06**: Exact X4 9.00 observation, identity, scheduling, transport, persistence, and Mail or Logbook semantics used by the implementation are backed by documented or disposable observed evidence.

### Parallel Hostile-Faction Research

- [x] **RES-01**: A versioned research artifact records observed XEN and KHK state, events, identity, visibility, scheduling, and control limitations relevant to future autonomous minds.
- [x] **RES-02**: XEN and KHK research runs independently of the ZYA and ARG critical path and cannot delay or silently expand milestone 0.1 implementation scope.
- [x] **RES-03**: Research separates documented, observed, inferred, and unknown claims and records provenance only for sources that materially influence Live Galaxy design.

## Later Milestones

### Autonomous Effects

- **AUTO-01**: Faction Minds can propose and execute bounded fleet, economy, diplomacy, and institution effects through deterministic game-side validation.
- **AUTO-02**: Closed-loop outcomes feed back into faction state without duplicating or partially applying actions after recovery.
- **AUTO-03**: Primitive institutions execute accepted initiatives while the Executive remains the brain and allocator rather than a direct game-world executor.
- **AUTO-04**: First-public-alpha diplomacy is limited to declaring and ending bilateral wars under deterministic legality, cooldown, compatibility, and safety constraints.

### Product Expansion

- **PROD-01**: A private gameplay-ready build supports the selected vanilla and DLC faction roster.
- **PROD-02**: The first public alpha passes evidence-based compatibility gates for KUDA AI Tweaks, More AI Economy Ships, and Add More Sectors while remaining explicitly incompatible with Faction Enhancer.
- **PROD-03**: Player missions and Player Influence are introduced only after the autonomous faction core is proven.
- **PROD-04**: A custom dossier, chronicle, or institution interface is introduced in a separate post-alpha milestone.
- **PROD-05**: Version 1.0.0 is released as the first public alpha only after private closed-loop gameplay, provenance, licensing, packaging, and recovery gates pass.
- **PROD-06**: Before publication, the owner completes a bounded normal-play campaign after automated and AFK/SETA gates pass.
- **PROD-07**: Public-alpha statements report material events truthfully while allowing faction-conditioned framing, and friendly-faction reports expose plan detail according to reputation.

## Out of Scope

| Capability | Reason |
| --- | --- |
| Game-state mutation in milestone 0.1 | Shadow Director must isolate observation, strategy, persistence, cost, and reliability first. |
| Autonomous XEN or KHK minds | Milestone 0.1 only researches their future architecture and treats them as observed pressures. |
| Private institutional knowledge or false beliefs | Primitive 0.1 institutions share the authoritative faction-visible snapshot; divergent epistemics remain later design work. |
| Mutable institutional influence, refusal, sabotage, or power struggle | Alpha institutions are bounded executors under final Executive arbitration, not an internal political simulation. |
| Rich diplomacy or treaty systems | First-public-alpha diplomacy is deliberately limited to declaring and ending bilateral wars. |
| Player missions and Player Influence | The autonomous faction core must be proven before player-directed workflows. |
| Custom in-game interface | Existing Mail or Logbook surfaces are sufficient for the observation prototype. |
| Public runtime API integration in milestone 0.1 | Real-model prototype evidence uses developer-controlled subscription tooling until the public-alpha path begins. |
| Full vanilla, DLC, or mod-added faction rollout | The first prototype deliberately exercises only full ZYA and ARG minds. |
| Faction Enhancer compatibility | It is an explicit first-public-alpha incompatibility, not a milestone 0.1 target. |
| XRSGE compatibility | No product commitment exists; a later evidence-based spike is required. |
| Direct player save-file access | Save files are prohibited research and mutation targets; persistence must use an X4-owned contract. |
| Public-ready or playable claims | Every 0.x release is an internal prototype; 1.0.0 is the first public alpha. |

## Traceability

Every milestone 0.1 requirement is assigned to exactly one roadmap phase.

| Requirement | Phase | Status |
| --- | --- | --- |
| OBS-01 | Phase 1 | Complete |
| OBS-02 | Phase 1 | Complete |
| OBS-03 | Phase 1 | Complete |
| OBS-04 | Phase 3 | Complete |
| OBS-05 | Phase 3 | Complete |
| OBS-06 | Phase 1 | Complete |
| OBS-07 | Phase 1 | Complete |
| OBS-08 | Phase 1 | Complete |
| MIND-01 | Phase 4 | Complete |
| MIND-02 | Phase 3 | Complete |
| MIND-03 | Phase 3 | Complete |
| MIND-04 | Phase 3 | Complete |
| MIND-05 | Phase 4 | Complete |
| MIND-06 | Phase 5 | Complete |
| MIND-07 | Phase 5 | Pending |
| INST-01 | Phase 3 | Complete |
| INST-02 | Phase 3 | Complete |
| INST-03 | Phase 4 | Complete |
| INST-04 | Phase 5 | Pending |
| INST-05 | Phase 5 | Pending |
| INST-06 | Phase 5 | Pending |
| INST-07 | Phase 5 | Pending |
| INST-08 | Phase 4 | Complete |
| MODEL-01 | Phase 5 | Pending |
| MODEL-02 | Phase 5 | Complete |
| MODEL-03 | Phase 5 | Complete |
| MODEL-04 | Phase 5 | Pending |
| MODEL-05 | Phase 4 | Complete |
| MODEL-06 | Phase 5 | Pending |
| MODEL-07 | Phase 5 | Pending |
| MODEL-08 | Phase 8 | Pending |
| MODEL-09 | Phase 8 | Pending |
| STATE-01 | Phase 4 | Complete |
| STATE-02 | Phase 4 | Complete |
| STATE-03 | Phase 4 | Complete |
| STATE-04 | Phase 4 | Complete |
| STATE-05 | Phase 4 | Complete |
| STATE-06 | Phase 4 | Complete |
| DIAG-01 | Phase 6 | Pending |
| DIAG-02 | Phase 6 | Pending |
| DIAG-03 | Phase 6 | Pending |
| DIAG-04 | Phase 6 | Pending |
| DIAG-05 | Phase 6 | Pending |
| VAL-01 | Phase 7 | Pending |
| VAL-02 | Phase 7 | Pending |
| VAL-03 | Phase 7 | Pending |
| VAL-04 | Phase 8 | Pending |
| VAL-05 | Phase 8 | Pending |
| VAL-06 | Phase 1 | Pending |
| RES-01 | Phase 2 | Complete |
| RES-02 | Phase 2 | Complete |
| RES-03 | Phase 2 | Complete |

**Coverage:** 52/52 milestone requirements mapped.

---

*Requirements defined: 2026-08-28*
*Last updated: 2026-08-28 after milestone 0.1 product brainstorm*
