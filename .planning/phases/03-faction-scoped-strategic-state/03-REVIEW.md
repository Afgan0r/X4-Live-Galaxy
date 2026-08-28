---
phase: 03-faction-scoped-strategic-state
reviewed: 2026-08-29T00:00:00Z
depth: deep
files_reviewed: 18
files_reviewed_list:
  - Cargo.toml
  - crates/strategic-state/Cargo.toml
  - crates/strategic-state/src/derive.rs
  - crates/strategic-state/src/fact.rs
  - crates/strategic-state/src/faction.rs
  - crates/strategic-state/src/fingerprint.rs
  - crates/strategic-state/src/lib.rs
  - crates/strategic-state/src/packet.rs
  - crates/strategic-state/src/policy.rs
  - crates/strategic-state/src/primitive.rs
  - crates/strategic-state/src/primitive_evidence.rs
  - crates/strategic-state/tests/capability_contract.rs
  - crates/strategic-state/tests/doctrine_priority.rs
  - crates/strategic-state/tests/mutation_baseline.rs
  - crates/strategic-state/tests/packet_determinism.rs
  - crates/strategic-state/tests/shadow_primitive_contract.rs
  - crates/strategic-state/tests/tracer_packet.rs
  - crates/strategic-state/tests/visibility_contract.rs
findings:
  critical: 1
  warning: 1
  info: 0
  total: 2
status: issues_found
---

# Phase 03: Code Review Report

**Reviewed:** 2026-08-29T00:00:00Z
**Depth:** deep
**Files Reviewed:** 18
**Status:** issues_found

## Summary

The crate is pure, bounded at the raw fact and primitive-evidence boundaries,
and all requested local quality checks pass. However, its replay identity is
not complete for the accepted projection, and its advertised primitive limit
is not enforced. Both break Phase 3's bounded/replayable admission contract.

## Critical Issues

### CR-01: Replay fingerprint aliases distinct accepted state

**File:** `crates/strategic-state/src/derive.rs:56-76`; `crates/strategic-state/src/fact.rs:79-92`; `crates/strategic-state/src/fingerprint.rs:25-54`

**Issue:** Derivation discards the observation entity kind/identity and all
state/version/content after mapping it to only `(owner, family, threat subject,
availability)`. `VisibleSnapshotId` is also a fixed faction constant. Thus two
different accepted projections with the same number of available ZYA economic
facts—for example `ZYA:economy:ore` replaced by `ZYA:economy:energy`—produce
the same admission inputs and `ReplayFingerprint`. Downstream replay/cache or
admission logic can reuse a result derived from materially different world
state, contrary to MIND-04's exact replay identity requirement.

**Fix:** Retain a typed canonical source identity plus the relevant accepted
observation/version payload fingerprint in `StrategicFact`/`FactReference`,
and include it with unambiguous length framing in admission inputs and replay
fingerprint bytes. Add a negative regression test proving same-family entity
replacement and changed accepted content/version change the fingerprint.

## Warnings

### WR-01: `PacketLimits.primitives` accepts values below actual output

**File:** `crates/strategic-state/src/derive.rs:11-24, 46-50`; `crates/strategic-state/src/primitive.rs:4, 64-104`

**Issue:** `PacketLimits::new(_, primitives)` stores a primitive budget, but
derivation rejects only zero. A caller selecting `PacketLimits::new(32, 1)` or
`new(32, 3)` receives packets from which `ShadowPrimitive::derive` always
constructs four primitives. The fixed internal maximum therefore bypasses the
configured caller budget; no test exercises a nonzero value below four.

**Fix:** Propagate the configured primitive limit into primitive derivation and
return `PrimitiveLimitExceeded` when the four candidate set exceeds it, or
remove the misleading independent field and expose the fixed four-primitive
contract explicitly. Add exact-boundary tests for 3, 4, and 5.

## Evidence Run

- `cargo test -p strategic-state` — passed.
- `cargo fmt --all --check` — passed.
- `cargo lint` — passed.
- `cargo test --workspace` — passed.
- `cargo run -p source-size-lint -- --max-lines 200` — passed.
- `git diff --check 2c37cfd..ae0e925` — passed.

---

_Reviewed: 2026-08-29T00:00:00Z_
_Reviewer: gsd-code-reviewer_
_Depth: deep_
