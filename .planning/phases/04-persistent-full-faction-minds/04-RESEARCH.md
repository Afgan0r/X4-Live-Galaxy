# Phase 4: Persistent Full Faction Minds - Research

**Researched:** 2026-08-29
**Domain:** Campaign-authoritative persistence and recovery for the Shadow Director
**Confidence:** HIGH for MD persistence semantics; MEDIUM for unobserved runtime limits

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Mind and institution continuity

- **D-01:** Each faction preserves doctrine, motives, priorities, short-term
  plans, long-term plans, and one Executive-owned typed diplomatic posture.
- **D-02:** Each of the three institutions owns at most one active typed Shadow
  initiative. Preemption preserves the previous initiative and its disposition.
- **D-03:** Executive and institution conversations may retain separate
  continuity, but the bridge-owned typed ledger is authoritative; conversation
  prose is not.

### Compaction

- **D-04:** History compacts by provider/model-relative token budgets into
  versioned typed-plus-narrative capsules. Typed facts and commitments remain
  authoritative.
- **D-05:** Exact thresholds and safety headroom are benchmark-derived rather
  than fixed by event count or elapsed game time.

### Persistence authority

- **D-06:** Compact runtime state uses an X4-owned persistence contract.
  External databases, caches, diagnostics, and prose are non-authoritative.
- **D-07:** The implementation never reads or modifies player save files.
- **D-08:** Accepted snapshot, mind, initiative, replay, admission, and report
  intent state is transactional and idempotent.

### Recovery and restart

- **D-09:** Corrupt, partial, incompatible, duplicate, out-of-order, and
  version-transition inputs fail closed or recover the last valid state with
  structured evidence.
- **D-10:** A compatible Rust process can restart, update, and reconnect to the
  same running X4 process without duplicating accepted state or report identity.
- **D-11:** An incompatible game-side protocol revision fails closed and names
  the required X4 restart.

### the agent's Discretion

Storage engine, transaction layout, schema versions, migration mechanics,
capsule encoding, and crash points are technical decisions. They must satisfy
the locked authority and recovery contract.

### Deferred Ideas (OUT OF SCOPE)

- Migration of public API credentials or public runtime settings is outside
  milestone 0.1.
- Mutable institutional power and multiple simultaneous initiatives per
  institution remain later work.
</user_constraints>

## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| MIND-01 | Independent full ZYA and ARG minds with doctrine, plans, and posture | Separate aggregate roots keyed by faction identity; typed ledger is the continuity source. [VERIFIED: 04-CONTEXT.md] |
| MIND-05 | Measurably distinct responses | Persist doctrine/policy version and frozen replay input with every admitted cycle. [VERIFIED: PROJECT.md] |
| INST-03 | At most one active initiative per institution | A unique active-slot invariant is committed in the same admission transaction. [VERIFIED: 04-CONTEXT.md] |
| INST-08 | Replayable causal evidence | Append immutable causal transitions; never overwrite the predecessor on preemption. [VERIFIED: 04-CONTEXT.md] |
| MODEL-05 | Model-relative typed-plus-narrative capsules | Store typed commitment projection independently from the non-authoritative narrative capsule. [VERIFIED: 04-CONTEXT.md] |
| STATE-01..06 | X4-owned, transactional, recoverable persistence | X4 checkpoint contract is the authority boundary; external SQLite is a rebuildable projection only. [VERIFIED: PROJECT.md] |

## Summary

Phase 4 must implement a durable **mind ledger**, not a Rust-owned campaign database. The locked product contract says compact runtime state is X4-owned and makes external cache, diagnostics, and prose non-authoritative. Therefore, a local SQLite database may accelerate restart, replay, diagnostics, and testing, but it cannot be used to decide which campaign state is current after its state differs from X4. [VERIFIED: PROJECT.md; VERIFIED: 04-CONTEXT.md]

The smallest safe design is an X4-owned opaque checkpoint record plus an append-only, acknowledged mutation protocol. The record needs only an extension-scoped schema/version, protocol compatibility identity, durable monotonic ledger sequence, canonical checkpoint hash, and opaque canonical payload. Rust owns the typed encoding and validates it; X4 owns its durable lifetime with the running campaign.

The Egosoft Mission Director Guide establishes the required persistence surface: loading a saved game restores saved MD state, variables added by newer script versions are absent from older savegames, and cue-version patches run when older saved cue state loads. This supports a versioned variable on a stable, long-lived static root cue as the authoritative compact checkpoint. It does not prove payload limits, interruption atomicity, or current X4 9.00 runtime behavior; those remain Phase 7 observations rather than blockers to the local Phase 4 contract. [DOCUMENTED: Egosoft Mission Director Guide, MD refreshing and patching]

**Primary recommendation:** Implement the Rust ledger and codec against an X4 checkpoint port, add the fixed MD cue/variable schema and migration shape now, and keep its runtime evidence explicitly pending until the existing Phase 7 disposable campaign. Do not add SQLite in this phase: a rebuildable external projection is optional and would add an unverified dependency without helping campaign authority.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
| --- | --- | --- | --- |
| Campaign checkpoint lifetime | X4 runtime | Rust bridge | X4 owns campaign continuity; Rust serializes and validates its payload. [VERIFIED: PROJECT.md] |
| Mind/initiative causal ledger | Rust bridge | X4 runtime | Rust owns normalized state, validation, recovery, and persistence mechanics behind the X4-owned contract. [VERIFIED: AGENTS.md] |
| Exact current checkpoint acknowledgement | X4 runtime | Rust bridge | Only the persisted MD checkpoint can establish campaign authority after restart. [DOCUMENTED persistence; acknowledgement protocol designed locally] |
| External read models and diagnostics | Rust bridge | — | Useful for evidence and fast startup, explicitly non-authoritative. [VERIFIED: PROJECT.md] |
| Player-facing report delivery | X4 runtime | Rust bridge | Phase 6 owns the Mail/Logbook projection and acknowledgement channel. [VERIFIED: ROADMAP.md] |

## Standard Stack

### Core

| Library / component | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| X4 checkpoint port | Pending disposable gate | Authoritative opaque checkpoint read/write/acknowledgement | It satisfies D-06 without reading player saves. [VERIFIED: 04-CONTEXT.md] |
| Rust typed domain ledger | Repository workspace | Canonical domain types, transitions, serialization boundary | Project rules require typed state machines, stable ordering, and no panic across recovery boundaries. [VERIFIED: live-galaxy-rust-conventions/SKILL.md] |
| `serde` with `derive` | `=1.0.229` already pinned | Versioned canonical checkpoint envelope encoding/decoding | The existing observation-ingest crate already pins it with `derive`; reuse it rather than adding a second serializer. Verbatim: `serde = { version = "=1.0.229", features = ["derive"] }`. [VERIFIED: crates/observation-ingest/Cargo.toml:12] |

**Installation:** No new crate is needed for Phase 4. Reuse the pinned workspace serialization stack and in-memory deterministic test projections.

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
| --- | --- | --- | --- | --- | --- | --- |
No new package is proposed.

**Packages removed due to [SLOP] verdict:** none.

**Packages flagged as suspicious [SUS]:** none. `rusqlite` was considered and rejected as unnecessary scope for this phase.

## Architecture Patterns

### System Architecture Diagram

```text
Frozen accepted snapshot + faction policy/version
  -> pure mind transition proposal
  -> validate identity, ordering, single-active-slot, causal predecessor
  -> canonical ledger entry + new aggregate projection
  -> one logical commit
       -> stable MD root-cue variable receives one complete envelope
       -> X4 returns the reread sequence/hash acknowledgement
          -> acknowledged: advance authoritative checkpoint cursor
          -> absent/invalid/no acknowledgement: fail closed; retain prior cursor
  -> optional local projection / diagnostics (rebuildable only)
  -> later Phase 6: report-intent outbox keyed by admitted report identity
```

### Recommended Project Structure

```text
crates/
├── mind-domain/          # typed minds, initiatives, causal events, pure transitions
├── mind-persistence/     # checkpoint codec, migration and recovery policies
└── x4-bridge/            # X4 checkpoint port and compatibility negotiation
tests/
├── fixtures/minds/       # canonical checkpoints and adversarial recovery cases
└── x4-disposable/        # X4 persistence-capability and restart probe evidence
```

### Pattern 1: Event ledger plus compact checkpoint

**What:** A canonical transition is immutable evidence; the compact aggregate is a deterministic projection of the ledger prefix. The checkpoint contains the latest committed sequence and hash, so recovery rejects a projection that does not match its ledger identity. [VERIFIED: live-galaxy-rust-tests/SKILL.md]

**When to use:** Every accepted mind update, initiative disposition, compaction replacement, migration completion, and future report-intent reservation.

**Rule:** The X4 port commits the complete opaque checkpoint atomically from X4's point of view. Rust must not mark a transition durable before receiving the matching acknowledgement. [ASSUMED]

### Pattern 2: Aggregate-root active initiative slot

**What:** Each `(faction, institution)` aggregate stores an optional active initiative ID and a causal history. A transition from active to replacement must record the predecessor, explicit disposition, trigger, and the Executive decision in the same logical commit. [VERIFIED: 04-CONTEXT.md]

**When to use:** Create, approve, revise, preempt, suspend, cancel, reject, complete, and fail transitions.

**Anti-pattern:** Mutating an `active` Boolean or replacing a row in place loses causal evidence and makes duplicate retry ambiguous. Use an explicit state enum and append-only causal record instead. [VERIFIED: live-galaxy-rust-conventions/SKILL.md]

### Pattern 3: Provider-relative compaction as a derived artifact

**What:** The durable typed ledger remains source of truth. A capsule records its codec/schema version, source ledger range and hash, provider/model budget profile identity, produced typed projection, and optional narrative. Narrative prose is never used to reconstruct commitments. [VERIFIED: 04-CONTEXT.md]

**When to use:** When a provider/model's configured token budget policy says compaction is needed; no provider call is required to calculate the policy or validate the capsule.

**Rule:** Store measured budget inputs and selected headroom profile, not a global token count. The actual thresholds stay configuration values derived from benchmark evidence in Phase 5/8. [VERIFIED: D-05]

### Pattern 4: Authority firewall

**What:** The X4 checkpoint port is the authority lane. Any local projection can be deleted/rebuilt from a verified X4 checkpoint plus retained runtime records; it never repairs or outranks X4. [VERIFIED: PROJECT.md]

**When to use:** Normal restart, diagnostics, development replay, and corrupt-local-store recovery.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Campaign authority | A Rust-only database treated as truth | X4-owned checkpoint port plus acknowledged cursor | External data must not decide campaign state. [VERIFIED: PROJECT.md] |
| Transaction semantics | Independent writes to snapshot, mind, initiative, and outbox tables | One canonical aggregate commit and X4 acknowledgement | Prevents partially accepted plans/report identities. [VERIFIED: D-08] |
| Schema evolution | Ad-hoc optional fields and silent fallback | Explicit envelope schema/version plus ordered migrations and fixtures | Unknown/incompatible state must fail closed. [VERIFIED: D-09] |
| Idempotency | Time-based duplicate suppression | Stable transition IDs and content hashes | Retries/reconnects must not duplicate accepted work. [VERIFIED: STATE-03] |
| Compaction | Trimming prose or event counts | Versioned typed-plus-narrative capsule derived from a ledger range | Preserves commitments and makes provenance replayable. [VERIFIED: D-04] |

## Exact X4-Owned Contract: Evidence and Gate

**Established evidence:** The project locks compact runtime state to an X4-owned persistence contract and prohibits player-save access. The Egosoft Mission Director Guide states that saved MD state is restored when a saved game loads, warns that newly introduced variables do not exist in older savegames, and defines cue-version patches for older saved cue state. A stable extension root cue can therefore own the checkpoint variable without direct save-file access. The installed `sn_mod_support_apis` manifest establishes only the existing Windows named-pipe transport; its `save="false"` content metadata is not evidence against MD cue-state persistence. [DOCUMENTED: Egosoft Mission Director Guide; VERIFIED: installed manifest]

**Required smallest contract (design, not yet X4-proven):**

| Operation | Required input/output | Safety property |
| --- | --- | --- |
| `load_checkpoint` | Stable root-cue variable -> opaque bytes, schema identity, sequence, hash, X4 runtime identity | Returns either a complete self-consistent record or explicit absence/failure. [DOCUMENTED persistence; locally verified codec required] |
| `store_checkpoint` | Expected predecessor sequence/hash plus complete opaque envelope -> reread acknowledgement | Logical compare-and-set prevents duplicate/out-of-order overwrite. [DESIGNED; runtime observation pending Phase 7] |
| `checkpoint_status` | Runtime/protocol identity -> compatibility disposition | Makes compatible Rust reconnect distinct from an incompatible game-side revision. [VERIFIED: D-10, D-11] |

**Runtime evidence:** The existing Phase 7 Creative Custom run must measure payload limits, persistence timing, save/load behavior, Rust-only reconnect, and last-good behavior after rejected/interrupted writes. It must not inspect or modify any player save. Phase 4 can implement and locally verify the contract now, but must label these properties `pending-X4` until Phase 7. [VERIFIED: AGENTS.md; VERIFIED: live-galaxy-x4-tests/SKILL.md]

## Recovery Matrix

| Condition | Required behavior | Durable evidence |
| --- | --- | --- |
| Rust crashes before X4 acknowledgement | Reload prior acknowledged X4 checkpoint; do not expose proposed transition as accepted. [ASSUMED] | Transition ID, predecessor cursor, failure class |
| Rust crashes after acknowledgement before local projection | Rebuild the local projection from X4 checkpoint; no second X4 write. [ASSUMED] | Checkpoint sequence/hash, projection cursor |
| Duplicate transition/request | Return prior admitted result when ID and content match; fail closed on same ID/different content. [VERIFIED: STATE-03] | Idempotency key and content hash |
| Out-of-order transition | Reject unless predecessor cursor/hash exactly matches. [ASSUMED] | Expected/received cursors |
| Corrupt/partial payload | Reject the record, retain last self-consistent acknowledged checkpoint, emit bounded diagnostic. [VERIFIED: STATE-04] | Decode/integrity failure and retained cursor |
| Schema upgrade | Apply ordered, tested migration to a copy; commit only a fully validated target envelope. [VERIFIED: STATE-05] | Source/target schema and migration identity |
| Compatible Rust release | Negotiate compatible protocol, reload X4 checkpoint, reconcile same report reservation identities. [VERIFIED: STATE-06] | Bridge generation, checkpoint identity |
| Incompatible X4-side revision | Fail closed and state that X4 restart is required. [VERIFIED: D-11] | Compatibility disposition and restart owner |

## Common Pitfalls

### Treating a local projection as recovery authority

**What goes wrong:** A newer local projection silently wins after X4 restarts or a checkpoint write fails.

**How to avoid:** Treat any local cursor beyond the last acknowledged X4 cursor as speculative/rebuildable; never emit an accepted plan/report from it. [VERIFIED: PROJECT.md]

### Losing preemption causality

**What goes wrong:** Replacement overwrites the active initiative, so replay cannot show why the former initiative ended.

**How to avoid:** Require predecessor identity, disposition, trigger, decision, and causal event in one transition. [VERIFIED: D-02]

### Compaction changes commitments

**What goes wrong:** A narrative summary becomes the only retained account of an objective or plan.

**How to avoid:** Keep typed facts/commitments in the ledger/projection; bind a capsule to exact source range/hash and mark narrative non-authoritative. [VERIFIED: D-03, D-04]

### Reserving report identity after an admission boundary

**What goes wrong:** A retry admits the same decision twice or produces multiple reports.

**How to avoid:** Reserve the report ID in the same logical mind transition. Phase 6 owns dispatch and acknowledgement, not reservation semantics. [VERIFIED: D-08; VERIFIED: 04-CONTEXT.md]

## Code Examples

```rust
// Pure domain boundary: no provider call and no direct SQLite/X4 access.
fn transition(
    prior: &MindAggregate,
    command: MindCommand,
) -> Result<PendingCommit, TransitionError> {
    // validate predecessor, causal link, and one-active-initiative invariant
    // return canonical event(s), updated typed projection, and report reservation
}
```

```text
read X4 checkpoint -> decode + validate -> replay/rebuild local projection
commit candidate -> X4 compare-and-set checkpoint -> receive acknowledgement
                   -> then update rebuildable SQLite/read diagnostics
```

These are proposed interfaces, not existing APIs. [ASSUMED]

## State of the Art

| Old Approach | Current Approach | Impact |
| --- | --- | --- |
| External database as the campaign state | X4-owned checkpoint plus optional non-authoritative local projection | Preserves campaign continuity when the bridge/cache is absent. [VERIFIED: PROJECT.md] |
| Conversation history as memory | Typed ledger plus optional narrative capsules | Ensures compaction cannot rewrite commitments. [VERIFIED: D-03, D-04] |
| Best-effort retry | Content-addressed idempotency and acknowledged cursor | Makes crash/reconnect behavior testable. [VERIFIED: STATE-03] |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | A stable long-lived root cue can retain the complete checkpoint variable with the documented saved MD state. | Exact X4-Owned Contract | Runtime evidence could require a different cue layout, while preserving the same port. |
| A2 | A single opaque checkpoint payload fits within the practical MD persistence surface. | Exact X4-Owned Contract | The contract may need bounded chunking or stricter compaction after Phase 7 measurement. |

## Open Questions (RESOLVED)

1. **What payload size and interruption semantics does the saved MD variable surface exhibit in X4 9.00?**
   - Recommendation: measure maximum safe bounded payload, write interruption behavior, and last-good retention before choosing checkpoint encoding/chunking.
   - Status: assigned to existing Phase 7 runtime evidence; not a blocker and no Phase 4 human gate is required.
2. **Can the MD adapter's logical compare-and-set survive every required lifecycle point?**
   - Recommendation: locally verify duplicate/stale rejection and re-read acknowledgement; Phase 7 exercises actual save/load and reconnect boundaries. If runtime evidence disproves it, retain only the strongest observed append/ack semantics and do not let a local store fill the gap.
   - Status: local contract required in Phase 4; runtime proof assigned to Phase 7 with no Phase 4 human gate.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| X4 installation | Disposable persistence gate | ✓ | target 9.00 | None for authoritative proof. [VERIFIED: PROJECT.md] |
| SirNukes Mod Support APIs | Existing Windows pipe precedent | ✓ | manifest `195` | Not a persistence fallback. [VERIFIED: installed manifest] |
| Rust/Cargo | Domain and fake-port tests | ✓ | available locally | None. [VERIFIED: Phase 1 research] |
| crates.io | No new Phase 4 dependency | not required | — | Reuse pinned workspace dependencies. |

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Cargo tests plus deterministic fakes; X4 black-box gate separately. [VERIFIED: live-galaxy-rust-tests/SKILL.md] |
| Quick run command | `cargo test -p mind-domain` [ASSUMED until workspace/crate exists] |
| Full suite command | `cargo test --workspace` [ASSUMED until workspace exists] |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| MIND-01 | Independent faction aggregate/replay identity | unit + property | `cargo test -p mind-domain faction_identity` | ❌ Wave 0 |
| MIND-05 | Same scenario produces doctrine-distinct typed state | fixture/property | `cargo test -p mind-domain divergence` | ❌ Wave 0 |
| INST-03 | Exactly one active slot; preemption preserves prior | state-machine/property | `cargo test -p mind-domain initiative_slot` | ❌ Wave 0 |
| INST-08 | Causal trace persists/replays canonically | integration | `cargo test -p mind-persistence causal_replay` | ❌ Wave 0 |
| MODEL-05 | Capsule preserves typed facts and source range/hash | unit + mutation | `cargo test -p mind-persistence capsule` | ❌ Wave 0 |
| STATE-01..05 | Recover from corrupt, crash-point, duplicate, order, migration fixtures | integration/property | `cargo test -p mind-persistence recovery` | ❌ Wave 0 |
| STATE-06 | Rust restart/reconnect and incompatible X4 revision | fake contract + disposable X4 gate | focused Cargo test plus manual recorded probe | ❌ Wave 0 |

### Crash and Mutation Strategy

- Inject failure at decode, validation, before X4 write, after X4 write/before acknowledgement, after acknowledgement/before SQLite projection, and during migration; assert the next startup uses the last acknowledged X4 cursor. [ASSUMED]
- Generate arbitrary legal and illegal transition sequences; assert no aggregate has more than one active initiative and every terminal/preempted initiative retains its causal predecessor. [VERIFIED: INST-03, INST-08]
- Mutate pure transition, integrity, cursor-comparison, capsule, and migration policy code with `cargo-mutants` only after a representative baseline exists. [VERIFIED: live-galaxy-rust-tests/SKILL.md]
- Keep X4 adapter code out of mutation scoring until the fake contract and disposable runtime harness demonstrate useful mutants. [VERIFIED: live-galaxy-x4-tests/SKILL.md]

### Wave 0 Gaps

- [ ] Fake X4 checkpoint port with scripted absence, duplicate acknowledgement, stale acknowledgement, corruption, and interruption outcomes.
- [ ] Canonical checkpoint codec and adversarial fixtures.
- [ ] State-machine/property harness for initiative lifecycle and idempotency.
- [ ] Static MD checkpoint schema/patch validation and a Phase 7 disposable evidence procedure.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V3 Session Management | Yes | Bind recovery to negotiated runtime/protocol identity and explicit compatibility disposition. [VERIFIED: D-10, D-11] |
| V4 Access Control | Yes | The checkpoint port is extension-scoped; player saves are never read or modified. [VERIFIED: D-07] |
| V5 Input Validation | Yes | Decode, version, integrity, predecessor, and semantic checks before any accepted projection. [VERIFIED: live-galaxy-rust-conventions/SKILL.md] |
| V6 Cryptography | No new cryptographic design | Use integrity/hash primitives selected by existing workspace evidence; do not invent cryptography. [ASSUMED] |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Corrupt/tampered checkpoint | Tampering | Strict decoding/integrity/version validation; retain last acknowledged valid cursor. [VERIFIED: STATE-04] |
| Replay/duplicate transition | Tampering | Stable identity plus content-hash idempotency check. [VERIFIED: STATE-03] |
| Stale/out-of-order writer | Tampering | Predecessor cursor/hash compare-and-set. [ASSUMED] |
| Oversized capsule/payload | DoS | Enforce measured budget and payload limits before checkpoint request. [VERIFIED: D-05] |
| Raw prompts/prose in diagnostics | Information disclosure | Typed safe summaries; prose is non-authoritative and redacted/bounded. [VERIFIED: AGENTS.md] |

## Phase Boundaries

| Phase | Owns | Phase 4 must leave untouched |
| --- | --- | --- |
| Phase 3 | Frozen faction-visible packets, deterministic facts, institution inputs | Do not redefine visibility, facts, or strategy derivation. [VERIFIED: 03-CONTEXT.md] |
| Phase 5 | Provider calls, candidate generation, bounded deliberation/admission behavior | Persist its admitted outcomes through the port; do not add live provider calls here. [VERIFIED: ROADMAP.md] |
| Phase 6 | Mail/Logbook dispatch, delivery acknowledgement, rich diagnostics | Reserve report identity only; do not implement the return channel. [VERIFIED: 04-CONTEXT.md] |
| Phase 7 | Disposable X4 restart/SETA operational proof | Supply probes/fixtures and explicit pending evidence; do not claim observed-in-X4 completion. [VERIFIED: ROADMAP.md] |

## Sources

### Primary (HIGH confidence)

- `AGENTS.md` — authority, save-file prohibition, typed/recoverable Rust rules, and layered X4 verification.
- `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, and `04-CONTEXT.md` — locked persistence, continuity, recovery, and phase ownership requirements.
- Project Rust and X4 skills — deterministic recovery, mutation, thin-adapter, and disposable-campaign test contracts.
- Installed `sn_mod_support_apis/content.xml` — current extension manifest; evidence limited to what it declares.

### Secondary (MEDIUM confidence)

- `.planning/research/ARCHITECTURE.md` and `PITFALLS.md` — local architecture and recovery precedent.
- `F:/Agent Projects/X4/tools/x4-live-protocol.md` and `tests/test_x4_live.py` — observed local atomic ingest/idempotency precedent, not Live Galaxy authority.
- `F:/Agent Projects/TALKER` planning contracts — typed causal history and idempotency precedent only; not X4 persistence evidence.

### Tertiary (LOW confidence)

- [Egosoft Mission Director Guide](https://wiki.egosoft.com/X%20Rebirth%20Wiki/Modding%20support/Mission%20Director%20Guide/?rev=32953.1) — documented saved MD state, old-save variables, and cue-version patching.
- [Serde docs](https://docs.rs/serde/latest/serde/) — already pinned workspace serialization stack.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — no new dependency is required.
- Architecture: MEDIUM — authority and documented persistence are established; payload and interruption behavior remain unobserved in X4 9.00.
- Pitfalls: HIGH — derive directly from locked recovery and test invariants.

**Research date:** 2026-08-29
**Valid until:** Revisit after the Phase 7 disposable X4 persistence evidence run.
