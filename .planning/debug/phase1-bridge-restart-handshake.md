---
status: resolved
trigger: Bridge restart accepts a new named-pipe connection but rejects the resumed Lua frame stream because the replacement bridge has not received a hello frame.
created: 2026-08-29
updated: 2026-08-29
---

# Phase 1 Bridge Restart Handshake

## Symptoms

- Expected behavior: after a bridge-only restart, the Lua runtime establishes a compatible session before the replacement bridge admits telemetry frames.
- Actual behavior: after restarting bridge PID 59152 while X4 remained running, the replacement bridge accepted a pipe connection but received `runtime_health` generation 1 sequence 27 before any `hello`, then rejected that and later frames.
- Error messages: bridge debug evidence records rejected `runtime_health`, `complete_marker`, and `heartbeat` frames; X4 logs `pipe_write_succeeded` for the same legacy sequence stream.
- Timeline: an earlier compatible restart had been observed; this reproduction followed an X4 relaunch while an old bridge still retained generation 3, then a bridge-only restart.
- Reproduction: run the disposable Creative Custom session, restart only `x4-bridge`, and inspect the correlated debug-sink log.

## Current Focus

- hypothesis: confirmed — a write failure must reset the Lua application session before any replacement-pipe write is attempted.
- test: focused Lua fake-adapter contract covers the first failed write, discarded legacy frame, and the following `hello`; Rust pipe contracts remain green.
- expecting: the replacement bridge receives `hello` before any telemetry frame.
- next_action: run the disposable in-game bridge-only restart probe before shipping.

## Evidence

- timestamp: 2026-08-29; `runtime/phase1-bridge-obs-x4-d09-v2-20260829-02-game-relaunch.log` recorded `listener_ready`, `client_connected`, then rejected generation-1 frames without `hello`.
- timestamp: 2026-08-29; X4 debug log recorded successful pipe writes for the rejected continuation stream.
- timestamp: 2026-08-29; installed `sn_mod_support_apis` source documents that a restarted server can appear healthy until the first write fails; `_Write_Pipe_Raw` opens the pipe and throws on that failure.
- timestamp: 2026-08-29; the old Lua adapter caught the failure, disconnected, and retried the same already-created telemetry frame. This is the first missing transition: `connected = false` before the next write can occur.
- timestamp: 2026-08-29; focused Lua regression was RED before the repair because `runtime.emit` reported `sent` after a second write. It is GREEN after the repair: the failed frame is discarded, the tick resets connection state, and the next tick emits `hello`.

## Eliminated

- hypothesis: named-pipe connection failed entirely; rejected because the replacement bridge logged `client_connected` and received frames.

## Resolution

- root_cause: the Lua adapter retried a legacy telemetry frame across a recovered OS pipe connection instead of transitioning the application protocol back to `AwaitingHello`.
- fix: after a write exception, disconnect once, discard the frame, return `pipe_reconnect`, and reset the runtime connection state so the next scheduled tick sends `hello` with a new generation.
- verification: `extensions/live_galaxy/tests/run_contracts.ps1 -Suite x4_discovery` and `cargo test -p x4-bridge` passed locally. The all-Lua-suite run is blocked by a shared-worktree failure at `telemetry_contract.lua:32`, outside this handshake change.
- guardrail_verdict: verified_locally; pending disposable X4 bridge-only-restart smoke test.
