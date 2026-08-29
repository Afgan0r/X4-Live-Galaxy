# Phase 01 Disposable X4 Evidence Ledger

## Ledger Contract

Ledger version: `1`

This ledger records evidence without promoting a weaker source to a stronger
level. Add an attempt row only after the matching procedure has been run or
explicitly deferred. `observed-in-X4` requires all setup, expected readback,
observed readback, and health fields from the procedure.

<!-- markdownlint-disable MD013 -->

| Evidence ID | Hypothesis | Baseline evidence | Game status | Evidence level |
| --- | --- | --- | --- | --- |
| OBS-X4-01 | Typed runtime identity and explicit section state | `documented-static`; `fake-local` | pending | `pending-X4` |
| OBS-X4-02 | Bounded producer and backpressure at normal speed and SETA | `documented-static`; `fake-local` | pending | `pending-X4` |
| OBS-X4-03 | Capability handshake, compatible reconnect, and X4-restart mismatch | `documented-static`; `fake-local` | pending | `pending-X4` |
| OBS-X4-04 | Scope-complete runtime enumeration | `documented-static`; `fake-local` | pending | `pending-X4` |

<!-- markdownlint-enable MD013 -->

## Attempt Template

Copy this section once per attempt. Keep failed and pending entries; do not
replace them with a later stronger result.

### Evidence ID: `OBS-X4-XX`

| Field | Recorded value |
| --- | --- |
| Attempt number | pending |
| Exact X4 version | pending |
| Active mod list | pending |
| Extension build identity | pending |
| Protocol identity | pending |
| Creative Custom scenario | pending |
| Isolated hypothesis | pending |
| Real elapsed time | pending |
| Game elapsed time | pending |
| SETA state | pending |
| Expected readback | pending |
| Observed readback | pending |
| Bounded health | pending |
| Evidence level | `pending-X4` |
| Game result | pending |
| Failure diagnostics | pending |
| Save access or modification | `none` |
| Game-state effect, report, or acknowledgement path | `none` |

## Local Preflight Baseline — Plan 01-09 Task 1 — 2026-08-29

<!-- markdownlint-disable MD013 -->

| Check | Result | Evidence level |
| --- | --- | --- |
| X4 process state before checks | No `X4` or `x4-bridge` process was observed before the preflight checks. | `documented-static` |
| `01-install-guard.ps1 -VerifyPackageOnly` | Passed; the package graph, XML links, trace helpers, and `x4-bridge.exe` were verified. No installation was performed. | `documented-static` |
| Focused bridge contracts | `named_pipe_contract` (7), `reconnect_idempotency` (2), and `backpressure_contract` (4) passed. | `fake-local` |
| `cargo fmt --check` | Passed. | `fake-local` |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed. | `fake-local` |
| `cargo test --workspace` | Passed. | `fake-local` |
| Installation result | Not attempted. The guard intentionally exposes verification only and rejects any non-verification invocation. | `documented-static` |
| Bridge readiness | One compatible `x4-bridge` process (PID `40820`) owns `\\.\pipe\live_galaxy`; no X4 process is running. | `documented-static` |

<!-- markdownlint-enable MD013 -->

The bridge was started after all green local checks and is ready for the human
Creative Custom checkpoint. This baseline is local proof only: OBS-X4-01 through
OBS-X4-04 remain `pending-X4`. No disposable X4 run, save access, game-state
effect, report, acknowledgement, or model capability was exercised.

## Plan 01-10 Task 3 Pre-Step — Deployment and Syntax Probe Preparation — 2026-08-29

<!-- markdownlint-disable MD013 -->

| Field | Recorded value |
| --- | --- |
| Package guard | `powershell -NoProfile -File tests/x4-disposable/01-install-guard.ps1 -VerifyPackageOnly` passed before deployment. |
| X4 deployment gate | `Get-Process -Name X4` was rechecked inside the deployment operation immediately before copying; no process was found. |
| Deployment result | The prepared repository `extensions/live_galaxy` contents were copied to the installed `live_galaxy` extension directory. No vanilla or other extension was edited. |
| Verified deployed files | `content.xml` `AED0DB6A8AA0D15235FD6A324720A173A559BAE043EE0DBF99A81F7FE28A1A53`; `ui.xml` `25C3EB340FAC45F83292FC256B74ED5B91DB24FE54F6CDC250B9467C879DD34E`; `md/live_galaxy_observation.xml` `7ECB9D8FE33FBD8C13ED19E1FA2A6D200FB9722857E839A730DCF9DBB08A7515`; `lua/live_galaxy_runtime.lua` `D2147BC75459EFE91080278507DFF602853704B79A564362710175A57AC2111B`; `lua/live_galaxy_x4_discovery.lua` `4B69D9C63829841CE03D7A9023C7B61091FE656C4E3787A662B7178D4F2EAC78`; `lua/live_galaxy_telemetry.lua` `85B8D9C7DF3A1E5D2867FDCF0FD826C6D50A04DC5012089628EF950E7AD1E92D`; `lua/live_galaxy_trace_config.lua` `1D25EA31B633C618A685263D3D3305ACF34736C617C369EB4020863B4732EF28`. |
| Disposable syntax/version probe | Prepared but not run. A new Creative Custom session must establish embedded-loader syntax acceptance and record an X4-exposed interpreter version, if any. |
| Runner / bridge / OBS-X4-04 | Not started, selected, provisioned, or claimed. No runtime discovery proof is implied. |
| Save access or modification | `none` |
| Game-state effect, report, or acknowledgement path | `none` |

<!-- markdownlint-enable MD013 -->

## Plan 01-10 Loader Layout Follow-Up — 2026-08-29

The initial loader failure established that bare
`require("live_galaxy_x4_discovery")` searched only
`extensions/live_galaxy_x4_discovery.lua`, while the packaged source is
`extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua`. The first correction
resolved discovery, but the fresh follow-up probe then failed on bare
`require("live_galaxy_telemetry")` with the same flattened lookup. This confirms
that every local and transitive import must use
`live_galaxy/lua/<module>`, rather than relying on an extension-local fallback.

The source now requires runtime, discovery, telemetry, trace configuration, and
normalization through that extension-relative module form. The package guard
checks the runtime helper has no bare fallback and checks telemetry's
normalization import. This is `documented-static` evidence only; no deployment
or observed-in-X4 result is claimed.

Before the next disposable probe, close X4, rerun the package guard, deploy the
updated extension, and start a new Creative Custom session. If the loader still
rejects a module, retain its bounded diagnostic as a failed probe and do not
infer a different package layout.

## Plan 01-10 Embedded-Lua Loader Probe — 2026-08-29

This is earned `observed-in-X4` evidence for embedded-loader syntax/load
acceptance only. It is not an OBS-X4 attempt and does not select or provision a
standalone Lua runner.

<!-- markdownlint-disable MD013 -->

| Field | Recorded value |
| --- | --- |
| Probe scope | Fresh disposable X4 session; X4 PID `43708`, started `2026-08-29 18:29:01` local. |
| Game-log observations | At game time `9075.57`, `trace_config_loaded detail=status=disabled enabled=false` and `handler_registered detail=event=live_galaxy_observation` were recorded. |
| Loader syntax/load disposition | Accepted for this deployed extension: no `module not found`, parser/syntax, or runtime module-load error occurred in the bounded observed log. |
| Interpreter version | `not exposed` by the observed bounded X4 log. No Lua release is inferred from loader acceptance or host tooling. |
| Bridge and transport | No bridge was started. At game time `9076.91`, `pipe_write_failed detail=raw writer exhausted its reconnect attempt` is the expected bounded result for this loader-only probe and is not transport evidence. |
| OBS-X4 classification | None. OBS-X4-01 through OBS-X4-04 remain unchanged and `pending-X4`. |
| Runner-selection gate | Blocked: X4 did not expose an interpreter version, so no compatible standalone Lua target, lock, provisioner, download, or runner selection is established. |
| Save access or modification | `none` |
| Game-state effect, report, or acknowledgement path | `none` |

<!-- markdownlint-enable MD013 -->

Next action: with X4 closed, deploy the default-off Plan 01-10 one-event
`_VERSION` diagnostic and enable only `version_diagnostic_enabled` for a new
loader-only Creative Custom probe. Record only the existing-debug-log's
sanitized, at-most-64-byte value, or `not exposed` / `invalid`; do not start a
bridge or trigger telemetry. If the value is unavailable or cannot identify a
compatible standalone target, halt before selecting, provisioning, or
downloading a runner. This result must not be used to claim runtime discovery
or bridge transport behavior.

## Plan 01-10 Embedded-Lua `_VERSION` Diagnostic — 2026-08-29

This is earned `observed-in-X4` evidence for the embedded runtime's
self-reported version only. It unlocks selection of a standalone Lua 5.1
runner target; it does not establish an OBS-X4 transport, discovery, or fact
result.

<!-- markdownlint-disable MD013 -->

| Field | Recorded value |
| --- | --- |
| Probe scope | Fresh disposable X4 session; X4 PID `33016`, started `2026-08-29 18:45:03` local. |
| Game-log observations | At game time `9075.57`, `embedded_lua_version detail=Lua 5.1` was recorded, followed by handler registration. |
| Loader syntax/load disposition | Accepted for this deployed extension: the bounded observed log contains no parser/syntax or runtime module-load error. |
| Embedded interpreter version | `Lua 5.1`, self-reported by the embedded runtime. This is the compatibility target for a pinned standalone Lua 5.1 runner selection only. |
| Trace configuration boundary | The enabled trace configuration was a temporary installed-extension probe setting. The repository source default remains disabled and this run does not change normal-operation defaults. |
| Bridge and transport | No bridge was started. Any pipe failure is the expected non-transport outcome of this loader-only probe and is not transport evidence. |
| OBS-X4 classification | None. OBS-X4-01 through OBS-X4-04 remain unchanged and `pending-X4`. |
| Runner-selection gate | Unlocked only for selecting a pinned standalone Lua 5.1 target. Provisioning, download, and contract execution are subsequent work, not earned by this probe. |
| Save access or modification | `none` |
| Game-state effect, report, or acknowledgement path | `none` |

<!-- markdownlint-enable MD013 -->

Next action: select and verify a pinned standalone Lua 5.1 runner using the
Plan 01-10 lock/provisioning procedure. Do not use this result to claim a
runtime discovery fact, OBS-X4 result, or bridge transport behavior.

## Observed Runtime Transport Trace — 2026-08-29

This is earned `observed-in-X4` evidence for the bounded transport and
compatible bridge-restart path only. It does **not** qualify any `OBS-X4-01`
through `OBS-X4-04` row: normal-versus-SETA comparison, incompatible
restart-required behavior, and scope-complete real-runtime enumeration remain
untested.

<!-- markdownlint-disable MD013 -->

| Field | Recorded value |
| --- | --- |
| Attempt ID | `obs-x4-01-20260829-02` |
| Exact X4 version | `9.0.0.0` |
| Active mod list | `ws_2042901274`, `live_galaxy` |
| Session | User-operated disposable Creative Custom; exact start scenario was not captured; X4 PID `16312` remained running from `2026-08-29 16:00:25`. |
| Extension and protocol identity | `live_galaxy`; protocol major `1`; capability `live-galaxy-observation-v1` |
| Game elapsed time | Not captured for this trace. |
| Real elapsed time and SETA | Bridge readiness and reconnect starts: `16:05:26` and `16:08:01`; normal-versus-SETA comparison was not run, so this evidence cannot support OBS-X4-02. |
| Expected readback | One accepted hello and ordered heartbeat, runtime-health, observation, and completion-marker frames |
| Observed readback | After user-account bridge start, Lua generation `9` sent accepted `hello`, `heartbeat`, `runtime_health`, `observation`, and `complete_marker` frames for `sector:live_galaxy`. After compatible bridge restart, stale heartbeat generation `9`, sequence `5` was rejected; Lua then emitted generation `10` `hello`, and the replacement bridge accepted `hello`, `heartbeat`, `runtime_health`, `observation`, `complete_marker`, and heartbeat sequence `5`. |
| Bounded health | Lua reported accepted sends; bridge reported accepted frames. The sandbox bridge reported bounded failure `pipe_write_failed detail=writer exhausted its reconnect attempt`. No raw payload was retained. |
| Compatible bridge restart | Bridge PID `54396` under `AFGAN0R\\pavlo` was stopped while X4 continued; replacement PID `52744` under the same account began at `16:08:01`. No X4 restart occurred. |
| Restart diagnostics | Initial sandbox bridge PID `40820` (`CodexSandboxOffline`) could not write the pipe and failed as recorded above. The compatible restart proved a stale-generation rejection followed by a new generation; incompatible protocol, capability, or extension-replacement behavior was not tested. |
| Evidence level | `observed-in-X4` for the transport trace; formal Phase 1 probe rows remain `pending-X4` |
| Diagnostics | `runtime/phase1-bridge-obs-x4-01-20260829-02-reconnect.log`; `runtime/phase1-bridge-obs-x4-01-20260829-02-compatible-restart.log`; X4 debug log retained machine-locally. |
| Save access or modification | `none` |
| Game-state effect, report, or acknowledgement path | `none` |

<!-- markdownlint-enable MD013 -->

## Observed Cadence Sampling — 2026-08-29

This sampling belongs to attempt `obs-x4-01-20260829-02` and bridge PID
`52744` under `AFGAN0R\\pavlo`. It is earned `observed-in-X4` transport
evidence, but it does not qualify OBS-X4-02: the procedure requires a fresh,
isolated normal-versus-SETA scenario and an explicit queue or backpressure
disposition. No backpressure condition was induced in this sampling.

<!-- markdownlint-disable MD013 -->

| Window | Bridge acceptance range | Real elapsed | Game elapsed | Observed rate | Health and disposition |
| --- | --- | --- | --- | --- | --- |
| SETA enabled before `2026-08-29T16:16:27.709+07:00` | Generation `10`, sequences `17` through `28` | Approximately `51.9` seconds | Approximately `330` game seconds | Approximately `6.35x`; `11` accepted frames | No rejection, backpressure, or health fault observed. |
| SETA disabled before `2026-08-29T16:17:57.909+07:00` | Generation `10`, sequences `33` through `37` | Approximately `95.1` seconds | Approximately `120` game seconds | Approximately `1.26x`; `4` accepted frames | No rejection, backpressure, or health fault observed. |
| Post-SETA normal speed | Generation `10`, sequences `43` through `45` from `16:22:28` to `16:23:28` | `60` seconds | Not captured | `2` accepted frames | No rejection or backpressure event observed. |

<!-- markdownlint-enable MD013 -->

Diagnostics: `runtime/phase1-bridge-obs-x4-01-20260829-02-compatible-restart.log`.
Only bounded metadata was recorded. No save access, game-state effect, report,
acknowledgement, or model path was exercised.

## Classification Rules

- `observed-in-X4`: qualifying disposable game evidence includes all required
  fields and confirms the single hypothesis.
- `failed`: the attempt has a concrete mismatch, safety stop, or missing
  required runtime signal; preserve its diagnostics.
- `pending`: the attempt has not yet been run or did not collect enough data to
  classify a runtime outcome.
- `documented-static` and `fake-local`: baseline-only evidence; neither can be
  rewritten as an X4 observation.

## Plan 01-11 Task 3 Strict-v2 Runtime Preparation — 2026-08-29

Attempt ID: `obs-x4-d09-v2-20260829-01` is prepared and remains
`pending-X4`; no X4 session, deployment, save access, or state-affecting path
was attempted by this preparation.

<!-- markdownlint-disable MD013 -->

| Field | Recorded value |
| --- | --- |
| Required game-facing identity | `live_galaxy`; `live-galaxy-x4-build-2`; `live-galaxy-observation-v2` |
| Built bridge identity | `target/debug/x4-bridge.exe`; SHA-256 `C049947574DEFA23D50FEFAB6162BFDD948F43737CB005319DFC18D17C10F313` |
| Process readiness before build/preflight | No `X4` or `x4-bridge` process observed. |
| Build command | `cargo build -p x4-bridge` passed. |
| Package and Lua preflight | `01-install-guard.ps1 -VerifyPackageOnly` and `run_contracts.ps1 -Suite x4_discovery` passed. |
| Rust admission preflight | `runtime_facts_contract` (3), `named_pipe_contract` (7), and `reconnect_idempotency` (2) passed. |
| Deployment result | Not attempted; extension replacement is blocked until X4 is closed for the human-operated attempt. |
| Required restart boundary | v1-to-v2 extension replacement requires an X4 restart. Once v2 is live, one compatible Rust-only bridge restart must leave X4 running. |
| Runtime classification | `pending-X4` until the same isolated attempt correlates sector, asset, capacity, and ownership facts through Rust admission with explicit quality and bounded health. |
| Save access or modification | `none` |
| Game-state effect, report, acknowledgement, command, persistence, or model path | `none` |

<!-- markdownlint-enable MD013 -->

## Plan 01-11 Task 3 Strict-v2 Runtime Retry — 2026-08-29

Attempt `obs-x4-d09-v2-20260829-02` is earned `observed-in-X4` evidence for
the bounded v2 transport and compatible bridge-only restart. It is a failed
D-09 fact-enumeration attempt: every required fact class was explicitly
`facts_unsupported`, so no known-empty or scope-complete enumeration is
claimed.

<!-- markdownlint-disable MD013 -->

| Field | Recorded value |
| --- | --- |
| Attempt ID | `obs-x4-d09-v2-20260829-02` |
| Isolated hypothesis | A deployed strict-v2 X4 client can reach the v2 bridge, survive a compatible bridge-only restart, and distinguish unavailable discovery facts from a known-empty scope. |
| Lua/X4 trace | Generation `1` sent `hello`, `heartbeat`, `runtime_health`, explicit `facts_unsupported` health, and the complete marker. After the first bridge-only restart, one bounded `pipe_unavailable` occurred; generation `2` then sent `hello`, `heartbeat`, and `runtime_health` and continued to report `facts_unsupported`. After the second bridge-only restart with the standard `DebugSink`, generation `3` sent `hello`, `heartbeat`, and `runtime_health`. |
| Bridge receipt/admission trace | `runtime/phase1-bridge-obs-x4-d09-v2-20260829-02-debug-sink.log` records `listener_ready`, `client_connected`, and `frame_received disposition=Accepted` for generation `3` `hello` (134 bytes), `heartbeat` (86 bytes, protocol version `4`, sequence `1`), and `runtime_health` (112 bytes, protocol version `4`, sequence `2`). No raw payload content is retained. |
| Compatible bridge-only restart | Observed in X4. X4 continued through both bridge-only restarts; the first has one bounded `pipe_unavailable`, followed by generation `2`, and the second is correlated through the standard debug sink with generation `3` accepted frames. |
| Fact-class disposition | `facts_unsupported` for sector, asset, capacity, and ownership discovery. This is an explicit unavailable state, not an empty collection, known-empty scope, or accepted fact enumeration. |
| X4 game time | Not retained in the supplied bounded v2 evidence. The machine-local X4 debug log is the source for Lua-side runtime observations, but this ledger does not invent a game-time value. |
| Bridge receipt Unix time | Not retained in the supplied sanitized `DebugSink` evidence. Receipt/order and accepted dispositions are observed; no Unix receipt timestamp is invented. |
| Evidence classification | `observed-in-X4` for v2 transport, bounded degraded health, and compatible bridge-only restart; `failed` for D-09 scope-complete fact enumeration. OBS-X4-04 remains unresolved because all required fact classes are explicitly unavailable. |
| Unresolved adapter limitation / next action | The X4 discovery adapter does not currently expose the required sector, asset, capacity, and ownership facts in this runtime. Keep `facts_unsupported` explicit and investigate the read-only adapter capability for each class in a separately authorized disposable attempt; do not coerce it into known-empty data or broaden Phase 1 authority. |
| Save access or modification | `none` |
| Game-state effect, report, acknowledgement, command, persistence, or model path | `none` |

<!-- markdownlint-enable MD013 -->

Skill learning: none — considered the correlated Lua/X4 and `DebugSink` traces,
the bounded `pipe_unavailable` recovery, and explicit `facts_unsupported`
outcome against `live-galaxy-x4-integration` and `live-galaxy-x4-tests`. Those
skills already require correlated bounded diagnostics and prohibit promoting an
unavailable value to known-empty; this retry demonstrates their enforcement,
not a missing reusable rule.

## Plan 01-11 Task 3 Capability-Vector Probe — Pending

<!-- markdownlint-disable MD013 -->

| Field | Recorded value |
| --- | --- |
| Attempt ID | `pending` — set one nonempty, bounded configured ID only for the disposable attempt. |
| X4 version / active mods / scenario | `pending-X4` |
| Preflight | `pending`: project-local Lua contracts, package guard, and strict-v2 Rust contracts must pass before deployment. |
| Aggregate disposition | `pending`: retain the existing `facts_unsupported` result. |
| `metadata_type` | `pending`: one of `ok`, `call_error`, `wrong_type`, or `invalid_value`. |
| `owner_id_validity` | `pending`: one of `ok`, `call_error`, `wrong_type`, or `invalid_value`. |
| `sector_capacity` | `pending`: one of `ok`, `call_error`, `wrong_type`, or `invalid_value`. |
| `first_cargo_ware_limit` | `pending`: one of `ok`, `call_error`, `wrong_type`, `invalid_value`, or `not_applicable` only for no cargo entry. |
| Trace bound and stop state | `pending`: exactly one sanitized four-class vector, then disabled and stopped. |
| Save access or modification | `none` |
| Game-state effect, report, acknowledgement, command, persistence, or model path | `none` |
| Successor routing | `pending`: only a selected-sector capacity/ware `call_error`, `wrong_type`, or `invalid_value` may request a separately planned real-component-enumeration successor. |

<!-- markdownlint-enable MD013 -->

This pending row stores class labels only. It must not be replaced with raw
values, identifiers, names, macros, ware labels, native errors, Lua tables,
frames, payloads, or a second candidate result.

## Plan 01-11 Task 3 Capability-Vector Probe — Observed Result — 2026-08-29

Attempt `obs-x4-d09-capability-20260829-03` earned narrow
`observed-in-X4` evidence for the selected-sector capability classes only. It
does not qualify OBS-X4-04: the required scope-complete fact enumeration still
failed and component enumeration is outside Plan 01-11.

<!-- markdownlint-disable MD013 -->

| Field | Recorded value |
| --- | --- |
| Attempt ID | `obs-x4-d09-capability-20260829-03` |
| Isolated hypothesis | The default-off, one-shot diagnostic can separate the selected-sector discovery predicates without widening the scan or retaining raw runtime data. |
| Probe configuration | The repository source and installed configuration explicitly enabled this attempt's default-off one-shot probe. |
| Capability vector | `metadata_type=ok`; `owner_id_validity=ok`; `sector_capacity=call_error`; `first_cargo_ware_limit=not_applicable` |
| Correlated bridge readback | Hello, heartbeat, and runtime-health were accepted through sequence `3`. No raw payload is retained. |
| Fact classification | `sector_capacity=call_error` identifies the selected-sector capacity operation as an exact unsupported call shape. It does not establish global absence of that API. |
| OBS-X4-04 / D-09 status | `failed` for scope-complete fact enumeration; metadata and owner validation are not the failing members in this cycle; the no-cargo ware path is non-failure. |
| Successor boundary | Do not enumerate components under Plan 01-11. Only a separately authorized bounded real-component-enumeration plan may investigate a valid capacity target. |
| Exact X4 version, active mods, scenario, elapsed time, and SETA state | Not retained in the supplied sanitized capability-vector evidence; no value is inferred. |
| Trace bound and stop state | One four-class vector only; no raw identifiers, values, payloads, native error text, saves, or unbounded logs were retained. |
| Save access or modification | `none` |
| Game-state effect, report, acknowledgement, command, persistence, or model path | `none` |

<!-- markdownlint-enable MD013 -->

Skill learning: none — considered the one-shot class-only X4 vector, the
correlated accepted bridge frames, and the existing Plan 01-11 capability-vector
procedure against `live-galaxy-x4-integration` and `live-galaxy-x4-tests`.
They already require bounded correlated diagnostics, explicit unsupported
results, and separately authorized scope expansion; this evidence applies those
rules without establishing a missing reusable rule.
