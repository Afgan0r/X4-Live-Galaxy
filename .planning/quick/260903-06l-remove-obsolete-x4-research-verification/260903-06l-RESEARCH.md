# Quick 260903-06l: Retire obsolete X4 research verification — Research

**Researched:** 2026-09-03. **Scope:** repository-only test infrastructure.

<user_constraints>

## User Constraints

Verbatim from the quick CONTEXT.md; these owner decisions govern the recommendations below.

<!-- DATA_92bce7a1_START -->

### Locked Owner Decisions

- The owner rejected repeated 15-20 minute feedback loops after small edits and
  explicitly accepted removing the research platform: Live Galaxy should test
  Live Galaxy.
- Remove obsolete research infrastructure instead of merely hiding it behind a
  slower optional suite or optimizing its current execution.
- X4 API knowledge research belongs to Docs MCP. Do not migrate the retired
  framework into that repository as part of this task.
- Retain product Lua, fake-adapter, native binding, package, persistence, and Rust
  bridge checks. Preserve committed Plan 09 RED contracts and their paused status.
- Extract a lean check of actual extension packaging and Lua imports before
  deleting the old dossier-bound package-conformance implementation.
- Preserve useful historical phase records and evidence limitations. Do not
  silently mark unverified game behavior or unfinished requirements complete.
- Research, plan checking, code review, and final verification are required.
  Present the research and checked plan to the owner before implementation.
- Do not launch X4, read saves, change installed extensions, or modify siblings.

### Technical Discretion

- Choose the smallest existing-product test entry points and names that preserve
  active callers. Keep `run_contracts.ps1 -Suite x4_discovery` compatible.
- Remove candidate-only suites and tests together with their implementation.
  Identify stale active instructions and documentation references in the plan.
- Use focused checks during edits and a single final product regression. Record
  exact commands, exit status, and elapsed time; do not invent a performance
  threshold or rerun the obsolete full gate just to establish another baseline.
- Keep runtime code unchanged unless research identifies a concrete dependency
  that must return to the owner as a scope conflict.
- Format only owned files. Preserve all pre-existing worktree changes.

<!-- DATA_92bce7a1_END -->

No separate deferred-ideas section exists; the task boundary explicitly excludes successor-phase implementation. [CITED: 260903-06l-CONTEXT.md, Task Boundary]

</user_constraints>

## Summary

Extract the existing lexical import traversal into one product package checker with a focused fixture harness; remove the research island and its runner wiring afterward. Do not relocate the authority framework under a new name. The current validator mixes genuine manifest/import checks with mandatory dossier/coverage admission and digests. [VERIFIED: tools/x4-verification/x4-package-conformance.ps1:418-487]

The similarly named component package guard only checks a fixed file allowlist and forbidden authority vocabulary; it cannot replace manifest or import validation. Preserve it separately. [VERIFIED: scripts/component_discovery_package_guard.ps1:27-40,74-91]

## Architectural Responsibility Map

| Capability | Recommended owner | Boundary |
| --- | --- | --- |
| Manifest, registration, import closure | Product test tooling | Read repository package; never run X4 |
| Policy, telemetry, scheduling | Existing Lua contracts | In-process fakes remain product evidence |
| Persistence and ingestion | Existing schema/Rust contracts | Preserve paused RED evidence |
| Game API research | Docs MCP | Shared contract, not a local candidate platform |

The table applies the owner boundary above and shared contract Repository Routing. [CITED: .agent-instructions/x4/AGENTS.md]

## Standard Stack and Architecture Patterns

Use existing PowerShell, locked Lua, and Cargo; install nothing. Local probes found PowerShell 7.6.4 and Cargo 1.97.1. The Lua lock quotes `"executableVersion": "Lua 5.1.5"`; executable availability was not re-probed because this research must not run tests. [VERIFIED: tools/lua-runner.lock.json:5; local version probes]

Recommended extraction:

1. Keep XML identity/dependency/registration checks, entrypoint traversal, exact-case contained path resolution, comment/string-aware import analysis, and native acquisition checks. Reuse the existing parser rather than replacing it with a regex. [VERIFIED: tools/x4-verification/x4-package-conformance.ps1:83-386,430-482]
2. Use stock file reads after ordinary existence, path containment, reparse-point, and file-size checks. Keep finite traversal safeguards. Do not copy native handle identity, concurrent-tamper protection, certificates, graph/dossier digests, evidence classes, or admission inputs. The old reader actually compiles native interop (`Add-Type -TypeDefinition`); package correctness does not require migrating that research threat model. [VERIFIED: tools/x4-verification/bounded-file.psm1:3-66; recommendation within locked extraction scope]
3. Keep one callable checker and one fixture harness beside product tests. Retain the existing suite name `x4-package-conformance` for the lean check; return a simple result/exit code and actionable relative-path failure. This suite already exists in the runner's `ValidateSet`. [VERIFIED: extensions/live_galaxy/tests/run_contracts.ps1:1-4]
4. Remove the old harness's aggregate self-invocation. It explicitly launches `x4-package-conformance`, then `x4-verification`, then `all`; copying this harness wholesale recreates the slow dependency. [VERIFIED: tools/x4-verification/tests/package_conformance_contract.ps1:313-342]

## Scoped Removal and Retention

| Action | Exact scope |
| --- | --- |
| Delete after extraction | Entire `tools/x4-verification/` and `tests/x4-candidates/`, including contracts, fixtures, certificates/anchors, attestation, retention, workers, builder, templates, candidate Lua, and obsolete run procedure |
| Delete candidate-only test entrypoints | `extensions/live_galaxy/tests/x4_candidate_runner_contract.lua` and `x4_candidate_runner_adversarial.lua` |
| Simplify runner | Remove research suites, prepared builds, candidate timing/marker machinery and recursive aggregate paths; preserve locked Lua resolution and nonempty behavior evidence |
| Retain | All production files and Rust tests; four product Lua contracts (component discovery, X4 discovery, telemetry, scheduler); native binding contract; component package guard and wrapper; persistence schema contract; Lua provisioning/lock; disposable product test docs |
| Preserve historical evidence | Existing phase/quick records, HANDOFF, paused Plan 09 tests and successor-phase scope; no bulk removal of historical references |

Deletion roots and entrypoints are the approved CONTEXT scope, confirmed by the focused literal-reference audit; retained runner branches and discovery behavior are directly read. [CITED: 260903-06l-CONTEXT.md, Verified Starting Evidence; extensions/live_galaxy/tests/run_contracts.ps1:381-486]

## Project Constraints and Active Instructions

Update only the X4 integration skill's Integration Admission, Known-Failure Gate, and Verification sections: remove mandatory local dossier/registry/candidate machinery and route unresolved API knowledge through the shared contract. Preserve extension-relative imports, full import closure, product fake tests, provenance, safety, and explicit evidence limitations. These obsolete mandates are active even after deleting scripts. [CITED: .agents/skills/live-galaxy-x4-integration/SKILL.md]

The active documentation search found no other direct obsolete-tool mandate outside this skill and runner. Preserve the skill changelog as history. Add only a concise retirement/current-test-route note to the verification authority if needed; do not rewrite target architecture or ADR-LG-023/024. The latter explicitly separate successor scope, evidence, and delivery state. [CITED: docs/architecture-decisions.md:416-449; scoped literal-reference audit]

Keep runtime unchanged; require checked plan approval, review, and verification; preserve user edits; write English artifacts; do not inspect secrets, saves, installed sources, or siblings. Markdownlint must respect the repository ignore `".planning/**"`. [CITED: AGENTS.md; .agent-instructions/x4/AGENTS.md; .markdownlint-cli2.jsonc]

## Validation and Common Pitfalls

**Package positives:** actual package including transitive imports; static concatenation/helper forms used by current code; comments and quoted decoys cannot become imports or bindings. **Negatives:** wrong/missing registration or entrypoint, missing transitive module, bare/test-only/dynamic import, root escape, cycle, and missing/duplicate native acquisition. Adapt existing fixtures to simple results; retain no candidate verdict or tamper-chain cases. [CITED: tools/x4-verification/tests/package_conformance_contract.ps1:88-301]

**Coverage regression:** make product `all` run every retained Lua contract, the lean package harness, and existing binding/guard/persistence checks exactly once. Today the binding and component guard are selected only by `component_discovery`, while Lua `all` enumerates contract files; persistence is a separate executable contract. This is a genuine product-coverage gap, not a reason to retain research meta-tests. Prove dispatch with one final observed stage/test manifest, child exit propagation, and a negative Lua assertion; reject removed suite names explicitly. [VERIFIED: extensions/live_galaxy/tests/run_contracts.ps1:381-400,457-486; CITED: extensions/live_galaxy/tests/persistence_schema_contract.ps1]

**Proposed execution commands** (not executed in research):

```powershell
pwsh -NoProfile -File extensions/live_galaxy/tests/run_contracts.ps1 -Suite x4-package-conformance
pwsh -NoProfile -File extensions/live_galaxy/tests/run_contracts.ps1 -Suite x4_discovery
pwsh -NoProfile -File extensions/live_galaxy/tests/run_contracts.ps1 -Suite component_discovery
pwsh -NoProfile -File extensions/live_galaxy/tests/run_contracts.ps1 -Suite all
cargo test -p x4-bridge --test named_pipe_contract --locked
cargo test --workspace --locked --no-fail-fast
```

Commands use existing runner selections and Cargo package `name = "x4-bridge"`. Capture named-pipe failures before edits, then compare the final workspace run with that baseline and unchanged Rust diffs; unexpected failures remain regressions. Never skip, ignore, weaken, or implement the paused tests to manufacture green. Exact current failures remain unknown until that focused baseline runs. [VERIFIED: extensions/live_galaxy/tests/run_contracts.ps1:1-4; crates/x4-bridge/Cargo.toml:1-3; CITED: .planning/HANDOFF.json; crates/x4-bridge/tests/named_pipe_contract.rs]

After deletion, repeat a tracked-file literal audit excluding historical planning and the retirement note: no executable inbound reference may survive. Record exact commands, exits, elapsed times and unresolved RED separately. Do not rerun the obsolete gate. [CITED: 260903-06l-CONTEXT.md, Technical Discretion]

## Runtime State Inventory and Open Questions

| Category | Finding and disposition |
| --- | --- |
| Stored data | Private retained evidence may exist; not inspected. Preserve it; no data migration is needed to stop invoking removed tooling. |
| Live service config | No service transition is part of this repository-only task; external configuration was not inspected. |
| OS-registered state | Retention code opens Windows CNG keys; actual registrations are unknown. Do not enumerate/delete them. |
| Secrets/env | Secret files were not read. Keep Lua runner environment selection; no key rotation or rename. |
| Build artifacts | Prepared candidate builds/caches may remain locally; retire their creation in the runner, preserve unrelated machine-local state. |

Source-supported possible state: retention opens `CngKey::Open` through the .NET call shown in source; prepared-build machinery exists in the runner. No claim of absent runtime state is made. [CITED: tools/x4-verification/retain-evidence.ps1:701-714; extensions/live_galaxy/tests/run_contracts.ps1:25-30]

**Remaining uncertainty:** package extraction behavior must be established by the listed fixtures; current RED failure identities must be measured. No runtime dependency requiring owner scope expansion was found. [CITED: focused source analysis; CONTEXT dependency audit]

## Sources and Confidence

All findings above are repository observations or explicitly proposed planning actions; no external API/package claim is introduced. External documentation research and package installation audits are inapplicable. Security enforcement and Nyquist are explicitly disabled in the loaded configuration. [CITED: .planning/config.json, workflow]

The confidence seam returned **LOW** for `query classify-confidence --provider codebase --verified`; this provider fallback does not grade the direct source citations. No behavior was executed, and no test success is claimed. The direct-read evidence identifies the extraction seam; runtime claims remain unknown. [VERIFIED: classifier output in this research session]
