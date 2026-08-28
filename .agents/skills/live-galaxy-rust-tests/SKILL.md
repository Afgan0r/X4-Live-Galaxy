---
name: live-galaxy-rust-tests
description: Test strategy for deterministic, recoverable, and bounded Live Galaxy Rust behavior.
---

# Live Galaxy Rust Tests

Use this skill when designing, writing, or reviewing Rust tests.

Read `.agents/skills/live-galaxy-rust-conventions/SKILL.md` first.

## Test Observable Contracts

- Test public behavior, state transitions, persisted effects, and emitted
  commands rather than private implementation details.
- Every accepted strategic primitive needs a success case, rejection cases, and
  an idempotent replay case.
- Invalid, stale, malformed, oversized, or unauthorized input must fail without
  partial state or game mutation.
- Model behavior is tested through recorded typed fixtures or deterministic
  fakes. Normal tests must not call live models.

## Required Risk Coverage

- deterministic replay from the same normalized snapshot and decision inputs;
- persistence recovery across interruption boundaries;
- duplicate delivery, retry, and out-of-order message handling;
- schema and semantic validation at game, model, storage, and configuration
  boundaries;
- budgets for actions, retries, time, payload size, and model usage;
- compatibility behavior when optional integrations are absent or changed;
- structured diagnostics for every rejected or degraded path.

## Test Shape

- Prefer small pure unit tests for policy and state transitions.
- Use contract tests at adapter boundaries and focused integration tests across
  persistence or transport seams.
- Keep end-to-end in-game probes minimal, disposable, and attributable to one
  hypothesis.
- A regression test must fail for the original defect and identify the behavior
  being protected.
- Avoid timing sleeps, network dependencies, order-dependent fixtures, and
  assertions that merely repeat implementation details.

## Workflow

Follow red-green-refactor when a phase enables implementation. Record the exact
focused command in the phase plan. Once the Cargo workspace exists, run focused
tests first and the full repository test suite before completion.
