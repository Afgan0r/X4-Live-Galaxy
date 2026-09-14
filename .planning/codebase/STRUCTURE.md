---
last_mapped_commit: 80881814c2b3753c50c9edc7cb79818ee7444dde
---

# Codebase Structure

**Analysis Date:** 2026-09-14

## Directory Layout

```text
live-galaxy/
├── Cargo.toml                         # Rust workspace and shared lints
├── rust-toolchain.toml                # Pinned Rust toolchain
├── crates/
│   ├── observation-domain/             # Dependency-free observation values
│   ├── observation-ingest/             # Wire decoding, staging, certificates
│   ├── observation-persistence/        # SQLite repository and retention
│   ├── observation-application/        # Lifecycle orchestration
│   ├── strategic-state/                # Faction visibility and packets
│   ├── mind-domain/                    # Typed shadow mind policy and state
│   ├── mind-orchestration/             # Provider boundary and runner
│   ├── mind-persistence/               # Checkpoint envelopes and recovery
│   ├── x4-carrier-native/              # Lua C ABI and Windows pipe carrier
│   └── x4-bridge/                      # External bridge binary
├── extensions/live_galaxy/
│   ├── content.xml                     # X4 extension manifest
│   ├── ui.xml                          # UI Lua registration
│   ├── md/                             # Mission Director event cues
│   ├── lua/                            # Runtime, adapters, and normalizers
│   └── tests/                          # Lua/package contract suites
├── tools/
│   ├── shadow-harness/                 # Explicit shadow-provider benchmark
│   ├── source-size-lint/               # Rust size and Cargo check runner
│   ├── package-live-galaxy.ps1         # Release/package assembly
│   └── provision-lua.ps1               # Locked Lua/Busted provisioning
├── config/                             # Checked-in bounded runtime limits
├── runbooks/                           # Operator procedures
├── tests/                              # Cross-component fixtures and probes
├── shadow-deliberation-evals/          # Versioned mind evaluation fixtures
├── docs/                               # Target architecture and decision ledger
├── .planning/                          # GSD state, phases, and maps
└── dist/                               # Generated package output
```

## Directory Purposes

**`crates/observation-domain/`:**

- Purpose: Own dependency-free identities and semantic observation records.
- Contains: `identity.rs`, `observation.rs`, `section.rs`, `section_state.rs`,
  `sender_evidence.rs`, and source session values.
- Key files: `crates/observation-domain/src/lib.rs` is the public surface;
  `crates/observation-domain/src/identity.rs` owns typed IDs and versions.

**`crates/observation-ingest/`:**

- Purpose: Convert wire bytes into validated, bounded observation revisions.
- Contains: wire serde records, digest encoding, generation candidates,
  completion certificates, scheduler admission, feedback slots, and authority.
- Key files: `crates/observation-ingest/src/wire.rs`,
  `crates/observation-ingest/src/generation.rs`,
  `crates/observation-ingest/src/completion.rs`, and
  `crates/observation-ingest/src/eligibility.rs`.

**`crates/observation-application/`:**

- Purpose: Sequence receiver admission, context checks, private staging, and
  durable publication through one lifecycle owner.
- Contains: `lifecycle.rs` plus focused modules for publication, reconciliation,
  queries, restoration, and public lifecycle types.
- Key files: `crates/observation-application/src/lifecycle.rs` and
  `crates/observation-application/src/lifecycle_publication.rs`.

**`crates/observation-persistence/`:**

- Purpose: Persist accepted revisions and make commit/recovery semantics explicit.
- Contains: repository port, SQLite schema, read/write paths, publication
  receipts, ambiguity reconciliation, retention, pins, hydration, and fakes.
- Key files: `crates/observation-persistence/src/sqlite.rs`,
  `crates/observation-persistence/src/sqlite_write.rs`, and
  `crates/observation-persistence/src/schema.rs`.

**`crates/x4-bridge/`:**

- Purpose: Compose the production external bridge and expose protocol types.
- Contains: `main.rs`, startup argument parsing, reconnect runtime, carrier
  codec, production session, admission, diagnostics, history, and an older
  `PipeServer`/listener surface.
- Key files: `crates/x4-bridge/src/production_startup.rs`,
  `crates/x4-bridge/src/production_runtime.rs`,
  `crates/x4-bridge/src/production.rs`, and
  `crates/x4-bridge/src/production_admission.rs`.

**`crates/x4-carrier-native/`:**

- Purpose: Build the `cdylib` loaded by X4 Lua and own Carrier B transport.
- Contains: Lua ABI resolution and operations, handle registry, producer and
  message assembly, Windows security/IO helpers, and asynchronous transport
  worker modules.
- Key files: `crates/x4-carrier-native/src/lib.rs`,
  `crates/x4-carrier-native/src/abi.rs`,
  `crates/x4-carrier-native/src/producer.rs`, and
  `crates/x4-carrier-native/src/transport.rs`.

**`extensions/live_galaxy/`:**

- Purpose: Package the X4 side of the observation prototype.
- Contains: manifest, UI Lua entrypoint, MD event cues, the active runtime and
  realtime observation adapter, alternate telemetry/normalization helpers,
  component discovery modules, and local contract tests.
- Key files: `extensions/live_galaxy/ui.xml`,
  `extensions/live_galaxy/md/live_galaxy_observation.xml`, and
  `extensions/live_galaxy/lua/live_galaxy_runtime.lua`.

**`crates/strategic-state/`, `crates/mind-domain/`,
`crates/mind-orchestration/`, and `crates/mind-persistence/`:**

- Purpose: Implement the typed shadow-director model boundary.
- Contains: faction profiles, visible snapshots, primitive policy, proposal
  admission, mind transitions, provider outcomes, checkpoints, and recovery.
- Key files: `crates/strategic-state/src/derive.rs`,
  `crates/mind-domain/src/admission.rs`,
  `crates/mind-orchestration/src/runner.rs`, and
  `crates/mind-persistence/src/checkpoint.rs`.
- Current composition: These crates are consumed by
  `tools/shadow-harness/src/benchmark.rs`; they are not imported by
  `crates/x4-bridge/Cargo.toml`.

**`tools/`:**

- Purpose: Keep developer-only validation, benchmark, packaging, and provisioning
  workflows outside runtime crates.
- Contains: Rust binaries/libraries under `tools/shadow-harness/` and
  `tools/source-size-lint/`, plus PowerShell tooling.
- Key files: `tools/shadow-harness/src/main.rs`,
  `tools/shadow-harness/src/benchmark.rs`, and
  `tools/source-size-lint/src/main.rs`.

## Key File Locations

**Entry Points:**

- `crates/x4-bridge/src/main.rs`: External `x4-bridge` process entrypoint.
- `crates/x4-carrier-native/src/lib.rs`: Exported
  `luaopen_live_galaxy_carrier` initializer.
- `extensions/live_galaxy/lua/live_galaxy_runtime.lua`: X4 Lua event
  registration and callback dispatch.
- `extensions/live_galaxy/md/live_galaxy_observation.xml`: X4 game-start,
  game-loaded, and recurring event cues.
- `tools/shadow-harness/src/main.rs`: Developer benchmark command entrypoint.

**Configuration:**

- `Cargo.toml`: Workspace members, Rust version, and deny-by-default lints.
- `rust-toolchain.toml`: Rust `1.97.1`, Clippy, and rustfmt selection.
- `clippy.toml`: Thresholds and test-only lint allowances.
- `config/carrier-b-limits.json`: Checked-in bounded production limits consumed
  by `x4-bridge` startup.
- `extensions/live_galaxy/content.xml` and `ui.xml`: X4 package identity and
  Lua registration.

**Core Logic:**

- `crates/observation-domain/src/`: Semantic IDs and source evidence.
- `crates/observation-ingest/src/`: Wire, candidate, certificate, and
  eligibility logic.
- `crates/observation-application/src/`: Lifecycle sequencing.
- `crates/observation-persistence/src/`: SQLite state owner.
- `crates/x4-bridge/src/`: Process composition and diagnostics.
- `crates/x4-carrier-native/src/`: X4 native boundary and producer.
- `extensions/live_galaxy/lua/live_galaxy_observation.lua` and
  `live_galaxy_scheduler.lua`: active realtime collection path.
- `extensions/live_galaxy/lua/live_galaxy_telemetry.lua` and
  `live_galaxy_normalize.lua`: alternate telemetry helper path used by Lua
  contracts; they are not loaded by `live_galaxy_runtime.lua`.

**Testing:**

- `crates/*/tests/`: Rust contract and integration tests organized by crate.
- `crates/*/src/*_tests.rs`: Inline or path-attached Rust unit tests where the
  source-size split keeps fixtures near a module.
- `extensions/live_galaxy/tests/`: Lua, loader, scheduler, package, and adapter
  contract suites.
- `tests/fixtures/`: Cross-component JSON fixtures for bounded message cases.
- `tests/x4-disposable/`: Human-run disposable X4 procedures and evidence
  records; do not treat written procedures as observed runtime proof.
- `shadow-deliberation-evals/v1/`: Versioned shadow proposal fixtures and
  schema/manifest for the harness.

## Naming Conventions

**Files:**

- Rust modules use lowercase `snake_case.rs`, such as
  `production_runtime.rs` and `sqlite_reconcile.rs`.
- Lua modules use lowercase `live_galaxy_<role>.lua`, such as
  `live_galaxy_scheduler.lua` and `live_galaxy_normalize.lua`.
- Mission Director files use the `live_galaxy_<scope>.xml` pattern.
- Rust integration tests use behavior-oriented names such as
  `reconnect_idempotency.rs`; Lua suites use `<area>_spec.lua` or
  `<area>_contract.lua`.
- Public protocol and domain types use PascalCase; IDs and enum variants are
  explicit rather than interchangeable strings.

**Directories:**

- Cargo packages live one level below `crates/` and use hyphenated package names
  (`observation-ingest`) with underscore module filenames.
- Source-specific Rust submodules group by lifecycle concern, for example
  `crates/observation-ingest/src/eligibility/` and
  `crates/x4-bridge/src/server/`.
- X4 package code remains under the extension ID directory
  `extensions/live_galaxy/`.

## Where to Add New Code

**New Feature:**

- Observation contract and typed value: start in
  `crates/observation-domain/src/`, then add decoding/validation in
  `crates/observation-ingest/src/`.
- Lifecycle or publication behavior: add a focused module under
  `crates/observation-application/src/`; keep repository effects behind
  `ObservationRepository`.
- SQLite schema or transaction behavior: update
  `crates/observation-persistence/src/schema.rs` and the smallest relevant
  `sqlite_*.rs` module, then update retained-state readers together.
- Production bridge composition: change
  `crates/x4-bridge/src/production_runtime.rs`,
  `production_admission.rs`, or `production.rs` according to ownership; keep
  protocol framing in `observation-ingest`.

**New X4 Source/Collector:**

- Lua collector or normalizer: add a focused module under
  `extensions/live_galaxy/lua/` and call it through the runtime-owned adapter
  seam in `live_galaxy_scheduler.lua`; a helper covered only by
  `live_galaxy_telemetry.lua` tests is not active production composition.
- Native producer input: extend the typed input and producer modules under
  `crates/x4-carrier-native/src/`; do not construct raw wire JSON in Lua.
- New MD lifecycle cue: add it under `extensions/live_galaxy/md/` and document
  its trigger, re-entry, boundary, and completion behavior beside the cue.

**New Component/Module:**

- Native ABI or transport code belongs in
  `crates/x4-carrier-native/src/abi*.rs`, `lua_*.rs`, or `transport*.rs`.
- Bridge-specific diagnostics and startup code belong in
  `crates/x4-bridge/src/`.
- Strategic policy belongs in `crates/strategic-state/src/`; mind state
  transitions belong in `crates/mind-domain/src/`; provider sequencing belongs
  in `crates/mind-orchestration/src/`; checkpoint/recovery belongs in
  `crates/mind-persistence/src/`.

**Tests:**

- Put crate behavior tests in the owning package's `tests/` directory, using
  subdirectories for a contract family when fixtures/helpers grow.
- Put small Rust unit tests beside the implementation only when the source file
  already owns the private seam; keep production files within the 200-line
  source-size check.
- Put Lua contract tests in `extensions/live_galaxy/tests/` and load shipped
  modules through their extension-relative paths.

**Utilities:**

- Shared observation helpers belong in the owning domain/ingest crate, not a
  catch-all utility module.
- Developer-only process, benchmark, and packaging helpers belong under
  `tools/` or `runbooks/` and must not become production dependencies.

## Special Directories

**`.planning/`:**

- Purpose: GSD project state, phase artifacts, research, and codebase maps.
- Generated: Partly workflow-generated; planning documents are committed as
  project artifacts, while machine-local routing under `.codex/` is excluded.
- Committed: Yes for planning artifacts; keep secrets, private evidence, logs,
  and runtime state out.

**`dist/`:**

- Purpose: Generated release bundle assembled by
  `tools/package-live-galaxy.ps1`.
- Generated: Yes.
- Committed: Treat as generated output; package only after the release workflow
  explicitly requires it.

**`target/`:**

- Purpose: Cargo build output and local test/probe artifacts.
- Generated: Yes.
- Committed: No; never use it as source-of-truth architecture evidence.

**`runtime/`:**

- Purpose: Local runtime state and operational outputs used by development.
- Generated: Yes or machine-local depending on the run.
- Committed: No; keep databases, logs, captures, and private evidence local.

**`extensions/live_galaxy/tests/` and `tests/x4-disposable/`:**

- Purpose: Automated package contracts and human-run X4 procedures.
- Generated: Automated results/evidence may be generated; procedures are source.
- Committed: Procedures and sanitized fixtures are committed; private runtime
  captures remain outside Git.

**`shadow-deliberation-evals/`:**

- Purpose: Versioned, bounded, redacted shadow proposal evaluation corpus.
- Generated: Fixtures are curated input artifacts, not build output.
- Committed: Yes, with no private prompts or credentials.

---

*Structure analysis: 2026-09-14*
