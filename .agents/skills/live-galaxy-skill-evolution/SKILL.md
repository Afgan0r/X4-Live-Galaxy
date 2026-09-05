---
name: live-galaxy-skill-evolution
description: Convert verified Live Galaxy implementation lessons into minimal durable project skill updates.
---

# Live Galaxy Skill Evolution

Use this skill after completed implementation, verification, debugging, or
review reveals a reusable project rule or a defect in a project-local skill.

## Mandatory Completion Gate

Before declaring an implementation, debugging, verification, or review task
complete, inspect its deviations, failed probes, review findings, and changed
rules. Record one of these outcomes in the task's existing summary, review, or
verification output; do not create a separate learning artifact:

- `Skill learning: none` — name the evidence considered and why it does not
  establish a reusable lesson.
- `Skill learning candidate` — name the evidence, single owning project skill,
  existing-rule check, minimal proposed rule, and whether owner approval is
  required or obtained.

Repeated violations of an existing rule are a workflow-enforcement candidate,
not a reason to duplicate that rule. A candidate remains open until the owner
approves it or explicitly defers it; never silently discard it.

## Gate

A lesson is durable only when it is supported by repository evidence, a
reproduced failure, an accepted product decision, or verified runtime behavior.
Do not persist guesses, one-off preferences, or unresolved alternatives.

An owner-approved prescriptive engineering decision is also valid evidence;
creating a quality standard does not require a prior production defect. Honor
the current task's explicit authorization to design or revise rules without
asking again for already-approved decisions.

Before editing a skill:

1. identify the exact recurring failure or missing rule;
2. name the single owning project skill;
3. check existing rules and changelog for duplication;
4. propose the minimal generalized change;
5. obtain explicit owner approval when the lesson changes behavior rather than
   merely correcting an objective error.

## Update

- Change only the owning skill and its `CHANGELOG.md`.
- Preserve concise, imperative, testable rules.
- Fix the class of defect demonstrated by evidence without widening into
  adjacent policy.
- Do not patch global or vendored skills from this repository.
- If the issue belongs upstream, record the local bridge needed now and route
  upstream feedback separately.

Use a single rule owner: common engineering/logging/tooling belongs to
`live-galaxy-code-conventions`; general test sufficiency and evidence to
`live-galaxy-tests`; Rust and X4-specific rules to their specialized skills;
review dispatch, adjudication, and verdicts to `live-galaxy-code-review`.
Update only necessary consumer links when routing changes. Do not copy the
same behavioral rule into tests, review, and language skills independently.

## Verify

Format the changed Markdown, run `git diff --check`, re-read the changed rule,
and confirm that a future agent can determine when it applies and how to verify
it.
