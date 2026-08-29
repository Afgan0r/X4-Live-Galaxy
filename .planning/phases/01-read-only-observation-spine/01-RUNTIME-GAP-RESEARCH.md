# Phase 1 Runtime Gap Research

**Date:** 2026-08-29
**Status:** Locally researched; disposable X4 evidence pending

## Correction

The existing Phase 1 implementation is not yet runnable in X4. The Mission
Director file has no active cues, the Lua modules are not registered as an X4
UI addon, and the Rust workspace has no named-pipe server process. Therefore,
the existing `OBS-X4-01` through `OBS-X4-04` human gate cannot be executed.
Static and fake-adapter evidence must remain `verified locally`.

## Evidence

| Claim | Classification | Evidence |
| --- | --- | --- |
| X4 UI Lua is loaded through `ui.xml` in a menus environment. | Documented and locally observed | The X4 Live precedent registers a UI file with `<environment type="menus">`. |
| Mission Director invokes UI Lua through named Lua events. | Documented and locally observed | The precedent registers callbacks with `RegisterEvent` and raises matching events with `raise_lua_event`. |
| Mission Director does not directly import the UI Lua module. | Observed | The current path is UI registration followed by event dispatch; no direct MD module call is used. |
| X4 acts as the named-pipe client and the external bridge owns the server endpoint. | Documented | The installed `sn_mod_support_apis` documentation assigns pipe-client ownership to X4. |
| The pipe is a Windows duplex message-mode pipe. | Locally observed | The precedent external bridge uses `CreateNamedPipeW`, `ConnectNamedPipe`, and message-mode reads. |
| Lua writes a bounded payload through `_Write_Pipe_Raw`. | Locally observed | The precedent calls the installed support API through a protected `pcall`. |
| Broken transport reconnects at the external server boundary. | Locally observed | The precedent closes a failed connection and accepts a new client while preserving semantic admission state. |
| Exact callback cadence and `_Write_Pipe_Raw` runtime semantics work in X4 9.00. | Unknown | These claims require a disposable in-game run. |

The reference repository is a dirty local checkout with no remote. It is a
process precedent, not remotely fresh authority. The installed support API is
version 195; its packed native implementation was not inspected.

## Required Runtime Topology

The smallest Phase 1 harness is one-way and telemetry-only:

```text
Mission Director scheduler
  -> named Lua event
  -> registered UI Lua callback
  -> sn_mod_support_apis named-pipe client
  -> Live Galaxy Rust named-pipe server
  -> bounded admission and diagnostic output
```

It requires:

- `content.xml` with an explicit `sn_mod_support_apis` dependency;
- `ui.xml` registering one Live Galaxy UI Lua entry point;
- a thin UI Lua adapter that owns the native pipe call, bounded queue, retry,
  and externally visible runtime-health frames;
- an active Mission Director scheduler that raises isolated telemetry events;
- a runnable Rust bridge binary that owns one named-pipe server endpoint,
  decodes one bounded frame at a time, and feeds existing admission logic;
- heartbeat, runtime-health, one bounded identity observation, and one explicit
  completion marker;
- no report, acknowledgement, effect, command, model, or game-state mutation
  vocabulary.

The extension and bridge must be implemented independently. The inspected
third-party and precedent sources do not establish redistribution permission.
Only the documented interoperability call shape may influence the design.

## Restart and Lifecycle Boundary

- Build and verify the extension and bridge before installing either.
- Install or replace game-facing extension files only while X4 is closed.
- Start the external bridge before X4 for the first disposable probe.
- A compatible bridge restart must reconnect without restarting X4.
- A game-facing extension change or incompatible protocol combination requires
  an X4 restart and must fail closed with that condition exposed.
- Do not run two bridge processes that compete for the same pipe.

## Verification Split

Automated evidence can prove XML structure, UI and MD event-name alignment,
Lua adapter contracts, pipe framing and bounds, Rust decoding and admission,
reconnect state, idempotency, reconciliation, packaging contents, and install
guards.

Only a disposable X4 9.00 run can prove UI loading, callback delivery, native
pipe availability, effective cadence, SETA behavior, bridge reconnect from the
running game, runtime discovery, and absence of game-thread degradation.

## Planning Consequence

Split the existing Plan 01-07 checkpoint:

1. Complete Plan 01-07 as the evidence-contract deliverable only.
2. Add an autonomous gap-closure plan for the runnable telemetry harness and
   its local verification.
3. Add a final human-verification plan for `OBS-X4-01` through `OBS-X4-04`.

The human gate remains blocking for Phase 1 completion, but it becomes
actionable only after the autonomous runtime-harness plan passes.

## 01-10 Discovery API Evidence Gate

**Date:** 2026-08-29
**Status:** Observed call-shape precedent available; runtime behavior and compatibility pending

### Installed-X4 evidence inspected

| Claim | Classification | Evidence | Consequence |
| --- | --- | --- | --- |
| `GetClusters(true)` can be iterated and each returned cluster passed to `GetSectors(cluster)`. | Observed | Installed `extensions/x4_live_mcp/ui/x4_live_mcp.lua`, `start_player_known_sector_scan()`. | This is evidence for a player-known sector scan only, not a documented all-runtime-sector contract. |
| `ConvertIDTo64Bit(value)` and `ConvertStringToLuaID(stable_id)` are used to derive numeric component identifiers. | Observed | Installed `extensions/x4_live_mcp/ui/x4_live_mcp.lua`, `lua_universe_id_string()`, `remember_known_sector()`, and `component_sector()`. | The example demonstrates a local normalization pattern, but does not establish identity stability or lifecycle semantics for Live Galaxy. |
| `GetComponentData(sector, "name", "owner", "macro")` returns sector metadata in the installed extension. | Observed | Installed `extensions/x4_live_mcp/ui/x4_live_mcp.lua`, `player_known_sector_event()`. | Owner data is evidenced only for sectors reached by that scan. |
| `GetComponentData(component, "cargo")` and `GetWareProductionLimit(component, ware)` are used for a component's cargo snapshot. | Observed | Installed `extensions/x4_live_mcp/ui/x4_live_mcp.lua`, `cargo_snapshot()`. | These calls do not establish an enumeration source for all required assets or their bounded cost. |
| `C.GetPeopleCapacity(component64, "", false)` is used for a component's crew capacity snapshot. | Observed | Installed `extensions/x4_live_mcp/ui/x4_live_mcp.lua`, `crew_snapshot()`. | The observed use is component-local and lacks a documented contract for availability, failure modes, or aggregate discovery. |

### Unknowns that constrain Task 1 production adapter

- **Assets:** No installed vanilla or support-library documentation establishes a read-only UI-Lua API that enumerates the required asset population, its visibility scope, identity form, or failure contract.
- **Capacity:** The observed component-local cargo and crew calls do not prove which capacity definition satisfies D-09 or how it can be sampled without unbounded per-asset work.
- **Ownership:** The observed sector `owner` field does not establish authoritative ownership semantics for discovered assets, unavailable ownership, or a stable normalized identity.
- **Bound:** No installed-X4 primary evidence establishes callback cost, pagination, slice size, or a maximum safe enumeration operation for the combined sector, asset, capacity, and ownership scope.
- **Provenance:** The API call examples occur in an installed third-party extension. Their implementation and licence provenance have not been reviewed, so they cannot be copied or treated as a Live Galaxy production contract.

### Gate result

The observed `x4_live_mcp` call sites provide a bounded implementation precedent, not a documented native X4 contract. Task 1 may create one original, bounded, read-only adapter from those explicit call shapes after recording provenance/license status and the unknowns above. It must expose unavailable, incomplete, and unsupported material explicitly; it must not copy third-party code, invent semantics, or claim compatibility. The fresh isolated OBS-X4-04 run in Task 3 remains the sole evidence that can establish actual X4 behavior or compatibility. If a safe minimal call cannot be constructed from the observed call sites, Task 1 halts and D-09 remains pending.

### Follow-up installed-source check (2026-08-29)

| Claim | Classification | Evidence | Consequence |
| --- | --- | --- | --- |
| The installed X4 root is available for read-only inspection. | Observed | `F:\SteamLibrary\steamapps\common\X4 Foundations` exists. | The Task 1 precondition's source path is available, but availability alone does not establish a production API contract. |
| The installed Mod Support API documents UI-Lua loading and pipe ownership, not discovery enumeration. | Documented | `extensions/sn_mod_support_apis/readme.txt` documents `Register_OnLoad_Init`, named-pipe ownership, and loading prerequisites. | It does not establish sector, asset, capacity, ownership, stable identities, or a one-operation discovery bound. |
| `GetClusters(true)`, `GetSectors(cluster)`, `GetComponentData`, `GetWareProductionLimit`, and `C.GetPeopleCapacity` occur in an installed extension. | Observed source evidence | Narrow call-site inspection of installed `extensions/x4_live_mcp/ui/x4_live_mcp.lua` at its sector scan, cargo snapshot, and crew snapshot functions. | These are call shapes from an installed third-party extension, not installed vanilla documentation or a Live Galaxy production guarantee. They cannot close the missing asset enumeration, ownership semantics, or bounded-work contract. |
| Lua semantic navigation is available for the installed X4 source tree. | Observed | The Lua LSP initially reported project initialization, then returned document symbols for the inspected installed extension. | LSP confirms source structure only; dynamic X4 globals still lack installed primary declarations, so literal source calls remain observed rather than documented contracts. |

The primary-source-first check found no installed vanilla Lua/documentation evidence that supplies a complete read-only enumeration contract for sectors, assets, capacity, and ownership with an identity and bounded-work rationale. This does not erase the observed third-party call-shape precedent: it permits a constrained original adapter that represents the remaining gaps explicitly. Task 3 remains the only place where a fresh disposable run can establish runtime behavior or compatibility.

### Task 01-10 original-adapter boundary

The project adapter is original code. It uses only the observed source shapes
`GetClusters(true)`, `GetSectors(cluster)`, `GetComponentData(sector, "name",
"owner", "macro", "cargo")`, `GetWareProductionLimit(sector, ware)`, and
`C.GetPeopleCapacity(ConvertIDTo64Bit(sector), "", false)`. The adapter selects
one deterministic sector from the first observed cluster and emits a single
`x4_runtime` section with `partial` quality even when its bounded metadata reads
return values. It makes no all-sector, asset-population, capacity-definition,
ownership-semantics, identity-stability, cost, API-availability, or compatibility
claim. A missing or malformed source result is `unsupported` or an explicit error,
never `known_empty` or a fixed fallback identity.

The call-shape provenance is the installed third-party `x4_live_mcp` extension.
Its implementation and redistribution license remain unreviewed; no source code was
copied and the observed shapes are not a documented contract. The installed Mod
Support API documentation does not fill these discovery gaps. Task 3 remains the
sole runtime evidence gate.

### Embedded-Lua loader compatibility result (2026-08-29)

**Classification:** Observed X4 loader evidence; standalone-runner target
remains unknown.

In a fresh disposable X4 session (PID `43708`, started `2026-08-29 18:29:01`
local), the bounded debug log recorded
`trace_config_loaded detail=status=disabled enabled=false` and
`handler_registered detail=event=live_galaxy_observation` at game time
`9075.57`. No `module not found`, parser/syntax, or runtime module-load error
occurred in the observed log. This establishes only that the deployed extension
was accepted by the embedded X4 loader for the observed session.

X4 exposed no interpreter version in that bounded log; its compatibility target
is therefore `not exposed`, not an inferred Lua release. No bridge was started.
The subsequent game-time `9076.91` bounded diagnostic,
`pipe_write_failed detail=raw writer exhausted its reconnect attempt`, is
expected for the loader-only probe and establishes neither bridge transport nor
any OBS-X4 outcome.

**Gate impact:** Plan 01-10 Task 2 remains blocked before standalone Lua
runner selection, lock creation, provisioning, or download. The next isolated
probe is a default-off, one-event development trace. With X4 closed before the
extension replacement, it enables only `version_diagnostic_enabled` and records
only the embedded runtime's ASCII-safe, at-most-64-byte sanitized `_VERSION`
value through the existing X4 debug-log path. It is loader-only: no bridge,
telemetry event, discovery, public UI, effect, or game-state path may run. If
`_VERSION` remains unavailable or invalid, or cannot identify a compatible
target, the plan stops before runner work. This result leaves OBS-X4-01 through
OBS-X4-04, including D-09 runtime discovery, pending.

### Embedded-Lua `_VERSION` compatibility result (2026-08-29)

**Classification:** Observed X4 embedded-version evidence; runner selection is
unlocked, while runtime discovery and transport remain pending.

In a fresh disposable X4 session (PID `33016`, started `2026-08-29 18:45:03`
local), the bounded X4 debug log recorded
`embedded_lua_version detail=Lua 5.1` at game time `9075.57`, followed by
handler registration. The observed log contains no parser/syntax or runtime
module-load error. This establishes the embedded runtime's self-reported Lua
version for standalone runner compatibility selection only.

The enabled trace configuration was a temporary installed-extension probe
setting; it is not evidence that the repository source default is enabled.
No bridge was started. Any pipe failure in this loader-only run is expected and
non-transport, so it cannot support an OBS-X4 transport, discovery, or fact
claim.

**Gate impact:** Plan 01-10 may now select a pinned standalone Lua 5.1 runner
target. That selection still needs its own lock, provenance, materialization,
and executable-version verification; this probe did not provision or download a
runner, run contracts, or establish an X4 runtime discovery result. OBS-X4-01
through OBS-X4-04, including D-09, remain pending.
