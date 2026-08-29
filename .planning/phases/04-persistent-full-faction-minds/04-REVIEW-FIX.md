---
phase: 04-persistent-full-faction-minds
review: 04-REVIEW.md
fixed: 2026-08-29
status: resolved
findings_resolved: [CR-01, CR-02, CR-03]
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

## CR-03: Authoritative capsule identity

Capsule identity now length-frames the schema, ledger range, provider-relative
budget, and every typed commitment field. Goal, plan, posture, and initiative
owner changes therefore produce distinct identities, while the optional
narrative remains deliberately outside the authoritative binding.

Regression coverage changes each commitment field independently and verifies
that a narrative-only replacement preserves identity.

## Verifier Gap: Reconstructible Typed Mind State

The checkpoint payload now stores a versioned `MindCheckpointState` instead of
Debug text. Its bounded owned identifiers and strict serde shape preserve the
complete aggregate, three initiative slots, history, causal ledger, recorded
commands, per-command events, and the pending mind transition.

Restore validates the locked faction profile and core state, reconstructs the
base mind from the pending event, replays every initiative command, compares
its exact events, and exposes only an exactly matching aggregate. Legacy
migration accepts the same typed state and rejects the former opaque string.

## Final Review: Canonical Chain Identifiers

Current and predecessor checkpoint checksums must use the canonical fixed-width
lowercase hexadecimal encoding before the state can enter recovery or CAS.
Malformed, short, overlong, and valid predecessor cases are covered. The
checksum remains corruption, identity, and chain evidence; hostile local
checkpoint rewriting is outside the locked trust boundary and no
authentication claim is made.

## Verification

- `cargo test -p mind-persistence --test recovery_contract` — passed, 6 tests.
- `cargo test -p mind-persistence --test capsule_contract` — passed, 4 tests.
- `cargo test -p mind-persistence --test checkpoint_tracer` — passed, 3 tests.
- `cargo test -p mind-domain --test mind_checkpoint` — passed, 3 tests.
- `cargo test -p mind-persistence --test mutation_baseline` — passed, 2 tests.
- `cargo run -p source-size-lint` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo test --workspace` — passed.
- `cargo fmt --all --check` — passed.
- `git diff --check` — passed.

No X4 runtime behavior or mutation score is claimed.
