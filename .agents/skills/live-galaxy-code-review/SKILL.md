---
name: live-galaxy-code-review
description: >-
  Review Live Galaxy changes against engineering and test contracts, with
  bounded specialist fan-out for substantive reviews (ревью кода, проверка
  конвенций, багов, тестов и логирования).
---

# Live Galaxy Code Review

## Review ownership

The review lead owns a complete review and the final verdict. In a GSD workflow
this may be the designated code-review agent; do not create a second competing
lead. Specialists provide candidates, not authoritative findings or votes.

Read root `AGENTS.md`, [code conventions](../live-galaxy-code-conventions/SKILL.md),
[tests](../live-galaxy-tests/SKILL.md) and its
[standards](../live-galaxy-tests/references/test-standards.md), then all
language/task-specific skills and their applicable references. Read
[logging](../live-galaxy-code-conventions/references/logging.md) for changed
decision/error/state paths and [tooling](../live-galaxy-code-conventions/references/tooling.md)
for scripts/check runners. Rust uses Rust conventions/tests; Lua/MD uses X4
integration, its Lua/MD reference, and X4 tests. Known loaded files need not be
read again in the same review.

## REV-01 — Establish the review

Identify the requested diff/base, current revision, affected contracts, and
unrelated work. Establish Git freshness before relying on a repository. Review
new/changed logic and necessary related corrections, including existing defects
activated by the change. Read relevant surrounding implementation, callers,
tests, and accepted architecture; the diff alone may omit the actual owner.

Existing code is not an exemplar. Do not expand ordinary review into an
unrequested whole-repository audit or import external stack conventions.

## REV-02 — Lead pass and specialist selection

Choose applicable independent lenses using [fan-out](references/fan-out.md).
For substantive reviews use up to three GPT-5.6 Luna / high specialists:
bugs/correctness, missing or ineffective tests, and diagnostic gaps. Keep tiny
localized changes inline; omit a lens without meaningful work. Follow the
runtime's available tools and routing constraints, with no hidden model upgrade.

While specialists work, the lead independently checks all applicable
conventions: responsibility, interfaces, types, error/state contracts,
resources, concurrency, persistence, compatibility, language rules,
configuration, logging, and tools. Trace cross-component effects and selected
risk gates. Do not reduce the lead pass to formatting or trust a clean lens as
proof of global correctness. An instruction-only change calls for consistency
and appropriate behavioral validation, not invented product-runtime findings.

## REV-03 — Evidence and adjudication

For each candidate, verify the trigger, applicable rule/contract, exact code
path, and observed or logically established consequence. Check caller-owned
logging, existing tests, explicit fallbacks, and permitted exceptions before
accepting a gap. A test that cannot detect its claimed defect is not useful
coverage. A fake cannot prove real storage durability or X4 semantics.

An explicit mandatory convention violation is actionable even without a
reproduced runtime bug, but name the concrete violation and maintenance or
correctness consequence. Mere taste, theoretical risk, missing per-function
logs, extra imaginable test cases, or arbitrary coverage targets are not
findings. For a test gap, identify the missing mandatory scenario or specific
incorrect behavior the tests would let pass.

Merge duplicates by root cause and affected behavior. Rejected candidates need
a brief evidence-based disposition in the existing review record or chat,
not a permanent candidate database. The lead assigns severity and smallest
viable correction; specialist confidence, repetition, and majority are not
evidence. Follow the selected security/AI/mutation/X4 gates when applicable;
Luna lenses do not replace a required specialist gate.

## REV-04 — Severity and output

- **P0:** destructive behavior, secret exposure, uncontrolled game mutation,
  or release-blocking corruption.
- **P1:** likely incorrect strategic behavior, broken recovery, duplicated
  effects, unsafe trust boundary, or major compatibility break.
- **P2:** concrete bounded correctness, maintainability, observability, or
  required-test gap.
- **P3:** low-risk actionable improvement within scope.

Use the smallest supported severity. A concrete mandatory-rule violation still
requires a fix or an explicitly authorized exception regardless of its label;
optional improvements do not become blockers merely by being numbered.

Lead with deduplicated findings ordered by severity. Each includes exact
location, trigger/violation, applicable rule ID or architecture contract,
evidence, consequence, and smallest correction. State unresolved evidence gaps
and the resulting readiness verdict separately from optional suggestions.
No findings means only no grounded findings in the examined scope.

## REV-05 — Re-review and completion

Re-read the current changed code and affected contracts after fixes. Confirm
each finding as fixed, still open, rejected with evidence, or explicitly
deferred by the owner; do not accept a description of the fix as proof. Check
for regressions and second-order effects. Reuse specialists only for a
remaining or newly affected question; do not rerun all lenses mechanically.

Wait for selected specialist results before the final verdict. If a specialist
fails or is unavailable, the lead covers the lens directly and states that
fallback, or reports its unresolved limit. Do not abandon a running lens
silently or claim full coverage from an incomplete pass.

Use focused verification during fixes and the required final relevant
regression once after convergence. Preserve GSD's completion gates and existing
review artifact format. Apply
[project skill maintenance](../../../AGENTS.md#maintaining-project-skills)
to confirmed review lessons.

When maintaining this skill, use [evaluation inputs](evals/inputs.md) for a
bounded independent behavioral pass and compare with
[expectations](evals/expectations.md) afterward. Never provide expectations or
suspected findings to the evaluating agent.
