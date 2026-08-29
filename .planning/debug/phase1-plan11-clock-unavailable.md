---
status: planned
trigger: "Diagnose the live Phase 1 Plan 01-11 v2 disposable X4 runtime failure: loader succeeds but runtime reports discovery_unavailable detail=clock_unavailable and bridge receives no frames."
created: 2026-08-29T00:00:00+07:00
updated: 2026-08-29T00:00:00+07:00
---

## Current Focus

bug_class: bohrbug
hypothesis: "The runtime adapter requires host-wall-clock Unix milliseconds from `os.time()`, but X4's UI Lua environment does not provide a usable result through that path; the adapter returns `clock_unavailable` before normalization and transport."
test: "Working backwards from the sole `clock_unavailable` return, compare the call and validation condition with the observed installed-X4 game-clock call shape and the trusted bridge-time precedent."
expecting: "The sole error branch will be the `pcall(api.observed_at_unix_millis)` validation, whose production implementation is `os.time() * 1000`; no pipe or fact normalization branch can run after it."
next_action: "Return the diagnosis: move Unix-millisecond authority to the Rust bridge receipt boundary, retain X4 `C.GetCurrentGameTime()` only as a separately named game-time value if Plan 01-11 requires it, and add fake/runtime admission contracts for missing or invalid clock sources."
reasoning_checkpoint:
  hypothesis: "`clock_unavailable` is caused by `runtime_api().observed_at_unix_millis` using `os.time() * 1000`, which fails X4's usable-clock validation before an observation reaches telemetry or the named pipe."
  confirming_evidence:
    - "`live_galaxy_x4_discovery.lua:51-54` defines the production clock exclusively through `os.time`; lines 118-121 return `clock_unavailable` whenever that protected call fails, is nonnumeric, or returns less than one."
    - "The live X4 diagnostic is exactly `discovery_unavailable detail=clock_unavailable`, and `live_galaxy_runtime.lua:145-150` logs that only when `telemetry.produce_observation` receives the adapter error; it then suppresses the observation frame."
    - "Installed X4 extension source uses `pcall(C.GetCurrentGameTime)` or `pcall(GetCurRealTime)`, not `os.time`; the installed X4 Live MCP source explicitly assigns wall-clock UTC at bridge receipt while X4 supplies game time."
  falsification_test: "A bounded X4 trace or fake reproducer showing that `os.time()` returns a positive numeric Unix-second value in the same registered UI-Lua context, while the adapter still returns `clock_unavailable`, would disprove this causal path."
  fix_rationale: "Do not reinterpret X4 game or elapsed time as Unix epoch milliseconds. Have the trusted Rust bridge stamp the accepted frame's Unix time at receipt; keep an optional X4-native game-clock value separately typed and named. This removes the unsupported host-clock dependency without fabricating a timestamp."
  blind_spots: "The current live log does not distinguish missing `os`, missing `os.time`, a thrown call, or a nonpositive return; all four produce the same intentionally bounded disposition. Exact native semantics of `GetCurRealTime` remain undocumented and must not be treated as Unix time."
  candidate_causes:
    - "code: the Plan 01-10 production adapter calls host-standard-library `os.time()` for a field that requires Unix milliseconds."
    - "environment: the X4 UI-Lua sandbox does not yield a usable value through that host-clock path in the observed live run."
    - "config: no Plan 01-11 bridge-receipt timestamp authority is selected, so Lua is forced to produce the Unix timestamp itself."
  and_gate: "yes — the failure requires both the adapter's host-clock dependency and the X4 UI-Lua environment where it has no usable result; the design-level timestamp-authority mismatch makes substituting game time incorrect."

## Symptoms

expected: "A disposable OBS-X4-04 attempt produces one bounded discovery-derived observation frame for the running v2 bridge."
actual: "At game time 9166.92, X4 debug.log reports `Live Galaxy runtime ... discovery_unavailable detail=clock_unavailable`; the bridge has no frames."
errors: "discovery_unavailable detail=clock_unavailable"
reproduction: "v2 bridge PID 45952 running; X4 PID 39904 started 2026-08-29 19:34:21; loader successfully loaded; then the registered runtime observation cycle executes."
started: "Observed during the live Phase 1 Plan 01-11 v2 disposable runtime attempt on 2026-08-29."

## Eliminated

## Evidence

- timestamp: 2026-08-29T00:00:00+07:00
  checked: Current authoritative live runtime report
  found: Loader completed, while the runtime cycle failed at discovery with `clock_unavailable` and emitted no bridge frames.
  implication: The failure is downstream of Lua loading and upstream of pipe transport; inspect the adapter's discovery/normalization path first.

- timestamp: 2026-08-29T00:00:00+07:00
  checked: Debug knowledge-base semantic recall
  found: No MemPalace recall tool is exposed in this task runtime; keyword fallback will be used if `.planning/debug/knowledge-base.md` exists.
  implication: No prior resolution is assumed as a diagnosis candidate.

- timestamp: 2026-08-29T00:00:00+07:00
  checked: SBFL applicability
  found: SBFL skipped: this is an in-game deterministic runtime observation with no failing/passing per-test coverage spectrum for the real X4 native call.
  implication: Use working-backwards and direct primary-source call-shape comparison.

- timestamp: 2026-08-29T00:00:00+07:00
  checked: `extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua:51-54,118-121`
  found: The sole production implementation of `observed_at_unix_millis` tests `os.time` and returns `os.time() * 1000`; its only consumer returns `clock_unavailable` on a protected-call failure, nonnumeric value, or value below one.
  implication: The reported disposition exactly localizes to the host-clock acquisition and validation branch, before fact construction, normalization, serialization, or pipe emission.

- timestamp: 2026-08-29T00:00:00+07:00
  checked: `extensions/live_galaxy/lua/live_galaxy_runtime.lua:123-128,145-150`
  found: A discovery error causes `produce_discovery_payload` to trace `discovery_unavailable`, return no body, and make the scheduled slot emit an unavailable health frame instead of an observation.
  implication: The absent bridge observation is the designed consequence of the clock failure, not an independent transport defect.

- timestamp: 2026-08-29T00:00:00+07:00
  checked: Installed X4 source: `extensions/dynamic_station_allocations/ui/dynamic_station_allocations.lua:152-165` and `extensions/x4_mission_offer_filter/ui/mission_offer_filter.lua:227-245`
  found: Observed installed-X4 UI-Lua call shapes obtain game time with `pcall(C.GetCurrentGameTime)` and real time with `pcall(GetCurRealTime)`, converting each with `tonumber`; neither establishes a Unix-epoch contract.
  implication: Replacing `os.time()` with either native call and multiplying by 1000 would not be a correct Unix-millisecond fix.

- timestamp: 2026-08-29T00:00:00+07:00
  checked: X4 Live MCP source precedent: `F:/Agent Projects/X4/extensions/x4_live_mcp/ui/x4_live_mcp.lua:72-80`
  found: Its event envelope carries `C.GetCurrentGameTime()` as game time and explicitly records that the bridge stamps wall-clock UTC on receipt.
  implication: The smallest semantically correct boundary is bridge-owned Unix receipt time plus an optionally separate X4 game-time field, not a Lua-derived fake Unix clock.

- timestamp: 2026-08-29T00:00:00+07:00
  checked: `crates/observation-ingest/src/wire.rs:133-141` and `src/batch.rs:109-122`
  found: The strict v2 wire schema presently requires client-supplied `observed_at_unix_millis: u64`, and admission constructs `ObservationTime` directly from it.
  implication: Plan 01-11 must move or introduce timestamp authority at Rust admission; a Lua-only substitution cannot preserve the field's stated Unix semantics.

## Resolution

root_cause: "Confirmed AND-gate: (1) `extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua` requires `os.time() * 1000` for `observed_at_unix_millis`; (2) the observed X4 UI-Lua runtime supplies no usable result through that path. The adapter therefore returns `clock_unavailable` at lines 118-121 before it builds, normalizes, serializes, or emits an observation. The design also incorrectly makes X4 responsible for a Unix-epoch timestamp: available native clocks are game/real-time values with no demonstrated Unix meaning."
fix: "Planned in `.planning/phases/01-read-only-observation-spine/01-11-PLAN.md`; not yet implemented or deployed. The Rust bridge will stamp `observed_at_unix_millis` from an injected trusted receipt clock only after strict frame validation, and the X4 Lua adapter will stop requiring a Unix-clock call. The envelope retains a separately named optional X4 game-time domain value from protected `C.GetCurrentGameTime()` when valid, never as Unix time. The retry ledger will record both values distinctly. Do not substitute `GetCurRealTime()*1000` or `C.GetCurrentGameTime()*1000`."
verification: "Add RED/green contracts: (1) Lua fake API with no host `os.time` still produces the complete fact payload; (2) Lua returns an explicit unavailable disposition if the separately optional X4 game clock is missing/invalid, without emitting an observation; (3) Rust stamps a positive receipt Unix time and rejects/masks any client-provided legacy Unix timestamp according to the selected strict-v2 schema; (4) a disposable X4 retry verifies that the frame passes the previous clock boundary and records bridge receipt time separately from X4 game time. The runtime result remains pending-X4 until that isolated retry is recorded."
files_changed: []

## Root-Cause Report

| Claim | Classification | Evidence |
| --- | --- | --- |
| The loaded v2 X4 runtime returns `clock_unavailable` before it sends an observation. | Observed | Authoritative live debug-log report; bridge receives no frames. |
| The sole project source path that can return that error is the protected `observed_at_unix_millis` call and numeric validation. | Documented | `live_galaxy_x4_discovery.lua:118-121`. |
| The production clock implementation is `os.time() * 1000`. | Documented | `live_galaxy_x4_discovery.lua:51-54`. |
| A discovery error suppresses the observation frame. | Documented | `live_galaxy_runtime.lua:123-128,145-150`. |
| Installed X4 UI code exposes `C.GetCurrentGameTime` and `GetCurRealTime` call shapes. | Observed source evidence | Installed extension sources cited in Evidence. |
| Either X4-native clock is Unix epoch time. | Unknown | No primary contract establishes that semantic; treating it as Unix would be an unsupported conversion. |
| Bridge receipt is the correct Unix timestamp authority. | Inferred, supported by precedent | The strict field name requires Unix milliseconds; X4 Live MCP separates game time from bridge-stamped UTC. |

Skill learning: none — the evidence is one newly confirmed adapter/sandbox mismatch; the existing integration skill already requires explicit value ownership and forbids treating missing data as a valid value, while the X4-tests skill already requires fake-adapter and disposable-runtime separation. No new durable rule is established.
