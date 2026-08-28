# Live Galaxy Agent Instructions

## Project Role

This repository develops **Live Galaxy**, a public X4: Foundations mod backed by
an LLM strategic director. The director is intended to influence faction
economies, fleets, and institutions while remaining observable, recoverable,
and bounded by deterministic validation.

All `0.x` releases are internal prototypes. Version `1.0.0` is the first public
alpha. Do not describe a prototype as playable or public-ready without evidence.

## Authority and Product Boundaries

- The project owner makes product decisions. Agents own technical research,
  design, implementation, and verification within those decisions.
- X4 owns authoritative game state and the final application of actions.
- The Rust bridge owns normalized state, validation, persistence, recovery,
  model orchestration, caching, and structured diagnostics.
- Models propose typed goals, plans, and strategic primitives. They must not
  directly mutate game state or bypass deterministic validation.
- Milestone and phase product scope must be discussed with the owner before
  planning. Do not silently promote deferred ideas into the active milestone.
- Missions, player influence, news, deeper diplomacy, historical simulations,
  and special XEN/KHK architecture remain outside the first autonomous-director
  slice unless a later milestone explicitly admits them.

## Required Skills

Before structural changes, game research, mod research, accumulated knowledge,
or session outcomes, read the global `game-repo-standard` skill and every
reference it routes for the task.

Use the following project skills and read their entire `SKILL.md` files before
acting:

- `.agents/skills/live-galaxy-rust-conventions/SKILL.md` for Rust design,
  implementation, or review.
- `.agents/skills/live-galaxy-rust-tests/SKILL.md` for test design,
  implementation, or review.
- `.agents/skills/live-galaxy-code-review/SKILL.md` for code review.
- `.agents/skills/live-galaxy-x4-integration/SKILL.md` for X4 XML, Mission
  Director, Lua, game-data, installed-mod, or compatibility work.
- `.agents/skills/live-galaxy-skill-evolution/SKILL.md` when completed work
  reveals a reusable project rule or a skill defect.

Use the global `lua` skill for Lua changes, `mcp-builder` for MCP contract work,
and `openai-docs` for current OpenAI API or model behavior. Current official
documentation and repository evidence outrank memory.

## GSD Workflow

- GSD owns project discovery, requirements, roadmap, phase discussion, planning,
  execution, verification, review, and milestone closure.
- Run a deep product brainstorm before each new milestone and before locking the
  product scope of each phase.
- `.planning/config.json` is the GSD configuration source of truth.
- Project-local `.codex/agents/gsd-*.toml` files are generated machine-local
  routing artifacts. Never commit them.
- After changing GSD routing, run
  `python3 ~/.agents/scripts/sync-gsd-project-routing.py` from the repository
  root and restart Codex before dispatching a typed GSD agent.
- Repository documents and planning artifacts are written in English. Russian
  is the default conversation language.

## Evidence and External Sources

- Treat the X4 installation, vanilla data, installed mods, and third-party code
  as read-only research sources. Never edit them from this repository workflow.
- Never read or modify save files. Use disposable Creative Custom campaigns or
  explicit test copies only after a test workflow is designed.
- Separate facts as documented, observed, inferred, or unknown. Do not turn an
  inference into a compatibility guarantee.
- Study behavior and architecture, then implement Live Galaxy's own algorithms.
  Do not copy third-party code without a compatible license and explicit
  provenance review.
- Add a provenance entry only when a specific external source materially
  influences implementation. Do not build an exhaustive source corpus.
- Live Galaxy is incompatible with the Faction Enhancer suite in the first
  public alpha. It should support KUDA AI Tweaks, More AI Economy Ships, and
  Add More Sectors, subject to verified compatibility tests.

## X4 Reference Researcher

The canonical read-only role is
`.agents/agents/live-galaxy-x4-reference-researcher.md`. It supports four modes:
vanilla files, installed mods, compatibility, and provenance/licensing.

The agent must write research into the current GSD-owned artifact requested by
the caller. Durable semantic conclusions belong in personal MemPalace wing
`wing_x4_live_galaxy`; raw copied corpora do not. The role must never edit the
repository, game installation, or installed mods.

## Memory

Use only personal MemPalace wing `wing_x4_live_galaxy` for durable Live Galaxy
knowledge. Recall relevant decisions before milestone and phase planning.
Deduplicate before writing and verify every retained drawer after writing.
Corrections, invalidations, and deletions require exact preview and explicit
owner approval.

## Engineering Invariants

- Keep model output outside the trust boundary until it passes schema,
  semantic, safety, budget, and current-state validation.
- Preserve deterministic replay inputs for strategic decisions.
- Reject invalid or stale actions without partial game mutation.
- Bound time, memory, payload size, retries, model calls, and game-side work.
- Make recovery idempotent. A restart must not duplicate an accepted action.
- Prefer explicit state machines and typed domain values over implicit strings
  and boolean combinations.
- Never expose secrets, raw prompts containing private data, or hidden reasoning
  in logs or public diagnostics.
- Do not optimize performance without a measured bottleneck, except for explicit
  safety bounds and token/cache budgets.

## Verification

- Follow the selected project skills' focused checks first.
- For Rust changes, run formatting, linting, focused tests, and the full test
  suite once the Cargo workspace exists.
- For X4 integration, verify schemas and use the smallest disposable in-game
  probe that can answer the question.
- Format every edited Markdown file with `markdownlint-cli2 --fix`.
- Before claiming completion, inspect the diff and separate implemented facts,
  verified behavior, open risks, and deferred work.

## Git and Releases

- The base branch is `master`; GSD uses milestone branches after bootstrap.
- Do not commit or push unless explicitly requested. The initial empty-repository
  bootstrap is explicitly authorized for direct commit and push to `master`.
- Never commit runtime state, saves, databases, logs, credentials, generated
  agent routing, build output, or local settings.
- Use the global `git-commit` skill for every commit.
- No project license is selected. A license and third-party provenance audit are
  required before `1.0.0`.
