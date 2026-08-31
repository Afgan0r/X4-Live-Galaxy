---
schema_version: 1
open_count: 11
waived_count: 0
fixed_count: 0
total_count: 11
last_updated: 2026-08-31T21:28:12.451Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 01 | stub | extensions/live_galaxy/md/live_galaxy_observation.xml | 3 | The MD scheduler is intentionally an empty shell until a disposable X4 runtime probe proves the exact event syntax and cadence. | open |  | 2026-08-28T18:18:35.175Z |  |
| 2 | 01 | unrun-verify | extensions/live_galaxy/tests/telemetry_contract.lua |  | Pure Lua contract execution remains pending a compatible X4-evidenced runner. | open |  | 2026-08-28T19:17:05.468Z |  |
| 3 | 01 | unrun-verify | extensions/live_galaxy/tests/scheduler_contract.lua |  | Pure Lua scheduler contract execution remains pending a compatible X4-evidenced runner. | open |  | 2026-08-28T19:19:05.974Z |  |
| 4 | 05.1 | unrun-verify | extensions/live_galaxy/tests/telemetry_contract.lua | 32 | Full Lua contract runner fails in unchanged telemetry_contract.lua:32; focused plan checks pass. | open |  | 2026-08-29T15:51:46.129Z |  |
| 5 | 05.2 | deviation | extensions/live_galaxy/tests/run_contracts.ps1 |  | Registered the focused candidate suite during Task 1 GREEN so the mandated tracer verify was runnable. | open |  | 2026-08-30T00:07:54.028Z |  |
| 6 | 05.2 | deviation | tools/x4-verification/contracts/runtime-evidence.v1.json |  | Added closed privacy-safe failure reasons required by the evidence threat contract. | open |  | 2026-08-30T00:07:54.420Z |  |
| 7 | 05.2 | deviation | .planning/ROADMAP.md |  | Corrected roadmap progress after PLAN-CHECK was miscounted as an executable plan. | open |  | 2026-08-30T00:07:54.818Z |  |
| 8 | 05.2 | deviation | tools/x4-verification/tests/candidate_build_contract.ps1 |  | Normalized one-item PowerShell validation collections before count comparisons. | open |  | 2026-08-30T00:28:28.330Z |  |
| 9 | 05.2 | deviation | tools/x4-verification/build-candidate-extension.ps1 |  | Summed generated-file bytes explicitly for ordered dictionary entries. | open |  | 2026-08-30T00:28:28.726Z |  |
| 10 | 05.1 | deviation | crates/observation-ingest/src/generation.rs |  | Explicit resume generation was added to preserve restart replay and stale-generation rejection. | open |  | 2026-08-31T21:28:12.067Z |  |
| 11 | 05.1 | deviation | crates/observation-ingest/tests/batch_bounds.rs |  | Task 2 began green because the Task 1 tracer already supplied streamed legacy-bound isolation. | open |  | 2026-08-31T21:28:12.451Z |  |

````json
[
  {
    "id": 1,
    "kind": "stub",
    "phase": "01",
    "file": "extensions/live_galaxy/md/live_galaxy_observation.xml",
    "line": 3,
    "description": "The MD scheduler is intentionally an empty shell until a disposable X4 runtime probe proves the exact event syntax and cadence.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-28T18:18:35.175Z",
    "resolved_at": null
  },
  {
    "id": 2,
    "kind": "unrun-verify",
    "phase": "01",
    "file": "extensions/live_galaxy/tests/telemetry_contract.lua",
    "line": null,
    "description": "Pure Lua contract execution remains pending a compatible X4-evidenced runner.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-28T19:17:05.468Z",
    "resolved_at": null
  },
  {
    "id": 3,
    "kind": "unrun-verify",
    "phase": "01",
    "file": "extensions/live_galaxy/tests/scheduler_contract.lua",
    "line": null,
    "description": "Pure Lua scheduler contract execution remains pending a compatible X4-evidenced runner.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-28T19:19:05.974Z",
    "resolved_at": null
  },
  {
    "id": 4,
    "kind": "unrun-verify",
    "phase": "05.1",
    "file": "extensions/live_galaxy/tests/telemetry_contract.lua",
    "line": 32,
    "description": "Full Lua contract runner fails in unchanged telemetry_contract.lua:32; focused plan checks pass.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-29T15:51:46.129Z",
    "resolved_at": null
  },
  {
    "id": 5,
    "kind": "deviation",
    "phase": "05.2",
    "file": "extensions/live_galaxy/tests/run_contracts.ps1",
    "line": null,
    "description": "Registered the focused candidate suite during Task 1 GREEN so the mandated tracer verify was runnable.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-30T00:07:54.028Z",
    "resolved_at": null
  },
  {
    "id": 6,
    "kind": "deviation",
    "phase": "05.2",
    "file": "tools/x4-verification/contracts/runtime-evidence.v1.json",
    "line": null,
    "description": "Added closed privacy-safe failure reasons required by the evidence threat contract.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-30T00:07:54.420Z",
    "resolved_at": null
  },
  {
    "id": 7,
    "kind": "deviation",
    "phase": "05.2",
    "file": ".planning/ROADMAP.md",
    "line": null,
    "description": "Corrected roadmap progress after PLAN-CHECK was miscounted as an executable plan.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-30T00:07:54.818Z",
    "resolved_at": null
  },
  {
    "id": 8,
    "kind": "deviation",
    "phase": "05.2",
    "file": "tools/x4-verification/tests/candidate_build_contract.ps1",
    "line": null,
    "description": "Normalized one-item PowerShell validation collections before count comparisons.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-30T00:28:28.330Z",
    "resolved_at": null
  },
  {
    "id": 9,
    "kind": "deviation",
    "phase": "05.2",
    "file": "tools/x4-verification/build-candidate-extension.ps1",
    "line": null,
    "description": "Summed generated-file bytes explicitly for ordered dictionary entries.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-30T00:28:28.726Z",
    "resolved_at": null
  },
  {
    "id": 10,
    "kind": "deviation",
    "phase": "05.1",
    "file": "crates/observation-ingest/src/generation.rs",
    "line": null,
    "description": "Explicit resume generation was added to preserve restart replay and stale-generation rejection.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T21:28:12.067Z",
    "resolved_at": null
  },
  {
    "id": 11,
    "kind": "deviation",
    "phase": "05.1",
    "file": "crates/observation-ingest/tests/batch_bounds.rs",
    "line": null,
    "description": "Task 2 began green because the Task 1 tracer already supplied streamed legacy-bound isolation.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T21:28:12.451Z",
    "resolved_at": null
  }
]
````
