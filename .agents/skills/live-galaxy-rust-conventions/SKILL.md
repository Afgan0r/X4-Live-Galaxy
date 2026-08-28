---
name: live-galaxy-rust-conventions
description: Rust architecture and correctness rules for the Live Galaxy bridge and strategic kernel.
---

# Live Galaxy Rust Conventions

Use this skill for Rust design, implementation, refactoring, and review.

## Boundaries

- Keep game adapters, model providers, persistence, strategic domain logic, and
  orchestration behind explicit interfaces.
- Domain code consumes normalized typed state, not raw XML, JSON, transport
  messages, or provider responses.
- Model output remains untrusted until schema, semantic, safety, budget, and
  current-state validation succeeds.
- Only the deterministic application layer may convert an accepted strategic
  primitive into a game command.

## Correctness

- Represent identifiers, quantities, time, reputation, priorities, and state
  transitions with domain types rather than interchangeable primitives.
- Make invalid states difficult to construct. Prefer enums and explicit state
  machines over boolean combinations and magic strings.
- Preserve replay inputs and stable ordering wherever a decision must be
  reproducible.
- Make command acceptance and recovery idempotent. Retries must not duplicate
  game effects.
- Reject stale or invalid actions before persistence or game mutation. Avoid
  partial success unless the contract explicitly models it.

## Errors and Safety

- Do not panic on game, model, storage, configuration, or transport input.
- Do not use unchecked indexing or `unwrap`/`expect` across external or
  recoverable boundaries.
- Preserve actionable context while preventing secrets, private prompts, and
  hidden reasoning from entering logs.
- Bound payloads, collections, loops, retries, queues, and external calls.
- Forbid unsafe Rust throughout the current workspace. A verified future need
  requires an explicitly approved dedicated boundary crate with a safety
  contract and focused tests.

## Design

- Prefer small cohesive modules and dependency inversion at external seams.
- Separate pure decision logic from I/O so it can be replayed and tested.
- Keep provider-specific behavior in adapters. Do not leak one model vendor's
  response shape into the domain.
- Avoid speculative abstractions. Add an interface when a real boundary or test
  seam requires it.
- Do not select crates or async architecture from convention alone; phase
  research owns dependency and runtime choices.

## Lint and Module Boundaries

- Keep every Rust source file at or below 200 physical lines, including blank
  lines, comments, documentation, and test-only sections.
- Give each module one bounded responsibility. Split by domain or adapter
  ownership before the size limit is reached; never satisfy the limit by moving
  arbitrary chunks into vaguely named files.
- Keep `lib.rs` and `mod.rs` focused on declarations, composition, and
  re-exports. Do not accumulate unrelated implementation logic in them.
- Apply the same Rust and Clippy policy to repository-owned tools as to product
  crates. A tool is not an escape hatch from workspace quality gates.
- Forbid `unwrap`, `expect`, and panic paths in production code. Tests and
  test-only helpers may use them when immediate test failure expresses the
  contract more directly than recoverable error handling.
- Never weaken a lint, threshold, or lint command merely to make a task pass.
  Use the narrowest possible `#[expect(..., reason = "...")]` only for a
  verified false positive or behavior intrinsic to an explicit contract.
- Treat stable `all` and `pedantic` lints as blocking except for individually
  audited cosmetic or documentation-only checks. Enable `nursery` lints
  individually, and re-audit that list whenever the pinned Rust version changes.

## Completion

Once a Cargo workspace exists, run the repository-defined formatting, linting,
focused tests, and full tests. Treat warnings as defects unless a documented
exception owns them.
