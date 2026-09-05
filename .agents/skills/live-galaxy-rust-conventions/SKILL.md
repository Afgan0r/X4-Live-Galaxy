---
name: live-galaxy-rust-conventions
description: >-
  Rust-specific correctness, ownership, and lint rules for Live Galaxy. Use for
  Rust design, implementation, refactoring, or review (конвенции Rust,
  написание и ревью Rust).
---

# Live Galaxy Rust Conventions

Read [common code conventions](../live-galaxy-code-conventions/SKILL.md) and
their applicable references first. They own common responsibility, errors,
state, recovery, bounds, logging, and tooling. This skill adds Rust-specific
obligations; it does not select dependencies or an async runtime.

## RUST-01 — Typed boundaries

Use distinct domain types where swapping identifiers, clocks, units, quantities,
or states changes meaning. Keep raw external representations outside validated
domain values until validation succeeds. Constructors, conversions, decoding,
and mutation preserve the same invariants; private fields alone do not prove
that every construction path is validated.

Use enums for meaningful closed alternatives and typed recoverable errors.
`Option` means expected absence, not erased failure. Match relevant safety/state
variants explicitly; a catch-all must not grant new variants silent success.
Keep provider-specific shapes at their boundary. Preserve error causes without
requiring one giant global error enum.

## RUST-02 — Ownership and cost

Borrow for access; take ownership when the operation needs to retain the value.
Prefer an honest ownership-taking signature over always borrowing and cloning.
Copying for an independent immutable snapshot is valid; unexplained cloning
or shared mutable containers used only to evade ownership design are not.
Consider aliases, lifetime, and synchronization consequences.

Choose traits, generics, trait objects, and smart pointers for actual boundary
or ownership requirements. Do not impose an interface per type or genericize
a local operation for hypothetical reuse. Check conversion/arithmetic range
assumptions or justify them through the explicit contract.

## RUST-03 — Production safety

Forbid `unwrap`, `expect`, and panic paths in production, including tools.
Do not panic on external/recoverable input or use unchecked indexing across
those boundaries. Tests and test-only helpers may fail immediately when that
expresses their contract; this must not create a production escape path.

Forbid unsafe Rust throughout the current workspace. A verified future need
requires an explicitly approved dedicated boundary crate with a safety
contract and focused tests.

## RUST-04 — Modules and lints

- Keep every Rust source file at or below 200 physical lines, including blank
  lines, comments, documentation, and test-only sections.
- Split by cohesive responsibility before the limit. Do not use arbitrary
  numbered chunks, source inclusion tricks, or renamed dumping grounds.
- Keep `lib.rs` and `mod.rs` focused on declarations, composition, and exports.
- Apply the same Rust/Clippy policy to repository tools.
- Treat stable `all` and `pedantic` as blocking except individually audited
  cosmetic/documentation-only checks. Enable `nursery` individually and
  re-audit when the pinned Rust version changes.
- Never weaken a lint or command to pass. Use the narrowest
  `#[expect(..., reason = "...")]` only for a verified false positive or
  behavior intrinsic to an explicit contract under the established policy.

## Verification

Use [Rust tests](../live-galaxy-rust-tests/SKILL.md) and repository-defined
formatting/lint commands for affected code. Follow focused iteration and one
required final regression after review convergence. Instruction-only changes
do not justify unrelated Rust runtime suites.
