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
- Isolate unsafe code behind a minimal reviewed boundary with an explicit safety
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

## Completion

Once a Cargo workspace exists, run the repository-defined formatting, linting,
focused tests, and full tests. Treat warnings as defects unless a documented
exception owns them.
