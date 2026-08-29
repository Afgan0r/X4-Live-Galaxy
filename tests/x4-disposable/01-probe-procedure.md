# Phase 01 Disposable X4 Observation Procedure

## Purpose

This procedure is the only accepted route for upgrading a Phase 01 runtime
claim to `observed-in-X4`. It uses a disposable Creative Custom X4 9.00
campaign and tests one hypothesis at a time. It never reads or modifies a save
file.

## Evidence Taxonomy

<!-- markdownlint-disable MD013 -->

| Evidence level | Meaning | May be upgraded to |
| --- | --- | --- |
| `documented-static` | A versioned source, manifest, or XML check supports the claim. | `pending-X4` or `observed-in-X4` after a separate run. |
| `fake-local` | A deterministic fake, fixture, or local Rust test supports the contract. | `pending-X4` or `observed-in-X4` after a separate run. |
| `pending-X4` | The exact X4 9.00 behavior has not yet been observed in a qualifying run. | `observed-in-X4` only. |
| `observed-in-X4` | A disposable game run recorded all required setup, readback, and health data. | Not applicable. |
| `failed` | The attempted game probe did not meet its expected result or safety bounds. | `pending-X4` only after recording a new attempt. |

<!-- markdownlint-enable MD013 -->

Static checks, local tests, and fake adapters never earn `observed-in-X4`.
Each game attempt needs an independent row in
[`01-probe-evidence.md`](01-probe-evidence.md) with the same Evidence ID.

## Safety Boundary

- Use a new disposable Creative Custom campaign; do not inspect, copy, load,
  edit, or otherwise access any save file.
- Keep the extension telemetry-only. Do not enable or add fleet, economy,
  diplomacy, institution, report, acknowledgement, model, public API, or any
  other game-state effect path.
- Stop an attempt immediately if it requires an effect path, blocks observable
  game scheduling, exceeds the configured bounded work, or cannot produce
  correlated setup, readback, and health evidence.
- Treat initial frame, queue, cadence, native Lua/Mission Director, and
  named-pipe details as assumptions until a row earns `observed-in-X4`.
- Start the bridge from the same interactive Windows account as X4. A pipe host
  started under a sandbox or service account can be rejected by the pipe ACL.

## Embedded-Lua Syntax/Version Probe — Plan 01-10 Task 3 Pre-Step

This narrow pre-step records only whether the currently deployed extension is
accepted by the embedded X4 Lua loader and whether X4 exposes an interpreter
version. It is not an OBS-X4-04 discovery attempt, does not start a bridge,
and cannot select or provision a standalone Lua runner.

1. Confirm the package guard passed and that X4 was closed immediately before
   the recorded extension deployment. Do not replace extension files while X4
   is running.
2. Launch a **new** disposable Creative Custom X4 9.00 campaign with only
   `live_galaxy` and its required support dependency active. Do not inspect,
   copy, load, edit, or otherwise access a save file.
3. Do not start `x4-bridge`, configure tracing, trigger telemetry, or exercise
   OBS-X4-01 through OBS-X4-04. Wait only long enough to determine whether the
   extension loads without an embedded-Lua parser or module-load error.
4. Record the exact X4 version, active extension list, Creative Custom
   scenario, real elapsed time, game elapsed time, SETA state, the embedded
   loader's syntax outcome, and any X4-exposed interpreter-version evidence.
   Record `not exposed` rather than inferring a Lua version from host tooling
   or syntax alone.
5. If the loader reports an error, preserve its bounded diagnostic and classify
   the probe `failed`. If the extension loads but exposes no interpreter
   version, classify syntax as observed and version as `not exposed`; runner
   selection remains blocked. If the session cannot be run, retain `pending`.

An accepted loader probe can satisfy only the loader syntax/load portion of
this pre-step. It does not select a standalone runner unless X4 also exposes a
compatible interpreter version. A trace configuration or registered handler is
loader evidence, not an observation, discovery, or transport result. When no
bridge is started, an expected bounded pipe-write failure must remain separate
from loader acceptance and cannot qualify any `OBS-X4-01` through `OBS-X4-04`
claim.

### Opt-In Embedded `_VERSION` Diagnostic

After the loader-only result above records the interpreter as `not exposed`,
the next disposable attempt may enable the Plan 01-10 development diagnostic.
This is a separate loader-only compatibility probe, not an OBS-X4 attempt.

1. With X4 closed, verify the package and deploy the extension. Set only the
   named attempt's **installed-extension**
   `version_diagnostic_enabled` flag to `true`; this temporary probe setting
   must not be represented as the repository source default. Do not start a
   bridge.
2. Start a new Creative Custom session with the same read-only boundary. Do not
   trigger a telemetry event, UI interaction, report, command, acknowledgement,
   persistence, or game-state effect.
3. Capture the single existing-debug-log event containing `_VERSION`. The value
   is expected to be ASCII-safe and no longer than 64 bytes. Record the exact
   bounded value, or `not exposed` / `invalid` if it is absent or unusable.
4. Disable the installed-extension flag for subsequent normal operation. If no
   usable target is observed, record `not exposed` or `invalid`, then stop
   before creating a runner lock, choosing a runner version, downloading a
   binary or source, or changing the contract runner.

The diagnostic proves only the embedded runtime's self-reported `_VERSION`.
It does not prove syntax beyond the earlier loader result, X4 API semantics,
discovery behavior, transport, or a compatible standalone runner.

### Loader Layout Preflight

The registered UI entry point is `lua/live_galaxy_runtime.lua`. Every local Lua
import, including transitive imports such as telemetry to normalization, must
use `live_galaxy/lua/<module>` so X4's `extensions/?.lua` package path maps it
to the packaged extension directory. Before replacing this extension, run the
package guard. If X4 reports a module load failure, record the complete bounded
loader diagnostic and stop the probe; do not substitute a root-level duplicate
module or inspect a save file.

## Required Fields for Every Attempt

Record the following in the matching evidence row before classifying the
result:

- Evidence ID and attempt number.
- Exact X4 version and active mod list.
- Extension build identity and protocol identity.
- Creative Custom scenario and the isolated hypothesis.
- Real elapsed time, game elapsed time, and SETA state.
- Expected readback, observed readback, and bounded health result.
- Evidence level, result (`observed-in-X4`, `failed`, or `pending`), and
  failure diagnostics when applicable.

## D-09 Strict-v2 Disposable Attempt Preparation — 2026-08-29

Attempt identity: `obs-x4-d09-v2-20260829-01`.

This is a game-facing incompatible evolution. A v1 extension cannot encode the
required strict-v2 fact classes and is not partial compatibility. Replace the
extension only with X4 closed, then start a new disposable Creative Custom
session. After the v2 extension connects successfully, a Rust-only bridge
restart is compatible and must not restart the still-running X4 process.

### Prepared Local Build and Static Preflight

<!-- markdownlint-disable MD013 -->

| Item | Recorded value |
| --- | --- |
| Extension identity | `live_galaxy`; game-facing build `live-galaxy-x4-build-2`; capability `live-galaxy-observation-v2` |
| Bridge executable | `target/debug/x4-bridge.exe` |
| Bridge SHA-256 | `C049947574DEFA23D50FEFAB6162BFDD948F43737CB005319DFC18D17C10F313` |
| Build command | `cargo build -p x4-bridge` |
| Package preflight | `powershell -NoProfile -File tests/x4-disposable/01-install-guard.ps1 -VerifyPackageOnly` |
| Lua contract preflight | `powershell -NoProfile -File extensions/live_galaxy/tests/run_contracts.ps1 -Suite x4_discovery` |
| Rust fact admission | `cargo test -p observation-ingest --test runtime_facts_contract` |
| Rust pipe and restart checks | `cargo test -p x4-bridge --test named_pipe_contract`; `cargo test -p x4-bridge --test reconnect_idempotency` |
| Deployment status | Not attempted. |

<!-- markdownlint-enable MD013 -->

### Controlled Launch and Restart Sequence

1. Confirm `Get-Process -Name X4 -ErrorAction SilentlyContinue` returns no
   process. Do not inspect, copy, load, edit, or otherwise access a save.
2. Run the prepared package and contract preflight commands above.
3. While X4 remains closed, use the approved extension deployment operation to
   replace the installed `live_galaxy` extension with this strict-v2 build.
   Verify the copied hashes and record them in the ledger. Do not change
   vanilla files or another extension.
4. Start exactly one `target/debug/x4-bridge.exe` from the same interactive
   Windows account as X4. Record its bounded readiness diagnostic.
5. Launch a new disposable Creative Custom X4 9.00 session with only
   `live_galaxy` and the required support dependency active. Perform exactly
   one isolated D-09 read-only attempt and record sector, asset, capacity, and
   ownership fact classes, each with its identity/value and explicit quality.
6. Without changing the extension or its protocol/build identity, stop the
   compatible bridge and start one replacement v2 bridge while X4 remains live.
   Record the new bridge/session generation and health readback. Do not run two
   bridge processes concurrently.
7. If any fact class is absent, malformed, unavailable, or over its bound,
   record the explicit disposition and classify the runtime attempt `failed` or
   `pending`; never call metadata-only output scope-complete discovery.

This preparation does not authorize deployment while X4 is running, a cadence
or SETA soak, reports, model calls, persistence, acknowledgements, commands,
or any game-state mutation.

### Recorded v2 Retry Outcome — 2026-08-29

Attempt `obs-x4-d09-v2-20260829-02` completed the controlled v2 transport and
bridge-only restart portion of this procedure. Its standard `DebugSink`
recorded listener readiness, connection, and accepted generation `3` hello,
heartbeat, and runtime-health frames. The Lua trace also recorded the bounded
recovery after the first bridge-only restart and the generation advances.

This does not complete D-09 enumeration. The runtime reported
`facts_unsupported` for every required fact class. Treat each as explicit
unavailable, never as a known-empty scope. The sanitized retained evidence does
not contain an X4 game-time value or a Unix receipt-time value; keep their
fields separate and unrecorded rather than deriving one from the other. The
next attempt must target the read-only X4 adapter capability gap without
introducing a game-state effect path.

## Automated Prerequisite Checks

Run these before opening X4 and record their outcome as `documented-static` or
`fake-local`; they are prerequisites, not runtime proof.

```powershell
cargo test --workspace
cargo lint
<!-- markdownlint-disable-next-line MD013 -->
powershell -NoProfile -Command "[xml](Get-Content -Raw 'extensions/live_galaxy/content.xml') | Out-Null; [xml](Get-Content -Raw 'extensions/live_galaxy/md/live_galaxy_observation.xml') | Out-Null"
```

## Probe Sequence

### OBS-X4-01 — Typed Runtime Identity and Section State

1. Start the disposable Creative Custom scenario with only the intended
   observation extension and required support dependencies active.
2. Identify one runtime-discovered sector or asset and one section state
   (`unknown`, `partial`, `stale`, `unsupported`, or known coverage) without
   assuming an empty value means known-empty.
3. Observe the proposed telemetry/readback surface once at normal speed.
4. Record identity, source, observation time, monotonic version, section
   freshness, coverage, quality, expected readback, observed readback, and
   bounded health in the matching ledger row.
5. Mark `observed-in-X4` only if the identity and explicit section state are
   visible in the same qualifying attempt. Otherwise mark `failed` or
   `pending` with diagnostics.

### OBS-X4-02 — Bounded Producer and Backpressure

1. Use a fresh disposable Creative Custom scenario and do not combine this
   attempt with identity, handshake, or enumeration confirmation.
2. Run the observation path at normal speed for the recorded time window, then
   repeat the same bounded window under the recorded SETA state.
3. Observe producer/readback timing, queue or backpressure disposition, and
   health without introducing additional game work or an effect path.
4. Record both timing windows and state whether scheduling remained observable.
5. Mark `failed` and stop if the game visibly blocks, a bound is exceeded, or
   health is missing. Do not infer numeric limits from a successful run; record
   only the observed values.

### OBS-X4-03 — Capability Handshake and Reconnect Boundary

1. Use a fresh disposable Creative Custom scenario and record the extension
   build and protocol identity before connecting.
2. Exercise a compatible Rust-side reconnect without changing the game-facing
   extension. Record the new bridge/session generation and health readback.
3. Separately exercise an incompatible protocol, capability, or game-facing
   build combination using the explicitly configured test condition.
4. Record whether the compatible case reconnects without restarting X4 and
   whether the incompatible case fails closed with an explicit X4 restart
   requirement.
5. Never substitute a local fake outcome for either result. If the test setup
   cannot safely produce both conditions, mark the missing condition `pending`.

### OBS-X4-04 — Scope-Complete Runtime Enumeration

1. Use a fresh disposable Creative Custom scenario and select one discovery
   scope for sectors, assets, capacity, or ownership.
2. Trigger only the bounded read-only enumeration for that scope.
3. Record the scope identity, completion marker/readback, discovered member
   count, section quality, and bounded health.
4. Confirm that a missing or incomplete marker does not become known-empty and
   does not remove prior membership. Record the actual readback rather than an
   expected topology.
5. Mark `observed-in-X4` only when the same attempt shows a validated
   scope-complete result. Otherwise retain `pending` or record `failed`.

## Completion Rule

The ledger is complete only when every attempted probe has all required fields
and is classified at the level it earned. It is valid for any row to remain
`pending-X4`; a local automated pass does not close that gap.

## Plan 01-11 Task 3 Capability-Vector Probe

Run this only after the strict-v2 retry is recorded as `facts_unsupported`, the
project-local Lua contracts and package guard pass, and X4 is closed for
deployment. Use one new disposable Creative Custom X4 9.00 session and one
nonempty attempt ID containing only letters, digits, `_`, or `-` (maximum 64
bytes).

1. Enable `capability_probe_enabled` only for this deployment and set the
   matching `capability_probe_attempt_id`; leave the repository default off.
2. Start one bridge only after the preflight passes, reproduce one existing
   selected-sector discovery cycle, and retain one aggregate
   `facts_unsupported` disposition plus at most one vector.
3. Accept only these four class labels: `metadata_type`, `owner_id_validity`,
   `sector_capacity`, and `first_cargo_ware_limit`. Each value is limited to
   `ok`, `call_error`, `wrong_type`, or `invalid_value`; the ware-limit field
   may additionally be `not_applicable` only when no cargo entry exists.
4. Stop immediately after the first vector. Disable the probe, stop the bridge
   attempt, and append the result to the ledger. Mark the attempt `failed` if
   the vector is absent or duplicated, has another field or class, contains a
   raw value/identifier/payload/native error, or needs a second-sector scan,
   component enumeration, retry, or mutation.

Never retain candidate-sector IDs, owner IDs, names, macros, ware labels,
numbers, native errors, Lua tables, frames, or raw payloads. Do not inspect or
modify saves, run a report/model/command path, claim normal-speed or SETA
behavior, or perform component enumeration in this attempt.

Route only from the recorded classes. A selected-sector capacity or ware-limit
`call_error`, `wrong_type`, or `invalid_value` permits a separately planned,
bounded real-component-enumeration successor. Otherwise keep D-09 explicitly
unsupported; a contradiction requires its own bounded corrective plan.

### Recorded Capability-Vector Outcome — 2026-08-29

Attempt `obs-x4-d09-capability-20260829-03` completed the one-shot vector
without expanding the selected-sector scan. The retained X4 result classes were
`metadata_type=ok`, `owner_id_validity=ok`, `sector_capacity=call_error`, and
`first_cargo_ware_limit=not_applicable`; correlated bridge hello, heartbeat,
and runtime-health were accepted through sequence `3`.

Classify only the selected-sector capacity operation as an exact unsupported
call shape. Do not infer global absence of `GetPeopleCapacity`, a valid
real-component target, or scope-complete D-09 facts. The next investigation,
if separately authorized, is the procedure's bounded
real-component-enumeration successor; Plan 01-11 remains closed to component
enumeration.
