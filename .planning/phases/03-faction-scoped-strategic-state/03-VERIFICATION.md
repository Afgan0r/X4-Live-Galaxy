---
phase: 03-faction-scoped-strategic-state
verified: 2026-08-28T23:30:11Z
status: passed
score: 5/5 must-haves verified
behavior_unverified: 0
overrides_applied: 0
decision_coverage:
  honored: 9
  total: 9
  not_honored: []
---

# Phase 3: Faction-Scoped Strategic State Verification Report

**Phase Goal:** ZYA, ARG, and their primitive institutions receive deterministic, replayable strategic state grounded in authoritative observations and permitted faction information.
**Verified:** 2026-08-28T23:30:11Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Frozen observations supply economic, military, territorial, and threat facts for both factions, including shared XEN pressure and observed KHK. | ✓ VERIFIED | `derive_packets` accepts only `ProjectionSnapshot`, derives all four `FactFamily` values, preserves quality as typed availability, and the paired tracer test passes with XEN and KHK fixture facts. |
| 2 | Each faction packet records its permitted facts and the policy that constructed them. | ✓ VERIFIED | `StrategicPacket` retains a faction-visible fact set and `visibility-v1`; foreign changing facts are explicitly `Inaccessible`, while own, threat, and static-map facts retain their availability. `visibility_contract` passes. |
| 3 | Equivalent frozen projections produce canonical facts, priorities, primitives, admission inputs, and replay identity. | ✓ VERIFIED | Ordered `BTreeMap` input plus sorted facts/evidence/primitives feed `AdmissionInputs` and the FNV-1a replay fingerprint. Permutation and changed identity/content/version tests pass. |
| 4 | Each faction has exactly three shared capability contracts with versioned, non-official internal labels and fixed doctrine priorities over one visible snapshot. | ✓ VERIFIED | Exhaustive `Capability::ALL` has three variants; packet construction creates three views sharing one snapshot ID. Capability and doctrine contract tests prove labels and ZYA/ARG order. |
| 5 | Missing, stale, inaccessible, private, or unsupported facts cannot silently become institution or Executive planning inputs. | ✓ VERIFIED | Typed availability is retained; primitive evidence accepts only `Available` facts and returns `UnavailableRequiredFact` otherwise. The focused contract test covers unknown, stale, and unsupported military evidence. |

**Score:** 5/5 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/strategic-state/src/derive.rs` | Pure, bounded projection-to-paired-packet compiler | ✓ VERIFIED | Accepts `&ProjectionSnapshot`; checks fact/primitive limits before construction; performs visibility filtering without I/O or X4/model/persistence dependencies. |
| `crates/strategic-state/src/fact.rs`, `policy.rs`, `packet.rs` | Explicit availability, visibility, and immutable faction packet boundary | ✓ VERIFIED | Quality maps to `Available`/`Unknown`/`Stale`/`Inaccessible`/`Unsupported`; views expose capability and snapshot ID, not raw projection or private facts. |
| `crates/strategic-state/src/faction.rs` | Exact shared capabilities and faction profiles | ✓ VERIFIED | Three exhaustive capabilities; `doctrine-v1` ZYA/ARG labels and differentiated priorities are explicitly Live Galaxy product policy, not X4 official names. |
| `crates/strategic-state/src/primitive.rs`, `primitive_evidence.rs`, `fingerprint.rs` | Finite planning-only primitive allowlist and deterministic replay input | ✓ VERIFIED | Four typed variants, Executive-only bilateral posture, priority/evidence bounds, canonical keys, and replay fingerprint. No effect executor or game command route exists. |
| `crates/strategic-state/tests/*.rs` | Behavioral boundary, determinism, capability, doctrine, and availability evidence | ✓ VERIFIED | All seven focused targets pass: 11 tests total. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| Phase 1 `ProjectionSnapshot` / `SectionQuality` | `derive_packets` | Pure accepted-projection input and availability mapping | ✓ WIRED | `observation-ingest::ProjectionSnapshot` is the sole compiler input; `fact::availability` maps the Phase 1 quality enum. |
| Visibility filtering | `FactionVisibleSnapshot` and `InstitutionView` | Pre-derivation filtering and one shared snapshot ID per faction | ✓ WIRED | `visible()` applies own/threat/static-map rules; the capability contract verifies every view references its packet snapshot ID. |
| Canonical packet fields / primitive evidence | `AdmissionInputs` / `ReplayFingerprint` | Sorted fact references and primitive canonical keys | ✓ WIRED | Permuted source insertion produces equal canonical facts, inputs, and fingerprint; changed record identity/content/version changes identity. |
| Available visible facts | `ShadowPrimitive::derive` | `primitive_evidence::collect` accepts only `Available` evidence | ✓ WIRED | Unknown, stale, and unsupported required military evidence return typed failure before any primitive output. |

### Data-Flow Trace (Level 4)

`AcceptedProjection` → immutable `ProjectionSnapshot` → `derive_packets` → faction-visible facts with availability → three institution views / bounded Shadow primitives / admission inputs / fingerprint. The chain uses accepted Phase 1 observations; it has no static fallback, mock-only production input, provider call, persistence, X4 command, or game mutation.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Paired facts and visibility | `cargo test -p strategic-state --test tracer_packet`; `--test visibility_contract` | 2/2 and 1/1 passed | ✓ PASS |
| Shared institutions and doctrine | `cargo test -p strategic-state --test capability_contract`; `--test doctrine_priority` | 1/1 and 1/1 passed | ✓ PASS |
| Bounded primitives and replay | `cargo test -p strategic-state --test shadow_primitive_contract`; `--test packet_determinism`; `--test mutation_baseline` | 2/2, 3/3, and 1/1 passed | ✓ PASS |
| Rust quality gates | `cargo fmt --all --check`; `cargo run -p source-size-lint`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace` | All commands succeeded | ✓ PASS |
| Diff hygiene | `git diff --check` | Passed on the stable full-suite revision; current in-progress Phase 3 test/matrix edits also pass their scoped diff check. | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plans | Status | Evidence |
| --- | --- | --- | --- |
| OBS-04 | 03-01, 03-03 | ✓ SATISFIED | Four typed fact families compile for both packets with bounded capacity and preserved availability. |
| OBS-05 | 03-01, 03-03 | ✓ SATISFIED | XEN is shared typed pressure; KHK remains a typed fact only when present in the accepted projection. |
| MIND-02 | 03-01 | ✓ SATISFIED | Versioned visibility policy and explicit inaccessible availability are recorded and contract-tested. |
| MIND-03 | 03-01, 03-02, 03-03 | ✓ SATISFIED | Pure bounded packet compiler, shared institution views, and finite typed planning candidates are implemented. |
| MIND-04 | 03-03 | ✓ SATISFIED | Canonical ordering and deterministic replay fingerprint are exercised by permutation and changed-input tests. |
| INST-01 | 03-02 | ✓ SATISFIED | Exactly three shared capability contracts and six locked descriptive labels are contract-tested. |
| INST-02 | 03-02 | ✓ SATISFIED | All three faction views reference the same immutable faction-visible snapshot ID. |

### Decision Coverage

`check.decision-coverage-verify` reports all 9/9 Phase 3 CONTEXT decisions honored. In particular, no private institution facts, fourth diplomacy institution, negotiation, X4 relation mutation, model call, persistence, or X4 command was introduced.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- |
| — | — | No production `unwrap`, `expect`, unsafe Rust, debt marker, or placeholder implementation found in the strategic-state source. | — | None |

## Residual Risk

The compiler is locally verified against accepted Phase 1 projection contracts, not observed X4 behavior. Exact X4 runtime semantics for visibility, freshness, and hostile observations remain Phase 1/Phase 7 evidence work and are deliberately not claimed as observed-in-X4 here. `cargo-mutants` is unavailable; the measured command failure is recorded in `03-03-SUMMARY.md` without inventing a mutation score, leaving the scoped mutation baseline for Phase 8 once the reviewed runner is available.

---

_Verified: 2026-08-28T23:30:11Z_
_Verifier: the agent (gsd-verifier)_
