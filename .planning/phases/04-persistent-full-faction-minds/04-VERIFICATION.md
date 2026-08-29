---
phase: 04-persistent-full-faction-minds
verified: 2026-08-29T00:00:00Z
status: passed
score: 8/8 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 3/8
  gaps_closed:
    - "Accepted mind state is persisted as a typed, reconstructible checkpoint rather than an opaque Debug string."
  gaps_remaining: []
  regressions: []
---

# Phase 04: Persistent Full Faction Minds Verification Report

**Phase Goal:** Full, distinct ZYA and ARG minds preserve coherent short- and long-term strategy plus one-owner institution initiatives across compaction, restart, retry, and schema transitions.
**Verified:** 2026-08-29T00:00:00Z
**Status:** passed
**Re-verification:** Yes — after closure commits `c3db89a` and `d6667ac`

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Independent ZYA and ARG doctrine, motives, priorities, plans, and typed Executive posture differ on a shared scenario. | VERIFIED | `mind_tracer::creates_independent_doctrine_divergent_replayable_minds` passed; `FactionProfile` supplies distinct locked priorities and `MindCommand::from_packet` derives the aggregate. |
| 2 | Model-relative compaction produces versioned typed-plus-narrative capsules without giving the narrative authority. | VERIFIED | `capsule_contract` passed 4/4: budget eligibility and identity bind typed commitments; a corrupted or replaced narrative does not alter them. |
| 3 | Each institution has at most one active typed Shadow initiative with identity, objective, evidence, priority, lifecycle, and owner. | VERIFIED | `initiative_lifecycle` passed 3/3; `ledger::apply` enforces the bounded command log and slot transitions. |
| 4 | Initiative proposal through terminal outcome remains replayable causal evidence. | VERIFIED | Lifecycle tests passed; restored aggregate compares exact causal events per replayed command. |
| 5 | Accepted snapshot/mind/initiative/replay/admission/report state recovers transactionally with no duplicate plan, tick, initiative, or report. | VERIFIED | `CheckpointPayload` holds all six identities plus typed `MindCheckpointState`; checkpoint, recovery, and fake-port contracts passed. |
| 6 | Corrupt, partial, incompatible, duplicate, out-of-order, and version-transition records fail closed or retain last valid state. | VERIFIED | Decode validates strict serde shape, payload restoration, checksum, and canonical predecessor digest; recovery tests cover typed v0 migration and opaque legacy-string rejection. |
| 7 | Compact authoritative state uses an X4-owned contract and does not read or modify player save files. | VERIFIED | PowerShell schema contract passed; XML has one extension-scoped checkpoint variable and prohibits model/return-channel/game-mutation terms. |
| 8 | Compatible Rust reconnect/retry preserves accepted state/report identity; incompatible game protocol names X4 restart. | VERIFIED | Fake-port and Rust/MD schema contracts passed. Actual X4 observation remains correctly deferred to Phase 7. |

**Score:** 8/8 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- |
| `crates/mind-domain/src/checkpoint.rs` | Versioned typed mind checkpoint | VERIFIED | `MindCheckpointState` serializes a `PendingMindCommit` with `deny_unknown_fields`. |
| `crates/mind-domain/src/restore.rs` | Validate and reconstruct full faction mind | VERIFIED | Rebuilds from locked `FactionProfile` and pending mind event, replays every initiative command, checks exact per-command events and final aggregate. |
| `crates/mind-persistence/src/checkpoint.rs` | Bounded envelope codec and typed payload | VERIFIED | Payload contains `MindCheckpointState`; encode/decode validate payload, checksum, sequence, and predecessor hash. |
| `crates/mind-persistence/src/legacy.rs` | Safe typed v0 migration | VERIFIED | `LegacyV0.mind` is `MindCheckpointState`; serde rejects opaque Debug strings and unknown/partial data before conversion. |
| `crates/mind-persistence/src/fake_port.rs` | CAS/retry/reconnect seam | VERIFIED | Genesis, successor cursor, exact retry, collision, and reread rules are enforced. |
| `extensions/live_galaxy/md/live_galaxy_persistence.xml` | Restricted X4-owned persistence schema | VERIFIED (static) | One extension-scoped payload root; runtime behavior is not claimed. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `PendingMindCommit` | `MindCheckpointState` | `checkpoint_state()` | WIRED | Commit is cloned as typed state, not formatted Debug text. |
| `CheckpointEnvelope::decode` | full-mind validation | `checkpoint_validation::payload` then `MindCheckpointState::restore` | WIRED | Decode rejects a syntactically valid but semantically unreplayable mind. |
| typed state | reconstructed aggregate | `restore::restore` | WIRED | Locked profile/core, pending event, slots/history/ledger, command idempotency log, exact events, and final aggregate are checked. |
| current envelope | successor envelope | cursor/predecessor CAS | WIRED | Predecessor sequence and 16-character lowercase-hex digest are validated; fake port requires exact current cursor. |
| legacy bytes | current envelope | `decode_and_convert` | WIRED | Typed legacy payload is validated and rehashed before recovery exposes a migration target. |

### Data-Flow Trace

| Artifact | Data variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| mind domain | `PendingMindCommit` | normalized `StrategicPacket` and locked faction profile | deterministic aggregate plus pending event | FLOWING |
| persistence envelope | `typed_mind_commit` | `commit.checkpoint_state()` | typed serialized state validated on encode and decode | FLOWING |
| recovery | `MindAggregate` | `MindCheckpointState::restore()` | exact reconstructed aggregate only after replay match | FLOWING |
| fake X4 port | acknowledged cursor | validated envelope CAS/reread | deterministic local adapter evidence; not X4 runtime | FLOWING (local) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Full typed checkpoint round trip and corruption buckets | `cargo test -p mind-domain --test mind_checkpoint` | 3 passed | PASS |
| Doctrine divergence and one-owner lifecycle | `cargo test -p mind-domain --test mind_tracer` and `--test initiative_lifecycle` | 1 + 3 passed | PASS |
| Typed envelope, tamper rejection, canonical predecessor identifier | `cargo test -p mind-persistence --test checkpoint_tracer` | 3 passed | PASS |
| Crash recovery and typed legacy migration | `cargo test -p mind-persistence --test recovery_contract` | 6 passed | PASS |
| Retry/idempotency and protocol compatibility | `cargo test -p mind-persistence --test fake_port_contract` | 2 passed | PASS |
| Static X4 persistence boundary | `powershell -NoProfile -ExecutionPolicy Bypass -File extensions/live_galaxy/tests/persistence_schema_contract.ps1` | `Persistence schema contract passed.` | PASS |

### Requirements Coverage

| Requirement | Status | Evidence |
| --- | --- | --- |
| MIND-01 | SATISFIED | Independent, profile-bound full aggregates and Executive posture are exercised by `mind_tracer`. |
| MIND-05 | SATISFIED | Shared normalized scenario produces doctrine-conditioned, non-cosmetic divergence. |
| INST-03 | SATISFIED | Three slots, one active owner, typed initiative data, and lifecycle preemption are behavior-tested. |
| INST-08 | SATISFIED | Causal ledger/events replay exactly and remain Shadow-only. |
| MODEL-05 | SATISFIED | Provider-relative budget and typed-authoritative capsule contracts passed. |
| STATE-01 | SATISFIED | Versioned, bounded X4-owned static contract plus validated Rust envelope; no player-save route. |
| STATE-02 | SATISFIED | Typed checkpoint includes accepted snapshot, mind, initiative, replay, admission, and reserved report identities atomically. |
| STATE-03 | SATISFIED | Exact retry/reload preserves accepted tick/report identity without duplication. |
| STATE-04 | SATISFIED | Invalid candidate envelopes retain only validated acknowledged projection with rejection diagnostics. |
| STATE-05 | SATISFIED | Crash, duplicate, stale/out-of-order, typed migration, malformed/unknown/opaque legacy fixtures are executable. |
| STATE-06 | SATISFIED (local contract) | Fake compatible reload and incompatible `X4RestartRequired` path passed; Phase 7 owns observed-in-X4 proof. |

Decision coverage is independently clean: `check.decision-coverage-verify` reported 11/11 CONTEXT decisions honored.

### Quality Gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | PASS |
| `cargo run -p source-size-lint -- --max-lines 200` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS |
| `git diff --check` | PASS |
| `cargo mutants --version` | NOT AVAILABLE: Cargo reports `no such command: mutants`; no score or survivor disposition is claimed. |

### Anti-Patterns Found

No blocker or warning was found in the Phase 04 implementation. The former opaque `format!("{commit:?}")` checkpoint representation is absent from the current persistence path. The clean `04-REVIEW.md` agrees, but this report relies on the code and independently run tests above.

## Re-verification of the Former Blocker

`c3db89a` replaces the opaque string with `CheckpointPayload.typed_mind_commit: MindCheckpointState`. `CheckpointEnvelope::decode` invokes `checkpoint_validation::payload`, which bounds the state then calls `restore()`.

`restore()` validates schema/capacity, uses the locked `FactionProfile` to validate doctrine/priorities/posture, checks the pending `MindUpdated` event, recreates the core from an empty faction aggregate, replays every stored initiative command, compares exact emitted events for each command, and finally requires aggregate equality. That equality includes slots, history, causal ledger, and command/idempotency records. `mind_checkpoint` corrupts core, profile, slot, history, ledger, and command buckets and verifies rejection.

`d6667ac` validates both envelope and predecessor checksums as canonical 16-character lowercase hexadecimal identifiers. The constructor, decoder, and tracer reject malformed/short/overlong predecessor hashes; the fake port requires the exact predecessor cursor for each successor.

Legacy v0 conversion accepts only `mind: MindCheckpointState`, validates it through the same restore path, and recalculates the current envelope checksum. The recovery contract proves typed legacy conversion and explicitly rejects the old opaque `"mind":"legacy"` shape without a write.

## Residual Risks and Deferred Evidence

- Phase 7 owns disposable X4 proof for payload capacity, interrupted writes, save/load restoration, Rust-only reconnect timing, and incompatible-protocol behavior. This report makes no observed-in-X4 claim.
- A hostile local editor able to rewrite X4-owned checkpoint storage and recompute the public checksum is outside the locked Phase 04 trust boundary. The checksum is corruption/identity/CAS evidence, not authentication; no secret/key authority was specified.
- `cargo-mutants` is unavailable in this environment. This is a recorded mutation-coverage residual, not a passing mutation result and not a blocker to the behavior-tested Phase 04 contract.

---

_Verified: 2026-08-29T00:00:00Z_
_Verifier: the agent (gsd-verifier)_
