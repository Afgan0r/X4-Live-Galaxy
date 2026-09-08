---
status: resolved
trigger: "Protected UI was already disabled; find why Carrier B remains peer-absent in the first real X4 run."
created: 2026-09-08
updated: 2026-09-08T21:55:00+07:00
---

# Debug Session: Carrier B Peer Absent

## Symptoms

- Expected: the installed Phase 05.4 Carrier B candidate connects to the already-running bridge and commits one revision within 35 real seconds after a new disposable Creative Custom campaign becomes controllable.
- Actual: `operational-status.json` remains `waiting / peer-absent`; bridge stdout, stderr, and operational history remain empty.
- Errors: no bridge-side error was emitted.
- Timeline: first real X4 run of the frozen candidate; Protected UI was already disabled before launch.
- Reproduction: start the verified bridge, launch X4 with Live Galaxy and required Mod Support APIs enabled, enter a new disposable Creative Custom campaign, and wait more than 35 seconds.

## Current Focus

- bug_class: bohrbug
- reasoning_checkpoint:
    hypothesis: "Slash-qualified local production imports stop X4 runtime initialization because the X4 9.00 UI loader admits extension-qualified dotted module names but does not expose the standalone runner's permissive `extensions/?.lua` search path."
    confirming_evidence:
      - "Two real X4 launches from different extension roots produced the same exact module-not-found failure before `carrier.new`."
      - "Repository contracts inject `extensions/?.lua`, and all five production/transitive local import call sites use slash-qualified names."
    falsification_test: "A loader-shaped test that admits dotted extension modules, rejects slash-qualified names, and omits `extensions/?.lua` would pass unchanged production source."
    fix_rationale: "Changing only local imports to the extension-qualified dotted form aligns production with the observed loader and leaves producer, transport, and game behavior unchanged."
    blind_spots: "Only a human-operated X4 retry can prove the real loader proceeds through native initialization and bridge connection; local tests cannot close that boundary."
    candidate_causes:
      - "code: production and transitive Lua modules use slash-qualified local `require` names."
      - "environment: the standalone test runner injects a search path absent from the actual X4 loader."
      - "config: extension installation root was tested and eliminated."
    and_gate: "yes — the defect escaped and manifested through the combination of wrong production import syntax and a permissive test-only loader surface; the product failure itself is caused by the import mismatch."
- hypothesis: confirmed — production uses slash-qualified local Lua imports that the real X4 9.00 UI loader does not resolve; the standalone test runner injected an `extensions/?.lua` path that masked this mismatch.
- test: complete — target loader, package conformance, aggregate contracts, package self-test, actual local DLL/bridge/SQLite scenarios, full Rust workspace, Clippy, format, and source-size all pass.
- expecting: human-operated X4 retry loads the rebuilt candidate, initializes native Carrier B, and connects to the bridge before B1 collection.
- next_action: install manifest `357b5b38c279efc10764ab1a772434928c748e8876a9a2ee76787ea7ba537c87` and repeat B1; this remains human evidence.

## Evidence

- timestamp: 2026-09-08T21:24:10+07:00
  source: `pwsh -NoProfile -File extensions/live_galaxy/tests/run_contracts.ps1 -Suite loader`
  finding: "RED: 0 successes / 0 failures / 1 error. `live_galaxy_runtime.lua:3` requested `live_galaxy/lua/live_galaxy_carrier`; the loader-shaped contract rejected it with `X4 loader requires an extension-qualified module name`."
  implication: the agent-authored regression reproduces the original defect without X4 and fails for the exact production import mismatch, not for compilation or empty discovery.

- timestamp: 2026-09-08T21:27:30+07:00
  source: focused loader and package-conformance checks after the production edit
  finding: "GREEN: the unchanged loader contract reports 1 success / 0 failures / 0 errors; package conformance reports 4 well-formed XML files and 7 rejection checks passed."
  implication: dotted extension-qualified imports load all nine production modules on the restricted surface, and the package contract now rejects the escaped slash-import class.

- timestamp: 2026-09-08T21:30:00+07:00
  source: bounded manual revert-and-reconfirm of only the five production import edits
  finding: reverting the dotted imports reproduced the exact 0-success/1-error module-not-found RED; reapplying them restored 1 success with zero failures or errors.
  implication: the import correction itself, rather than the new test harness or package assertion, removes the reproduced defect.

- timestamp: 2026-09-08T20:44:22+07:00
  source: `C:/Users/pavlo/Documents/Egosoft/X4/128881100/debug.log`, lines 111-119
  finding: X4 reports `Error while executing the Lua script 'extensions/live_galaxy/lua/live_galaxy_runtime.lua' for addon 'live_galaxy': module 'live_galaxy/lua/live_galaxy_carrier' not found`; the reported search then names only `ui\\core\\lualibs\\live_galaxy/lua/live_galaxy_carrier_64.dll`.
  implication: the load/connect chain stops while `live_galaxy_runtime.lua` resolves its Carrier module, before Carrier can open the bridge pipe. The exact production `require` and `ui.xml` module-loading contract are now the leading boundary.

- timestamp: 2026-09-08T20:50:26+07:00
  source: repository source and `C:/Users/pavlo/Documents/Egosoft/X4/128881100/extensions/live_galaxy`
  finding: the installed candidate exactly matches the repository `ui.xml`, `live_galaxy_runtime.lua`, and `live_galaxy_carrier.lua`; it is installed only in the per-user X4 profile, while `F:/SteamLibrary/steamapps/common/X4 Foundations/extensions/live_galaxy` is absent. The entrypoint uses `require("live_galaxy/lua/live_galaxy_carrier")`; the carrier wrapper later uses `.\\extensions\\live_galaxy\\ui_c_library_live_galaxy_carrier_64.txt`.
  implication: content mismatch is not the failure. Installation root is a candidate common cause for both the failed nested Lua resolution and the carrier DLL path, and must be tested before changing import syntax.

- timestamp: 2026-09-08T21:02:00+07:00
  source: `tests/x4-disposable/01-probe-evidence.md`, `tests/x4-disposable/01-probe-procedure.md`, `05.4-03-PLAN.md`, and the live X4 process/log
  finding: the slash-qualified imports intentionally rely on the game-root `extensions/?.lua` search root; the standalone contract runner explicitly injects that same search path. The Phase 05.4 human procedure instead said "user extensions directory", and the candidate's native path is explicitly `.\\extensions\\live_galaxy\\ui_c_library_live_galaxy_carrier_64.txt`. X4 PID 51212 remained live during diagnosis.
  implication: the procedure selected an incompatible deployment root, while local tests reproduced the game-root resolver and therefore could not catch that deployment instruction error.

- timestamp: 2026-09-08T21:08:30+07:00
  source: second real X4 launch from game-root installation and `C:/Users/pavlo/Documents/Egosoft/X4/128881100/debug.log`, lines 109-114
  finding: after relocating the unchanged, hash-verified candidate to the game-root extension directory, X4 emits the identical `module 'live_galaxy/lua/live_galaxy_carrier' not found` error and searches only `package.preload` plus `ui/core/lualibs/..._64.dll`.
  implication: installation root is eliminated. The slash-qualified production import itself is incompatible with the observed X4 loader; the bridge cannot connect because runtime execution stops before `carrier.new` and `package.loadlib`.


## Eliminated

- hypothesis: Protected UI was enabled.
  reason: user confirmed it was disabled before launch.

- hypothesis: the per-user extension root alone caused the import failure.
  reason: the exact candidate produced the identical loader error after relocation to the X4 game-root extension directory.

## Resolution

- root_cause: `live_galaxy_runtime.lua` and transitive modules use slash-qualified local imports. The actual X4 9.00 UI loader does not provide the standalone runner's injected `extensions/?.lua` lookup, so runtime initialization stops before the native module and pipe open.
- fix: use X4's extension-qualified dotted local module names consistently and add a loader contract that reproduces the real search surface instead of injecting the slash lookup. The human procedure must also name the required game-root installation explicitly because the native `.txt` path is game-root-relative.
- oracle_type: specified — the project X4 integration contract requires extension-qualified production `require` paths, and the regression asserts dotted names load while slash-qualified local names do not.
- tdd_checkpoint:
    test_file: extensions/live_galaxy/tests/x4_loader_contract.lua
    test_name: loads every production module without a slash search path
    status: green
    failure_output: "live_galaxy_runtime.lua:3: module 'live_galaxy/lua/live_galaxy_carrier' not found: X4 loader requires an extension-qualified module name"
- verification:
    target_test: { result: pass }
    mutation_check: { result: skipped, reason_if_skipped: "No configured Lua mutation runner for import-string replacement; the unchanged driving loader contract plus revert-and-reconfirm directly exercises the fix site.", mutant_killed: null }
    no_op_deletion: { result: pass, deletion_justified_by_rca: false }
    adjacent_tests: { result: pass, suites_run: ["aggregate 50 Lua / 9 syntax", "XML/package", "persistence", "three actual local scenarios", "full Rust workspace", "Clippy", "format", "source-size"] }
    revert_and_reconfirm: { result: pass, bug_returned_on_revert: true, fixed_on_reapply: true }
    guardrail_verdict: passed; independent re-review CLEAN and focused security audit SECURED 5/5
- files_changed: five production import call sites across `live_galaxy_runtime.lua`, `live_galaxy_telemetry.lua`, and `live_galaxy_x4_discovery.lua`; loader/local/package tests; packaging startup guidance; Phase 05.4 procedure, evidence, review, security, verification, summary, and this debug record.
