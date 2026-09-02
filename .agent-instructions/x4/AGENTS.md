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

Apply Proportionate Engineering below when assessing local plans, technical
choices, review findings, and verification scope.

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

## Proportionate Engineering

These rules apply to all registered X4 modding repositories, including the
canonical instruction repository. Keep engineering effort proportional to the
current task and its actual use. Many research hypotheses, detailed searchable
logs, and necessary product checks are legitimate complexity; line count or
test count alone does not establish overengineering.

- **Justify additional mechanisms.** Before adding a layer, protocol, generic
  framework, or defensive mechanism, identify the concrete failure in the
  current scenario and explain why the simpler approach is insufficient.
  Speculative future reuse, a merely imaginable attacker, and generic claims
  of reliability are not sufficient reasons. Keep disposable experiments
  specific to their questions rather than building a permanent platform.
- **Keep agent duties with the agent.** Source research, code review,
  application of previous lessons, and honest interpretation of results are
  agent responsibilities. Do not automatically turn them into certificates,
  operator receipts, admission engines, or other project infrastructure.
  An additional mechanism must satisfy the same necessity test.
- **Question plans and review findings.** An agent-authored requirement does
  not become justified merely by appearing in a plan or specification. A
  severity label, including Critical, is not proof that a proposed mechanism
  is needed. Independently assess findings against the agreed behavior and
  concrete evidence; reject unjustified additions without asking the user
  each time, and briefly explain the decision in the existing review record
  or chat. Continue required review and fix substantiated defects.
- **Respect the change boundary.** Ordinary local simplifications within the
  agreed task need no additional approval. Before materially redesigning or
  removing an existing mechanism, or changing an approved approach, explain
  what would change, why, and its consequences, then obtain user approval.
  Do not silently cancel explicit user requirements. Once the user approves
  the scoped change, update its affected plans and implementation without
  repeated permission requests for each file.
- **Verify the affected behavior.** Select checks for the changed behavior
  and affected contracts. Each additional or repeated run must resolve a
  concrete remaining uncertainty or satisfy an applicable required gate.
  Do not repeatedly run full suites after small edits, include unrelated
  suites in an instruction-only change, or build a verification subsystem
  solely to gain confidence in that subsystem. Stop when the agreed result
  has sufficient evidence. Preserve necessary research coverage, product
  checks, and existing evidence and safety boundaries.

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
