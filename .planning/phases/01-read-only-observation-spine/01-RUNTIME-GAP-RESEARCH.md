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
