---
last_mapped_commit: 80881814c2b3753c50c9edc7cb79818ee7444dde
---
<!-- refreshed: 2026-09-14 -->

# Coding Conventions

**Analysis Date:** 2026-09-14

## Naming Patterns

**Files:**

- Use `snake_case` for Rust modules and source files, as in `crates/observation-ingest/src/accepted_versions.rs` and `crates/x4-bridge/src/operational_history.rs`.
- Name Rust tests after the contract or lifecycle they exercise: `crates/observation-ingest/tests/contract_lifecycle.rs`, `crates/x4-carrier-native/tests/transport_recovery_contract.rs`, and `crates/x4-bridge/tests/production_runtime_recovery.rs`.
- Prefix production Lua modules with `live_galaxy_`, as in `extensions/live_galaxy/lua/live_galaxy_normalize.lua` and `extensions/live_galaxy/lua/live_galaxy_observation.lua`.
- Name Lua suites with `*_contract.lua` for boundary checks and `*_spec.lua` for focused module checks, as in `extensions/live_galaxy/tests/component_discovery_contract.lua` and `extensions/live_galaxy/tests/telemetry_spec.lua`.

**Functions:**

- Use `snake_case` for Rust functions, methods, and test names, as shown by `crates/observation-ingest/src/completion.rs` and `crates/mind-domain/tests/shadow_deliberation_evals.rs`.
- Use `snake_case` for Lua helpers and module methods, such as `normalize_observation` in `extensions/live_galaxy/lua/live_galaxy_normalize.lua`.
- Name tests by observable scenario and outcome, for example `reconciliation_proves_an_absent_attempt_retryable` in `crates/observation-persistence/tests/crash_recovery.rs`.

**Variables:**

- Use descriptive `snake_case` locals and keep Rust fields private where validation is required, as in `crates/observation-domain/src/identity.rs`.
- Use uppercase constants for bounds and protocol limits, such as `MAX_RUST_FILE_LINES` in `tools/source-size-lint/src/main.rs` and the `MAX_*` constants in `extensions/live_galaxy/lua/live_galaxy_normalize.lua`.
- Keep Lua state local to the module or fixture; test globals are installed and restored explicitly in `extensions/live_galaxy/tests/test_helper.lua`.

**Types:**

- Use `PascalCase` for Rust structs, enums, and type aliases, with typed newtypes for identifiers and versions in `crates/observation-domain/src/identity.rs`.
- Derive comparison and display-relevant traits deliberately on domain values, as in `crates/observation-domain/src/identity.rs` and `crates/mind-domain/src/deliberation.rs`.
- Model finite dispositions with enums rather than strings or boolean combinations, as in `crates/observation-ingest/src/completion.rs` and `crates/x4-bridge/src/facade.rs`.

## Code Style

**Formatting:**

- Run `cargo fmt --all --check`; the pinned formatter is provided by `rust-toolchain.toml` and CI invokes it from `.github/workflows/phase-05.yml`.
- Keep Rust files at or below 200 physical lines; `tools/source-size-lint/src/main.rs` enforces this for `crates/` and `tools/` and also performs formatting and lint checks.
- Split cohesive responsibilities into neighboring modules, as in `crates/observation-persistence/src/sqlite_*.rs` and `crates/x4-bridge/src/production_*.rs`.
- Keep Lua compatible with the pinned Lua 5.1 runner in `tools/lua-runner.lock.json`; use the repository's Busted 2.3.0 setup from `tools/live-galaxy-tests-1.0-1.rockspec`.

**Linting:**

- Treat the workspace lint policy in `Cargo.toml` as a build contract: warnings, `unsafe_code`, Clippy `all` and `pedantic`, and selected panic-prone lints are denied.
- Use narrow, explained `#[expect(..., reason = "...")]` exceptions, as in `crates/mind-domain/src/deliberation.rs` and `crates/x4-carrier-native/src/lib.rs`.
- Keep the native FFI exception limited to `crates/x4-carrier-native/Cargo.toml`; its ABI calls in `crates/x4-carrier-native/src/abi.rs` require explicit `// SAFETY:` rationale and deny `unsafe_op_in_unsafe_fn`.
- Follow complexity thresholds in `clippy.toml`: cognitive complexity 15, nesting 3, too-many-lines 60, and type complexity 200.

## Import Organization

**Observed practice:**

- Rust modules commonly group `crate::{...}` imports, workspace dependencies, and standard-library imports; follow `rustfmt` and the nearest module rather than treating this incidental order as a semantic rule, as seen in `crates/observation-persistence/src/lib.rs` and `crates/x4-bridge/src/facade.rs`.

**Path Resolution:**

- Use the existing Cargo crate names and `crate::` paths at Rust boundaries, as shown in `Cargo.toml` and `crates/*/src/lib.rs`.
- Use dotted extension-relative module names for production Lua requires, as in `extensions/live_galaxy/lua/live_galaxy_runtime.lua` and `extensions/live_galaxy/lua/live_galaxy_telemetry.lua`.
- The production dotted loader contract is checked by `extensions/live_galaxy/tests/x4_loader_contract.lua`; test setup in `extensions/live_galaxy/tests/test_helper.lua` also supports the test module path it needs.

## Error Handling

**Patterns:**

- Return typed boundary errors and match every disposition explicitly; examples are `crates/observation-ingest/src/completion.rs`, `crates/observation-persistence/src/types.rs`, and `crates/x4-bridge/src/facade.rs`.
- Use `Option` for expected absence and `Result` for rejection, I/O, protocol, or persistence failure; constructors in `crates/observation-domain/src/identity.rs` validate before constructing values.
- Do not use production `unwrap`, `expect`, `panic`, `todo`, or `unimplemented`; the deny policy is declared in `Cargo.toml` and test-only exceptions are scoped in `crates/observation-persistence/tests/support.rs`.
- Wrap X4 and native Lua calls in `pcall`, preserve the returned error, and emit an explicit failure shape as done in `extensions/live_galaxy/lua/live_galaxy_component_discovery.lua` and `extensions/live_galaxy/lua/live_galaxy_carrier.lua`.

## Logging

**Framework:**

- Use the project's structured diagnostic and operational-history types rather than an ad hoc logging framework; see `crates/x4-bridge/src/diagnostics.rs` and `crates/x4-bridge/src/operational_history.rs`.
- Lua integration diagnostics use explicit X4 functions such as `DebugError` at the boundary in `extensions/live_galaxy/lua/live_galaxy_runtime.lua`.

**Patterns:**

- Emit stable, privacy-safe fields that identify stage, disposition, and bounded metadata; the diagnostic contracts live in `crates/x4-bridge/src/diagnostics.rs` and `extensions/live_galaxy/tests/telemetry_spec.lua`.
- Never log secrets, raw prompts, or private payloads; redaction and evidence shaping are exercised by `tools/shadow-harness/src/` and `tools/shadow-harness/tests/evidence_contract.rs`.
- Preserve the startup/runtime split: `OperationalHistory::open` records `startup`/`journal-accepted` and `crates/x4-bridge/src/production_startup.rs` refuses startup on a journal error, while `crates/x4-bridge/src/operational_history.rs` reports a bounded degraded status and emergency stderr record after an operational write gap so `crates/x4-bridge/src/production_runtime.rs` can continue otherwise valid work. This follows `.agents/skills/live-galaxy-code-conventions/references/logging.md` (LOG-05); diagnostic degradation never authorizes acknowledging a state whose required persistence failed.

## Comments

**When to Comment:**

- Comment unsafe boundary assumptions with `// SAFETY:` immediately beside the unsafe operation in `crates/x4-carrier-native/src/abi.rs`.
- Explain a narrow lint exception or protocol invariant in its `#[expect(..., reason = "...")]` attribute, as in `crates/mind-domain/src/cache_identity.rs`.
- Use comments for contract rationale and recovery cuts; do not restate obvious control flow in `crates/x4-bridge/src/production_runtime.rs`.

**Rust/Lua API Documentation:**

- Rust public API documentation is sparse; add documentation when a public item or safety contract needs it, following the public exports in `crates/observation-ingest/src/lib.rs` and `crates/x4-bridge/src/lib.rs`.
- Lua modules rely on readable names and contract tests rather than a documentation generator; extend behavior with a focused suite under `extensions/live_galaxy/tests/`.

## Function Design

**Size:**

- Keep production Rust functions cohesive and below the configured complexity thresholds; split long flows into modules such as `crates/x4-bridge/src/production_admission.rs` and `crates/x4-bridge/src/production_runtime.rs`.
- Keep Lua helpers small enough to make validation order and failure behavior visible in `extensions/live_galaxy/lua/live_galaxy_normalize.lua`.

**Parameters:**

- Pass explicit ports, clocks, repositories, and adapters instead of reaching for mutable global state; examples are `crates/observation-ingest/src/scheduler.rs` and `crates/x4-bridge/src/facade.rs`.
- Use typed values at domain boundaries and validate raw strings at constructors in `crates/observation-domain/src/identity.rs`.

**Return Values:**

- Mark stable domain values and side-effect-free accessors `#[must_use]`, following `crates/observation-domain/src/identity.rs` and `crates/mind-domain/src/ledger.rs`.
- Return explicit result or disposition values from adapters and orchestration paths, as in `crates/x4-bridge/src/facade.rs` and `crates/mind-orchestration/src/lib.rs`.

## Module Design

**Exports and Module Boundaries:**

- Keep `lib.rs` focused on `mod` declarations and selected `pub use` exports, as in `crates/observation-application/src/lib.rs`, `crates/observation-ingest/src/lib.rs`, and `crates/mind-domain/src/lib.rs`.
- Keep implementation modules private unless their types form an intentional crate boundary; expose ports and domain types through the crate root in `crates/observation-persistence/src/lib.rs`.
- Keep re-exports narrow and tied to a crate boundary, following `crates/x4-carrier-native/src/lib.rs` and `crates/observation-persistence/src/lib.rs`.
- Put integration tests in `crates/*/tests/` and split large suites with `#[path = ...] mod ...` as in `crates/observation-ingest/tests/contract_lifecycle.rs` and `crates/x4-bridge/tests/named_pipe_contract.rs`.

---

*Convention analysis: 2026-09-14*
