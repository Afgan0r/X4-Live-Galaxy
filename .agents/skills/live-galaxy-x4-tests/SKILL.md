---
name: live-galaxy-x4-tests
description: Test Live Galaxy Lua, Mission Director XML, X4 adapters, packages, and runtime evidence. / Тесты Lua, Mission Director XML, адаптеров, пакетов и X4 runtime Live Galaxy.
---

# Live Galaxy X4 Tests

Use this skill for Lua, Mission Director XML, X4 adapter, package, game-command,
or runtime-integration tests. Read
[`live-galaxy-tests`](../live-galaxy-tests/SKILL.md),
[`live-galaxy-x4-integration`](../live-galaxy-x4-integration/SKILL.md), and the
global `lua` skill first. The common skill owns general test sufficiency,
oracles, doubles, fixtures, diagnostics assertions, and evidence reporting.

## X4-Specific Test Layers

- **XT-01 — Real package paths:** Parse XML and check the manifest, UI
  registration, Mission Director structure, identifiers, entrypoints, and
  generated package claims. Compile shipped Lua with the actual interpreter and
  load real product modules through normal extension-relative `require` paths.
- **XT-02 — Pure Lua:** Keep policy, serialization, normalization, scheduling,
  batching, budgeting, and diff logic independent from X4 globals. Run it with
  deterministic fixtures in the compatible, pinned standalone Lua runner
  selected by current X4 evidence.
- **XT-03 — Adapter contract:** Fake only explicit X4 seams. Exercise
  successful observations plus absent or malformed identities, rejected
  context, and thrown native failures. For commands, check identity, preview or
  state-version validation, idempotency, bounded retry, rejection without
  partial mutation, and independent readback semantics.
- **XT-04 — Cross-language path:** When Lua/MD changes affect Rust boundaries,
  run the actual owned producer/consumer path. A fake adapter proves the local
  seam; it cannot establish a real X4 API or game behavior.
- **XT-05 — In-game evidence:** Use a disposable Creative Custom campaign or
  approved test copy under a written plan. The user performs all X4 actions.
  Report a scenario as `observed in X4` only after expected behavior and its
  health surface succeed with exact version, mod set, setup, elapsed real/game
  time, SETA state when relevant, and independent readback recorded.

## Runtime and Diagnostic Evidence

- Keep static, pure-Lua, fake-adapter, locally integrated, pending-game, and
  observed-in-X4 evidence separate. No fake or source assertion substitutes for
  a required probe.
- Select X4 probe and SETA soak gates from the project risk matrix: apply a
  probe when local evidence cannot establish runtime behavior and a SETA soak
  when timing, scheduling, lifecycle, recovery, or accelerated-time load
  changes. They are not automatic for every Lua or XML edit.
- Test the relevant semantic diagnostic event identity, reason, correlation,
  and state. Significant-operation history and diagnostic failure policy are
  defined by
  [`code-conventions logging`](../live-galaxy-code-conventions/references/logging.md):
  a runtime log failure must be independently visible, must not stop X4, and
  does not waive durability or idempotency evidence. Do not assert formatted
  lines or unbounded per-frame traces.

## Lua Mutation Testing

- Mutate only executable pure Lua modules initially. Native X4 adapters and
  Mission Director XML remain outside mutation scoring until a useful harness
  is demonstrated.
- Run a bounded `Universal Mutator` spike against one representative pure module
  and its compatible standalone tests. Pin evaluated tool versions and keep the
  spike out of public runtime dependencies.
- Measure invalid, trivial, equivalent, killed, and surviving mutants before
  making it a required gate. Universal Mutator does not list Lua as supported;
  generic rewriting alone is not acceptance. Do not build a replacement mutation
  engine. If the spike is noisy or incompatible, defer the gate and retain the
  selected unit, contract, and runtime evidence.

Follow the focused suite and final regression commands in
`live-galaxy-x4-integration`; use its tool provisioning path rather than
inventing another runner workflow.
