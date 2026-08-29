---
schema_version: 1
open_count: 4
waived_count: 0
fixed_count: 0
total_count: 4
last_updated: 2026-08-29T15:51:46.129Z
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
  }
]
````
