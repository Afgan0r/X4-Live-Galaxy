# Project Research Summary

**Project:** Live Galaxy
**Domain:** Observation-only LLM strategic director for X4: Foundations
**Milestone:** 0.1 — Shadow Director
**Researched:** 2026-08-28
**Confidence:** MEDIUM-HIGH

## Executive Summary

Live Galaxy is an internal 0.1 prototype for persistent, faction-specific strategic minds observing an X4 galaxy. X4 remains authoritative: the Lua/Mission Director adapter only extracts bounded observations, Rust owns normalization, deterministic policy, persistence, recovery, model orchestration, validation, evaluation, and diagnostics, and providers produce untrusted proposals. The acceptance question is whether full ZYA and ARG minds can maintain coherent short- and long-term plans under shared XEN pressure, recognize KHK when observed, and remain explainable and recoverable during unattended AFK/SETA runs.

The recommended implementation is a one-way replayable pipeline: bounded X4 envelopes over a versioned transport, typed Rust ingest and quality-aware snapshots, a pure deterministic faction kernel, provider-neutral model adapters with Ollama evaluated first, complete proposal admission, transactional state plus an idempotent report outbox, and external correlated evidence. Use SQLite for compact authoritative runtime state, JSONL/tracing for non-authoritative diagnostics and evaluation, exact content-addressed cache keys, recorded fixtures, and deterministic fakes. Do not add mutation, player control, custom UI, save-file access, or autonomous XEN/KHK minds to 0.1.

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
- Typed strategic decisions, explanations, and safe concise Mail/Logbook reports; never expose hidden reasoning or raw private prompts.
- Persistence, compact save-authoritative state boundary, restart recovery, deterministic caching, context compaction, token/time/call budgets, and idempotency.
- External correlated diagnostics, snapshots, decision traces, quality/cost/failure evidence, and automated strategic-quality evaluation.
- Reliability degradation and AFK/SETA soak behavior without stalling or corrupting X4.

**Should have (competitive/research-instrument differentiators):**

- Shared pressure with measurably distinct ZYA/ARG doctrines, rather than prompt copies.
- Information-bounded faction viewpoints and replayable decision packets.
- Provider/model benchmark matrix covering latency, cost, cache behavior, failures, and strategic quality.
- Operator-readable health summaries and a separate parallel XEN/KHK research track.

**Defer (later milestones):**

- Fleet, economy, institution, or diplomacy mutation; player missions and Player Influence; autonomous XEN/KHK minds.
- Full vanilla/DLC or mod-added faction rollout and compatibility guarantees; test KUDA AI Tweaks, More AI Economy Ships, and Add More Sectors only in later evidence gates, while Faction Enhancer remains explicitly incompatible for the first public alpha.
- Custom dossier/chronicle/institution UI, save-file readers/writers, and unbounded chat/prose.

### Architecture Approach

Use a one-way pipeline from X4 observation producers through bounded transport, Rust ingest/normalization, immutable sectioned snapshots/checkpoints, deterministic faction views and kernel, untrusted provider proposal, schema/semantic/information/safety/budget/freshness admission, typed shadow plan, and an idempotent report outbox. Reports are projections only; detailed traces and replay/evaluation artifacts stay external, while compact authoritative runtime state follows the X4-owned persistence boundary and external cache/diagnostics remain non-authoritative.

**Major components:**

1. X4 Lua/MD observation adapter and transport/session layer — read-only extraction, stable IDs, freshness/quality, cooperative scheduling, framing, correlation, backpressure, and health.
2. Rust ingest, normalizer, snapshot/checkpoint store — reject malformed/oversized input, canonicalize typed values, retain section quality/unknowns, advance monotonic state versions, and recover atomically.
3. Deterministic strategic kernel — construct faction-specific information views, bounded derived facts, priorities, budgets, stable ordering, and replayable decision inputs under shared XEN pressure.
4. Provider ports, proposal validator, and admission boundary — isolate provider behavior, enforce versioned schemas and all policy checks, and reject without side effects.
5. Persistence/outbox/diagnostics/evaluation — atomically persist accepted plan and report intent, deduplicate retries, emit concise Mail/Logbook output, and correlate observable evidence.

### Critical Pitfalls

1. **Stale or incomplete observation treated as truth** — carry timestamp, coverage, freshness, quality, source, and explicit unknown/unsupported states through every layer; discover map/assets/capacity at runtime.
2. **Unstable identities, ordering, and recovery** — use typed stable IDs, event/state versions, correlation IDs, canonical ordering, transactional admission, and duplicate/out-of-order/crash tests.
3. **Faction or diagnostic information leakage** — filter visibility before context construction, tag provenance, separate player output from debug diagnostics, and redact secrets, raw prompts, hidden reasoning, and machine paths.
4. **Provider/schema drift and nondeterminism** — version schemas/prompt packages/policies, isolate adapters, fingerprint replay inputs, inject clock/ID sources, and use recorded typed fixtures rather than live models in normal tests.
5. **SETA overload and false integration confidence** — hard-bound game-thread work, queues, payloads, retries, model calls, and retention; require static, pure Lua, fake-adapter, and disposable in-game black-box evidence with independent readback.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Contracts, Observation, and Normalization

**Rationale:** Every later feature depends on trustworthy typed state; quality and identity errors poison strategy and replay.
**Delivers:** Versioned envelopes, stable identity/event contracts, normalized snapshots, section freshness/quality/unknown semantics, bounded Lua/MD scheduling and transport health, and fixture tests.
**Addresses:** Authoritative observation, XEN/KHK recognition, no-mutation boundary, and the first part of AFK safety.
**Avoids:** Fabricated truth, unstable identity/order, oversized payloads, and full-callback SETA stalls.
**Research flag:** Yes — verify exact X4 9.00 runtime API/event semantics, embedded Lua limits, named-pipe dependency/version, and disposable observation behavior.

### Phase 2: Persistence, Faction Kernel, and Replayable State

**Rationale:** Persistence and deterministic faction views must exist before model integration so continuity and information boundaries are testable independently.
**Delivers:** SQLite transaction/checkpoint model, compact save-authoritative state boundary, recovery markers, idempotent ingest, deterministic ZYA/ARG minds, faction visibility filtering, XEN pressure inputs, bounded context compaction, replay tuples, and offline evaluation fixtures.
**Addresses:** Persistent minds, short/long-term plans, distinct doctrines, replayability, and restart recovery.
**Uses:** Rust typed domain, `serde`, `thiserror`, SQLite, pure kernel tests, and deterministic fakes.
**Avoids:** Context loss, stale caches, duplicate recovery, hidden-information leakage, and nondeterministic decisions.
**Research flag:** Usually no for persistence/replay patterns; yes only if X4-owned save integration semantics are admitted to this phase.

### Phase 3: Provider Boundary, Validation, Budgets, and Shadow Decisions

**Rationale:** With frozen state and replay inputs established, provider behavior can be measured without confusing transport or persistence defects with model quality.
**Delivers:** Provider trait, Ollama-first benchmark harness, exact cache keys, typed candidate parsing, schema/semantic/safety/information/budget/current-state admission, safe explanations, rejection diagnostics, and accepted typed shadow plans.
**Addresses:** Typed decisions, explanations, model neutrality, cost/reliability budgets, and strategic-quality evaluation.
**Implements:** Provider adapters, deterministic kernel-to-provider packet boundary, and all-or-nothing validator.
**Avoids:** Model-to-game shortcuts, contract drift, vendor lock-in, hidden reasoning leakage, and live-model nondeterminism in normal tests.
**Research flag:** Yes — benchmark Ollama server/model options and current provider behavior at phase time; do not make Ollama a permanent requirement before evidence.

### Phase 4: Reports, Diagnostics, and X4 AFK/SETA Evidence

**Rationale:** Player-visible output and endurance are integration/release evidence after the core is deterministic and recoverable.
**Delivers:** Idempotent Mail/Logbook report outbox, external JSONL/tracing diagnostics, health summaries, reconnect/recovery behavior, disposable in-game observation and report probes, normal-speed and SETA soak evidence, and a parallel non-blocking XEN/KHK research report.
**Addresses:** Concise in-game sanity output, structured diagnostics, unattended AFK/SETA acceptance, and observed-in-X4 status.
**Avoids:** Duplicate reports, simulation stalls, source-text/fake-test overclaiming, and accidental mutation scope.
**Research flag:** Yes — exact in-game event/report surfaces, SETA scheduling limits, installed-mod behavior, and compatibility must be verified with captured version/mod/scenario/readback evidence.

### Phase 5: Evaluation and 0.1 Release Gate

**Rationale:** Release confidence requires held-out strategic and failure evidence, not merely successful runs.
**Delivers:** Scenario corpus and evaluation matrix for factuality, continuity, information discipline, divergence/consistency, cost, reliability, determinism, redaction, and recovery; measured mutation baselines for pure high-risk logic; explicit verified/pending/observed status; package/provenance audit.
**Addresses:** Automated strategic-quality evaluation and internal prototype release readiness.
**Avoids:** Weak fixtures, premature mutation thresholds, unsupported compatibility claims, and calling 0.1 playable or public-ready.
**Research flag:** Targeted research only for unresolved test harness/tool compatibility; most evaluation design follows established repository contracts.

### Phase Ordering Rationale

- Contracts and quality-aware observation precede strategy because every decision must be grounded in authoritative, replayable state.
- Persistence and information-bounded deterministic views precede provider calls so failures and model quality remain attributable.
- Admission and outbox boundaries precede in-game evidence so reports cannot bypass validation or duplicate on recovery.
- AFK/SETA and disposable in-game probes come after local/fake coverage; they prove X4 behavior, not merely source structure.
- XEN/KHK research runs in parallel and must not become a hidden dependency for the ZYA/ARG Shadow Director slice.

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 1:** Exact X4 9.00 observation APIs, embedded Lua/MD behavior, transport dependency, event identity, and scheduling caps.
- **Phase 3:** Ollama/provider benchmark and current schema/usage behavior; keep provider choice evidence-based.
- **Phase 4:** Mail/Logbook integration, SETA soak, reconnect semantics, installed-mod ownership/load order, and compatibility probes.
- **Phase 5:** Only targeted tool validation and corpus adequacy review; mutation gates require a measured baseline.

Phases with standard patterns (skip research-phase unless scope changes):

- **Phase 2:** Typed Rust domain modeling, transactional SQLite recovery, deterministic replay, and fixture/fake testing are well covered by project conventions, subject to exact implementation decisions.
- **Most of Phase 3's pure validation:** schema barriers, adapter isolation, bounded retries, and rejection-without-side-effects are established patterns.

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
- No project licence is selected; provenance/licensing review remains a release gate before 1.0.0.

## Sources

### Primary (HIGH confidence)

- [`PROJECT.md`](../PROJECT.md) — current milestone 0.1 requirements, boundaries, authority, persistence, privacy, compatibility, and release maturity.
- [`AGENTS.md`](../../AGENTS.md) — repository authority, evidence handling, X4 research routing, and engineering invariants.
- [`STACK.md`](STACK.md), [`FEATURES.md`](FEATURES.md), [`ARCHITECTURE.md`](ARCHITECTURE.md), [`PITFALLS.md`](PITFALLS.md) — synthesized research inputs.
- Live Galaxy project skills under `.agents/skills/` — Rust conventions/tests and X4 integration/testing contracts.
- [Rust Book](https://doc.rust-lang.org/book/), [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html), [Serde](https://serde.rs/), [Tokio](https://tokio.rs/), [SQLite transactions](https://www.sqlite.org/lang_transaction.html), [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/), [tracing](https://docs.rs/tracing/latest/tracing/), and [Ollama API](https://github.com/ollama/ollama/blob/main/docs/api.md) — official/primary technology guidance.

### Secondary (MEDIUM confidence)

- `F:\Agent Projects\X4\tools\x4-live-protocol.md` — observed local transport, bounded scheduling, freshness, and retry precedent; exact revision freshness unresolved.
- `F:\Agent Projects\X4\tests\test_x4_live.py` — observed local test patterns for atomic batches, idempotent replay, reconciliation, unknowns, and bounded coverage.
- `F:\Agent Projects\X4\extensions\x4_live_mcp\content.xml` — observed dependency and extension precedent; not a Live Galaxy runtime dependency.
- [Egosoft X4 Foundations](https://www.egosoft.com/games/x4/info_en.php) and [official X4 Wiki](https://wiki.egosoft.com:1337/x4wiki/) — product/modding context, insufficient alone for runtime guarantees.

### Tertiary (LOW confidence)

- None used as roadmap authority. Any future community or analogue evidence must remain explicitly secondary and cannot establish X4 compatibility.

---
*Research completed: 2026-08-28*
*Ready for roadmap: yes*
