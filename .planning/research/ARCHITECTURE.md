# Architecture Patterns

**Domain:** Observation-only LLM strategic director for X4: Foundations
**Researched:** 2026-08-28
**Milestone:** 0.1 Shadow Director
**Overall confidence:** MEDIUM-HIGH

## Recommended Architecture

Use an asymmetric bidirectional, replayable integration with X4 as the
authority and Rust as the trust boundary. The broad path is X4-to-Rust
telemetry; the only 0.1 return path is an allowlisted report-and-acknowledgement
channel:

```text
X4 Lua/MD producers
  -> bounded observation envelopes (transport/session)
  -> Rust ingest + normalization
  -> immutable snapshot/checkpoint store
  -> deterministic faction kernel
  -> primitive institution views
  -> provider adapter(s) (untrusted institution/Executive candidates)
  -> bounded Executive arbitration
  -> schema/semantic/information/budget/current-state validation
  -> typed shadow plan + institution initiative lifecycle + explanation
  -> idempotent report outbox
       -> allowlisted return channel -> low-cost X4 Mail/Logbook
       <- delivery acknowledgement
  -> diagnostics/evaluation/replay artifacts
```

The milestone has no game-mutating command path. Keep a future command
application interface behind the validator, but do not implement fleet,
economy, diplomacy, missions, custom UI, or full-faction rollout now.

### Component Boundaries

| Component | Responsibility | Communicates With |
| --- | --- | --- |
| X4 observation adapter (Lua/MD) | Read-only, player-visible/authorized facts; stable IDs; timestamps; freshness and quality; cooperative scheduling and bounded serialization | Pipe transport |
| Transport/session layer | Asymmetric versioned frames, capability negotiation, correlation, retry/backpressure, reconnect, degraded-mode health, and explicit compatibility failure; no domain interpretation | X4 adapter, Rust ingest, report outbox |
| Ingest/normalizer | Decode and validate envelopes; canonicalize IDs/quantities/time; reject malformed or oversized input; assign monotonic state versions | Transport, snapshot store |
| Snapshot/checkpoint store | Atomic section replacement, immutable event log, current projection, retention, recovery markers | Ingest, kernel, replay, diagnostics |
| Deterministic strategic kernel | Derive bounded facts, faction information views, goals, priorities, budgets, and allowed strategic primitives from typed snapshots | Snapshot store, provider orchestration, evaluator |
| Primitive institution layer | Apply fixed faction-conditioned priorities; own at most one active Shadow initiative per institution; emit typed proposals, objections, and preemption requests | Kernel, Executive arbitration, persistence |
| Executive arbitration | Originate, approve, revise, preempt, reject, and assign initiatives; invoke at most two dialogue cycles only on material disagreement | Institution layer, provider orchestration, validator |
| Model-provider port/adapters | Prompt/package construction and provider-specific calls; bounded timeout/retry/token budget; return raw response as untrusted data | Kernel, validator |
| Proposal validator/admission | Schema, semantic, safety, information, budget, freshness and faction-policy checks; all-or-nothing admission | Kernel, persistence, report outbox |
| Plan/explanation projection | Store typed accepted shadow plan and safe rationale summary; never expose hidden reasoning or raw private prompts | Validator, diagnostics, report outbox |
| Report outbox | Idempotent concise Mail/Logbook delivery; deduplicate by report identity; degrade if X4 unavailable | X4 adapter, persistence |
| Diagnostics/evaluation | Correlated traces, quality gaps, latency/cost, replay fixtures, provider scores, failure classification | Every Rust component; external tooling |

### Data Flow

Each observation envelope carries `release_id`, `session_id`, `event_id`,
`captured_at`, game time, source/visibility scope, entity identity, schema
version, quality/coverage, payload size, and correlation ID. The adapter must
publish facts and explicit `unknown`/`unsupported` values; absence is not
`known-empty`.

Rust accepts a frame only after transport, envelope, size, and schema checks.
Ingest atomically validates all events in a frame, writes the immutable event
record, updates touched snapshot sections, and advances a monotonic
`state_version`. A complete bounded asset cycle may create a checkpoint;
partial cycles retain section freshness and never imply deletion.

The kernel reads one frozen snapshot/checkpoint and builds separate ZYA and ARG
information views under shared observed XEN pressure. Primitive institutions
inside each faction consume the same authoritative faction-visible snapshot and
apply fixed faction-conditioned priorities. Each may own at most one active
Shadow initiative and may propose a new initiative or an explicit preemption
request carrying the current initiative state and replacement rationale.

The Executive Brain composes and allocates strategy rather than executing it.
It may originate, approve, revise, preempt, reject, or assign an initiative.
Aligned proposals proceed directly to validation. A material objection, forced
mandate, preemption, or revision may invoke at most two full dialogue cycles;
the final Executive disposition must still pass deterministic admission. The
provider output remains outside the trust boundary throughout. Rejected
candidates produce diagnostics and no plan, initiative, or report side effect.

Accepted plans are persisted transactionally with the decision input hash,
provider/model identity, schema version, validation result, and report identity.
The report outbox then emits only a short player-safe summary through the
allowlisted return channel and correlates its acknowledgement. Detailed traces,
quality gaps, and evaluation data remain external diagnostics.

The X4 adapter and Rust bridge negotiate protocol and capability identities
before accepting traffic. A protocol-compatible Rust release may restart,
update, and reconnect while the X4 process continues running; accepted state
and report identity survive the interruption. Unsupported combinations enter a
bounded fail-closed degraded mode. X4 restart is required only when game-facing
code changes or the game-side protocol is incompatible. Pipe topology,
negotiation schema, buffering, and acknowledgement framing are phase-level
technical decisions.

### Trust and Ownership Rules

- X4 owns live world state, game time, and any eventual effect application.
- Lua/MD owns extraction and scheduling only; it must not decide strategy or
  infer missing facts.
- Rust owns normalization, deterministic policy, persistence, recovery,
  validation, provider orchestration, caching, and diagnostics.
- Providers may propose goals/plans/explanations only; they cannot issue native
  X4 calls or mutate storage directly.
- Institutions own proposals and Shadow initiative lifecycle, not authoritative
  facts, Executive authority, or X4 effects.
- The Executive owns final initiative disposition but cannot bypass kernel
  legality, compatibility, budgets, or the no-mutation milestone boundary.
- Reports are projections, not authority. A report must identify its snapshot
  and decision, and must not claim an action occurred in 0.1.

## Replay, Idempotency, and Recovery

Persist the complete replay tuple: normalized snapshot/checkpoint ID and hash,
ordered input event IDs, state version, faction policy/version, deterministic
kernel configuration, context-compaction result, provider/model/configuration
identity, prompt-package hash (not necessarily private prompt contents),
budget, candidate bytes/hash, validator version, and accepted plan/report IDs.
Recorded typed provider fixtures and deterministic fakes must reproduce normal
tests without live model calls.

Use stable IDs at every boundary. Ingest deduplicates `event_id`; plan admission
deduplicates `(faction, decision_input_hash, policy_version)`; report delivery
deduplicates `report_id`. Commit plan and outbox intent atomically, then retry
delivery by the same identity. On restart, reconcile pending outbox entries and
resume from the last durable cursor/checkpoint. Never advance a cursor before a
successful durable write, and never partially apply a future game command.

Persist institution identity, fixed-priority version, active initiative ID,
initiative owner, proposal and objection records, Executive disposition,
preemption reason, validator result, and terminal outcome. An institution cannot
silently replace active work; suspension or cancellation is an explicit state
transition that preserves the previous initiative history.

## Patterns to Follow

### Pattern 1: Frozen snapshot plus pure decision kernel

**What:** Read one immutable snapshot, produce typed derived facts and a
deterministic decision input, then perform provider I/O outside pure logic.
**When:** Every faction cycle and every replay/evaluation.

### Pattern 2: Sectioned, quality-aware observations

**What:** Store fast and slow sections independently with per-section freshness,
coverage, and quality. Use explicit `unknown`, `unsupported`, and stale states.
**When:** Asset scans, station/fleet details, and any rotating X4 producer.

### Pattern 3: Atomic admission with an outbox

**What:** Validate the complete candidate before one transactional persistence
boundary; emit player-facing reports asynchronously by idempotent identity.
**When:** Every model cycle, including degraded or provider-failure paths.

### Pattern 4: Bounded initiative state machine

**What:** Represent proposal, active work, objection, revision, preemption,
completion, failure, cancellation, and rejection as typed transitions. Direct
agreement is the fast path; exceptional dialogue is capped at two cycles.
**When:** Every primitive institution and Executive disposition in 0.1.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Model-to-game shortcut

**What:** Letting a model response call Lua/native APIs or writing commands before
validation. **Why bad:** It violates authority and permits stale, unsafe, or
partial effects. **Instead:** Keep 0.1 report-only and retain a future command
port behind deterministic admission.

### Anti-Pattern 2: Full snapshot on one game callback

**What:** Enumerating the galaxy and serializing one large payload synchronously.
**Why bad:** It stalls the simulation, worsens SETA behavior, and creates false
completeness. **Instead:** Cooperative rotating producers with hard per-tick,
payload, queue, and retry caps.

### Anti-Pattern 3: Treating missing data as empty or current

**What:** Converting absent fields, failed detail producers, or stale sections to
zero/empty/idle. **Why bad:** It fabricates strategic evidence. **Instead:** carry
quality and coverage through normalization, kernel, explanation, and diagnostics.

### Anti-Pattern 4: Turning the idea catalogue into runtime scope

**What:** Prebuilding private knowledge, political resistance, rich councils,
news chains, or treaty systems because a Bannerlord reference demonstrates them.
**Why bad:** It recreates the scope-growth and idea-overload problem that the
confirmed milestone policy rejects. **Instead:** Implement only the accepted
primitive institution lifecycle; retain all other mechanisms as phase-triggered
hypotheses.

## Scalability Considerations

| Concern | At 100 users | At 10K users | At 1M users |
| --- | --- | --- | --- |
| X4 work | One bounded producer cycle per process; report-only | Same game-thread caps; tune by measured soak data | Per-process isolation; never scale by removing caps |
| Storage | SQLite/event log and compact checkpoints | Partition/rotate diagnostics; keep runtime projection compact | Separate telemetry backend; preserve local replay contracts |
| Model cost | One bounded cycle per faction; cache exact inputs | Queue and provider quotas; degrade to deterministic reports | Fleet of provider workers with per-faction fairness |
| Context | Compact frozen packet with explicit omitted sections | Cache stable facts and evaluate truncation | Versioned feature projections and sampling |

## Build-Order Implications

1. Define typed envelope, normalized snapshot, quality, identity, and version
   contracts; add fixture/replay tests.
2. Build the thin X4 adapter and bounded transport/health surface, including
   capability negotiation and fail-closed degraded behavior; prove static,
   fake-adapter, and disposable in-game observation behavior.
3. Implement atomic persistence, section freshness, checkpoints, restart
   recovery, and idempotent ingestion before model integration.
4. Implement deterministic ZYA/ARG kernel, information views, primitive
   institution priorities, and typed one-active-initiative state with recorded
   snapshots.
5. Add provider ports, Executive arbitration, exceptional two-cycle dialogue,
   exact cache keys, typed candidate parsing, and complete validation/admission;
   evaluate with recorded fixtures only.
6. Add the bounded report return channel, report outbox, acknowledgements, and
   external diagnostics, then prove compatible Rust restart/reconnect and run
   AFK/SETA soak with health evidence. Keep future mutation seams as interfaces
   only.

## Sources

- **Documented:** Live Galaxy project boundary and requirements, local
  [`PROJECT.md`](../PROJECT.md), updated from confirmed MemPalace decisions on
  2026-08-28.
- **Confirmed decisions:** primitive institutions in 0.1 and small verified
  milestone progression,
  `drawer_wing_x4_live_galaxy_decisions_a376ced07a211aa8271352e6` and
  `drawer_wing_dialogue_sessions_dd1780a21bd9ded3e9c4e997`.
- **Verified reference plus hypotheses:** Bannerlord AI-mod architecture and
  Live Galaxy adaptation candidates,
  `drawer_wing_bannerlord_operations_7675741e0bb9147f4d2ed3f1` and the six
  2026-08-28 `wing_x4_live_galaxy/observations` drawers. These inform seams but
  do not establish product scope or X4 runtime behavior.
- **Observed local precedent:** X4 Live protocol and bounded scheduling,
  `F:/Agent Projects/X4/tools/x4-live-protocol.md`, local revision available at
  research time; remote refresh was attempted but blocked by `.git/FETCH_HEAD`
  permission failure. Relevant evidence includes named-pipe framing, 3 MiB
  envelope cap, cooperative 50 ms/4 ms budgets, section freshness, atomic
  SQLite transactions, same-handle retry, and fail-closed transport.
- **Observed local tests:** `F:/Agent Projects/X4/tests/test_x4_live.py`, local
  revision available at research time; tests cover atomic batches, idempotent
  replay, reconciliation/tombstones, quality-aware unknowns, and bounded
  sector/asset coverage.
- **Documented official modding entry point:** [Egosoft X4 Foundations
  Wiki](https://wiki.egosoft.com:1337/x4wiki/), consulted for the official
  documentation boundary; exact runtime behavior remains an in-game research
  question.
- **Inferred:** The pipeline and build order above apply the repository's
  stated authority, trust, recovery, and budget invariants to the 0.1
  observation-only scope. No claim here proves an unsupported X4 API or
  headless-test capability.
