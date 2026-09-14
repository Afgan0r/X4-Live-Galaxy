---
last_mapped_commit: 80881814c2b3753c50c9edc7cb79818ee7444dde
---

# Technology Stack

**Analysis Date:** 2026-09-14

## Languages

**Primary:**

- Rust 2024, pinned to `1.97.1` - domain, ingestion, persistence, orchestration, the Windows bridge, and the native Lua carrier in `Cargo.toml` and `rust-toolchain.toml`.

**Secondary:**

- Lua 5.1 - X4 extension modules under `extensions/live_galaxy/lua/` and Busted contract tests under `extensions/live_galaxy/tests/`.
- XML - X4 extension registration and Mission Director cues in `extensions/live_galaxy/content.xml`, `extensions/live_galaxy/ui.xml`, and `extensions/live_galaxy/md/`.
- PowerShell - packaging, Lua provisioning, local integration harnesses, and repository checks in `tools/`, `tests/`, and `extensions/live_galaxy/tests/`.

## Runtime

**Environment:**

- Windows is the production target for the X4 extension, the `x4-carrier-native` DLL, and the `x4-bridge` executable; the native transport and bridge peer return unavailable on non-Windows, while the legacy listener returns an unsupported named-pipe error, in `crates/x4-carrier-native/src/transport.rs`, `crates/x4-carrier-native/src/transport_peer.rs`, and `crates/x4-bridge/src/listener.rs`.
- X4 loads the UI Lua entry point declared by `extensions/live_galaxy/ui.xml`; the exact embedded X4 runtime behavior remains evidence-scoped in `tests/x4-disposable/01-probe-evidence.md`.

**Package Manager:**

- Cargo with workspace resolver 3 and a checked-in `Cargo.lock` for Rust dependencies in `Cargo.toml` and `Cargo.lock`.
- LuaRocks/Busted for standalone Lua contract tests, provisioned by `tools/provision-lua.ps1` from `tools/live-galaxy-tests-1.0-1.rockspec`.
- Lockfiles: `Cargo.lock`, `tools/shadow-harness/Cargo.lock`, and `tools/lua-runner.lock.json` are present.

## Frameworks

**Core:**

- Rust workspace crates, without a web framework - typed observation contracts, admission, publication, faction strategic state, mind persistence, orchestration, and X4 transport are split across `crates/observation-*`, `crates/strategic-state`, `crates/mind-*`, `crates/x4-bridge`, and `crates/x4-carrier-native`.
- X4 UI Lua and Mission Director - event registration, bounded observation scheduling, and game-facing runtime integration in `extensions/live_galaxy/lua/live_galaxy_runtime.lua` and `extensions/live_galaxy/md/live_galaxy_observation.xml`.

**Testing:**

- Rust `cargo test` with integration tests colocated in each crate's `tests/` directory, as configured by `.github/workflows/phase-05.yml`.
- Busted 2.3.0 against Lua 5.1.5, pinned in `tools/live-galaxy-tests-1.0-1.rockspec` and `tools/lua-runner.lock.json`.

**Build/Dev:**

- `rustfmt` and Clippy with blocking workspace lint policy in `Cargo.toml` and thresholds in `clippy.toml`.
- `source-size-lint` enforces the repository's Rust source-size rule through `tools/source-size-lint/src/main.rs`.
- PowerShell package and local Carrier B harnesses in `tools/package-live-galaxy.ps1` and `tests/carrier-b-local.ps1`.

## Key Dependencies

**Critical:**

- `serde` 1.0.229 and `serde_json` 1.0.151 - typed serialization and strict JSON contracts in `crates/observation-ingest`, `crates/mind-domain`, `crates/mind-persistence`, and `crates/x4-bridge`.
- `sha2` 0.11.0 - canonical content and publication digests in `crates/observation-ingest` and `crates/observation-persistence`.
- `rusqlite` 0.40.2 with the `bundled` feature - durable observation revisions and current pointers in `crates/observation-persistence`.

**Infrastructure:**

- `interprocess` 2.4.3 and `recvmsg` 1.0.0 - a separate legacy Windows message-mode listener contract in `crates/x4-bridge/Cargo.toml` and `crates/x4-bridge/src/listener.rs`; the active production client is `BridgePeer` from `x4-carrier-native`.
- `windows-sys` 0.61.2 - native Lua ABI lookup, overlapped pipe I/O, worker lifecycle, security descriptors, and performance clock in `crates/x4-carrier-native/Cargo.toml` and `crates/x4-carrier-native/src/abi_windows_*.rs`.
- `doctest-file`, `widestring`, and Windows support crates are transitive locked dependencies in `Cargo.lock`.

## Configuration

**Environment:**

- Production bridge arguments are `--data-dir` and `--limits-file`; optional `--readback`, `--section-key`, and `--section-revision` are parsed in `crates/x4-bridge/src/production_startup.rs`.
- Active production startup requires a writable `operational-history.jsonl` and status path through `OperationalHistory::open` in `crates/x4-bridge/src/production_startup.rs` and `crates/x4-bridge/src/operational_history.rs`.
- `LIVE_GALAXY_DEBUG_EVIDENCE_PATH` and `LIVE_GALAXY_TRACE_ATTEMPT_ID` opt into bounded diagnostics only for the separately exported legacy listener in `crates/x4-bridge/src/listener.rs`.
- `LIVE_GALAXY_C_COMPILER` is an optional local toolchain selector for `tools/provision-lua.ps1`; no environment values are read into committed documentation.

**Build:**

- Rust version and components are fixed by `rust-toolchain.toml`; lint policy is in `Cargo.toml` and `clippy.toml`.
- Carrier limits are validated from `config/carrier-b-limits.json` by `crates/x4-bridge/src/production_limits.rs` and `crates/x4-bridge/src/bin/validate_limits.rs`.
- Release packaging builds `x4_carrier_native.dll` and `x4-bridge.exe`, then assembles the extension and manifest in `tools/package-live-galaxy.ps1`.

## Platform Requirements

**Development:**

- Rust 1.97.1 with `rustfmt` and `clippy`, PowerShell, a Windows C/LLVM toolchain for Lua 5.1.5 source builds, and LuaRocks/Busted as specified by `tools/lua-runner.lock.json`.
- A Windows x86-64 host is required for native Carrier B and named-pipe checks; Lua-only tests use the pinned standalone runner from `tools/provision-lua.ps1`.

**Production:**

- A Windows x86-64 X4 installation with `extensions/live_galaxy` installed, the packaged native DLL renamed to `ui_c_library_live_galaxy_carrier_64.txt`, and `live-galaxy-bridge.exe` launched with a private data directory and validated limits file, as described by `tools/package-live-galaxy.ps1`.
- The repository describes version `0.1.0` as a pre-alpha, observation-only prototype in `README.md`; no public release deployment is established.

---

*Stack analysis: 2026-09-14*
