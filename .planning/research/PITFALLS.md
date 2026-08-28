# Domain Pitfalls

**Domain:** Observation-only LLM strategic director for X4: Foundations
**Project:** Live Galaxy milestone 0.1 (Shadow Director)
**Researched:** 2026-08-28
**Overall confidence:** MEDIUM

This catalog treats X4 as the authority and the model as an untrusted proposal
source. Claims about the project are documented in `AGENTS.md` and
`.planning/PROJECT.md`; claims about the integration boundary are documented in
the Live Galaxy integration and test skills. Runtime behavior of the installed
game remains **unknown** until a disposable Creative Custom campaign produces
attributable evidence.

## Critical Pitfalls

### 1. Stale or incomplete observations become strategic “truth”

**What goes wrong:** Missing, delayed, paginated, or degraded X4 observations
are normalized as empty/current state. A faction mind then plans against a
fictional economy or fleet picture.

**Why it happens:** Polling and event streams have different freshness and
coverage; adapter errors are easy to collapse into defaults.

**Consequences:** Wrong reports, false causal explanations, and decisions that
cannot be replayed or audited.

**Prevention:** Every snapshot carries timestamp, source, schema/version,
freshness, completeness, and quality metadata. Distinguish absent from empty;
reject or explicitly degrade on stale data. Discover sectors, assets, capacity,
and ownership at runtime rather than assuming a fixed map.

**Detection:** Fixtures for missing/oversized/duplicate/out-of-order data;
snapshot coverage counters; stale-data rejection traces; disposable X4 readback
with exact version, mod list, scenario, and elapsed time.

**Phase ownership:** Phase 1 (observation and normalization); Phase 4 (X4
black-box/SETA evidence).

### 2. Unstable identities and event ordering corrupt continuity

**What goes wrong:** Objects, events, decisions, or acknowledgements are keyed
by display names, array positions, or transient transport IDs. Renames,
duplicates, reconnects, and out-of-order delivery fork history or apply an event
twice.

**Why it happens:** X4-facing identity semantics are adapter-specific and must
be verified rather than inferred from labels.

**Consequences:** Duplicate reports, impossible timelines, and non-idempotent
recovery.

**Prevention:** Define typed stable identities, source identity, observation
sequence, and state version. Preserve deterministic ordering and reject
ambiguous identity; use correlation IDs through observation → decision → outcome.

**Detection:** Duplicate/out-of-order contract tests, reconnect tests, identity
collision diagnostics, and replay comparison.

**Phase ownership:** Phase 1 (identity contract); Phase 3 (persistence and
replay).

### 3. Hidden-information leakage violates faction knowledge boundaries

**What goes wrong:** A shared packet, prompt, cache, report, or diagnostic leaks
facts unavailable to a faction, private prompts, hidden reasoning, or secrets.

**Why it happens:** Normalization and prompt assembly often share broad structs;
logging is added before visibility filtering.

**Consequences:** Unfair strategy, invalid evaluation results, and privacy or
security incidents.

**Prevention:** Apply information filtering before model context construction;
tag facts with visibility/provenance; separate player-facing reports from debug
diagnostics; never log credentials, raw private prompts, or hidden reasoning.

**Detection:** Negative authorization tests (faction A cannot see B-only facts),
redaction tests, packet-size/content audits, and diagnostic secret scanners.

**Phase ownership:** Phase 2 (typed strategic packets and model contract).

### 4. Model contract drift is mistaken for strategic failure

**What goes wrong:** Provider response shapes, schema versions, defaults, or
tool/model configuration change silently. Invalid output reaches domain logic,
or evaluation compares unlike contracts.

**Why it happens:** Provider-specific fields leak across the adapter boundary;
prompt and schema revisions lack explicit versioning.

**Consequences:** Brittle runtime failures, unsafe coercion, unreproducible
decisions, and misleading quality scores.

**Prevention:** Version typed request/response schemas; isolate providers behind
adapters; validate schema, semantics, safety, budget, information, and current
state before admission. Store model/provider/config fingerprints with replay
inputs.

**Detection:** Recorded typed fixtures, malformed/oversized response tests,
contract tests per provider, and a compatibility matrix for schema versions.

**Phase ownership:** Phase 2 (model boundary); Phase 5 (evaluation and release
gate).

### 5. Context loss, cache staleness, and accidental nondeterminism erase intent

**What goes wrong:** Compaction drops durable goals or constraints; stale cache
entries are reused after world/schema/model changes; unordered maps, wall-clock
time, random IDs, or concurrency change a decision on replay.

**Why it happens:** Token budgets encourage aggressive compaction, while cache
keys omit relevant inputs and runtime ordering is implicit.

**Consequences:** Faction personalities drift, reports contradict history, and
the same snapshot produces different output without an explainable cause.

**Prevention:** Keep compact authoritative state with X4 save integration and
external caches non-authoritative. Use exact cache keys containing normalized
snapshot, faction, contract, provider/model, and policy versions; bound TTL and
invalidate on schema/config changes. Persist canonical replay inputs and use
stable sorting plus injected clock/ID sources.

**Detection:** Same-input replay tests, compaction round trips, cache hit/miss
probes, stale-key tests, and byte-for-byte decision-input comparisons.

**Phase ownership:** Phase 3 (persistence, cache, deterministic replay).

### 6. Persistence, retry, and recovery produce duplicate or partial work

**What goes wrong:** A crash between acceptance, persistence, dispatch, and
acknowledgement causes a decision/report to be lost or emitted twice.

**Why it happens:** Recovery boundaries and idempotency keys are not modeled as
an explicit state machine.

**Consequences:** Corrupt history and repeated effects if mutation is added
later; observation reports become untrustworthy now.

**Prevention:** Persist accepted work with unique identity and validation
context; make retries idempotent; reject stale work before persistence or
mutation; define interruption boundaries and reconciliation rules.

**Detection:** Crash/restart tests, duplicate delivery and out-of-order tests,
recovery journals, and no-partial-state assertions.

**Phase ownership:** Phase 3.

### 7. Unbounded SETA workload stalls or destabilizes X4

**What goes wrong:** Polling, queue growth, model retries, payload sizes, or
diagnostic writes scale with accelerated game time and eventually starve the
game or bridge.

**Why it happens:** Normal-speed testing hides workload amplification.

**Consequences:** AFK runs stall, memory grows, observations lag, or X4 state
becomes unsafe.

**Prevention:** Bound cadence, queue depth, payloads, retries, model calls,
memory, and game-thread work; coalesce observations; degrade safely when the
bridge is unavailable.

**Detection:** Normal-speed and SETA soak with health/timing bounds, queue and
latency telemetry, and explicit overload behavior.

**Phase ownership:** Phase 4 (runtime scheduling and soak).

### 8. Source-text tests create false confidence

**What goes wrong:** Tests assert that a Lua/XML string exists while the runtime
never executes the path, or a fake adapter is treated as proof that X4 behaves
the same way.

**Why it happens:** In-game automation is costly and X4 headless support is
unknown.

**Consequences:** Broken registration, event semantics, or readback ships as a
“passing” integration.

**Prevention:** Layer static/schema checks, pure Lua tests, fake adapter contract
tests, and minimal disposable in-game black-box probes. Label results as
verified locally, pending game smoke, or observed in X4.

**Detection:** Require executable behavior and independent readback for every
integration claim; record exact scenario and health surface.

**Phase ownership:** Phase 4.

### 9. Weak evaluation corpora and premature mutation architecture mislead

**What goes wrong:** A tiny or unrepresentative fixture corpus rewards generic
reports; mutation scoring is imposed before representative pure logic and a
measured baseline exist.

**Why it happens:** Quality metrics are sought before observation and contract
semantics stabilize.

**Consequences:** Optimizing the wrong behavior, noisy gates, and maintenance
cost without evidence.

**Prevention:** Build evaluation fixtures from ZYA/ARG/XEN observation cases,
including degraded and adversarial inputs; evaluate factuality, boundary
compliance, continuity, cost, and determinism separately. Apply mutation only
to pure high-risk logic after baseline measurement; review every survivor.

**Detection:** Held-out fixtures, disagreement review, coverage by scenario and
failure mode, and mutation reports with survivor disposition.

**Phase ownership:** Phase 5 (evaluation); mutation spike only after the
relevant pure kernel exists.

## Moderate Pitfalls

### Diagnostic privacy leaks

Machine-local paths, provider credentials, prompt contents, hidden reasoning,
or raw payloads can escape through structured diagnostics. Keep public reports
minimal and debug diagnostics separate, redact at the logging boundary, and
test representative error paths. **Phase ownership:** Phase 2 and Phase 5.

### Compatibility assumptions become guarantees

Installed mods and X4 APIs change behavior through patches, hooks, and load
order. Map ownership and versions first; call compatibility “verified” only
after a disposable test. **Phase ownership:** Phase 4.

### Observation-only scope quietly becomes mutation

Adding command plumbing before Shadow Director evidence expands the trust and
recovery surface. Keep milestone 0.1 read-only and reject unsupported commands
explicitly. **Phase ownership:** Phase 0 scope gate and Phase 4 release gate.

## Phase-Specific Warnings

| Phase topic | Likely pitfall | Mitigation |
| --- | --- | --- |
| Observation/normalization | Stale, partial, unstable X4 data | Quality metadata, typed IDs, replay fixtures |
| Model boundary | Contract drift or hidden-information leakage | Versioned schemas, pre-context filtering, redaction |
| Persistence/cache | Context loss, stale cache, duplicate recovery | Canonical inputs, exact keys, idempotent state machine |
| SETA runtime | Unbounded queue/model/game-thread load | Cadence, payload, retry, and soak budgets |
| X4 verification | Source-text/fake-test false confidence | Layered tests plus disposable black-box readback |
| Evaluation/release | Weak corpus or premature mutation gate | Held-out fixtures and measured mutation baseline |

## Sources

- Live Galaxy project constraints and milestone scope: `AGENTS.md` and
  `.planning/PROJECT.md` (repository, current checkout; documented).
- Rust trust-boundary, determinism, idempotency, privacy, and bounds rules:
  `.agents/skills/live-galaxy-rust-conventions/SKILL.md` (repository; documented).
- Required Rust test coverage and mutation policy:
  `.agents/skills/live-galaxy-rust-tests/SKILL.md` (repository; documented).
- X4 adapter safety, identity/freshness, compatibility, diagnostics, and
  verification requirements:
  `.agents/skills/live-galaxy-x4-integration/SKILL.md` (repository; documented).
- Layered X4 testing, SETA soak, black-box evidence, and mutation limits:
  `.agents/skills/live-galaxy-x4-tests/SKILL.md` (repository; documented).
- Egosoft X4 official portal (version-sensitive game/mod documentation entry
  point; consult the installed 9.00 build before relying on API semantics):
  https://www.egosoft.com/games/x4/info_en.php (external; authoritative for
  product-level facts, not sufficient alone for runtime API behavior).

