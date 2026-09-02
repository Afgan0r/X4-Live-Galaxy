<!-- markdownlint-disable MD013 MD041 -->
<!-- Managed by x4/agent-instructions. Do not hand-edit in a consumer repo. -->

# Shared X4 Modding Contract

## Bundle Integrity

Before product work, confirm that these committed files are present and
readable:

- `.agent-instructions/x4/AGENTS.md`;
- `.agent-instructions/x4/MEMORY.md`;
- `.agent-instructions/x4/CONTRACT_VERSION`;
- `.agent-instructions/x4/handoffs/knowledge-gap.md`;
- `.agent-instructions/x4/handoffs/validated-example.md`.

If any file is missing or unreadable, stop product work and restore the whole
bundle. Repository-local instructions remain authoritative for product
architecture, development workflow, engineering conventions, tests, release,
and branch/release policy. Commit and push authorization is owned by the
shared Git Completion section below.

## Repository Routing

The current repository identity is `x4/live-galaxy`; its local workflow
classification is `interactive-gsd`.

- **Docs MCP** owns read-only, provenance-bearing X4 API and modding knowledge,
  its ingestion pipeline, and disposable runtime probes used to close
  knowledge gaps.
- **Live Galaxy** is an independent X4 mod with Rust, X4 runtime, and LLM
  layers. It consumes Docs MCP during development.
- **Live MCP** is an independent gameplay mod and Docs MCP consumer. It is not
  part of Live Galaxy and is not an automatic evidence provider for Docs MCP.
- **agent-instructions** owns only this cross-repository contract and rollout.

Do not edit a sibling repository merely because the current task discovers a
need there. Use the applicable handoff and wait for the user to coordinate the
other task.

## Git Completion

The user explicitly authorizes agents to stage, commit, and push their work
autonomously in every repository listed in `config/repositories.tsv` in the
canonical `x4/agent-instructions` repository, including that repository itself.
Agents perform the development work in this workspace; the user does not
manually maintain the code. This standing authorization overrides generic or
older repository-local requirements to ask separately before committing or
pushing. A specific user instruction to pause or omit either action takes
precedence.

- After the task's required verification, inspect the diff, commit the scoped
  durable changes using the global `git-commit` skill, and push to the existing
  intended remote branch without another permission round trip.
- Follow the repository's branch and active workflow rules, including GSD
  milestone branches and review gates. Authorization does not bypass checks,
  authorize force pushes, or authorize merging or publishing releases.
- Repositories with no configured remote remain local-only: commit locally
  and report that push is unavailable. Do not create a remote or publish a
  new repository under this authorization.
- Before finishing, remove task-created disposable artifacts or keep them in
  the appropriate ignored local storage, then check
  `git status --short --branch`. Leave no uncommitted changes from the completed task and verify
  that its commits reached the intended remote when one is configured.
- Preserve changes owned by another active task and any user changes. Do not
  stage unrelated work or discard it merely to make the worktree clean. If
  ownership is unclear, identify the remaining changes rather than guessing.
- If verification, a workflow gate, a commit, or a push is blocked, preserve
  the work and report the exact blocker and remaining Git state. Do not
  describe blocked or unpublished work as fully complete.

## Common Evidence and Safety Boundaries

- Current repository evidence and fresh runtime observations outrank memory.
- Classify material claims as `documented`, `observed`, `inferred`, `unknown`,
  or `unsupported` as applicable. Preserve provenance and conflicts.
- Treat installed X4 files, installed extensions, third-party code, and local
  source snapshots as read-only unless the user explicitly requests the owning
  maintenance workflow.
- Never read or modify saves. Never publish private paths, raw corpora, runtime
  captures, prompts, credentials, or unlicensed source.
- The user starts X4 and performs every in-game action. Agents may prepare a
  run, read logs live, interpret evidence, and ask the user to enable SETA or
  perform another bounded action, but must never start or control the game.

## Docs MCP Lookup Gate

Before guessing about an unknown or uncertain X4 API, schema, event, loader,
lifecycle, or runtime behavior, query the registered read-only MCP server named
exactly `x4-docs`.

Use `search` for a bounded query. For each selected entity, call `inspect` with
the same snapshot set and continue with every returned `next_cursor` until it
is `null`. Preserve the exact request, snapshot identities, coverage,
provenance, conflicts, and residual unknowns in the current task evidence.

Do not treat one example as a universal rule. If Docs MCP lacks a load-bearing
claim, do not invent it or silently replace source research with an X4 probe.
First exhaust sources available to the current research task.

If the missing knowledge still blocks all or part of the task, read
`.agent-instructions/x4/handoffs/knowledge-gap.md`, emit its handoff in chat,
and stop only the dependent slice. Write no proposal file or queue and do not
poll. After the user says the data was added, repeat the exact original Docs
MCP request; that message is permission to retry, not proof of success.

After an adapted API use has executed successfully in X4 and its intended
behavior has been checked, read
`.agent-instructions/x4/handoffs/validated-example.md` and emit the scoped
receipt in chat. The consumer must not mutate Docs MCP directly.

## Memory

Read `.agent-instructions/x4/MEMORY.md` before project-memory use. The main
agent owns recall and capture for the top-level task. Specialists and
subagents receive filtered context and do not independently query or mutate
X4 project memory.

The shared contract intentionally does not select GSD, OpenSpec, or another
development framework. Follow the current repository's local workflow.
