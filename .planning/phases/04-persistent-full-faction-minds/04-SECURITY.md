---
phase: 04-persistent-full-faction-minds
audited: 2026-08-29
reaudited_after: [c3db89a, d6667ac]
asvs_level: 1
block_on: high
threats_open: 0
status: secured
---

# Phase 04 Security Audit

## SECURED

**Phase:** 04 — Persistent Full Faction Minds  
**Threats Closed:** 15/15  
**ASVS Level:** 1

All implemented mitigation paths were verified in code or executable contracts.
Runtime X4 save/load, interruption, and reconnect observations remain explicitly
pending Phase 7; they are not represented as observed-in-X4 evidence.

## Threat Verification

| Threat ID | Category | Severity | Disposition | Evidence |
| --- | --- | --- | --- | --- |
| T-04-01 | Tampering | high | mitigate | Command replay/collision checks and bounded causal transition are implemented in `crates/mind-domain/src/ledger.rs`; `initiative_lifecycle` passes duplicate, preemption, and replay cases. |
| T-04-02 | Repudiation | high | mitigate | Immutable causal events, predecessor, disposition, and ownership transitions are emitted by `crates/mind-domain/src/ledger.rs`; `initiative_lifecycle` passes. |
| T-04-03 | Elevation of privilege | high | mitigate | `mind-domain` exports planning-state transitions only; its manifest depends solely on workspace-local `strategic-state`. No provider, transport, report, persistence, or X4 operation is exported. |
| T-04-04 | Tampering | high | mitigate | `CheckpointEnvelope` requires schema/protocol/sequence/predecessor/integrity validation and deny-unknown-fields decode in `crates/mind-persistence/src/checkpoint.rs`; `checkpoint_tracer` and the XML schema contract pass. |
| T-04-05 | Elevation of privilege | high | mitigate | `extensions/live_galaxy/tests/persistence_schema_contract.ps1:70-72` rejects model, report, acknowledgement, pipe, Lua, and game-mutation terms; the static MD cue contains storage declaration only. |
| T-04-06 | Repudiation | medium | mitigate | `04-X4-PERSISTENCE-EVIDENCE.md` separates Documented, Verified locally, Pending-X4, and Observed in X4; the static contract verifies all runtime properties remain pending-X4. |
| T-04-07 | Tampering | high | mitigate | Canonical encode/decode checks envelope size, bounded fields, schema/protocol identities, and recomputed integrity binding (`checkpoint.rs:71-147`); adversarial decode tests pass. |
| T-04-08 | Tampering | high | mitigate | `FakeCheckpointPort` validates predecessor cursor/hash and returns an exact retry acknowledgement (`fake_port.rs:24-94`); fake-port contract passes stale and content-collision cases. |
| T-04-09 | Denial of service | medium | mitigate | Envelope, identifier, mind-state, legacy, capsule field, narrative, and evidence limits are enforced before persistence/recovery allocation (`checkpoint.rs`, `legacy.rs`, `capsule.rs`); focused contracts pass. |
| T-04-10 | Repudiation | high | mitigate | Typed cursor/hash acknowledgement plus explicit compatible/restart-required disposition are implemented in `port.rs`; fake-port protocol-mismatch test passes. |
| T-04-11 | Tampering | high | mitigate | Recovery retains only validated acknowledged state and migration converts bounded, deny-unknown-fields v0 input before requesting one target write (`recovery.rs`, `migration.rs`, `legacy.rs`); six recovery fixtures pass. |
| T-04-12 | Information disclosure | medium | mitigate | Capsule narrative is bounded to 1024 bytes, remains outside authoritative identity, and cannot replace typed commitments (`capsule.rs:3, 121-139, 178-183`); capsule contract passes. |
| T-04-13 | Denial of service | medium | mitigate | Capsule range/profile/field checks, checked budget addition, and recovery/checkpoint byte caps fail closed (`capsule.rs:50-53`, `checkpoint.rs:7-9`, `legacy.rs:5, 19-22`). |
| T-04-14 | Repudiation | medium | mitigate | `04-04-SUMMARY.md:68,88` records the exact unavailable `cargo mutants` command, no installation/substitution, and that no score or survivor category was inferred. |
| T-04-SC | Tampering | high | mitigate | `cargo mutants --version` currently returns `no such command: mutants`; no package was installed. `mind-persistence/Cargo.toml` uses workspace crates plus pinned `serde`/`serde_json` only. |

## Verification Run

- `cargo test -p mind-domain` — passed (3 tests).
- `cargo test -p mind-persistence` — passed (17 tests).
- `pwsh -NoProfile -File extensions/live_galaxy/tests/persistence_schema_contract.ps1` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo run -p source-size-lint -- --max-lines 200` — passed.
- `cargo test --workspace` and `cargo fmt --all --check` — passed.
- `cargo mutants --version` — unavailable; recorded without installation or fabricated mutation evidence.

## Residual Risks

- Runtime X4 payload capacity, interruption, save/load restoration, and reconnect
  timing require the disposable Creative Custom procedure owned by Phase 7.
- The reviewed mutation runner is unavailable, so mutation score and survivor
  counts remain unavailable rather than assumed.
- A hostile local editor able to rewrite X4-owned checkpoint storage and
  recompute the public checksum is outside this phase's locked trust boundary.
  The checksum is deterministic corruption detection, chaining, and identity;
  it is not authentication and no secret/signing authority is in scope.

## Re-audit: Typed Restore and Canonical Digest

Commits `c3db89a` and `d6667ac` preserve the secured verdict.

- `CheckpointPayload` persists `MindCheckpointState`, and
  `checkpoint_validation::payload` validates its serialized bound and calls
  `restore()` before an envelope can be encoded or decoded
  (`crates/mind-persistence/src/checkpoint_validation.rs:8-29`).
- Typed restoration rejects unsupported schema, invalid core fields, invalid
  nested values, over-capacity collections, and non-replayable aggregate state
  before exposing a `MindAggregate` (`crates/mind-domain/src/restore.rs:9-52`).
  `mind_checkpoint` passes canonical round-trip plus unknown, malformed, and
  oversized state rejection.
- Legacy v0 conversion accepts only bounded, deny-unknown-fields typed state;
  opaque legacy `mind` values are rejected and recovery retains a valid fallback
  without requesting a write (`crates/mind-persistence/src/legacy.rs:7-48`; the
  `recovery_contract` invalid-legacy fixture passes).
- Current and predecessor digest strings must be exactly 16 lowercase hex
  characters before construction or decode. The checksum binds the predecessor
  cursor and complete payload; malformed, short, and overlong predecessor
  digests are rejected (`crates/mind-persistence/src/integrity.rs:1-15`,
  `crates/mind-persistence/src/checkpoint.rs:42-49,130-171`, and
  `checkpoint_tracer` pass).
- CAS/recovery remains exact: successor writes require the acknowledged cursor
  and matching predecessor, while corrupt, stale, duplicate, out-of-order, and
  crash inputs retain only the validated acknowledged state
  (`fake_port.rs:59-94`, `recovery.rs:144-180`).

Re-audit gates passed: `mind_checkpoint`; `checkpoint_tracer` (3 tests);
`recovery_contract` (6 tests); workspace Clippy; source-size lint; complete
workspace tests; and formatter.

**threats_open:** 0
