# XEN and KHK Static Evidence Register

**Status:** Versioned static-evidence skeleton; no runtime observation is asserted.

This record isolates installed X4 9.00 configuration and repository contract
evidence from future runtime probes. It is read-only, non-gating, and never
selects hostile autonomy, architecture, write primitives, or control channels.

```json hostile-claim-register
{
  "schema_version": "1.0",
  "source_boundary": "static X4 9.00 and repository contracts only",
  "scope": {
    "xen_primary_pressure": true,
    "khk_observed_when_present": true,
    "autonomous_hostile_minds": false,
    "government_institutions": false,
    "hostile_motives": false,
    "hostile_diplomacy": false,
    "hostile_architecture_selected": false,
    "hostile_write_primitives": false,
    "hostile_control_channels": false,
    "critical_path_dependency": false,
    "phase8_inventory_only": true
  },
  "coverage": {
    "requirements": ["RES-01", "RES-02", "RES-03"],
    "decisions": ["D-01", "D-02", "D-03", "D-04", "D-05", "D-06", "D-07", "D-08"]
  },
  "sources": [
    {"id":"x4-version-dat","kind":"installed_x4_file","path":"version.dat","boundary":"installed X4 9.00","allowed_conclusions":["installed_version"]},
    {"id":"x4-jobs","kind":"installed_x4_file","path":"08.cat::libraries/jobs.xml","boundary":"installed X4 9.00 catalog","allowed_conclusions":["xen_job_configuration","xen_patrol_configuration"]},
    {"id":"x4-khaak-activity","kind":"installed_x4_file","path":"08.cat::md/khaak_activity.xml","boundary":"installed X4 9.00 catalog","allowed_conclusions":["khk_activity_configuration","khk_spawn_configuration"]},
    {"id":"khaakfinder-precedent","kind":"installed_extension_precedent","path":"extensions/z_ram_khaakfinder","boundary":"installed extension v101","allowed_conclusions":["visibility_precedent"]},
    {"id":"project-contract","kind":"repository_contract","path":"AGENTS.md|.planning/PROJECT.md|.planning/REQUIREMENTS.md|.planning/ROADMAP.md|02-CONTEXT.md","boundary":"repository planning","allowed_conclusions":["observation_only_scope","runtime_contract_unknown","non_gating_deferral"]}
  ],
  "claims": [
    {"id":"XEN-STATE-01","faction":"XEN","area":"state","classification":"observed","source_id":"x4-jobs","permitted_conclusion":"xen_job_configuration","non_gating":true,"future_owner":"Phase 1","evidence_needed":"disposable X4 9.00 runtime readback"},
    {"id":"XEN-EVENTS-01","faction":"XEN","area":"events","classification":"unknown","source_id":"project-contract","permitted_conclusion":"runtime_contract_unknown","non_gating":true,"future_owner":"Phase 1 and Phase 7 X4 validation","evidence_needed":"attributable disposable X4 9.00 event export and independent readback"},
    {"id":"XEN-IDENTITY-01","faction":"XEN","area":"identity","classification":"observed","source_id":"x4-jobs","permitted_conclusion":"xen_job_configuration","non_gating":true,"future_owner":"Phase 1","evidence_needed":"runtime identity readback"},
    {"id":"XEN-VISIBILITY-01","faction":"XEN","area":"visibility","classification":"unknown","source_id":"project-contract","permitted_conclusion":"runtime_contract_unknown","non_gating":true,"future_owner":"Phase 3 information-boundary work","evidence_needed":"authoritative faction visibility observation and policy test"},
    {"id":"XEN-SCHEDULING-01","faction":"XEN","area":"scheduling","classification":"observed","source_id":"x4-jobs","permitted_conclusion":"xen_patrol_configuration","non_gating":true,"future_owner":"Phase 1","evidence_needed":"runtime cadence observation"},
    {"id":"XEN-ECONOMY-01","faction":"XEN","area":"economy_or_spawning_ownership","classification":"observed","source_id":"x4-jobs","permitted_conclusion":"xen_job_configuration","non_gating":true,"future_owner":"Phase 1","evidence_needed":"authoritative job and asset readback"},
    {"id":"XEN-CONTROL-01","faction":"XEN","area":"control_limits","classification":"documented","source_id":"project-contract","permitted_conclusion":"observation_only_scope","non_gating":true,"future_owner":"later milestone product discussion","evidence_needed":"explicit future scope decision"},
    {"id":"KHK-STATE-01","faction":"KHK","area":"state","classification":"observed","source_id":"x4-khaak-activity","permitted_conclusion":"khk_activity_configuration","non_gating":true,"future_owner":"Phase 1","evidence_needed":"disposable X4 9.00 runtime readback"},
    {"id":"KHK-EVENTS-01","faction":"KHK","area":"events","classification":"unknown","source_id":"project-contract","permitted_conclusion":"runtime_contract_unknown","non_gating":true,"future_owner":"later hostile-design research","evidence_needed":"KHK activity, quota, and spawn transition readback without MD internals"},
    {"id":"KHK-IDENTITY-01","faction":"KHK","area":"identity","classification":"observed","source_id":"x4-khaak-activity","permitted_conclusion":"khk_spawn_configuration","non_gating":true,"future_owner":"Phase 1","evidence_needed":"authoritative owner and station identity readback"},
    {"id":"KHK-VISIBILITY-01","faction":"KHK","area":"visibility","classification":"unknown","source_id":"project-contract","permitted_conclusion":"non_gating_deferral","non_gating":true,"future_owner":"later compatibility research","evidence_needed":"versioned extension interaction inventory and disposable scenario"},
    {"id":"KHK-SCHEDULING-01","faction":"KHK","area":"scheduling","classification":"observed","source_id":"x4-khaak-activity","permitted_conclusion":"khk_activity_configuration","non_gating":true,"future_owner":"Phase 1","evidence_needed":"runtime activity interval observation"},
    {"id":"KHK-ECONOMY-01","faction":"KHK","area":"economy_or_spawning_ownership","classification":"observed","source_id":"x4-khaak-activity","permitted_conclusion":"khk_spawn_configuration","non_gating":true,"future_owner":"Phase 1","evidence_needed":"authoritative hive, outpost, and quota readback"},
    {"id":"KHK-CONTROL-01","faction":"KHK","area":"control_limits","classification":"documented","source_id":"project-contract","permitted_conclusion":"observation_only_scope","non_gating":true,"future_owner":"later milestone product discussion","evidence_needed":"explicit future scope decision"}
  ]
}
```

## Boundary

Static catalog and extension evidence describes configured mechanisms or a
read-only precedent. It does not prove a live entity, visibility state, event
stream, or externally supported control capability. Runtime gaps stay owned by
later disposable X4 probes and cannot block the ZYA/ARG Shadow Director path.

## Permitted conclusions

- XEN static jobs show configured mining, energy, and patrol mechanisms; X4
  retains economy, replenishment, and scheduling ownership.
- KHK static Mission Director data shows activity-driven hive and outpost
  configuration; it does not establish an exported runtime contract.
- Runtime event exports, faction visibility, KHK quota observability, and
  extension interaction remain owned, non-gating unknowns.
- Phase 8 may inventory this evidence only. It cannot elevate any row into a
  hostile architecture or game-control decision.
