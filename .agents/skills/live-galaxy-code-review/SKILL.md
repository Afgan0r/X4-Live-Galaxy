---
name: live-galaxy-code-review
description: Risk-first code review for the Live Galaxy Rust and X4 integration stack.
---

# Live Galaxy Code Review

Use this skill for review of implementation, tests, protocols, or integration
changes.

Read the matching convention and test skills before reviewing. For any
X4-facing change, read both
`.agents/skills/live-galaxy-x4-integration/SKILL.md` and
`.agents/skills/live-galaxy-x4-tests/SKILL.md`.

## Review Order

1. Confirm the requested scope and preserve unrelated user changes.
2. Trace changed behavior across model, Rust, persistence, transport, and game
   boundaries that are actually touched.
3. Verify trust-boundary validation, idempotency, recovery, deterministic
   replay, resource bounds, and observability.
4. Verify tests exercise behavior and failure paths rather than only happy-path
   structure.
5. Reject source-text assertions as primary behavioral evidence when executable
   pure, contract, or in-game verification is possible.
6. Check applicable mutation-test survivors and their dispositions.
7. Check compatibility and provenance claims against evidence.

## Severity

- **P0** — destructive behavior, secret exposure, uncontrolled game mutation,
  or release-blocking corruption.
- **P1** — likely incorrect strategic behavior, broken recovery, duplicated
  actions, unsafe trust boundary, or major compatibility break.
- **P2** — bounded correctness, maintainability, observability, or test gap with
  a concrete failure mode.
- **P3** — low-risk improvement that is still actionable and in scope.

Do not report style preferences without a concrete project rule or failure
mode. Do not ask for unrelated refactors.

## Output

Lead with findings ordered by severity. Each finding names the exact location,
observable failure, evidence, and smallest viable fix. Then list open questions
and residual verification gaps. If there are no findings, say so and name the
specific risk classes checked.
