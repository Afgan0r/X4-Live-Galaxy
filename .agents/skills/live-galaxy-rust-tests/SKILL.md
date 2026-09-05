---
name: live-galaxy-rust-tests
description: >-
  Test deterministic, recoverable, and bounded Live Galaxy Rust behavior.
  Тесты Rust-логики Live Galaxy: детерминизм, восстановление и ограничения.
---

# Live Galaxy Rust Tests

Use this skill when designing, writing, or reviewing Rust tests. Read
[`live-galaxy-tests`](../live-galaxy-tests/SKILL.md) and
[`live-galaxy-rust-conventions`](../live-galaxy-rust-conventions/SKILL.md)
first. The common skill owns general scenarios, oracles, doubles, fixtures,
diagnostic assertions, and test-run evidence.

## Rust-Specific Coverage

- **RT-01 — Deterministic admission:** Test normalized snapshots and decision
  inputs for stable replay and ordering. Exercise schema, semantic, safety,
  budget, and current-state validation before a strategic primitive is
  accepted.
- **RT-02 — Atomic rejection:** For malformed, stale, oversized, unauthorized,
  duplicate, and out-of-order input, assert the precise disposition and no
  partial persisted state, published state, or emitted game command.
- **RT-03 — Receipts and recovery:** Cover interruption boundaries, exact replay,
  conflicting duplicate identity, and recovery from an actual isolated store.
  A restart must retain the accepted state without publishing the action twice.
- **RT-04 — Bounded operation:** Test the relevant action, retry, payload,
  queue, candidate-work, time, or model-use limits. Include the refusal or
  degraded result at the boundary, not only successful use below it.
- **RT-05 — Adapter and compatibility seams:** Use focused contract tests for
  storage, transport, configuration, and optional integrations. Recorded typed
  fixtures or deterministic model fakes replace live models in normal tests.

## Mutation Testing

- Apply `cargo-mutants` to high-risk pure domain logic such as validation,
  state transitions, budgets, idempotency, reconciliation, and strategic
  primitives. Run it after the relevant phase and before a release gate, not
  on every commit.
- Establish a measured baseline before setting a required score. Triage every
  survivor as a missing behavioral test, equivalent mutant, or bounded gap; a
  headline score is not acceptance.
- Exclude generated code and side-effect-heavy adapters unless a phase proves
  mutation evidence there is actionable. Pin the runner in development or CI
  tooling, and use its normal `cargo test` or approved `cargo nextest` contract.

## Execution

Run repository formatting, linting, focused tests, and the relevant full test
suite. Record mutation scope, command,
baseline, survivors, and dispositions when that gate applies.
