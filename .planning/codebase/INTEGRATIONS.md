---
last_mapped_commit: 80881814c2b3753c50c9edc7cb79818ee7444dde
---

# External Integrations

**Analysis Date:** 2026-09-14

## APIs & External Services

**X4 game runtime:**

- X4 UI Lua supplies observation callbacks through `RegisterEvent`, `GetCurRealTime`, and `DebugError` in `extensions/live_galaxy/lua/live_galaxy_runtime.lua` and `extensions/live_galaxy/lua/live_galaxy_observation.lua`.
- Mission Director raises `live_galaxy_observation` on a recurring 50 ms cue and once at game load in `extensions/live_galaxy/md/live_galaxy_observation.xml`; the exact loader and lifecycle semantics remain evidence-scoped in `tests/x4-disposable/01-probe-evidence.md`.
- The active observation path reads one `GetCurRealTime` value per scheduled sample and gets its game-time clock through `ffi.C.GetCurrentGameTime` in `extensions/live_galaxy/lua/live_galaxy_observation.lua`.
  - SDK/Client: X4 embedded Lua plus LuaJIT-style `ffi`; no separate X4 SDK package is declared in `Cargo.toml`.
  - Auth: none; the active extension path observes X4-owned state and does not expose mutation commands.
- Station discovery calls (`GetNumAllFactionStations`, `GetAllFactionStations`, `GetPeopleCapacity`, `ConvertStringToLuaID`, `ConvertIDTo64Bit`, and `GetComponentData`) are implemented in `extensions/live_galaxy/lua/live_galaxy_component_discovery.lua` but are not wired into the active `extensions/live_galaxy/lua/live_galaxy_runtime.lua`; their runtime integration remains unwired/pending.

**Native Lua carrier:**

- Lua loads the packaged native image using `package.loadlib` and the `luaopen_live_galaxy_carrier` initializer in `extensions/live_galaxy/lua/live_galaxy_carrier.lua`.
- The Rust `cdylib` exports the initializer and an exact ten-operation API (`open`, section lifecycle, progress/control, reset, and close) in `crates/x4-carrier-native/src/lib.rs` and `crates/x4-carrier-native/src/lua_operations.rs`.
- The native carrier resolves X4's already-loaded Lua 5.1 symbols and binds the Lua state through Windows ABI calls in `crates/x4-carrier-native/src/abi_windows.rs`.

**Windows named-pipe transport:**

- The packaged native DLL creates `\\.\pipe\live_galaxy` with message-mode, duplex, overlapped I/O and a current-logon security descriptor in `crates/x4-carrier-native/src/transport.rs`, `crates/x4-carrier-native/src/abi_windows_io.rs`, and `crates/x4-carrier-native/src/abi_windows_security.rs`.
- The active external bridge is a `BridgePeer` client that connects to that DLL-owned server, receives bootstrap/data messages, and sends control/demand messages in `crates/x4-bridge/src/production_runtime.rs` and `crates/x4-carrier-native/src/transport_peer.rs`.
- `crates/x4-bridge/src/listener.rs` uses `interprocess`/`recvmsg` for a separately exported legacy contract listener; `crates/x4-bridge/src/main.rs` enters `run_production` and does not call that listener.
- Control and data envelopes are versioned, bounded, and validated by `crates/observation-ingest/src/wire.rs`, `crates/observation-ingest/src/wire_decode.rs`, `crates/x4-bridge/src/carrier_b_codec.rs`, and `crates/x4-bridge/src/protocol.rs`.
- Pipe handoff is intentionally distinct from semantic receipt and durable publication; the production receive/admission sequence is implemented in `crates/x4-bridge/src/production_runtime.rs` and `crates/x4-bridge/src/production_admission.rs`.

## Data Storage

**Databases:**

- SQLite database `observations.sqlite3` stores accepted revisions, receipts, current pointers, dependency pins, and ambiguity/recovery state through `rusqlite` in `crates/observation-persistence/src/sqlite.rs` and its `sqlite_*` modules.
  - Connection: `--data-dir` supplied to `x4-bridge.exe` in `crates/x4-bridge/src/production_startup.rs`.
  - Client: `rusqlite` 0.40.2 with bundled SQLite, configured in `crates/observation-persistence/Cargo.toml`.
- Normal production startup also requires `--limits-file`; `ProductionLimits::read` validates all 23 positive fields and maps them into generation, lifecycle, and publication limits in `crates/x4-bridge/src/production_limits.rs` and `crates/x4-bridge/src/production_startup.rs`.
- `--readback` bypasses the limits file but still requires `--data-dir`, `--section-key`, and `--section-revision`; it opens `<data-dir>/observations.sqlite3` with fixed limits of 4,096 records and 16 MiB content in `crates/x4-bridge/src/diagnostics.rs`.
- Mind checkpoints and capsules are typed JSON structures with fake/port abstractions in `crates/mind-persistence/src/` and are not wired to a production external database in the current bridge startup path.

**File Storage:**

- Local filesystem only for the bridge database, mandatory operational history, optional legacy-listener debug evidence, package manifests, and development fixtures in `crates/x4-bridge/src/diagnostics.rs`, `crates/x4-bridge/src/operational_history.rs`, `crates/x4-bridge/src/listener.rs`, and `shadow-deliberation-evals/v1/`.
- X4 checkpoint storage is an extension-scoped Mission Director variable, declared by `extensions/live_galaxy/md/live_galaxy_persistence.xml` and described by `extensions/live_galaxy/checkpoint_schema.json`; save/load runtime proof is still pending.

**Caching:**

- No network or distributed cache is configured. Provider-relative cache identity and replay constraints exist as domain contracts in `crates/mind-domain/src/cache_identity.rs`; current model execution remains explicit/manual.

## Authentication & Identity

**Auth Provider:**

- Windows current-logon ACL and remote-client rejection protect the named pipe in `crates/x4-carrier-native/src/abi_windows_security.rs`; this is local process authorization, not user authentication.
- Transport identity uses session, producer incarnation, transport epoch, sequence, and digest fields in `crates/observation-ingest/src/wire.rs` and `crates/x4-carrier-native/src/producer_types.rs`.
- No OAuth, API-key, account, or external identity provider is implemented in the current checkout; transport and producer identities are assembled across `crates/observation-ingest/src/wire.rs` and `crates/x4-carrier-native/src/producer_types.rs`.

## Monitoring & Observability

**Error Tracking:**

- No hosted error tracker is configured. Active production diagnostics require `OperationalHistory::open` to create and accept an initial journal record in `crates/x4-bridge/src/production_startup.rs` and `crates/x4-bridge/src/operational_history.rs`.

**Logs:**

- The active bridge records bounded operational states, reasons, session/epoch/message context, suppression counts, and history gaps through `crates/x4-bridge/src/operational_history.rs` and `crates/x4-bridge/src/operational_status.rs`.
- Bounded frame summaries and `LIVE_GALAXY_DEBUG_EVIDENCE_PATH` are limited to the legacy listener path in `crates/x4-bridge/src/listener.rs`.
- Lua emits deduplicated safe diagnostic classes through `DebugError` in `extensions/live_galaxy/lua/live_galaxy_runtime.lua`; raw prompts, payloads, and secrets are excluded by design.

## CI/CD & Deployment

**Hosting:**

- No hosted runtime is configured. The intended deployment is a local X4 extension plus local Windows bridge packaged by `tools/package-live-galaxy.ps1`.

**CI Pipeline:**

- GitHub Actions workflow `.github/workflows/phase-05.yml` runs on `windows-latest`, checks formatting and Clippy, executes focused/full Rust tests, checks the shadow harness, and runs the Rust source-size lint.
- Lua and XML checks are repository PowerShell workflows under `extensions/live_galaxy/tests/run_contracts.ps1`; no CI job currently publishes a release artifact.

## Environment Configuration

**Required env vars:**

- None are required for normal bridge startup; command-line `--data-dir` and `--limits-file` are required by `crates/x4-bridge/src/production_startup.rs`.
- Optional `LIVE_GALAXY_DEBUG_EVIDENCE_PATH` and `LIVE_GALAXY_TRACE_ATTEMPT_ID` enable bounded legacy-listener diagnostics in `crates/x4-bridge/src/listener.rs`; they are not the active production history sink.
- Optional `LIVE_GALAXY_C_COMPILER` selects the approved local compiler for `tools/provision-lua.ps1`.

**Secrets location:**

- No secret store or committed credential integration is detected in the workspace manifests or bridge startup path (`Cargo.toml`, `crates/x4-bridge/src/production_startup.rs`). Environment and credential files are intentionally outside this map.

## Webhooks & Callbacks

**Incoming:**

- X4 Mission Director events call the Lua callback registered by `extensions/live_galaxy/lua/live_galaxy_runtime.lua`; the callback accepts `telemetry_tick` and `telemetry_game_loaded` boundaries.

**Outgoing:**

- The native DLL's transport worker sends bounded observation messages and receives controls through the local Windows named pipe in `crates/x4-carrier-native/src/transport_worker.rs`.
- The external bridge's `BridgePeer` client sends handshake, collection intent, demand, and bounded control responses back over that pipe in `crates/x4-bridge/src/production_runtime.rs`; no game-state mutation command is exposed.
- No HTTP webhook, cloud queue, email, or third-party callback endpoint is implemented in `crates/x4-bridge/src/production_runtime.rs` or the extension Lua modules.

**Model-provider seam:**

- `mind-orchestration` defines the `ShadowProvider` trait and typed provider metadata in `crates/mind-orchestration/src/provider_port.rs`.
- The concrete benchmark process is `CodexProcess`, which invokes `codex exec --json --output-schema` through the generic `BenchmarkProcess` seam in `tools/shadow-harness/src/process.rs`; `tools/shadow-harness/src/subscription_adapter.rs` adapts that process and returns `Unavailable` when no explicit process is supplied.
- There is no OpenAI/Anthropic SDK or public API runtime integration in `Cargo.toml`; model-provider behavior is manual harness evidence in `tools/shadow-harness/src/process.rs` and remains separate from production X4 observation transport.

---

*Integration audit: 2026-09-14*
