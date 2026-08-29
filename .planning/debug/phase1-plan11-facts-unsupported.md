---
status: diagnosed
trigger: "Diagnose only the remaining facts_unsupported result from the live Phase 1 Plan 01-11 v2 retry."
created: 2026-08-29T00:00:00+07:00
updated: 2026-08-29
goal: find_root_cause_only
---

## Current Focus

hypothesis: "Confirmed: `facts_unsupported` aggregates the discovery adapter's completeness predicate, and the one-shot capability vector localizes this attempt's failed member to `sector_capacity`: the selected sector used as the capacity target returns `call_error`."
test: "Correlate the one-shot class-only vector with the accepted bridge frames and compare the failing selected-sector call shape with installed X4 Live's component-local capacity precedent."
expecting: "A `sector_capacity=call_error` with metadata and owner validation `ok` rules out those predicate members for this attempt, but proves neither global API absence nor the valid shape for real-component enumeration."
next_action: "Keep D-09 explicitly unsupported for scope-complete facts. If the owner separately authorizes it, plan a bounded real-component-enumeration investigation; do not enumerate components under Plan 01-11."
bug_class: bohrbug
candidate_causes:
  - "code: adapter import or native-call shape does not match X4's Lua API"
  - "environment: supported API is unavailable in this UI/runtime startup context"
and_gate: "Unknown; test whether both a wrong call shape and runtime availability condition contribute."

## Symptoms

expected: "A live Phase 1 Plan 01-11 v2 retry exports supported normalized discovery facts after bridge transport and restart succeed."
actual: "Lua reports `facts_unsupported` on every discovery cycle while bridge transport and restart are observed."
errors: "facts_unsupported"
reproduction: "Run the existing live Phase 1 Plan 01-11 v2 retry in its established X4 environment; every discovery cycle emits `facts_unsupported`."
started: "Observed during the Plan 01-11 v2 retry."

## Eliminated

<!-- APPEND only - prevents re-investigating -->

## Evidence

- timestamp: 2026-08-29
  checked: "Reporter-provided live retry evidence"
  found: "Bridge transport and restart were observed; only discovery reports `facts_unsupported` every cycle."
  implication: "The diagnosis starts at the discovery capability branch, not bridge connectivity or restart recovery."
- timestamp: 2026-08-29T00:12:00+07:00
  checked: "Plan 01-11, runtime-gap research, current discovery, telemetry, runtime, normalizer, and fake adapter contracts"
  found: "`facts_unsupported` has exactly one production origin: the combined `complete` predicate in `live_galaxy_x4_discovery.lua`. It requires nonempty name and macro, syntactically valid sector/asset/owner identifiers, a successful finite integer `get_people_capacity` call, and a successful nonnegative ware-limit read when cargo is nonempty. The runtime logs only the combined result."
  implication: "The live result proves one member is false but does not reveal which member; current diagnostics cannot identify a single failed capability."
- timestamp: 2026-08-29T00:12:00+07:00
  checked: "Installed X4 Live UI Lua source and Lua LSP document-symbol parse"
  found: "The installed reference calls `C.GetPeopleCapacity(component64, \"\", false)` in its component-local crew snapshot; it calls `GetWareProductionLimit(component, ware)` after `GetComponentData(component, \"cargo\")`; and it receives sector metadata via `GetComponentData(sector, \"name\", \"owner\", \"macro\")`. The Live Galaxy adapter passes its selected sector to all three reads and promotes that sector to a synthetic asset."
  implication: "The capacity call has observed precedent only for a component/asset, not a sector. This is a bounded call-shape/category mismatch candidate; no installed source documents it as a sector-capacity API."
- timestamp: 2026-08-29T00:12:00+07:00
  checked: "Runtime-gap research and installed Mod Support API scope"
  found: "Current primary research records no installed vanilla contract for asset enumeration, capacity semantics, ownership semantics, or bounded combined discovery. The support API documents loading and named pipes, not these discovery calls."
  implication: "This cannot be classified as a genuine API absence; it is an unsupported/unproven use of otherwise observed UI-Lua call shapes until a one-call runtime probe separates return states."
- timestamp: 2026-08-29T00:20:00+07:00
  checked: "Live retry ledger and all current producer error paths"
  found: "The retry has accepted v2 hello, heartbeat, and runtime-health frames across bridge-only restarts, while Lua reports `facts_unsupported` every discovery cycle. No other code path emits that exact reason. A normalizer failure would instead report `runtime_facts_invalid`; transport and bridge failures have separate `pipe_*` and frame dispositions."
  implication: "The first failing boundary is X4 API value/category -> discovery adapter completeness, before telemetry serialization and bridge admission."
- timestamp: 2026-08-29T00:20:00+07:00
  checked: "Static failure predicate coverage"
  found: "The combined predicate can fail only on: absent/non-string `name`; absent/non-string `macro`; invalid `owner` identifier; failed/non-integer/out-of-range `C.GetPeopleCapacity(ConvertIDTo64Bit(sector), \"\", false)`; or, when sector cargo is nonempty, failed/non-numeric/negative `GetWareProductionLimit(sector, ware)`. Sector and synthetic-asset identifier terms are already valid after `read_one_sector` succeeds."
  implication: "No exact member can be honestly selected from the retained evidence. The adapter needs a bounded per-predicate result vector; guessing one would turn an inference into a runtime fact."
- timestamp: 2026-08-29T00:00:00+07:00
  checked: "Reporter-provided bounded X4 capability-vector attempt `obs-x4-d09-capability-20260829-03` and correlated bridge acceptance"
  found: "The X4 debug log recorded `metadata_type=ok`, `owner_id_validity=ok`, `sector_capacity=call_error`, and `first_cargo_ware_limit=not_applicable`; the bridge accepted hello, heartbeat, and runtime-health through sequence 3. The source and installed configuration had the default-off one-shot probe explicitly enabled for this attempt."
  implication: "For this selected-sector cycle, the exact unsupported call shape is the sector-as-capacity-target operation. Metadata type and owner-ID validation are not the failing predicate members, and no-cargo ware handling is non-failure. This does not establish global absence of the capacity API or authorize component enumeration."

## Resolution

root_cause: "The immediate defect is diagnostic aggregation: `live_galaxy_x4_discovery.lua` collapses several X4 metadata and capability predicates into one `facts_unsupported` outcome. The bounded X4 vector now identifies this selected-sector cycle's failing member as `C.GetPeopleCapacity(ConvertIDTo64Bit(sector), \"\", false)` returning `call_error`; installed X4 Live observes that API with a component/asset, so this is an exact unsupported sector-as-capacity-target call shape, not evidence that the API is globally absent."
fix: "No fix applied (diagnose-only). Keep D-09 scope-complete facts explicitly unsupported. A separately authorized, bounded real-component-enumeration plan is required before changing the adapter; Plan 01-11 forbids component enumeration."
verification: "Verified in X4 for one sanitized vector: metadata type and owner-ID validation are `ok`; selected-sector capacity is `call_error`; no-cargo ware-limit is `not_applicable`; correlated bridge hello, heartbeat, and runtime-health were accepted through sequence 3. Scope-complete sector, asset, capacity, and ownership enumeration remains unverified."
files_changed:
  - ".planning/debug/phase1-plan11-facts-unsupported.md"

## Authorized Plan Correction

classification: "planned; not implemented or observed in X4"
plan: "01-11 Task 3"
contract: "A default-off, attempt-scoped developer diagnostic may emit at most one vector with exactly four class-only fields: `metadata_type`, `owner_id_validity`, `sector_capacity`, and `first_cargo_ware_limit`. It retains the aggregate `facts_unsupported` disposition and does not alter the observation schema or admission path."
privacy_and_scope: "The vector may retain its configured attempt ID and result classes only. It must not include runtime IDs, owner IDs, names, macros, wares, numeric values, native errors, Lua tables, frames, raw payloads, component enumeration, a second-sector scan, save access, or any mutation/effect path."
trace_bound: "Disabled by default; one nonempty bounded attempt ID enables no more than one four-field vector for one disposable cycle. A missing, invalid, repeated, or over-limit attempt suppresses the vector and retains the aggregate unsupported result."
ledger: "The append-only disposable ledger records the attempt ID, X4 version, active mods, scenario, aggregate disposition, four result classes, trace-stop state, and no-save/no-effect attestations. It stores no raw vector inputs or payload data."
stop_conditions:
  - "Stop and disable the probe immediately after the first vector."
  - "Stop as failed if the vector is absent, duplicated, has a non-closed field/value class, contains any raw or identifying content, requires a new X4 call, or expands the selected-sector scan."
  - "Do not proceed to component enumeration within Plan 01-11."
successor_routing:
  - "A `call_error`, `wrong_type`, or `invalid_value` result for selected-sector capacity or first-cargo ware limit supports a separately authorized plan for deterministic, bounded real-component enumeration; it does not establish the API is globally absent."
  - "`not_applicable` for an empty cargo table is retained as the existing non-failure cargo case, not a proof of an asset fact."
  - "Any other outcome retains D-09 as explicit unsupported; a contradiction between all admissible result classes and the aggregate result needs its own bounded corrective plan."
