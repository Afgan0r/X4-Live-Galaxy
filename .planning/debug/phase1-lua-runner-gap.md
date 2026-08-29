---
status: diagnosed
trigger: "Diagnose the Phase 1 local pure-Lua runner gap only: run_contracts.ps1 cannot find lua or luajit."
created: 2026-08-29T00:00:00+07:00
updated: 2026-08-29T00:20:00+07:00
---

## Current Focus

bug_class: Bohrbug (deterministic environment/configuration gap)
hypothesis: "The pure-Lua contracts cannot start because run_contracts.ps1 resolves only a PATH command named lua, while the repository contains no pinned project-local interpreter or provisioner."
test: "Run the specified suite and inspect its executable resolution, repository tooling manifests, and Phase 1 validation prerequisites."
expecting: "The runner will throw before dofile() and no Lua test case will execute; the Phase validation contract will show runner selection was deferred until an X4 syntax probe."
next_action: "Return the diagnosed plan inconsistency and a bounded provisioner design; do not install, download, or edit production files."
reasoning_checkpoint:
  hypothesis: "Plan 01-10's local contract command is blocked because its runner requires a globally discoverable lua executable, but neither the project nor its toolchain declares or provisions one; selecting a pinned version is additionally gated on an unperformed X4 runtime syntax probe."
  confirming_evidence:
    - "The exact prescribed command throws at run_contracts.ps1:9 before loading the requested Lua suite."
    - "run_contracts.ps1 calls Get-Command lua only and has no project-local path, parameter, lock file, bootstrap, or luajit fallback."
    - "01-VALIDATION.md and 01-RESEARCH.md explicitly defer the pure Lua runner until an X4 runtime syntax probe confirms compatibility."
  falsification_test: "A tracked project-local interpreter/provisioner selected after a recorded X4 syntax probe, with run_contracts.ps1 resolving it and the suite reaching a Lua assertion, would disprove this root cause."
  fix_rationale: "Amending the plan to make the syntax probe and a pinned, verified project-local runner an explicit prerequisite creates executable fake-adapter evidence without changing X4 runtime, transport, or production authority."
  blind_spots: "The embedded X4 Lua version and accepted standalone compatibility range are still unobserved; official Lua provides source archives, so the Windows binary build/provenance path needs an approved implementation choice."
  candidate_causes:
    - "code: the runner's Get-Command lua-only lookup cannot discover a project-local tool."
    - "config: no lock/provisioner or documented Lua version/path exists in repository manifests or readmes."
    - "environment: no lua or luajit command is installed in the current execution environment."
  and_gate: "yes — the visible failure requires both an absent interpreter in the environment and a runner/configuration that supplies no project-local alternative; the plan inconsistency prevents choosing a compliant version."

## Symptoms

expected: "The x4_discovery pure fake-adapter contract suite executes locally."
actual: "extensions/live_galaxy/tests/run_contracts.ps1 -Suite x4_discovery fails because neither lua nor luajit is available."
errors: "Lua interpreter executable not found (lua/luajit unavailable)."
reproduction: "Run powershell -File extensions/live_galaxy/tests/run_contracts.ps1 -Suite x4_discovery."
started: "Phase 1 Plan 01-10 Tasks 1-2 implementation environment."

## Eliminated

<!-- APPEND ONLY -->

## Evidence

- timestamp: 2026-08-29T00:00:00+07:00
  checked: "git status and remote freshness"
  found: "Working tree contains substantial pre-existing Phase 1 changes; git fetch could not write .git/FETCH_HEAD due to sandbox permission."
  implication: "Preserve all existing changes and treat local checked-out files as the evidence source; remote refs could not be refreshed in this sandbox."

- timestamp: 2026-08-29T00:10:00+07:00
  checked: "ast-index readiness and extension test topology"
  found: "The project index was present (126 files) and updated; x4_discovery_contract.lua imports discovery, telemetry, and runtime modules, but its execution is delegated entirely to run_contracts.ps1."
  implication: "The missing executable is an external runner boundary, not a missing Lua test file or unresolved project module."

- timestamp: 2026-08-29T00:12:00+07:00
  checked: "The exact Plan 01-10 automated command and shell command resolution"
  found: "powershell -NoProfile -File extensions/live_galaxy/tests/run_contracts.ps1 -Suite x4_discovery failed deterministically at line 9 with 'Lua runner unavailable'; Get-Command found neither lua nor luajit."
  implication: "No contract case executes, so the claimed pure/fake-adapter proof is currently unavailable."

- timestamp: 2026-08-29T00:14:00+07:00
  checked: "extensions/live_galaxy/tests/run_contracts.ps1"
  found: "The script resolves only Get-Command lua, then invokes lua -e dofile(...). It has no parameter/environment override, no project-local lookup, no luajit fallback, and no bootstrap or checksum verification."
  implication: "The runner has an undeclared machine-global dependency and cannot provide reproducible local proof."

- timestamp: 2026-08-29T00:16:00+07:00
  checked: "repository manifests/readmes and Phase 1 validation/research"
  found: "Cargo.toml and README.md declare no Lua tooling; 01-VALIDATION.md requires an X4 runtime Lua syntax probe before runner selection, and 01-RESEARCH.md records Lua and Busted as absent/deferred."
  implication: "Plan 01-10 requires executable pure-Lua contracts without adding the prerequisite that its own phase validation contract requires."

- timestamp: 2026-08-29T00:18:00+07:00
  checked: "official Lua release and distribution documentation"
  found: "Lua.org lists current source releases (5.5.1 and final 5.4.9) with SHA-256 values and states official Lua is distributed in source form; it does not make an official Windows interpreter binary the project can simply pin."
  implication: "A compliant Windows runner must either build a source-pinned Lua release with an available C toolchain or use a separately approved, license/provenance-reviewed prebuilt binary; the X4 syntax probe must select the compatible major/minor first."

## Resolution

root_cause: "Confirmed: Plan 01-10's executable pure-Lua contract requirement has no provisioned interpreter. run_contracts.ps1 depends solely on a machine-global lua command, which is absent, while Phase 1 separately requires an X4 runtime syntax probe before a compatible standalone Lua version may be selected."
fix: "Diagnose-only: correct Plan 01-10/validation sequencing to add a prerequisite X4 syntax/version probe and a pinned project-local Lua provisioner. Then make run_contracts.ps1 resolve that tool (with an explicit override for CI) before any PATH fallback."
verification: "Before accepting: record the X4 Lua version/syntax probe; verify the archive hash and interpreter --version; run x4_discovery through the project-local executable; then run all pure contract suites. Keep all results classified verified locally, not observed in X4."
files_changed: []

## Root-Cause Report

### Claim classification

| Claim | Classification | Evidence |
| --- | --- | --- |
| The prescribed x4_discovery command cannot execute a Lua case. | Observed | PowerShell run fails at `run_contracts.ps1:9` before `dofile()`. |
| The runner depends only on a globally resolved `lua`. | Documented | `extensions/live_galaxy/tests/run_contracts.ps1`. |
| Neither `lua` nor `luajit` is available in this environment. | Observed | `Get-Command lua,luajit` returned no commands. |
| A compatible standalone version has not been selected. | Documented | `01-VALIDATION.md` and `01-RESEARCH.md` defer selection until an X4 syntax probe. |
| Lua.org provides a source archive and checksum, not an official Windows executable. | Documented | Lua.org download and version-history pages. |

### Smallest compliant repair

This needs a **plan correction**, not a silent bounded execution repair. The plan's task already requires executable fake contracts, but the phase contract makes version selection conditional on a missing X4 syntax probe. Adding an arbitrary system interpreter would bypass that safety gate and reproduce the current machine-global dependency.

Amend Task 1 before its existing automated verification with one explicit prerequisite:

1. Record a disposable, read-only X4 embedded-Lua version and syntax probe, and select the standalone compatibility target from that observation.
2. Add a project-owned provisioner and lock record. The runner must resolve an explicit `-LuaPath`/`LIVE_GALAXY_LUA`, then the pinned project-local executable, and only then an optional PATH fallback that verifies the selected version.
3. Pin the exact archive URL, SHA-256, archive version, executable version, platform, and source/binary provenance. Official source candidates currently include Lua 5.4.9 (`https://www.lua.org/ftp/lua-5.4.9.tar.gz`, SHA-256 `2335b6c582a52654f94612bf10d2f4672805d05329aa6568b1d8cd9e5c6fb8e6`) and Lua 5.5.1, but neither may be chosen until the X4 probe defines the compatible minor line.
4. Because official Lua is source-only, choose and document exactly one Windows materialization route: source build with a pre-existing approved C toolchain, or a separately approved prebuilt binary with license/provenance review and its own hash. Keep downloaded/built tools in a project-local ignored cache; do not commit an unreviewed binary.

Acceptance verification: provisioner checksum passes, `lua -v` equals the lock, x4_discovery and all contract suites reach Lua assertions, and the syntax-probe compatibility record is attached. This creates local/fake-adapter proof only; it does not satisfy the Task 3 in-game gate.

Skill learning: none — the X4 test skill already requires confirming embedded Lua constraints before choosing a standalone runner, and the phase validation contract already encodes the missing probe. Evidence shows an execution/plan-sequencing lapse, not a missing reusable project rule.
