# Phase 5: Bounded Shadow Deliberation - Research

**Researched:** 2026-08-29
**Domain:** Rust trust-boundary orchestration and developer-controlled Codex subscription harness
**Confidence:** HIGH for local contracts and harness capability; MEDIUM for measured operating bounds

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** All real-model prototype and benchmark runs before `1.0.0` use a developer-controlled subscription harness.
- **D-02:** Public runtime API integration is not part of milestone 0.1 and begins on the public-alpha release path.
- **D-03:** Deterministic fakes are required for contracts, failure tests, and replay, but fake trajectories cannot satisfy strategic-quality acceptance.
- **D-04:** The subscription harness and fake remain behind the same typed provider-neutral domain boundary.
- **D-05:** Provider output remains untrusted until schema, semantic, information, safety, budget, and current-state validation all pass.
- **D-06:** Scheduling combines strategic ticks, relevant events, and cooldowns while remaining bounded and deduplicated per faction.
- **D-07:** Each institution proposes at most one active initiative. The Executive may originate, assign, approve, revise, preempt, reject, or terminate it but cannot bypass admission.
- **D-08:** Agreement does not open dialogue. Material objection, mandate, revision, or preemption may open at most two complete Executive–institution dialogue cycles before a final kernel-valid disposition.
- **D-09:** The Executive may maintain, de-escalate, intensify, or seek limited threat-driven coordination in the typed ZYA–ARG Shadow posture.
- **D-10:** There is no diplomacy institution, cross-faction model negotiation, or X4 relationship mutation in 0.1.
- **D-11:** Provider outage or timeout pauses new strategic decisions, records bounded degraded evidence, preserves accepted state, and reconciles current observations before replanning after recovery.
- **D-12:** Exact versioned cache identity includes faction, snapshot, policy, prompt package, schema, provider, model, and relevant generation settings.

### the agent's Discretion

The planner owns harness process mechanics, concrete subscription models, model-role routing, invocation relevance policy, prompt/schema design, queue limits, retry/backoff, cache storage, and benchmark-derived budgets. These choices may not introduce an API runtime dependency before alpha.

### Deferred Ideas (OUT OF SCOPE)

- OpenAI API and other public runtime adapters begin on the alpha path.
- Local-model public support remains outside milestone 0.1.
- Inter-faction negotiations and executable diplomacy are later work.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| MIND-06 | Typed accepted plans and postures without negotiation | Candidate schema, validator chain, and typed projection after atomic admission |
| MIND-07 | Bounded, deduplicated deliberation requests | Per-faction scheduler state and deterministic trigger coalescing |
| INST-04 | Executive actions cannot bypass admission | Executive produces the same untrusted candidate/disposition envelope as institutions |
| INST-05 | Explicit replacement causal record | Preemption command is constructed only after validation and includes predecessor evidence |
| INST-06 | Agreement has no dialogue | State machine has a zero-cycle direct-admission branch |
| INST-07 | Exceptional dialogue has two-cycle cap | Cycle counter is in kernel-owned request state, not provider prose |
| MODEL-01 | Provider-neutral real-model harness | Typed `DeliberationProvider` port with fake and manual Codex CLI harness adapters |
| MODEL-02 | Full validation chain | Parse bytes first, then pure ordered validators, then one commit |
| MODEL-03 | Safe rejection/timeout | Typed degradation result; no candidate or state write before admission |
| MODEL-04 | Exact cache keys | Canonical tuple includes every D-12 component and mandatory hit revalidation |
| MODEL-06 | Enforceable resource bounds | Configuration supplies all limits; benchmark records decide values |
| MODEL-07 | Offline deterministic fixture evidence | Versioned corpus plus fake provider, separate from manual benchmark evidence |
</phase_requirements>

## Summary

Implement Phase 5 as a Rust-only application/domain layer above the existing `mind-domain` aggregate and `mind-persistence` checkpoint port. Provider interaction is an effect at the edge: it receives a canonical, frozen request and returns bounded raw bytes or a classified failure. The deterministic kernel owns scheduling, exact cache identity, validation, dialogue state, causal evidence, and the sole atomic persistence attempt. [VERIFIED: crates/mind-domain/src/ledger.rs:26-76; crates/mind-persistence/src/port.rs:35-44]

Use a developer-run **Codex CLI subscription harness** for manual real-model benchmarks. Local product evidence shows `codex exec` is non-interactive and supports `--json` and `--output-schema`; it is therefore suitable as an explicitly invoked, newline-delimited benchmark process. [VERIFIED: local `codex exec --help`, 2026-08-29] The harness must never be launched by the X4 bridge, normal CI, or unattended runtime. OpenAI documents that ChatGPT/Codex subscription billing and API billing are separate, so the harness must not accept or imply an API credential or API entitlement. [CITED: https://help.openai.com/en/articles/9039756]

**Primary recommendation:** Create a provider-neutral port and deterministic `DeliberationKernel`; implement the fake first, then a manual `codex exec --json --output-schema` adapter that is enabled only by an explicit benchmark command and records measured capability/usage evidence.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
| --- | --- | --- | --- |
| Trigger coalescing, cooldown, and queue bound | API / Backend | Database / Storage | The deterministic Rust kernel decides whether a frozen faction tuple may request work. |
| Request canonicalization and exact cache identity | API / Backend | Database / Storage | Canonical identity must survive restarts and cannot depend on provider session state. |
| Subscription benchmark invocation | Developer harness | API / Backend | An explicit developer process calls the local Codex client; it is not a mod runtime service. |
| Candidate parsing and all admission validators | API / Backend | Database / Storage | Untrusted bytes remain outside authoritative state until one successful commit. |
| Initiative/preemption and posture projection | API / Backend | Database / Storage | Existing aggregate owns typed lifecycle state; persistence owns durable compare-and-set. |
| Player-facing effects | — | — | Phase 5 admits no X4 command, report intent, or relationship mutation. |

## Project Constraints (from AGENTS.md)

- X4 is authoritative; models propose typed goals, plans, and primitives only.
- Rust owns validation, persistence, recovery, model orchestration, caching, and diagnostics.
- Preserve deterministic replay inputs; reject stale or invalid work without partial mutation.
- Bound time, memory, payloads, retries, calls, and game-side work; do not invent performance targets.
- Never log credentials, private prompts, hidden reasoning, or machine-local paths.
- Use only disposable X4 scenarios for later X4 evidence; Phase 5 makes no observed-in-X4 claim.
- Keep repository artifacts in English; do not commit this research output.

## Standard Stack

### Core

| Library / tool | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| Rust workspace | `rust-version = "1.97.1"` | Deterministic domain/application code | Existing workspace baseline. [VERIFIED: Cargo.toml:78-80] |
| `serde` | `=1.0.229` | Strict typed request/candidate codec | Existing domain dependency; use `deny_unknown_fields` on all external candidate records. [VERIFIED: crates/mind-domain/Cargo.toml:11-12; crates/mind-domain/src/initiative.rs:39-40] |
| `serde_json` | `=1.0.151` | Fixture/candidate JSON decoding | Existing test/persistence dependency. [VERIFIED: crates/mind-domain/Cargo.toml:15-17] |
| Codex CLI | `0.149.0` locally | Explicit subscription-backed manual benchmark harness | Local CLI exposes non-interactive `exec`, JSONL events, and final-response JSON Schema. [VERIFIED: local `codex --version`; local `codex exec --help`, 2026-08-29] |

### Supporting

| Library / tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| Existing `mind-domain` | local | Aggregate, initiative lifecycle, causal events | Extend with pure deliberation types and pending commit only. |
| Existing `mind-persistence` | local | Checkpoint compare-and-set and recovery | Persist an admitted aggregate exactly once after validation. |
| `cargo test` | local Cargo 1.97.1 | Deterministic tests and replay corpus | CI and local contract evidence. [VERIFIED: local `cargo --version`, 2026-08-29] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Manual Codex CLI harness | Public Responses/API adapter | Explicitly deferred by D-02; subscriptions do not establish API billing or entitlement. [CITED: https://help.openai.com/en/articles/9039756] |
| Typed hand-written Rust port | Prompt/evaluation framework | No framework is authorized; adding one widens the trust and dependency surface. [VERIFIED: 05-AI-SPEC.md: evaluation tooling section] |
| Exact cache lookup | Semantic similarity reuse | Forbidden by D-12 because near matches can bypass visibility/current-state validation. |

**Installation:** None. Phase 5 should not add a package until a concrete capability is absent and its legitimacy review is completed.

## Architecture Patterns

### System Architecture Diagram

```text
frozen snapshot + faction-visible packet + persisted mind
  -> deterministic trigger classifier
  -> per-faction bounded scheduler
  -> canonical request / exact cache key
       -> revalidated cache hit -> validator chain -> pending commit
       -> cache miss -> DeliberationProvider
                         -> fake (CI/replay)
                         -> manual Codex CLI harness (benchmark only)
                      -> bounded raw candidate or failure
  -> schema -> semantic -> information -> safety -> budget -> current-state
  -> Executive/institution dialogue state machine (0 or <= 2 full cycles)
  -> pending atomic admission -> checkpoint compare-and-set
  -> typed accepted plan, initiative transition, posture, bounded evidence
```

### Recommended Project Structure

```text
crates/
├── mind-domain/src/
│   ├── deliberation.rs        # Pure request/candidate/validator/state-machine types
│   ├── scheduler.rs           # Deterministic per-faction request eligibility
│   ├── cache_identity.rs      # Canonical exact identity and revalidation input
│   └── admission.rs           # Pending commit creation; no I/O
├── mind-persistence/src/
│   └── deliberation_checkpoint.rs # Durable accepted/degraded records and CAS bridge
├── shadow-harness/            # Manual-only process adapter; never linked by X4 bridge
└── crates/mind-domain/tests/
    └── shadow_deliberation_evals.rs # CI corpus contract target
```

### Pattern 1: Effect port outside the trust boundary

**What:** Define a narrow provider trait whose input is a canonical request and whose output is raw bytes or a typed transport failure. The trait cannot receive a persistence port, initiative aggregate mutation capability, X4 adapter, or report outbox.

**When to use:** For both the deterministic fake and manual subscription harness.

```rust
pub trait DeliberationProvider {
    fn deliberate(&mut self, request: &CanonicalDeliberationRequest)
        -> Result<BoundedProviderBytes, ProviderFailure>;
}
```

The identifiers above are proposed implementation names, not existing public API. [ASSUMED]

### Pattern 2: Canonical tuple, exact cache, mandatory revalidation

**What:** Construct a canonical serialization/digest from D-12 components plus the allowed primitive vocabulary and relevant compaction result. A hit may supply candidate bytes only; it must pass schema, information, safety, budget, and current-state validation against the current frozen tuple before admission.

**Why:** Existing initiative application already deduplicates identical command identity/content and rejects a collision before state transition. [VERIFIED: crates/mind-domain/src/ledger.rs:26-48]

### Pattern 3: Pending commit then one compare-and-set

**What:** Validators return an immutable `AdmissionDecision`. Only `Accepted` produces a pending aggregate/checkpoint candidate; persistence calls `CheckpointPort::compare_and_set` once. [VERIFIED: crates/mind-persistence/src/port.rs:35-44]

**Do not:** write a proposal, cache entry marked admitted, initiative state, report intent, or X4 effect during parsing or individual validation.

### Pattern 4: Kernel-owned dialogue state machine

**What:** Direct agreement reaches final admission with zero cycles. Material objection, mandate, revision, or preemption moves to a kernel-owned state carrying `cycles_completed`; after each complete Executive–institution pair, the kernel either reaches a final disposition or stops at the configured cap of two.

**Why:** Provider conversation/session identifiers are non-authoritative harness metadata; persistent typed causal events remain authoritative. [CITED: 05-MEMORY-RECALL.md]

### Pattern 5: Degrade, reconcile, then replan

**What:** On timeout/outage, record only a bounded failure classification correlated to the frozen request; retain last accepted state and mark the faction scheduler paused. Recovery first requires a newer/reconciled strategic packet, then a new canonical request.

**Why:** Recovery must not replay stale provider work or duplicate an accepted initiative. Existing checkpoint recovery is fail-closed on invalid candidates. [VERIFIED: crates/mind-persistence/src/recovery.rs:120-178]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Subscription access | An undocumented HTTP client that imitates ChatGPT/Codex | Explicit local `codex exec` benchmark adapter | Subscription use is a product-client capability, not API authorization. [CITED: https://help.openai.com/en/articles/9039756] |
| Candidate enforcement | Prompt-only “follow JSON” convention | `--output-schema` in the manual harness plus Rust parse/validation | Harness formatting does not replace deterministic admission. [VERIFIED: local `codex exec --help`, 2026-08-29] |
| Atomicity | Ad hoc staged writes | Existing checkpoint compare-and-set seam | It preserves a single durable acceptance boundary. [VERIFIED: crates/mind-persistence/src/port.rs:35-44] |
| Initiative lifecycle | Free-form conversation history | Existing typed initiative commands and causal ledger | The aggregate already encodes active slot, preemption, terminal transitions, and idempotency. [VERIFIED: crates/mind-domain/src/initiative.rs:95-183; crates/mind-domain/src/ledger.rs:26-190] |

## Common Pitfalls

### Subscription client mistaken for API access

**What goes wrong:** A benchmark harness silently depends on API keys, Responses endpoint behavior, or API rate/cost assumptions.

**How to avoid:** Invoke only the authenticated local Codex client from a manual benchmark command, capture its observed model/generation identity when exposed, and make missing local authentication an explicit harness-unavailable result. Do not add API configuration, API keys, or an API crate. ChatGPT/Codex subscriptions and API billing are separate. [CITED: https://help.openai.com/en/articles/9039756]

### Cache hit bypasses current-state validation

**What goes wrong:** An exact prior response is admitted after a snapshot/mind transition.

**How to avoid:** Cache identity includes the frozen snapshot and every D-12 version component; re-run validators on all hits and reject stale candidate facts.

### Dialogue becomes provider-owned and unbounded

**What goes wrong:** The provider asks for another turn indefinitely or direct agreement opens a discussion.

**How to avoid:** The kernel determines eligibility and cycle count before each call; the third full cycle is impossible by construction. Persist no conversational prose as authority.

### Failure records become partial admission

**What goes wrong:** A timeout advances a tick, allocates an initiative ID, or overwrites an active initiative.

**How to avoid:** Make degradation a separate durable record type; it cannot contain an admitted-plan/initiative transition and always retains the accepted aggregate.

### Benchmark claims become release thresholds

**What goes wrong:** One model run yields invented latency, quality, token, retry, or cost numbers.

**How to avoid:** Record measurements by corpus/model/harness/configuration fingerprint. Phase 8, not Phase 5, derives thresholds. [VERIFIED: 05-AI-SPEC.md: Evaluation Strategy and Production Monitoring]

## Code Examples

### Ordered admission skeleton

```rust
fn admit(input: &AdmissionInput, bytes: &BoundedProviderBytes) -> AdmissionDecision {
    let candidate = parse_strict(bytes)?;
    validate_schema(&candidate)?;
    validate_semantics(input, &candidate)?;
    validate_information(input, &candidate)?;
    validate_safety(input, &candidate)?;
    validate_budget(input, &candidate)?;
    validate_current_state(input, &candidate)?;
    build_pending_commit(input, candidate)
}
```

This is pseudocode: validator names and return types are proposed. [ASSUMED] The ordering is locked by D-05 and the AI-SPEC guardrail contract.

### Manual harness invocation contract

```text
canonical request JSON -> manual wrapper -> codex exec --json --output-schema <schema>
  -> capture final structured response and event metadata
  -> impose wrapper timeout/byte limits
  -> return bytes/failure to the provider port
```

The actual wrapper must supply a dedicated, non-secret benchmark working directory and must pass a prompt that forbids tool use and instructs only the candidate schema. It is a developer action, never a production service. The availability of `--json` and `--output-schema` is local evidence, not a guarantee for other Codex versions. [VERIFIED: local `codex exec --help`, 2026-08-29]

## State of the Art

| Old Approach | Current Approach | Impact |
| --- | --- | --- |
| Provider output treated as a plan | Provider output is untrusted bytes until deterministic admission | Protects authority, replay, and no-mutation guarantees. [VERIFIED: AGENTS.md; 05-CONTEXT.md D-05] |
| Generic cache reuse | Exact versioned identity plus hit revalidation | Prevents semantic/visibility/state drift. [VERIFIED: 05-CONTEXT.md D-12] |
| “Conversation” as durable state | Typed aggregate, causal ledger, and bounded state machine | Enables replay and explicit preemption evidence. [VERIFIED: crates/mind-domain/src/mind.rs:52-125; crates/mind-domain/src/ledger.rs:26-190] |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | A separate `shadow-harness` package is the best workspace placement. | Recommended Project Structure | Planner may choose a tools directory or binary crate instead. |
| A2 | The proposed provider trait and validator identifiers are suitable names. | Architecture Patterns / Code Examples | Naming may conflict with existing conventions. |
| A3 | The manual wrapper can disable/avoid all Codex tool behavior through its prompt and sandbox configuration. | Manual harness invocation contract | Must be verified by an actual controlled benchmark before relying on it. |

## Resolved Execution Decisions

1. **Subscription availability and model selection — resolved by a manual preflight, not an assumed model.**
   - Plan 05 must expose a developer-only preflight that invokes the locally authenticated `codex exec --json --output-schema` path against one schema-valid corpus request and records the CLI version plus any exposed provider/model/generation metadata.
   - If authentication, model selection, structured output, or required metadata is unavailable, the harness returns a typed `Unavailable` result and writes a redacted availability record. It neither retries through an API nor substitutes a different provider. A successful preflight selects only that recorded model/configuration for the matching manual corpus run; it creates no runtime support or quality claim.
2. **Scheduler bounds — resolved as required versioned configuration with a disabled-until-measured real-model path.**
   - Plans 02–04 must define checked `RequestBounds` fields for queue depth, request/context/output bytes, provider calls, retries, timeout, retained history, and dialogue cycles. Missing, zero, malformed, or over-cap values are a deterministic fail-closed disposition before provider invocation.
   - CI fixtures declare bounded values solely to prove enforcement. The manual harness accepts a versioned measured profile tied to corpus/model/configuration fingerprints; until a profile exists, it remains unavailable for real-model runs. No benchmark result or numeric performance claim is invented by this phase.
3. **Non-authoritative traces — resolved as a deletable redacted sidecar.**
   - Plan 05 must write manual benchmark traces only beneath an explicit harness `--evidence-dir`, outside the X4-owned checkpoint and excluded from normal runtime/CI. Each record is keyed by exact request/cache identity and contains only the redacted monitoring tuple, availability/failure class, and corpus/configuration fingerprint.
   - The sidecar is safe to delete without changing admitted state. Phase 6 may consume a derived diagnostic projection later; Phase 5 does not add a second authoritative persistence writer or put raw provider material in the checkpoint.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| Rust toolchain | Domain, tests, harness wrapper | ✓ | Cargo `1.97.1`; rustc `1.97.1` | — |
| Node.js / npm | Markdown lint / local developer tooling | ✓ | Node `24.16.0`; npm `11.13.0` | — |
| Codex CLI | Manual subscription benchmark harness | ✓ | `codex-cli 0.149.0` | Deterministic fake for CI/replay only |
| Authenticated/usable Codex subscription | Real-model benchmark | Unknown | Account-dependent | Benchmark remains unavailable; do not substitute an API |
| markdownlint-cli2 | Required formatting command | Not currently resolvable in sandbox | `npx` registry-cache `EPERM` during probe | Run the required command under approved host access |

**Missing dependencies with no fallback:** None for deterministic Phase 5 implementation; an unavailable subscription blocks only real-model benchmark evidence, not CI contracts.

**Missing dependencies with fallback:** The real-model harness has no equivalent evidence fallback; deterministic fake tests remain a different, explicitly insufficient evidence class.

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Rust built-in test harness |
| Config file | `Cargo.toml` workspace |
| Quick run command | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` |
| Full suite command | `cargo test --workspace --locked` |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| MIND-06 / MODEL-02 | Valid candidate passes all ordered validators then commits once | unit/integration | focused corpus command | ❌ Wave 0 |
| MIND-07 / MODEL-06 | Trigger coalescing, cooldown, queue, retry, and timeout bounds | unit | focused corpus command | ❌ Wave 0 |
| INST-04 / INST-05 | Executive admission and causal preemption | unit | focused corpus command | ❌ Wave 0 |
| INST-06 / INST-07 | Zero-cycle agreement and two-cycle maximum | state-machine/property | focused corpus command | ❌ Wave 0 |
| MODEL-01 / MODEL-07 | Fake/provider port replay and manual harness exclusion from CI | integration | focused corpus command | ❌ Wave 0 |
| MODEL-03 / MODEL-04 | Failure no-side-effect and exact cache hit/miss/revalidation | integration | focused corpus command | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** focused corpus plus targeted crate tests.
- **Per wave merge:** `cargo test --workspace --locked`, `cargo fmt --check`, and repository strict Clippy command.
- **Phase gate:** all deterministic corpus cases pass; manual subscription evidence is recorded separately and cannot claim strategic-quality acceptance.

### Wave 0 Gaps

- [ ] `crates/mind-domain/tests/shadow_deliberation_evals.rs` — implement AI-SPEC cases SD-001 through SD-013.
- [ ] `shadow-deliberation-evals/v1/manifest.*` — pin fixtures, hashes, schema/policy/prompt versions, and expected deterministic outcome.
- [ ] Controlled fake provider fixtures for malformed, timeout, cache, replay, dialogue, and recovery paths.
- [ ] Manual-only harness runner that writes redacted benchmark evidence and is excluded from CI.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | Yes | Harness delegates interactive account authentication to the local Codex client; Rust code never reads subscription/API secrets. |
| V3 Session Management | Yes | Provider session identifiers are non-authoritative and scoped to manual benchmark metadata. |
| V4 Access Control | Yes | Faction-visible packet is frozen before context construction; Executive cannot bypass admission. |
| V5 Input Validation | Yes | Strict decode, size bound, schema, semantic, information, safety, budget, and current-state validators. |
| V6 Cryptography | No new cryptography | Reuse existing approved platform/storage mechanisms; do not hand-roll cryptography. |

### Known Threat Patterns for the Stack

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Prompt/candidate injection | Tampering | Treat provider content as bytes; strict typed admission with finite Shadow vocabulary. |
| Cross-faction fact leak | Information disclosure | Construct and hash faction-filtered packet before any provider call; test negative fixtures. |
| Credential or prompt leakage in evidence | Information disclosure | Log hashes/classification only; never raw prompts, provider payloads, hidden reasoning, or paths. |
| Retry/cache replay duplication | Repudiation / tampering | Exact identity, idempotent command IDs, revalidation, and one checkpoint CAS. |
| Resource exhaustion | Denial of service | Kernel-enforced bounded queue, bytes, context, calls, retries, and dialogue cycles. |

## Sources

### Primary (HIGH confidence)

- [Local Codex CLI help](local command: `codex exec --help`) - non-interactive execution, JSONL, JSON Schema, model and sandbox options; verified 2026-08-29.
- [OpenAI billing separation](https://help.openai.com/en/articles/9039756) - ChatGPT subscription and API platform billing are separate.
- [05-CONTEXT.md](05-CONTEXT.md) - locked D-01 through D-12.
- [05-AI-SPEC.md](05-AI-SPEC.md) - deterministic corpus, guardrails, benchmark-only evidence, and manual-harness constraints.
- [initiative.rs](../../../crates/mind-domain/src/initiative.rs) and [ledger.rs](../../../crates/mind-domain/src/ledger.rs) - existing lifecycle, idempotency, and pending-commit patterns.
- [port.rs](../../../crates/mind-persistence/src/port.rs) and [recovery.rs](../../../crates/mind-persistence/src/recovery.rs) - compare-and-set and fail-closed recovery seam.

### Secondary (MEDIUM confidence)

- [05-MEMORY-RECALL.md](05-MEMORY-RECALL.md) - durable decision/pattern context; checked against phase constraints.

### Tertiary (LOW confidence)

- None. Proposed names and package placement are listed separately in the Assumptions Log.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - current workspace manifests and local CLI were inspected.
- Architecture: HIGH - locked phase decisions align with existing aggregate/checkpoint seams.
- Harness behavior: MEDIUM - CLI capability is local evidence; account/model availability must be benchmarked.
- Pitfalls: HIGH - backed by phase AI specification and project constraints.

**Research date:** 2026-08-29
**Valid until:** 2026-09-05 for subscription-harness details; local code evidence remains valid until the next revision.
