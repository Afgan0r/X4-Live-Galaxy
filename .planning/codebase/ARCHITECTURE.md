---
last_mapped_commit: 80881814c2b3753c50c9edc7cb79818ee7444dde
---
<!-- refreshed: 2026-09-14 -->

# Architecture

**Analysis Date:** 2026-09-14

## System Overview

```text
Active production flow:
X4 MD cues -> Lua runtime/scheduler -> Carrier B DLL -> Windows named pipe
-> x4-bridge startup/runtime -> observation lifecycle -> ingest/domain checks
-> SQLite revisions, receipts, and current pointers.

Separate benchmark flow:
fixture frames -> admit_batch + derive_packets -> DeliberationRequest
-> DeliberationRunner -> ShadowProvider -> mind_domain::admit
-> redacted benchmark evidence.

Capability graph (not an active benchmark chain):
AcceptedProposal::pending_commit -> mind_domain::transition -> pending mind
commit -> mind-persistence checkpoint APIs.
```

The repository implements a layered, observation-only Rust workspace with a
game-facing Lua package and a Windows native carrier. X4 remains the authority
for game state. The production bridge accepts only bounded, typed observation
messages and publishes immutable observation revisions. The current runtime
sample is the `GetCurRealTime` value in
`extensions/live_galaxy/lua/live_galaxy_observation.lua`.

The architecture contains two distinguishable Rust paths. The active production
path starts in `crates/x4-bridge/src/production_startup.rs` and uses
`x4_carrier_native::BridgePeer` through
`crates/x4-bridge/src/production_runtime.rs`. The exported
`PipeServer`/`run_windows_listener` path in
`crates/x4-bridge/src/server.rs` and `crates/x4-bridge/src/listener.rs` is a
separate in-process protocol surface used by contract tests; `main` does not
call it. `CarrierBFacade` in `crates/x4-bridge/src/carrier_b.rs` is another
facade-level contract surface and is not used by the production runtime.

The strategic policy and mind crates implement typed packet, proposal,
admission, initiative, checkpoint, and provider abstractions. They are reached
by `tools/shadow-harness/src/benchmark.rs`, not by
`crates/x4-bridge/src/main.rs`. Current code therefore does not prove a
production observation-to-mind-to-provider flow.

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| X4 Mission Director seam | Raises game-start, game-loaded, and recurring telemetry events | `extensions/live_galaxy/md/live_galaxy_observation.xml` |
| Lua runtime | Initializes adapters, guards re-entry, handles boundary markers, and dispatches ticks | `extensions/live_galaxy/lua/live_galaxy_runtime.lua` |
| Lua scheduler | Polls carrier progress/control, captures one bounded fact, and closes or fails a section | `extensions/live_galaxy/lua/live_galaxy_scheduler.lua` |
| Lua telemetry helper | Provides normalized/serialized telemetry for an alternate helper and contract path; the active runtime does not load it | `extensions/live_galaxy/lua/live_galaxy_telemetry.lua` and `extensions/live_galaxy/lua/live_galaxy_normalize.lua` |
| Native producer | Copies Lua evidence, builds start/batch/completion messages, and retains exact retry bytes | `crates/x4-carrier-native/src/producer.rs` and `crates/x4-carrier-native/src/producer_message.rs` |
| Native transport | Owns bounded asynchronous duplex pipe I/O and connection generations | `crates/x4-carrier-native/src/transport.rs` and `crates/x4-carrier-native/src/transport_worker.rs` |
| Bridge runtime | Connects, negotiates, receives complete messages, admits dispositions, and sends demand | `crates/x4-bridge/src/production_runtime.rs` |
| Bridge session | Combines lifecycle, generation staging, revision index, and repository access | `crates/x4-bridge/src/production.rs` |
| Observation lifecycle | Coordinates stop-and-wait slot, context validation, private candidates, and publication | `crates/observation-application/src/lifecycle.rs` |
| Observation ingest | Decodes wire data, stages keyed candidates, verifies completion certificates, and tracks authority | `crates/observation-ingest/src/` |
| Observation repository | Stores immutable revisions, receipts, current pointers, pins, ambiguity, and retention state | `crates/observation-persistence/src/` |
| Operational diagnostics | Writes bounded history and status events with redacted identities | `crates/x4-bridge/src/operational_history.rs` and `crates/x4-bridge/src/operational_status.rs` |
| Strategic packet derivation | Converts accepted projections into faction-visible ZYA/ARG packets | `crates/strategic-state/src/derive.rs` |
| Mind transition | Validates proposals and creates pending typed mind commits | `crates/mind-domain/src/admission.rs` and `crates/mind-domain/src/mind.rs` |
| Provider orchestration | Invokes a `ShadowProvider`, classifies degradation, and records redacted evidence | `crates/mind-orchestration/src/runner.rs` |
| Shadow harness | Runs explicit benchmark fixtures through the mind/provider path | `tools/shadow-harness/src/benchmark.rs` |

## Pattern Overview

**Overall:** Layered ports-and-adapters composition around a bounded,
replication-shaped observation lifecycle.

**Key Characteristics:**

- X4/Lua owns collection and callback-local work; Rust owns protocol assembly,
  receiver validation, persistence, and recovery.
- Observation data remains private while staged and becomes current only after
  certificate, dependency, authority, and durable publication checks.
- Transport, producer, lifecycle, and repository expose typed outcomes for
  capacity, receipt, commit, rejection, ambiguity, and stale identity.
- Strategic state and mind orchestration are pure or dependency-injected domain
  surfaces, currently exercised through the benchmark harness.

## Layers

**X4 package and event layer:**

- Purpose: Connect X4 lifecycle cues to Lua and declare the extension package.
- Location: `extensions/live_galaxy/`
- Contains: `content.xml`, `ui.xml`, Mission Director XML, Lua modules, and
  package-local tests.
- Depends on: X4 event registration and the native module loader.
- Used by: X4's UI/MD loader and the Lua runtime callback.

**Lua collection layer:**

- Purpose: Perform bounded, cooperative reads and preserve explicit quality,
  coverage, and source-boundary evidence.
- Location: `extensions/live_galaxy/lua/`
- Contains: `live_galaxy_scheduler.lua`, the active realtime observation adapter,
  plus alternate telemetry/normalization and component-discovery helpers.
- Depends on: Injected adapters or X4 globals at the runtime boundary.
- Used by: `live_galaxy_runtime.lua`.

**Native carrier layer:**

- Purpose: Bridge Lua to the external process without retaining Lua/X4 pointers.
- Location: `crates/x4-carrier-native/src/`
- Contains: C ABI resolver, Lua operation registrations, producer state,
  message assembly, transport facade, Windows worker, and security helpers.
- Depends on: `observation-domain`, `observation-ingest`, and Windows APIs on
  the Windows target.
- Used by: `package.loadlib` in `live_galaxy_carrier.lua`.

**Bridge and application layer:**

- Purpose: Own process startup, session negotiation, complete-message admission,
  context assembly, lifecycle sequencing, and reconnect behavior.
- Location: `crates/x4-bridge/src/`
- Contains: `main.rs`, startup, production runtime, admission, carrier codec,
  diagnostics, and session/protocol types.
- Depends on: all observation crates and `x4-carrier-native`.
- Used by: the `x4-bridge` binary and integration callers.

**Observation domain/ingest layer:**

- Purpose: Represent identities, source evidence, section quality, wire envelopes,
  candidate limits, certificates, versions, and acceptance state.
- Location: `crates/observation-domain/src/` and
  `crates/observation-ingest/src/`
- Contains: dependency-free domain records plus serde/digest-backed decoding,
  staging, scheduling, and eligibility modules.
- Depends on: `serde`, `serde_json`, `sha2` only in ingest as declared by
  `crates/observation-ingest/Cargo.toml`.
- Used by: application, persistence, bridge, native carrier, and tests.

**Persistence layer:**

- Purpose: Commit immutable revisions and receipts atomically in SQLite and
  reconcile unknown outcomes.
- Location: `crates/observation-persistence/src/`
- Contains: repository port, schema, SQLite read/write/publish/reconcile files,
  retention, pins, hydration, and fake repository.
- Depends on: `rusqlite` with bundled SQLite.
- Used by: `observation-application` and `x4-bridge`.

**Strategic and mind layer:**

- Purpose: Derive faction-visible packets, schedule deliberation, validate
  typed proposals, transition mind aggregates, and persist checkpoints.
- Location: `crates/strategic-state/src/`, `crates/mind-domain/src/`,
  `crates/mind-orchestration/src/`, and `crates/mind-persistence/src/`.
- Contains: pure domain policy, provider ports, checkpoint envelopes, recovery,
  and typed initiative state.
- Depends on: observation projection types and local domain crates.
- Used by: `tools/shadow-harness/src/benchmark.rs`; no bridge dependency exists.

## Data Flow

### Primary Request Path

1. X4's game-start and game-loaded cues, plus the recurring 50 ms cue, raise
   `live_galaxy_observation` (`extensions/live_galaxy/md/live_galaxy_observation.xml`).
2. `RegisterEvent` invokes `runtime.dispatch`, which lazily initializes the
   carrier and realtime observation adapter (`extensions/live_galaxy/lua/live_galaxy_runtime.lua:74`).
3. `scheduler.tick` polls native progress/control, begins a section, captures
   `GetCurRealTime`, pushes one typed record, and finishes the section
   (`extensions/live_galaxy/lua/live_galaxy_scheduler.lua:22`).
4. The native operation table in `crates/x4-carrier-native/src/lua_operations.rs`
   routes calls to `lua_producer_operations.rs`; `Producer` assembles start,
   batch, and completion bytes in `producer_message.rs`.
5. `NativeTransport::try_send` hands one bounded message to its Windows worker,
   which owns connect/read/write/reconnect operations in
   `crates/x4-carrier-native/src/transport_worker.rs`.
6. `run_production` loads validated limits, opens operational history and the
   SQLite-backed observation session, then enters the reconnect loop
   (`crates/x4-bridge/src/production_startup.rs:30`).
7. `production_runtime::serve` decodes the bootstrap, sends handshake,
   collection intent, and demand, then receives complete messages
   (`crates/x4-bridge/src/production_runtime.rs:42`).
8. `production_admission::admit` verifies carrier identity, decodes the message,
   binds diagnostic identity, submits it to `ProductionObservationSession`, and
   sends the typed disposition back (`crates/x4-bridge/src/production_admission.rs:49`).
9. `ObservationLifecycle` validates context, stages the immutable message in a
   stop-and-wait slot, and routes section start, batch, or completion to
   `GenerationStager` (`crates/observation-application/src/lifecycle.rs:39`).
10. Completion recomputes batch and content digests, checks coverage, versions,
    dependencies, and current pointers, then prepares an authoritative revision
    (`crates/observation-application/src/lifecycle_publication.rs:49`).
11. `SqliteObservationRepository` writes revision content, receipt, and the
    current pointer in an immediate transaction; ambiguous commit outcomes are
    retained for reconciliation (`crates/observation-persistence/src/sqlite_write.rs:8`).
12. The bridge returns `received`, `committed`, or a typed negative disposition;
    the native producer advances its exact pending message or enters a bounded
    retry/degraded state (`crates/x4-carrier-native/src/producer_feedback.rs:1`).

### Shadow Mind Harness Flow

1. `run_case` parses one fixture, obtains its faction, and creates a provider
   adapter (`tools/shadow-harness/src/benchmark.rs:39`).
2. `fixture.request()` calls `admit_batch` on fixture frames, then calls
   `derive_packets` and selects the faction packet before constructing the
   `DeliberationRequest` (`tools/shadow-harness/src/benchmark_fixture.rs:43`).
3. `run_case` creates a `DeliberationScheduler`, marks a strategic-tick
   eligibility check, and calls `DeliberationRunner::run`
   (`tools/shadow-harness/src/benchmark.rs:52`).
4. `DeliberationRunner::run` invokes the injected `ShadowProvider`, then calls
   `mind_domain::admit` on provider bytes and returns `RunnerOutcome`; the
   harness accepts only `AdmissionDecision::Accepted` and emits redacted
   evidence (`crates/mind-orchestration/src/runner.rs:37`).
5. `mind_domain::transition` is reachable from
   `AcceptedProposal::pending_commit` (`crates/mind-domain/src/admission.rs:56`),
   but `run_case` and `DeliberationRunner` do not call that method. The
   `mind-persistence` checkpoint APIs are likewise available capabilities, not
   calls made by this benchmark or by `crates/x4-bridge/src/main.rs`.

**State Management:**

- The Lua runtime owns initialization, callback re-entry, and pending game/load
  boundaries in module-local state (`extensions/live_galaxy/lua/live_galaxy_runtime.lua`).
- The native carrier owns one producer, one transport, a handle registry, and
  connection generation state; process-global `Mutex` values are declared in
  `crates/x4-carrier-native/src/abi.rs`.
- The bridge keeps one volatile stop-and-wait slot and keyed generation
  candidates, while accepted revisions and receipts live in SQLite.
- `DecisionRevisionIndex` is restored from current SQLite revisions on session
  startup (`crates/observation-application/src/lifecycle_restore.rs`).
- Mind checkpoints retain the last acknowledged envelope and validate candidate
  identity in `crates/mind-persistence/src/recovery.rs`.

## Key Abstractions

**Complete observation message:**

- Purpose: Encapsulates section start, immutable batch, completion, and control
  envelopes with explicit typed identities.
- Examples: `crates/observation-domain/src/observation.rs`,
  `crates/observation-ingest/src/wire_decode.rs`.
- Pattern: Decode raw serde shapes at the boundary, then construct validated
  domain values.

**Generation stager:**

- Purpose: Own private per-section candidates, independent candidate/aggregate
  bounds, and completion certificate validation.
- Examples: `crates/observation-ingest/src/generation.rs` and
  `crates/observation-ingest/src/completion.rs`.
- Pattern: Keep incomplete candidates private; publish only a validated revision.

**Observation repository port:**

- Purpose: Abstract publication, current reads, decision pins, and unpinning.
- Examples: `crates/observation-persistence/src/port.rs`,
  `crates/observation-persistence/src/sqlite.rs`.
- Pattern: Application code depends on the trait; SQLite owns transaction and
  recovery details.

**Carrier operation contract:**

- Purpose: Define the exact ten Lua-callable operations and ABI version.
- Examples: `crates/x4-carrier-native/src/lib.rs` and
  `extensions/live_galaxy/lua/live_galaxy_carrier.lua`.
- Pattern: Validate operation shape and version before opening a producer handle.

**Faction-visible strategic packet:**

- Purpose: Limit ZYA/ARG minds to visible facts, three institution views, policy,
  and replay/admission fingerprints.
- Examples: `crates/strategic-state/src/packet.rs` and
  `crates/strategic-state/src/derive.rs`.
- Pattern: Pure derivation from an accepted projection with explicit
  inaccessible/unknown/stale/unsupported availability.

## Entry Points

**X4 Lua registration:**

- Location: `extensions/live_galaxy/lua/live_galaxy_runtime.lua`
- Triggers: `RegisterEvent("live_galaxy_observation", dispatch)` from MD cues.
- Responsibilities: Initialize Carrier B and observation modules, guard callback
  re-entry, and dispatch the scheduler.

**Native Lua initializer:**

- Location: `crates/x4-carrier-native/src/lib.rs:110`
- Triggers: `package.loadlib` with `luaopen_live_galaxy_carrier`.
- Responsibilities: Resolve Lua host symbols and return the registered ABI table.

**Bridge executable:**

- Location: `crates/x4-bridge/src/main.rs`
- Triggers: Process startup with `--data-dir` and `--limits-file`, or readback
  arguments handled by `production_startup.rs`.
- Responsibilities: Start the production bridge and print explicit readback
  output when requested.

**Shadow benchmark executable:**

- Location: `tools/shadow-harness/src/main.rs`
- Triggers: `--benchmark --corpus <path>`.
- Responsibilities: Run explicit fixture/provider cases and emit redacted
  evidence records.

## Architectural Constraints

- **Threading:** Lua collection is callback-local and cooperative; Carrier B
  uses a Windows worker thread for pipe I/O (`crates/x4-carrier-native/src/transport_worker.rs`).
- **Global state:** Carrier ABI, registry, transport, and producer are static
  mutex-backed values in `crates/x4-carrier-native/src/abi.rs`; bridge session
  state is owned by `ProductionObservationSession`.
- **Circular imports:** No known production crate cycle; Cargo dependencies flow
  from bridge/application toward domain and persistence, while mind crates are a
  separate graph.
- **Platform:** Native transport is available only on Windows; the non-Windows
  implementation returns `TransportError::Unavailable` in
  `crates/x4-carrier-native/src/transport.rs`.
- **Source size:** Rust source is capped at 200 physical lines by
  `tools/source-size-lint/src/main.rs`; modules are split by responsibility.
- **Authority:** X4 owns game state; no current bridge command mutates X4. Model
  output, where exercised by the harness, remains outside the accepted mind
  state until typed admission.

## Anti-Patterns

### Treating test protocol surfaces as production composition

**What happens:** `PipeServer` and `run_windows_listener` in
`crates/x4-bridge/src/server.rs` and `crates/x4-bridge/src/listener.rs` expose a
separate in-process path, while production uses Carrier B's `BridgePeer`.
**Why it's wrong:** Wiring callers to the test-oriented path would bypass the
native producer and its bounded worker contract.
**Do this instead:** Extend `crates/x4-bridge/src/production_runtime.rs` and
`crates/x4-carrier-native/src/` when changing the active carrier path; preserve
the facade/server APIs as explicitly separate compatibility surfaces.

### Claiming strategic minds are fed by the bridge

**What happens:** `strategic-state`, `mind-domain`, `mind-orchestration`, and
`mind-persistence` are not dependencies of `crates/x4-bridge/Cargo.toml` and are
used by `tools/shadow-harness/src/benchmark.rs`.
**Why it's wrong:** A document or feature that assumes automatic production
observation-to-mind wiring would describe planned composition as implemented.
**Do this instead:** Add bridge composition only through an approved phase and
trace an accepted projection into packet derivation, then typed admission and
checkpoint persistence.

### Treating component discovery as the current runtime source

**What happens:** `extensions/live_galaxy/lua/live_galaxy_component_discovery.lua`
implements bounded station enumeration, but `live_galaxy_runtime.lua` creates
`live_galaxy_observation.lua` directly and never loads discovery.
**Why it's wrong:** The discovery module's source/API work is not evidence that
station observations are currently produced by the installed runtime.
**Do this instead:** Add a composed adapter only with the phase's exact source
contract and verify the resulting Lua → DLL → bridge flow.

## Error Handling

**Strategy:** Fail closed at each boundary, retain the last accepted revision,
and distinguish volatile receipt, durable commit, terminal rejection, and
ambiguous publication.

**Patterns:**

- Lua uses `pcall`, explicit result-shape checks, and bounded safe diagnostic
  classes (`extensions/live_galaxy/lua/live_galaxy_runtime.lua`).
- Native ABI wrappers catch panics and return integer status codes; producer and
  transport expose typed failures (`crates/x4-carrier-native/src/lua_operations.rs`).
- Bridge admission maps decode, identity, lifecycle, storage, and response-loss
  errors to bounded reason codes (`crates/x4-bridge/src/production_admission.rs`).
- SQLite marks ambiguous publications before attempting a transaction and
  reconciles exact revision/receipt/pointer state (`crates/observation-persistence/src/sqlite_publish.rs`).

## Cross-Cutting Concerns

**Logging:** `OperationalHistory` writes bounded JSONL history and a separate
status surface, suppresses identical events, and reports gaps to stderr
(`crates/x4-bridge/src/operational_history.rs`).

**Validation:** Raw Lua input and JSON remain outside trusted state until exact
shape, identity, size, version, digest, dependency, and current-pointer checks
complete (`crates/observation-ingest/src/wire_decode.rs` and
`crates/observation-application/src/lifecycle_publication.rs`).

**Authentication:** Carrier B validates a session/producer/transport identity
and uses Windows current-logon pipe security in
`crates/x4-carrier-native/src/abi_windows_security.rs`; this is local transport
peer admission, not user authentication or a model authorization system.

---

*Architecture analysis: 2026-09-14*
