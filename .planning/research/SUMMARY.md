# Project Research Summary

**Project:** Live Galaxy
**Domain:** Observation-only LLM strategic director for X4: Foundations
**Milestone:** 0.1 — Shadow Director
**Researched:** 2026-08-28
**Confidence:** MEDIUM-HIGH

## Executive Summary

Live Galaxy is an internal 0.1 prototype for persistent, faction-specific strategic minds and primitive institutions observing an X4 galaxy. X4 remains authoritative: the Lua/Mission Director adapter only extracts bounded observations, Rust owns normalization, deterministic policy, initiative lifecycle, persistence, recovery, model orchestration, validation, evaluation, and diagnostics, and providers produce untrusted proposals. The acceptance question is whether full ZYA and ARG minds can maintain coherent short- and long-term plans under shared XEN pressure, coordinate fixed-priority institutions through bounded Executive arbitration, recognize KHK when observed, and remain explainable and recoverable during unattended AFK/SETA runs.

The recommended implementation is a one-way replayable pipeline: bounded X4 envelopes over a versioned transport, typed Rust ingest and quality-aware snapshots, a pure deterministic faction kernel, primitive institution views and one-active-initiative state machines, bounded Executive arbitration, provider-neutral model adapters with Ollama evaluated first, complete proposal admission, transactional state plus an idempotent report outbox, and external correlated evidence. Use SQLite for compact authoritative runtime state, JSONL/tracing for non-authoritative diagnostics and evaluation, exact content-addressed cache keys, recorded fixtures, and deterministic fakes. Do not add mutation, player control, custom UI, save-file access, internal political simulation, or autonomous XEN/KHK minds to 0.1.

The main risks are fabricated truth from stale or partial observations, identity/order and recovery errors, hidden-information or privacy leakage, contract drift, and unbounded SETA workload. Prevent them with explicit unknown/stale/quality states, typed stable identities and monotonic versions, frozen replay inputs, schema and information filtering before model context construction, atomic admission/outbox state machines, hard game/queue/payload/retry/model budgets, and layered tests that distinguish local verification, pending game smoke, and observed-in-X4 evidence.

## Key Findings

### Recommended Stack

The stack recommendation is strong on boundaries but intentionally leaves versions to implementation-phase benchmarks; no crate, model, CI image, or Lua runner version should be invented now.

**Core technologies:**

- Rust stable with a Cargo workspace: typed domain, bridge, persistence, provider, diagnostics, and orchestration boundaries; keep pure strategic logic synchronous and isolate I/O.
- `serde`/`serde_json` and `thiserror`: versioned typed envelopes/configuration/fixtures and recoverable errors without panics at external boundaries.
- `tokio`: bounded async transport/provider work only; runtime features and budgets require measurement.
- Mission Director XML plus embedded X4 Lua: thin cooperative observation producers with bounded serialization; confirm runtime syntax and semantics in a disposable probe.
- Versioned JSON envelopes over the installed named-pipe support seam: adapt the local X4 Live MCP precedent, without making that campaign repository a runtime dependency or claiming compatibility before tests.
- SQLite via `rusqlite`: transactional compact runtime projection, event/checkpoint data, idempotency, and restart recovery; external JSONL plus `tracing` carries bounded diagnostics/evaluation.
- Provider-owned trait plus Ollama-first evaluation: preserve provider neutrality, offline fixtures, usage/timeout/retry metadata, and later benchmark alternatives without leaking vendor types into the domain.
- `cargo test`, focused contract tests, Busted for pure Lua after runtime confirmation, XML/package checks, and measured `cargo-mutants`/Lua mutation spikes only after representative pure logic exists.

### Expected Features

**Must have (table stakes):**

- Read-only authoritative normalized observation for ZYA, ARG, XEN pressure, and observed KHK, with stable identities, timestamps, freshness, quality, and explicit unknowns.
- Independent full ZYA and ARG Faction Minds with doctrine, motives, goals, short/long-term plans, and bounded history.
- Primitive fixed-priority institutions for both factions, each owning at most one typed Shadow initiative under final Executive arbitration.
- Typed strategic decisions, explanations, and safe concise Mail/Logbook reports; never expose hidden reasoning or raw private prompts.
- Persistence, compact save-authoritative state boundary, restart recovery, deterministic caching, context compaction, token/time/call budgets, and idempotency.
- External correlated diagnostics, snapshots, decision traces, quality/cost/failure evidence, and automated strategic-quality evaluation.
- Reliability degradation and AFK/SETA soak behavior without stalling or corrupting X4.

**Should have (competitive/research-instrument differentiators):**

- Shared pressure with measurably distinct ZYA/ARG doctrines, rather than prompt copies.
- Auditable proposal, objection, preemption, disposition, and initiative-outcome records without open-ended council simulation.
- Information-bounded faction viewpoints and replayable decision packets.
- Provider/model benchmark matrix covering latency, cost, cache behavior, failures, and strategic quality.
- Operator-readable health summaries and a separate parallel XEN/KHK research track.

**Defer (later milestones):**

- Fleet, economy, institution, or diplomacy mutation; player missions and Player Influence; autonomous XEN/KHK minds.
- Full vanilla/DLC or mod-added faction rollout and compatibility guarantees; test KUDA AI Tweaks, More AI Economy Ships, and Add More Sectors only in later evidence gates, while Faction Enhancer remains explicitly incompatible for the first public alpha.
- Custom dossier/chronicle/institution UI, save-file readers/writers, and unbounded chat/prose.

### Architecture Approach

Use a one-way pipeline from X4 observation producers through bounded transport, Rust ingest/normalization, immutable sectioned snapshots/checkpoints, deterministic faction views and kernel, primitive institution proposals, bounded Executive disposition, schema/semantic/information/safety/budget/freshness admission, typed shadow plan and initiative lifecycle, and an idempotent report outbox. Reports are projections only; detailed traces and replay/evaluation artifacts stay external, while compact authoritative runtime state follows the X4-owned persistence boundary and external cache/diagnostics remain non-authoritative.

**Major components:**

1. X4 Lua/MD observation adapter and transport/session layer — read-only extraction, stable IDs, freshness/quality, cooperative scheduling, framing, correlation, backpressure, and health.
2. Rust ingest, normalizer, snapshot/checkpoint store — reject malformed/oversized input, canonicalize typed values, retain section quality/unknowns, advance monotonic state versions, and recover atomically.
3. Deterministic strategic kernel — construct faction-specific information views, bounded derived facts, priorities, budgets, stable ordering, and replayable decision inputs under shared XEN pressure.
4. Primitive institutions and Executive arbitration — apply fixed priorities, enforce one active initiative per institution, preserve explicit preemption, and cap exceptional dialogue at two cycles.
5. Provider ports, proposal validator, and admission boundary — isolate provider behavior, enforce versioned schemas and all policy checks, and reject without side effects.
6. Persistence/outbox/diagnostics/evaluation — atomically persist accepted plan, initiative lifecycle, and report intent, deduplicate retries, emit concise Mail/Logbook output, and correlate observable evidence.

### Critical Pitfalls

1. **Stale or incomplete observation treated as truth** — carry timestamp, coverage, freshness, quality, source, and explicit unknown/unsupported states through every layer; discover map/assets/capacity at runtime.
2. **Unstable identities, ordering, and recovery** — use typed stable IDs, event/state versions, correlation IDs, canonical ordering, transactional admission, and duplicate/out-of-order/crash tests.
3. **Faction or diagnostic information leakage** — filter visibility before context construction, tag provenance, separate player output from debug diagnostics, and redact secrets, raw prompts, hidden reasoning, and machine paths.
4. **Provider/schema drift and nondeterminism** — version schemas/prompt packages/policies, isolate adapters, fingerprint replay inputs, inject clock/ID sources, and use recorded typed fixtures rather than live models in normal tests.
5. **SETA overload and false integration confidence** — hard-bound game-thread work, queues, payloads, retries, model calls, and retention; require static, pure Lua, fake-adapter, and disposable in-game black-box evidence with independent readback.
6. **Unbounded institution dialogue or silent preemption** — direct agreement is the fast path; material disagreement gets at most two cycles, and every replacement preserves owner, reason, prior state, and outcome.
7. **Idea-catalogue scope creep** — the 103 Bannerlord candidates remain references unless a milestone decision explicitly promotes one.

## Implications for Roadmap

The confirmed scope fits the existing eight-phase roadmap; no extra phase is
needed. Primitive institutions deepen Phases 3–5 rather than creating a new
political-simulation workstream.

### Phase 1: Read-Only Observation Spine

Deliver versioned observations, stable identities, explicit data quality,
bounded Lua/MD scheduling, and proof that no mutation path exists. Institution
work must not pull private knowledge or council design into this phase.

### Phase 2: Hostile-Faction Research Track

Retain independent XEN/KHK evidence without blocking the ZYA/ARG critical path.
The Bannerlord idea catalogue is not part of this research track.

### Phase 3: Faction-Scoped Strategic State

Build deterministic faction-visible snapshots, fixed institution priorities,
and typed proposal inputs. In 0.1 all institutions in one faction consume the
same authoritative faction-visible snapshot; private institutional knowledge
and false beliefs remain later-scope candidates.

### Phase 4: Persistent Full Faction Minds

Persist full ZYA/ARG mind state plus institution identity, priority version,
one active initiative, owner, lifecycle, preemption history, and outcome. This
phase establishes recovery and causal continuity before live providers.

### Phase 5: Bounded Shadow Deliberation

Add provider neutrality, exact caching, complete admission, Executive
origination and disposition, and exceptional two-cycle institution dialogue.
Aligned proposals bypass dialogue; no proposal or initiative mutates X4.

### Phase 6: Correlated Reports and Diagnostics

Correlate institution proposal, objection, Executive disposition, validation,
initiative lifecycle, and report output while keeping player-visible Mail and
Logbook concise and safe.

### Phase 7: X4 Operational Proof

Demonstrate bounded normal-speed, SETA, reconnect, recovery, and unattended
behavior with primitive institutions active in Shadow mode.

### Phase 8: Evaluation and Internal Prototype Gate

Measure strategic coherence, faction divergence, institution contribution,
initiative causality, reliability, cost, and recovery. Package 0.1 only as an
internal prototype and keep all unpromoted Bannerlord candidates outside its
acceptance gate.

### Phase Ordering Rationale

- Contracts and quality-aware observation precede strategy because every decision must be grounded in authoritative, replayable state.
- Institution priority and initiative contracts follow faction-scoped state and precede provider calls so disagreement, ownership, and failures remain attributable.
- Persistence precedes Executive dialogue so preemption and restart cannot erase causal history.
- Admission and outbox boundaries precede in-game evidence so reports cannot bypass validation or duplicate on recovery.
- AFK/SETA and disposable in-game probes come after local/fake coverage; they prove X4 behavior, not merely source structure.
- XEN/KHK research runs in parallel and must not become a hidden dependency for the ZYA/ARG Shadow Director slice.

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 1:** Exact X4 9.00 observation APIs, embedded Lua/MD behavior, transport dependency, event identity, and scheduling caps.
- **Phase 3:** Exact faction-visible fact semantics; private institutional knowledge is explicitly not a 0.1 research requirement.
- **Phase 4:** X4-owned persistence contract and initiative recovery semantics.
- **Phase 5:** Ollama/provider benchmark and current schema/usage behavior; keep provider choice evidence-based.
- **Phase 6:** Mail/Logbook integration and safe initiative/report correlation.
- **Phase 7:** SETA soak, reconnect semantics, installed-mod ownership/load order, and compatibility probes.
- **Phase 8:** Only targeted tool validation and corpus adequacy review; mutation gates require a measured baseline.

Phases with standard patterns (skip research-phase unless scope changes):

- **Most of Phases 3–5:** Typed Rust domain modeling, initiative state machines,
  deterministic replay, schema barriers, bounded retries, and
  rejection-without-side-effects are established patterns.

## Confidence Assessment

| Area | Confidence | Notes |
| --- | --- | --- |
| Stack | MEDIUM-HIGH | Rust/SQLite/typed-provider recommendations are well supported; exact toolchain, crate, model, CI, and Lua versions remain phase decisions. |
| Features | HIGH | Milestone boundary, must-haves, differentiators, and anti-features derive directly from `PROJECT.md`; runtime details are less certain. |
| Architecture | MEDIUM-HIGH | Strongly aligned with project invariants and local X4 precedent; exact X4 runtime semantics and save integration remain unproven. |
| Pitfalls | MEDIUM-HIGH | Failure modes are coherent and directly tied to documented invariants; incidence and thresholds require soak evidence. |

**Overall confidence:** MEDIUM-HIGH for roadmap shape; MEDIUM for exact X4 integration behavior.

### Gaps to Address

- Exact X4 9.00 event, identity, Lua, Mission Director, Mail/Logbook, and named-pipe semantics need clean-revision or disposable in-game confirmation.
- The local X4 Live MCP precedent was dirty and remote refresh was blocked; treat its protocol and test details as observed local precedent, not compatibility guarantees.
- X4-owned compact save persistence integration is a boundary decision; no player save files may be read or modified during research.
- Ollama server/model choice, latency/context quality, hosted-provider comparison, and token/cost budgets need recorded benchmark evidence.
- Headless X4 automation capability is unknown; plan black-box evidence as disposable scenarios and do not assume a headless runner.
- Initial AFK/SETA budget thresholds, evaluation corpus size, mutation thresholds, and compatibility matrix must be measured rather than invented.
- Phase planning must define the concrete primitive institution roster and
  initiative schema without adding private knowledge, mutable influence, or
  political resistance to 0.1.
- No project licence is selected; provenance/licensing review remains a release gate before 1.0.0.

## Sources

### Primary (HIGH confidence)

- [`PROJECT.md`](../PROJECT.md) — current milestone 0.1 requirements, boundaries, authority, persistence, privacy, compatibility, and release maturity.
- [`AGENTS.md`](../../AGENTS.md) — repository authority, evidence handling, X4 research routing, and engineering invariants.
- [`STACK.md`](STACK.md), [`FEATURES.md`](FEATURES.md), [`ARCHITECTURE.md`](ARCHITECTURE.md), [`PITFALLS.md`](PITFALLS.md) — synthesized research inputs.
- Live Galaxy project skills under `.agents/skills/` — Rust conventions/tests and X4 integration/testing contracts.
- `drawer_wing_x4_live_galaxy_decisions_a376ced07a211aa8271352e6` and related 2026-08-28 decisions — confirmed 0.1 primitive-institution lifecycle.
- `drawer_wing_dialogue_sessions_dd1780a21bd9ded3e9c4e997` — confirmed public-alpha boundary and one-verified-milestone-at-a-time principle.
- [Rust Book](https://doc.rust-lang.org/book/), [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html), [Serde](https://serde.rs/), [Tokio](https://tokio.rs/), [SQLite transactions](https://www.sqlite.org/lang_transaction.html), [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/), [tracing](https://docs.rs/tracing/latest/tracing/), and [Ollama API](https://github.com/ollama/ollama/blob/main/docs/api.md) — official/primary technology guidance.

### Secondary (MEDIUM confidence)

- `F:\Agent Projects\X4\tools\x4-live-protocol.md` — observed local transport, bounded scheduling, freshness, and retry precedent; exact revision freshness unresolved.
- `F:\Agent Projects\X4\tests\test_x4_live.py` — observed local test patterns for atomic batches, idempotent replay, reconciliation, unknowns, and bounded coverage.
- `F:\Agent Projects\X4\extensions\x4_live_mcp\content.xml` — observed dependency and extension precedent; not a Live Galaxy runtime dependency.
- `drawer_wing_bannerlord_operations_5616c5cee3cb7fb1d4281fb2`, `drawer_wing_bannerlord_operations_b56d90c7d2d6048b12545a7f`, and `drawer_wing_bannerlord_operations_0ecf0d0d620fcdb86d5ca3b7` — verified static reference architecture from AI Influence, ChatAi, and AliceMM.
- `drawer_wing_bannerlord_operations_7675741e0bb9147f4d2ed3f1` — exhaustive 103-item candidate catalogue; reference only, not roadmap scope.
- [Egosoft X4 Foundations](https://www.egosoft.com/games/x4/info_en.php) and [official X4 Wiki](https://wiki.egosoft.com:1337/x4wiki/) — product/modding context, insufficient alone for runtime guarantees.

### Tertiary (LOW confidence)

- Six Bannerlord-derived Live Galaxy hypotheses in `wing_x4_live_galaxy/observations` cover publication, commitments, follow-ups, intelligence artifacts, prompt packs, and deliberation records. They may inform later phase design but do not establish requirements or X4 compatibility.

---
*Research completed: 2026-08-28*
*Ready for roadmap: yes*
