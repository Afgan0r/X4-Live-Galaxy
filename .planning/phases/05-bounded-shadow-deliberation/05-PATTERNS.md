# Phase 5: Bounded Shadow Deliberation - Pattern Map

**Mapped:** 2026-08-29  
**Files analyzed:** 9 planned production/test/artifact areas  
**Analogs found:** 7 / 9 (role or data-flow matches; no exact Phase 5 implementation exists)

## File Classification

The following names come from `05-RESEARCH.md` and the Wave 0 requirements. Names and
symbols marked planned are proposals, not existing public APIs.

| Planned file | Role | Data flow | Closest analog | Match quality |
| --- | --- | --- | --- | --- |
| `crates/mind-domain/src/deliberation.rs` | model boundary, dialogue state machine | request-response / transform | `crates/mind-domain/src/mind.rs`, `initiative.rs` | role-match |
| `crates/mind-domain/src/scheduler.rs` | scheduler | event-driven / batch | `extensions/live_galaxy/lua/live_galaxy_scheduler.lua` | data-flow match |
| `crates/mind-domain/src/cache_identity.rs` | utility / identity | transform | `crates/strategic-state/src/fingerprint.rs`, `crates/mind-persistence/src/capsule_identity.rs` | strong match |
| `crates/mind-domain/src/admission.rs` | validator / application boundary | request-response / transform | `crates/mind-domain/src/ledger.rs`, `crates/strategic-state/src/fingerprint.rs` | role-match |
| `crates/mind-persistence/src/deliberation_checkpoint.rs` | persistence integration | CRUD / atomic write | `crates/mind-persistence/src/port.rs`, `checkpoint.rs` | strong match |
| `shadow-harness/` (manual Codex CLI adapter) | provider adapter / harness | file-I/O / request-response | `crates/mind-persistence/src/fake_port.rs` | partial; no provider analog |
| `shadow-deliberation-evals/v1/manifest.*` | versioned corpus/config artifact | batch / file-I/O | `crates/mind-persistence/src/checkpoint.rs` | partial |
| `crates/mind-domain/tests/shadow_deliberation_evals.rs` | contract/integration/property tests | request-response / replay | `crates/mind-domain/tests/initiative_lifecycle.rs`, `mind_checkpoint.rs` | strong match |
| fake-provider fixtures | deterministic test adapter/data | request-response / replay | `crates/mind-persistence/tests/fake_port_contract.rs` | role/data-flow match |

`shadow-harness` and the corpus are intentionally not runtime dependencies. The manual
runner must remain outside the X4 bridge and CI; fake trajectories are contract evidence,
not strategic-quality evidence.

## Existing Symbols (verified)

- `MindAggregate::apply_initiative` in `crates/mind-domain/src/mind.rs:117-125` delegates
  to `ledger::apply` and returns a pending, immutable commit.
- `InitiativeCommand::{Accept,Preempt,Terminal}` in `initiative.rs:95-145` and
  `ledger::apply` in `ledger.rs:26-48` provide typed lifecycle, idempotent same-command
  replay, and content-collision rejection.
- `ledger::preempt` in `ledger.rs:104-153` enforces one active capability slot,
  predecessor identity, bounded history/events, and causal replacement evidence.
- `StrategicPacket::canonical_facts` and `admission_inputs` in
  `crates/strategic-state/src/packet.rs:99-122` and `fingerprint.rs:18-32` sort inputs
  before identity construction; `replay_fingerprint` in `fingerprint.rs:35-54` binds
  policy/profile/faction/snapshot/facts/primitives.
- `Capsule::new` and `eligible` in `crates/mind-persistence/src/capsule.rs:57-83`
  validate bounds before construction; `capsule_identity::build` in
  `capsule_identity.rs:3-29` frames every identity component with length prefixes.
- `CheckpointPort::compare_and_set` in `crates/mind-persistence/src/port.rs:35-44` is
  the sole atomic persistence seam. `FakeCheckpointPort::preflight` and
  `validate_successor` in `fake_port.rs:47-80` implement exact retry, collision, stale
  predecessor, and sequence checks.
- `CheckpointEnvelope::encode/decode/validate` in `checkpoint.rs:74-145` enforce typed
  Serde decoding, size bounds, schema/protocol identity, integrity hash, and payload
  validation before persistence.
- `recover_candidate` and `retained_valid` in `recovery.rs:91-124` reject malformed,
  stale, out-of-order, or colliding candidates while retaining acknowledged state.
- `admit_batch`/`derive_packets` usage in `mind_checkpoint.rs:12-35` is the fixture setup
  pattern for frozen observations and faction-visible packets.

## Pattern Assignments

### `crates/mind-domain/src/deliberation.rs` (planned; model boundary/dialogue)

**Copy from:** `mind.rs:1-35, 52-125`; `initiative.rs:95-183`.

Use Serde typed structs with `#[serde(deny_unknown_fields)]`, domain enums, bounded
constructors, and `Result` errors. Keep the deterministic request, proposal, dialogue
cycle counter, institution role, and posture in pure domain types. The provider port is
only a planned trait (the index confirms no existing Rust provider/deliberation symbol):

```rust
// Planned shape; names are not existing APIs.
pub trait DeliberationProvider {
    fn deliberate(&mut self, request: &CanonicalDeliberationRequest)
        -> Result<BoundedProviderBytes, ProviderFailure>;
}
```

Copy the aggregate pattern: `MindAggregate::empty` (`mind.rs:68-87`) gives explicit typed
defaults; `MindAggregate::apply_initiative` (`mind.rs:117-125`) returns a pending result;
`InitiativeCommand` (`initiative.rs:95-145`) models lifecycle operations as enums. Direct
agreement must be a zero-cycle terminal branch; objection/mandate/revision/preemption must
carry a kernel-owned cycle count capped at two. Provider session/conversation text is not
authoritative state.

### `crates/mind-domain/src/scheduler.rs` (planned; deterministic scheduler)

**Copy from:** `live_galaxy_scheduler.lua:3-47` for bounded disposition flow and
`strategic-state/src/packet.rs:99-122` for canonical ordering. This is the only scheduler
analog and is Lua, not a Rust API.

The Lua scheduler clamps work (`MAX_SLICES_PER_TICK = 1`), returns explicit dispositions
for unavailable/backpressure paths, and never waits. Port that shape to pure Rust: merge
strategic ticks, relevant events, and cooldown eligibility; deduplicate one outstanding
request per faction; enforce queue/call/retry/timeout bounds; return typed decisions rather
than invoking I/O. Do not copy Lua strings as domain state.

### `crates/mind-domain/src/cache_identity.rs` (planned; exact cache identity)

**Copy from:** `fingerprint.rs:18-54` and `capsule_identity.rs:3-29`.

`admission_inputs` clones and `sort_unstable_by_key`s primitives before hashing. The capsule
identity frames every field with its byte length. Build a canonical, versioned identity with
the locked tuple: faction, snapshot hash, policy version, prompt-package hash, schema
version, provider ID, model ID, and generation settings (plus relevant compaction/primitive
vocabulary). Avoid wall-clock/session IDs. Cache hits provide candidate bytes only and must
re-enter the full validation pipeline against the current frozen request.

### `crates/mind-domain/src/admission.rs` (planned; ordered validation)

**Copy from:** `ledger::apply` (`ledger.rs:26-48`) and `Capsule::new` validation
(`capsule.rs:57-83`).

Implement a pure, ordered chain: strict byte-size/decode, schema, semantic, information /
visibility, safety, budget, then current-state validation. Return an immutable planned
`AdmissionDecision`; only its accepted variant creates a pending aggregate/checkpoint. Any
failure returns a classified diagnostic with no partial initiative, cache-admitted marker,
cursor, report intent, or X4 effect. Use `InitiativeError`-style typed errors and preserve
same-command idempotency/content collision behavior from `ledger.rs:34-48`.

### `crates/mind-persistence/src/deliberation_checkpoint.rs` (planned; atomic integration)

**Copy from:** `CheckpointEnvelope::from_pending_commit` (`checkpoint.rs:26-72`) and
`CheckpointPort` (`port.rs:35-44`).

Represent accepted and bounded degraded records as typed versioned payload fields. Validate
identities and payload before encoding, calculate integrity after the payload is complete,
then call `compare_and_set` exactly once. Preserve the existing checkpoint predecessor,
sequence, integrity, and replay identity semantics. Degraded records must retain accepted
state and a bounded failure classification/correlation ID, never a speculative initiative
transition. Reuse `CheckpointError`/`PortError`-style fail-closed outcomes; do not add a
second persistence writer.

### `shadow-harness/` (planned; manual-only Codex CLI adapter)

**Copy from:** `FakeCheckpointPort::compare_and_set` (`fake_port.rs:19-44`) for a small
adapter implementing one boundary and explicit failure mapping. There is no existing Rust
provider or process-harness analog (index exploration found none).

The adapter should take canonical request bytes, invoke the developer-controlled local
`codex exec --json --output-schema` process only through an explicit manual command, impose
timeout/byte limits, capture redacted metadata, and return bounded bytes or typed failure.
It must not receive persistence/X4 ports, API credentials, hidden prompts, or tools; it must
not be linked by normal runtime/CI. Missing authentication/model availability is an explicit
unavailable result, not a fallback to an API.

### `shadow-deliberation-evals/v1/manifest.*` (planned; versioned corpus)

**Copy from:** `CheckpointEnvelope`’s version and integrity binding (`checkpoint.rs:6-24,
145-177`) and `capsule_identity::build` framing (`capsule_identity.rs:3-29`).

Pin fixture IDs/hashes, schema/policy/prompt versions, expected deterministic outcomes,
evidence class, and configuration fingerprint. Keep corpus entries bounded and deterministic;
never store raw prompts, secrets, hidden reasoning, or machine-local paths. Manual benchmark
results are separate evidence and cannot alter CI expectations or create acceptance
thresholds.

### `crates/mind-domain/tests/shadow_deliberation_evals.rs` (planned; contract/property)

**Copy from:** `initiative_lifecycle.rs:16-64` for duplicate/preemption/idempotency cases;
`mind_checkpoint.rs:112-199` for round-trip, malformed, unknown-field, and corruption
fixtures; `checkpoint_tracer.rs:52-142` for deterministic encode/hash assertions.

Cover SD-001–SD-013 and validation map cases: valid ordered admission/one CAS; hidden,
stale, malformed, oversized, unsafe, and over-budget candidates; trigger coalescing and
cooldowns; zero-cycle agreement and two-cycle maximum; executive preemption with causal
predecessor; exact cache component changes and hit revalidation; timeout/outage no-side
effects; recovery after fresh observation; fake replay; and a static/contract assertion that
the manual harness is excluded from CI. Tests may use immediate `assert!` failure helpers,
but production modules may not use `unwrap`, `expect`, or panic paths.

### Fake-provider fixtures (planned; deterministic adapter)

**Copy from:** `fake_port_contract.rs:29-64` for deterministic fixture construction and
exact retry/collision assertions, plus `mind_checkpoint.rs:137-198` for mutation of one
serialized field per negative case.

The fake must implement the same planned provider-neutral port as the manual adapter,
replay recorded bytes without network/time dependence, and expose malformed, timeout,
cache, dialogue, recovery, and valid candidates. It cannot be used to claim strategic
quality. Keep fixture ordering and hashes stable.

## Shared Patterns

### Trust boundary and typed state

Use `deny_unknown_fields` (`initiative.rs:4-6`, `mind.rs:52-66`, `checkpoint.rs:11-24`),
enums and domain identifiers. Provider output is bytes until schema parse and all ordered
validators pass. No async, I/O, locks, provider response types, or X4 command types belong
inside deterministic state.

### Determinism and bounds

Canonical-sort lists as in `StrategicPacket::canonical_facts` (`packet.rs:99-104`) and
`admission_inputs` (`fingerprint.rs:22-31`); frame identity fields as in
`capsule_identity.rs:23-29`; enforce explicit size/capacity checks as in
`ledger.rs:82-89` and `checkpoint.rs:78-85`. Every source file must remain <=200 physical
lines, with one cohesive responsibility.

### Idempotency, recovery, and atomicity

Reuse `ledger::apply` same-command replay (`ledger.rs:34-48`), `FakeCheckpointPort`
exact-retry preflight (`fake_port.rs:47-80`), and recovery’s retain-on-rejection behavior
(`recovery.rs:91-124`). Accepted state changes only through one pending commit and one
checkpoint CAS. Provider retry reuses the same canonical request and cache identity.

### Diagnostics and secrecy

Return bounded typed failure classifications and correlation IDs. Follow
`RecoveryDiagnostic::Rejected { code }` (`recovery.rs:11-15`) rather than logging payloads.
Never persist raw prompts, provider bytes, hidden reasoning, credentials, or local paths.

## No Analog Found

| Planned area | Why no exact analog | Planner implication |
| --- | --- | --- |
| Provider port/adapters and Codex CLI process | No Rust provider/process symbols in the index; only capsule budget tests mention provider/model | Define a new narrow effect seam; keep fake/manual adapters behind it and verify CLI manually only |
| Ordered schema/information/safety/current-state admission | Existing `ledger` validates domain commands, but no model candidate pipeline exists | Create pure validators and preserve the locked order; do not infer API names from pseudocode |
| Executive/institution dialogue state machine | No dialogue symbols found | Introduce explicit finite state/cycle enum; direct agreement is zero-cycle, exceptional path max two |
| Versioned shadow corpus and redacted benchmark evidence | Existing checkpoint/capsule artifacts are persistence precedents, not evaluation corpus | Pin hashes/versions and evidence class; keep manual evidence outside CI |

## Metadata

**Analog search scope:** `crates/mind-domain`, `crates/mind-persistence`,
`crates/strategic-state`, existing `extensions/live_galaxy` scheduler, and tests; structural
queries used `ast-index rebuild`, `outline`, `explore`, `refs`, `usages`, and `callers`.

**Instruction/skill files read:** `AGENTS.md`; `05-CONTEXT.md`; `05-AI-SPEC.md`;
`05-RESEARCH.md`; `05-VALIDATION.md`; `C:/Users/pavlo/.agents/skills/game-repo-standard/SKILL.md`;
`discord-har.md`; `initialization-discovery.md`; `memory-lifecycle.md`; `migration.md`;
`mod-research.md`; `profiles-and-registry.md`; `research-and-spoilers.md`;
`reusable-runbooks.md`; `.agents/skills/live-galaxy-rust-conventions/SKILL.md`; and
`.agents/skills/live-galaxy-rust-tests/SKILL.md`.
