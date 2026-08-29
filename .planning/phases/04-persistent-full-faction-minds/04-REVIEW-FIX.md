---
phase: 04-persistent-full-faction-minds
review: 04-REVIEW.md
fixed: 2026-08-29
status: resolved
findings_resolved: [CR-01, CR-02]
---

# Phase 04 Code Review Fixes

## CR-01: Provider-relative budget eligibility

`BudgetProfile` now uses checked addition and accepts a capsule only when its
measured tokens plus required headroom fit within the provider context limit.
The capsule identity also binds the context limit, measured tokens, and
headroom so changing any budget input changes canonical identity.

Regression coverage exercises below-limit, exact-limit, above-limit, and
integer-overflow profiles.

## CR-02: Real v0-to-v1 migration

Migration now accepts bounded raw `mind-checkpoint-v0` bytes, decodes a
deny-unknown-fields legacy wire type, validates every authoritative payload
identity, constructs a current envelope, and recomputes its canonical integrity
hash before exposing it. A successful conversion requests one durable target
write. Malformed, partial, unknown-field, oversized, and unsupported inputs
retain an available fallback without a write or fail closed without a
projection.

Migration conversion lives separately from current-envelope recovery so the
strict source-file limit remains enforced.

## Verification

- `cargo test -p mind-persistence --test recovery_contract` — passed, 6 tests.
- `cargo test -p mind-persistence --test capsule_contract` — passed, 3 tests.
- `cargo test -p mind-persistence --test mutation_baseline` — passed, 2 tests.
- `cargo run -p source-size-lint` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo test --workspace` — passed.
- `cargo fmt --all --check` — passed.
- `git diff --check` — passed.

No X4 runtime behavior or mutation score is claimed.
