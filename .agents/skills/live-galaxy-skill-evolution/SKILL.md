---
name: live-galaxy-skill-evolution
description: Convert verified Live Galaxy implementation lessons into minimal durable project skill updates.
---

# Live Galaxy Skill Evolution

Use this skill after completed implementation, verification, debugging, or
review reveals a reusable project rule or a defect in a project-local skill.

## Gate

A lesson is durable only when it is supported by repository evidence, a
reproduced failure, an accepted product decision, or verified runtime behavior.
Do not persist guesses, one-off preferences, or unresolved alternatives.

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

## Verify

Run the shared skill validator, format Markdown, re-read the changed rule, and
confirm that a future agent can determine when it applies and how to verify it.
