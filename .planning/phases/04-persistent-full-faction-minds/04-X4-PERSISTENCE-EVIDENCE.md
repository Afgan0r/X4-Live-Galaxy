---
phase: 04-persistent-full-faction-minds
runtime-proof-owner: Phase 7
status: pending-X4
---

# Phase 7 X4 Persistence Evidence Protocol

This procedure is the disposable Creative Custom evidence handoff for the X4-owned checkpoint surface. It does not authorize save-file inspection or modification, nor does it claim that local static checks prove runtime behavior.

## Evidence Classification

| Level | Current fact |
| --- | --- |
| Documented | Mission Director state is saved with X4-owned campaign state; old variables can be absent in older saves; cue-version patches can accommodate schema evolution. |
| Verified locally | `live_galaxy_persistence.xml` and `checkpoint_schema.json` parse, and the PowerShell contract validates their stable identities and restricted action surface. |
| Pending-X4 | Payload limits, write interruption retention, save/load restoration, Rust-only reconnect, and incompatible-protocol behavior require a disposable game observation. |
| Observed in X4 | None. |

All runtime properties remain pending-X4 until the scheduled Phase 7 gate earns an observation.

## Required Phase 7 Environment

- Use one disposable Creative Custom scenario; never inspect or modify a player save file.
- Record the X4 build and full extension/mod set before every attempt.
- Capture the scenario, elapsed game time, elapsed real time, checkpoint sequence/hash, expected result, reread result, and bounded diagnostics for every observation.
- Stop the attempt if the checkpoint surface exposes model invocation, report delivery, acknowledgement delivery, pipe transport, Lua scheduling, or a game-state mutation path.

## Observation Matrix

| Probe | Setup and expected result | Required reread | Status before Phase 7 |
| --- | --- | --- | --- |
| Payload boundary | Submit only a contract-valid opaque checkpoint at each measured payload size; record accepted/rejected boundary without assuming a limit. | Reread the checkpoint sequence/hash after each accepted attempt. | pending-X4 |
| Interrupted write | Interrupt only the written disposable procedure at its defined checkpoint boundary; expect last-good retention or an explicit failure. | Reread the prior checkpoint sequence/hash and record bounded diagnostics. | pending-X4 |
| Save/load | Save and reload the disposable Creative Custom campaign after a contract-valid checkpoint; expect either attributable restoration or a recorded failure. | Reread the checkpoint sequence/hash after reload. | pending-X4 |
| Rust-only reconnect | Restart only a compatible Rust process while X4 remains open; expect no duplicate accepted sequence or report identity. | Reread the acknowledged checkpoint sequence/hash after reconnect. | pending-X4 |
| Incompatible protocol | Present an incompatible game-side protocol identity in the disposable procedure; expect fail-closed `x4_restart_required`. | Reread the compatibility status and bounded diagnostics after the required X4 restart path. | pending-X4 |

## Evidence Record Template

| Field | Required value |
| --- | --- |
| Classification | observed-in-X4, failed, or pending-X4 |
| X4 build | Exact installed build |
| Extension/mod set | Exact enabled extension identities and versions |
| Scenario | Disposable Creative Custom setup |
| Elapsed game time | Measured value |
| Elapsed real time | Measured value |
| Checkpoint sequence/hash | Expected and reread sequence/hash |
| Expected result | Hypothesis-specific outcome |
| Reread result | Independent readback outcome |
| Bounded diagnostics | Correlation-safe health and failure metadata only |

## Phase 7 Handoff

Phase 7 owns execution and recording of these runtime observations. Phase 4 contributes only the static schema contract and this procedure; it must not simulate a game result or promote local parsing to runtime proof.
