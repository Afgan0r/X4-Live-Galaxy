---
last_mapped_commit: 80881814c2b3753c50c9edc7cb79818ee7444dde
---
<!-- refreshed: 2026-09-14 -->

# Testing Patterns

**Analysis Date:** 2026-09-14

This map inventories source-defined checks and recorded evidence. No product
test suite or X4 probe was executed as part of the mapping task.

## Test Framework

**Runner:**

- Rust uses Cargo's built-in test runner across the workspace declared in `Cargo.toml`; CI runs the complete workspace from `.github/workflows/phase-05.yml`.
- Lua uses Busted 2.3.0-1 with Lua 5.1.5, pinned by `tools/lua-runner.lock.json` and `tools/live-galaxy-tests-1.0-1.rockspec`.
- XML and package conformance use PowerShell scripts in `extensions/live_galaxy/tests/x4-package-conformance.ps1` and `extensions/live_galaxy/tests/persistence_schema_contract.ps1`.
- `tools/verify_xen_khk_evidence.Tests.ps1` follows Pester-style `Describe`/`It`/`Should` syntax, but no CI workflow invokes that file.

**Config:**

- Rust lint and test policy is centralized in `Cargo.toml`, `clippy.toml`, `rust-toolchain.toml`, and `.github/workflows/phase-05.yml`.
- The root Cargo alias `lint` points to `tools/source-size-lint/Cargo.toml` in `.cargo/config.toml`; `tools/source-size-lint/src/main.rs` enforces the 200-line source bound as well as formatting and lint checks.
- Lua suite selection and stage orchestration are implemented by `extensions/live_galaxy/tests/run_contracts.ps1`.

**Assertion Library:**

- Rust tests use `assert!`, `assert_eq!`, `assert_ne!`, and exact enum or typed-value matching, as in `crates/observation-ingest/tests/contract_lifecycle.rs` and `crates/x4-bridge/tests/protocol_contract.rs`.
- Lua tests use Busted assertions such as `assert.equals`, `assert.same`, `assert.is_nil`, and `assert.is_table` in `extensions/live_galaxy/tests/component_discovery_contract.lua` and `extensions/live_galaxy/tests/telemetry_spec.lua`.
- PowerShell conformance uses `Assert-*` helpers in `extensions/live_galaxy/tests/x4-package-conformance.ps1`; Pester assertions use `Should` in `tools/verify_xen_khk_evidence.Tests.ps1`.

**Run Commands:**

```text
cargo fmt --all --check                                      # Rust formatting
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked                              # Rust workspace tests
cargo run -p source-size-lint --locked -- crates tools        # size, fmt, and lint gate
pwsh -File extensions/live_galaxy/tests/run_contracts.ps1     # Lua/XML contract suite
pwsh -File extensions/live_galaxy/tests/run_contracts.ps1 -Suite lua -Filter "pattern"
pwsh -File tests/carrier-b-local.ps1 -SelfTest                # local Carrier B process harness
```

The command set mirrors `.github/workflows/phase-05.yml`, `extensions/live_galaxy/tests/run_contracts.ps1`, and `tests/carrier-b-local.ps1`; the CI workflow currently does not invoke the Lua/XML suite.

## Test File Organization

**Location:**

- Rust integration tests are separate from production modules under each crate's `tests/` directory, for example `crates/observation-ingest/tests/` and `crates/x4-bridge/tests/`.
- Narrow Rust unit tests are colocated in `#[cfg(test)]` modules in files such as `crates/observation-ingest/src/accepted_versions.rs` and `crates/x4-carrier-native/src/transport.rs`.
- Lua tests are separate from runtime modules under `extensions/live_galaxy/tests/`; production code lives under `extensions/live_galaxy/lua/`.
- PowerShell contract tests sit beside the X4 package in `extensions/live_galaxy/tests/`, while the evidence verifier is under `tools/verify_xen_khk_evidence.Tests.ps1`.

**Naming:**

- Use `*_contract.rs` for protocol, boundary, and schema contracts; use `*_recovery.rs`, `*_lifecycle.rs`, and `*_determinism.rs` for behavior families, as in `crates/x4-carrier-native/tests/producer_recovery_contract.rs` and `crates/observation-persistence/tests/crash_recovery.rs`.
- Use `*_contract.lua` for adapter or loader contracts and `*_spec.lua` for module behavior, as in the active suites `extensions/live_galaxy/tests/component_discovery_contract.lua`, `extensions/live_galaxy/tests/carrier_b_contract.lua`, and `extensions/live_galaxy/tests/telemetry_spec.lua`.
- Name tests after the scenario and observable outcome, as in `immutable_batch_distinguishes_progress_handoff_and_volatile_receipt` in `crates/observation-ingest/tests/contract_lifecycle.rs`.

**Runner Reachability:**

- The default Busted list in `extensions/live_galaxy/tests/run_contracts.ps1` runs `component_discovery_contract.lua`, `telemetry_spec.lua`, `carrier_b_contract.lua`, and `x4_loader_contract.lua`.
- The same runner invokes `module_loading_spec.lua` in the separate `Lua-syntax` stage for `all`/`lua`; `-Suite syntax` selects that tagged file directly.
- `-Suite scheduler` and `-Suite x4_discovery` both remap to `carrier_b_contract.lua`; the tracked `extensions/live_galaxy/tests/scheduler_contract.lua` and `extensions/live_galaxy/tests/x4_discovery_contract.lua` are dormant direct suites and must not be counted as active Busted coverage.
- `extensions/live_galaxy/tests/carrier_b_local.lua` is active through the process harness `tests/carrier-b-local.ps1`, not through Busted.

**Structure:**

```text
crates/<crate>/tests/<contract_or_lifecycle>.rs
crates/<crate>/tests/<contract_or_lifecycle>/<focused_case>.rs
extensions/live_galaxy/tests/<suite>_contract.lua
extensions/live_galaxy/tests/run_contracts.ps1
```

Large Rust suites are split with `#[path = ...] mod ...`, as in `crates/observation-ingest/tests/contract_lifecycle.rs`, `crates/x4-bridge/tests/named_pipe_contract.rs`, and `crates/mind-domain/tests/shadow_deliberation_evals.rs`.

## Test Structure

**Suite Organization:**

The following Rust block is an excerpt from `crates/observation-persistence/tests/crash_recovery.rs`.

```rust
#[test]
fn reconciliation_proves_an_absent_attempt_retryable() {
    let database = TempDatabase::new("retryable");
    let request = publish_request(validated("ships", 1, None, BTreeMap::default()));
    let mut repository = SqliteObservationRepository::open(database.path(), limits()).unwrap();

    assert_eq!(
        repository.reconcile_publication(&request),
        ReconciliationOutcome::ProvenNotCommitted
    );
}
```

This scenario/outcome style is used in `crates/observation-persistence/tests/crash_recovery.rs`, with shared fixtures in `crates/observation-persistence/tests/support.rs`.

The following Lua block is an excerpt from `extensions/live_galaxy/tests/telemetry_spec.lua`.

```lua
describe("telemetry", function()
    it("rejects invalid runtime game time", function()
        for _, game_time in ipairs({ -1, 1.5, "3600" }) do
            local section, err = normalize.normalize_section({
                entity_id = "sector:alpha",
                source = "x4_runtime",
                version = 1,
                quality = "fresh",
                runtime_facts = runtime_facts("sector:alpha", game_time),
            })
            assert(section == nil)
            assert(err == "runtime_facts_invalid")
        end
    end)
end)
```

The lifecycle and isolation pattern is implemented by `extensions/live_galaxy/tests/test_helper.lua` and exercised by `extensions/live_galaxy/tests/telemetry_spec.lua`.

**Patterns:**

- Assert complete typed outcomes and preserved state, rather than only a success flag, in `crates/observation-ingest/tests/contract_lifecycle.rs` and `crates/x4-bridge/tests/production_runtime_recovery.rs`.
- Use committed JSON fixtures with `include_str!` or `include_bytes!` for malformed, oversized, and reordered inputs in `tests/fixtures/`, and for shadow-evaluation inputs in `shadow-deliberation-evals/v1/fixtures/`.
- Keep fixture setup fail-fast with narrowly scoped test-only lint exceptions in `crates/observation-persistence/tests/support.rs` and `crates/observation-application/tests/support.rs`.
- Do not treat source-text assertions as runtime proof; the evidence boundary is documented in `tests/x4-disposable/` and implemented by layered contract harnesses.

## Mocking

**Framework:**

- Rust uses hand-written fakes and ports, not a mocking crate; examples include `FakeClock` in `crates/observation-ingest/tests/scheduler_policy.rs` and `FakeProvider` in `crates/mind-orchestration/tests/provider_contract.rs`.
- Lua tests use local fixture fakes and spies supplied by `extensions/live_galaxy/tests/test_helper.lua` and `extensions/live_galaxy/tests/carrier_b_contract.lua`.

**Patterns:**

- Use the explicit adapter seam and exact typed outcomes in `crates/x4-bridge/tests/protocol_contract.rs`.
- The suite-local fakes in `extensions/live_galaxy/tests/carrier_b_contract.lua` capture external calls and inject controlled failures. Keep the real scheduler and product modules under test.

**What to Mock:**

- Mock clocks, providers, adapters, native calls, and failure injection points where the production port is explicit, following `crates/observation-ingest/tests/scheduler_policy.rs`, `crates/mind-orchestration/tests/provider_contract.rs`, and `crates/x4-bridge/tests/protocol_contract.rs`.
- Mock process boundaries only for deterministic unit or contract tests; use the local process harness in `tests/carrier-b-local.ps1` when cross-language framing matters.

**What NOT to Mock:**

- Use the real isolated SQLite repository for schema, reopen, publication, and recovery behavior in `crates/observation-persistence/tests/port_contract.rs` and `crates/observation-persistence/tests/crash_recovery.rs`.
- Do not call a fake adapter an in-game result; X4 runtime proof remains a separate layer described in `crates/x4-bridge/tests/` and `tests/x4-disposable/`.

## Fixtures and Factories

**Test Data:**

The following Rust block is an excerpt from `crates/observation-persistence/tests/crash_recovery.rs`.

```rust
fn precommit_cut(cut: PublicationFailpoint) {
    let database = TempDatabase::new("precommit-cut");
    let request = publish_request(validated("ships", 1, None, BTreeMap::default()));
    let mut repository = SqliteObservationRepository::open(database.path(), limits()).unwrap();
    assert!(matches!(
        repository.publish_with_failpoint(&request, cut),
        PublishOutcome::PermanentRejection(_)
    ));
    drop(repository);
    let reopened = SqliteObservationRepository::open(database.path(), limits()).unwrap();
    assert_eq!(reopened.current(&key("ships")), Ok(None));
}
```

Temporary database and fixture lifecycle helpers live in `crates/observation-persistence/tests/support.rs`.

The following Lua block is an excerpt from `extensions/live_galaxy/tests/component_discovery_contract.lua`.

```lua
after_each(function() if fixture then fixture.restore() end end)
before_each(function()
    fixture = helper.new()
    discovery = fixture.load("live_galaxy_component_discovery")
    telemetry = fixture.load("live_galaxy_telemetry")
end)
```

Global snapshots, module cache cleanup, and fixture restoration are centralized in `extensions/live_galaxy/tests/test_helper.lua`.

**Location:**

- Shared Rust helpers are local to the owning crate's support module, with concrete examples in `crates/observation-persistence/tests/support.rs` and `crates/x4-bridge/tests/production_startup/support.rs`.
- Stable serialized fixtures are committed under `tests/fixtures/` and `shadow-deliberation-evals/v1/fixtures/`; Rust-specific fixture helpers remain beside their suites, such as `crates/x4-bridge/tests/named_pipe_contract/fixtures.rs`.
- Lua fixtures stay in `extensions/live_galaxy/tests/test_helper.lua`; test-specific shapes remain in the suite that owns the contract.

## Coverage

**Requirements:**

- Current sufficiency is scenario-based and follows `.agents/skills/live-galaxy-tests/references/test-standards.md` and `.agents/skills/live-galaxy-x4-tests/SKILL.md`; no numeric percentage target is stated in the repository configuration.
- Five deferred tests in `crates/x4-bridge/tests/named_pipe_contract/deferred_publication.rs` carry explicit `#[ignore]` reasons; report them as pending contract work rather than executed coverage.
- The configured suite uses deterministic edge and recovery scenarios; property-based tests are not part of the current tracked manifests.

**View Coverage:**

- Coverage is represented by the scenario suites and CI results defined in `Cargo.toml` and `.github/workflows/phase-05.yml`; a dedicated percentage report is not part of the current configured commands.
- Mutation testing is a conditional gate: use a measured `cargo-mutants` baseline and survivor triage when the phase changes deterministic safety validation, following `.agents/skills/live-galaxy-rust-tests/SKILL.md` and `crates/mind-persistence/tests/mutation_baseline.rs`.
- Lua mutation scoring is not a standing gate; `.agents/skills/live-galaxy-x4-tests/SKILL.md` limits it to a bounded spike until a useful harness exists.

## Test Types

**Unit Tests:**

- Colocated `#[cfg(test)]` modules cover narrow pure helpers and private seams in `crates/observation-ingest/src/accepted_versions.rs` and `crates/x4-carrier-native/src/transport_types.rs`.
- Pure Lua normalization and telemetry behavior is covered by `extensions/live_galaxy/tests/telemetry_spec.lua`.

**Integration Tests:**

- Rust contract suites cover ingest, persistence, orchestration, protocol, and recovery through public crate boundaries in `crates/observation-ingest/tests/`, `crates/observation-persistence/tests/`, and `crates/x4-bridge/tests/`.
- XML and package integration checks parse actual extension files and reject forbidden nodes in `extensions/live_galaxy/tests/x4-package-conformance.ps1` and `extensions/live_galaxy/tests/persistence_schema_contract.ps1`.
- `tests/carrier-b-local.ps1` builds and runs the real local native producer/bridge processes; it is cross-language local evidence, not in-game evidence.

**E2E Tests:**

- No automated end-to-end X4 game runner is wired into `.github/workflows/phase-05.yml`; disposable in-game probes and observed runtime evidence are tracked separately under `tests/x4-disposable/`.
- `tools/shadow-harness/tests/manual_contract.rs` explicitly marks its manual harness evidence as `EvidenceClass::ManualHarness` and unavailable preflight, so it must not be reported as a live X4 result.

## Common Patterns

**Async Testing:**

The following Rust block is an excerpt from `crates/x4-carrier-native/tests/transport_recovery_contract.rs`.

```rust
fn poll(transport: &NativeTransport) -> TransportPoll {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let result = transport.poll_control(token(), 512);
        if result != TransportPoll::NoMessage || Instant::now() >= deadline {
            return result;
        }
        std::thread::yield_now();
    }
}
```

Use bounded watchdog loops and short waits in `crates/x4-carrier-native/tests/transport_recovery_contract.rs` and `crates/x4-bridge/tests/production_startup/support.rs`; serialize process-global state with the existing mutexes in `crates/x4-bridge/tests/production_runtime_recovery.rs`.

**Error Testing:**

- Exercise malformed input, stale sequences, oversize payloads, and explicit rejection dispositions using fixtures in `tests/fixtures/` and scenarios in `crates/observation-ingest/tests/contract_lifecycle.rs`.
- Exercise ambiguous publication cuts and reopen recovery with `PublicationFailpoint` in `crates/observation-persistence/tests/crash_recovery.rs`.
- Exercise invalid X4/native return shapes and `pcall` failures in the active suites `extensions/live_galaxy/tests/component_discovery_contract.lua` and `extensions/live_galaxy/tests/carrier_b_contract.lua`.
- Keep every error assertion tied to the stable error or disposition contract in `crates/x4-bridge/src/diagnostics.rs` and `extensions/live_galaxy/lua/live_galaxy_telemetry.lua`.

---

*Testing analysis: 2026-09-14*
