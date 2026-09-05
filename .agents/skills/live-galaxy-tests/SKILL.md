---
name: live-galaxy-tests
description: Design, implement, or review Live Galaxy tests and verification evidence. Use for test strategy, fixtures, test doubles, regressions, and test results. / Тестовая стратегия, фикстуры, двойники и проверка результатов Live Galaxy.
---

# Live Galaxy Tests

Use this skill for any Live Galaxy test design, implementation, review, or
verification result. It owns cross-language test sufficiency. Read the relevant
specialization as well: `live-galaxy-rust-tests` for Rust and
`live-galaxy-x4-tests` for Lua, Mission Director, adapters, packaging, or X4
runtime work.

Read
[`live-galaxy-code-conventions`](../live-galaxy-code-conventions/SKILL.md)
before adding or judging diagnostic assertions. Its
[`logging reference`](../live-galaxy-code-conventions/references/logging.md)
owns event and confidentiality policy; this skill defines how tests use that
policy.

## Select Evidence by Claim

- Apply [TEST-01](references/test-standards.md#test-01-scenarios-and-oracles):
  identify the observable contract and mandatory scenarios before choosing a
  test level. A percentage is diagnostic, never a universal acceptance gate.
- Apply [TEST-02](references/test-standards.md#test-02-minimum-capable-level):
  choose the smallest level that can prove the claim; use more than one level
  when claims cross a boundary. Test pure logic directly and do not mock the
  system under test.
- Apply [TEST-03](references/test-standards.md#test-03-offline-and-real-boundaries):
  normal unit and contract tests are offline. A cross-language boundary needs
  the actual producer/consumer product path, and product modules load through
  their normal loader.
- Apply [TEST-04](references/test-standards.md#test-04-persistence-and-recovery):
  prove durability and recovery with isolated real storage and an independent
  read or restart, using clean in-memory state.

## Working Rules

- Scope coverage to new or changed behavior and necessary related fixes. A
  suitable existing test may cover a pure refactor; do not manufacture tests
  for a low-impact reversible edit.
- Keep test names scenario- and outcome-oriented. One coherent behavior may
  require several actions and assertions.
- Preserve red-green-refactor when a phase enables implementation. A regression
  must fail for the reported defect before the fix and state the protected
  behavior.
- Report evidence precisely: `verified locally`, `pending game evidence`, and
  `observed in X4` are distinct states. A missing mandatory scenario cannot be
  reported as passing.

## Completion Evidence

- While iterating, run focused checks. After review convergence, run the final
  relevant full regression once. Record commands, results, and material
  coverage gaps in existing workflow artifacts; do not create a separate test
  dossier or a universal matrix.
- Select mutation, property, fuzz, performance, X4 probe, and SETA evidence by
  current risk and uncertainty. Follow the specialization and the project risk
  matrix; no universal fuzz or performance-framework gate exists.

Read [Test Standards](references/test-standards.md) for assertion, fixture,
double, determinism, diagnostic, and evidence details.
