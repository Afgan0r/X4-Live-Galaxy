---
schema_version: 1
open_count: 1
waived_count: 0
fixed_count: 0
total_count: 1
last_updated: 2026-08-28T18:18:35.175Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 01 | stub | extensions/live_galaxy/md/live_galaxy_observation.xml | 3 | The MD scheduler is intentionally an empty shell until a disposable X4 runtime probe proves the exact event syntax and cadence. | open |  | 2026-08-28T18:18:35.175Z |  |

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
  }
]
````
