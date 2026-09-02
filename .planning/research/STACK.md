# Technology Stack

**Project:** Live Galaxy — milestone 0.1 Shadow Director  
**Researched:** 2026-08-28  
**Scope:** Windows, X4 9.00 (`version.dat` = `900`), observation-only ZYA/ARG faction minds under XEN pressure.

## Recommendation

Use a small Rust workspace for the deterministic kernel and bridge, with thin Mission Director/Lua adapters in the X4 extension. Keep SQLite as the local durable store, JSON Lines plus structured tracing for diagnostics, and a provider-neutral async interface around a developer-controlled subscription harness for all pre-alpha real-model work. Use recorded fixtures and deterministic fakes for offline tests and replay, but require subscription-backed trajectories for strategic-quality acceptance. Public runtime API integration begins on the alpha path. Do not add mutation commands, save-file readers, or a custom UI in 0.1.

Confidence labels below are **HIGH** for stable official tool documentation, **MEDIUM** for ecosystem choices that remain phase benchmarks, and **OBSERVED** for exact local X4 precedent.

## Recommended Stack

### Core Framework and Kernel

| Technology | Version | Purpose | Why |
| --- | --- | --- | --- |
| Rust stable toolchain | Pin in `rust-toolchain.toml` when workspace is created | Bridge, typed domain, validation, orchestration | Strong enum/newtype/error tooling and deterministic, bounded execution. Exact toolchain is a phase decision; do not invent it now. **MEDIUM** |
| Cargo workspace | Current stable Cargo | Package boundaries and reproducible builds | Native Rust dependency/build boundary; split domain, bridge, persistence, provider, and diagnostics crates only where seams are real. **HIGH** |
| `serde` + `serde_json` | Pin in `Cargo.lock` | Typed wire/config/fixture serialization | Standard Rust serialization; preserves an explicit schema barrier before model data enters domain logic. **HIGH** |
| `thiserror` | Pin in `Cargo.lock` | Typed recoverable errors | Keeps transport, storage, provider, and validation failures actionable without panics. **HIGH** |
| `tokio` | Pin in `Cargo.lock` | Bounded async bridge/provider work | Appropriate only for I/O concurrency; keep pure strategic logic synchronous and testable. Runtime features and budgets require phase benchmarking. **MEDIUM** |

### X4 Adapter

| Technology | Version | Purpose | Why |
| --- | --- | --- | --- |
| Mission Director XML | X4 9.00 runtime | Cooperative scheduler and lifecycle hooks | Existing X4 precedent schedules bounded callbacks; do not perform a full snapshot in one game-thread callback. **OBSERVED** |
| Embedded X4 Lua | X4 9.00 runtime | Thin observation, normalization, envelope serialization | Keep pure batching/diff/budget logic separate from X4 globals; confirm syntax/runtime limits with a disposable probe. **MEDIUM** |
| SirNukes Mod Support APIs | Installed dependency; exact installed version must be recorded in phase smoke | Named-pipe/native support seam | Local precedent `F:\Agent Projects\X4\extensions\x4_live_mcp\content.xml` declares dependency `ws_2042901274`; compatibility is evidence-based, not guaranteed. **OBSERVED** |
| Versioned JSON envelopes over Win32 named pipe | Protocol v1 precedent | Bridge transport for observations/health | Existing protocol documents `\\.\\pipe\\x4_live_mcp`, bounded messages, identity, timestamps, freshness and quality. Adapt the contract for Live Galaxy rather than importing campaign tooling. **OBSERVED** |

### Database, Cache, and Diagnostics

| Technology | Version | Purpose | Why |
| --- | --- | --- | --- |
| SQLite via `rusqlite` | Pin crate; SQLite library version to verify in build | Authoritative compact runtime state and idempotency records | Durable local persistence, transactions, restart recovery, and unique event/decision identities. Production readers should use read-only/query-only mode where they consume external data. **MEDIUM** |
| `tracing` + `tracing-subscriber` | Pin in `Cargo.lock` | Correlated structured diagnostics | Emit correlation IDs, stage, duration, quality, rejection/degradation reason; never log prompts, secrets, or hidden reasoning. **HIGH** |
| JSONL evidence files | Internal format, schema versioned | Offline traces, snapshots, evaluation inputs | Append-friendly and inspectable; rotate by size/time and enforce payload/retention limits. **MEDIUM** |
| Exact content-addressed cache keys | Project contract | Cache normalized snapshot/model request/result | Include provider/model/schema/prompt-template version and canonical input; cache is non-authoritative and must be invalidated on schema changes. **HIGH** |

### Model-Provider Abstraction

| Technology | Version | Purpose | Why |
| --- | --- | --- | --- |
| Provider trait owned by Live Galaxy | Project API | Uniform typed request/result, usage, timeout, retry, and failure metadata | Prevent subscription-harness or later API response types from leaking into domain code. **HIGH** |
| Developer-controlled subscription harness | Pin the invoked client and model identity in each benchmark | Pre-alpha real-model deliberation and evaluation | Uses the owner's subscriptions for prototypes while retaining typed requests, structured results, usage evidence, and reproducible run identity. It is not a public runtime dependency. **HIGH** |
| Deterministic fake | Project-versioned fixtures | Contract, replay, failure, and normal automated tests | Keeps normal tests offline and deterministic but cannot satisfy strategic-quality acceptance. **HIGH** |
| Public runtime API adapter | Deferred to the alpha path | Supported public model access | Keep behind the same provider trait; credentials and API billing are not milestone 0.1 requirements. **HIGH** |
| JSON Schema or equivalent typed output contract | Project-versioned schema | Constrain goals/plans/reports | Parse into typed values, then apply semantic, information, safety, budget, and current-state checks. **HIGH** |

### Testing, Evaluation, Packaging, and Tooling

| Technology | Version | Purpose | Why |
| --- | --- | --- | --- |
| Rust built-in tests (`cargo test`) | Stable Cargo | Unit/contract/integration tests | Deterministic fakes and recorded fixtures; cover replay, persistence interruption, duplicate/out-of-order input, budgets, malformed model output, and diagnostics. **HIGH** |
| `cargo-nextest` | Phase benchmark/pin if adopted | Faster isolated Rust test execution | Optional; use only if measured workspace size justifies it. The test contract remains equivalent to `cargo test`. **MEDIUM** |
| `cargo-mutants` | Pin when mutation gate is admitted | Mutation testing for pure kernel validation/state transitions | Required project direction, but establish a measured baseline first; exclude adapters and generated code. **HIGH** |
| Busted | Confirm/pin Lua-compatible version | Pure Lua unit tests | Project test guidance recommends Busted after confirming embedded Lua syntax/runtime. **MEDIUM** |
| XML/schema/package checks | CI script/tool to be selected | Validate extension manifest, MD structure, identifiers, and package contents | Static checks precede disposable in-game probes; source-string matching is not behavioral proof. **HIGH** |
| GitHub Actions or equivalent Windows CI | Phase decision | Format, lint, test, package audit | Run reproducible Windows checks; exact runner/tool versions remain unknown until workspace exists. **MEDIUM** |
| Zip/package manifest with release identity and hashes | Project format | Installable developer artifact and provenance | Include extension, bridge, schemas, and diagnostics contract; exclude saves, credentials, runtime DBs, and generated local routing. **HIGH** |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
| --- | --- | --- | --- |
| Bridge language | Rust | Python | Python is useful as a reference/tooling language, but Rust better enforces bounded typed runtime boundaries and single-binary Windows packaging. **MEDIUM** |
| Persistence | SQLite | JSON-only files | JSONL is excellent evidence, but SQLite transactions/unique constraints are needed for restart recovery and idempotency. **HIGH** |
| Model runtime | Provider trait + subscription harness before alpha | Hard-code the development client into the domain | Couples strategic state to a developer tool and makes the later API migration invasive. **HIGH** |
| Game integration | Thin MD/Lua adapter | Broad game-thread polling/hook | Risks SETA stalls and simulation impact; cooperative bounded scheduling is the observed safer precedent. **HIGH** |
| Lua testing | Busted pure-module tests + fakes | In-game-only testing | Slow, nondeterministic, and unable to isolate malformed/budget/retry cases. **HIGH** |
| Evaluation | Recorded fixtures and replay | Live model calls in normal tests | Live calls are nondeterministic, costly, and conceal regressions. **HIGH** |

## Installation / Bootstrap (after workspace creation)

Do not execute this as a current dependency prescription; crate versions and the Rust/Lua toolchain must be pinned by the implementation phase after benchmark and compatibility checks.

```text
# Rust dependencies to evaluate and pin in Cargo.toml
serde, serde_json, thiserror, tokio, rusqlite, tracing, tracing-subscriber

# Development tools to evaluate and pin
cargo-nextest (optional), cargo-mutants

# Lua test runner (developer-only, after confirming X4's embedded Lua)
Busted
```

## Explicit Deferrals for 0.1

- No game-state mutation, command dispatcher, or mutation-capable adapter.
- No player save-file reader or writer; X4-owned persistence integration remains a later evidence gate.
- No custom dossier/chronicle/institution UI; use low-cost Mail/Logbook reports and external diagnostics.
- No fixed vanilla map, hard-coded asset counts, or mod-added-faction assumptions.
- No Faction Enhancer compatibility claim; KUDA AI Tweaks and Add More Sectors require later compatibility tests. The owner dropped More AI Economy Ships compatibility on 2026-09-03; it is not a release gate.
- No final Rust toolchain, crate versions, subscription model, public API model, CI image, or Lua runner version until phase benchmarks verify them.

## Sources

- Rust Book, ownership/type/error foundations: <https://doc.rust-lang.org/book/>
- Cargo workspaces: <https://doc.rust-lang.org/cargo/reference/workspaces.html>
- Serde documentation: <https://serde.rs/>
- Tokio documentation: <https://tokio.rs/>
- SQLite transactions and atomic commit: <https://www.sqlite.org/lang_transaction.html>
- `rusqlite` crate documentation: <https://docs.rs/rusqlite/latest/rusqlite/>
- `tracing` crate documentation: <https://docs.rs/tracing/latest/tracing/>
- Local X4 9.00 observation: `F:\SteamLibrary\steamapps\common\X4 Foundations\version.dat` (`900`) (**OBSERVED**, read-only).
- Local X4 adapter precedent: `F:\Agent Projects\X4\extensions\x4_live_mcp\content.xml`, extension v051 (**OBSERVED**; reference repository Git refresh blocked by ownership/FETCH_HEAD permissions).
- Local transport/scheduling precedent: `F:\Agent Projects\X4\tools\x4-live-protocol.md` (**OBSERVED**).
- Local test/evidence precedent: `F:\Agent Projects\X4\tools\README.md` and `F:\Agent Projects\X4\tests\test_x4_live.py` (**OBSERVED**).
- Project constraints and boundaries: `AGENTS.md`, `.planning/PROJECT.md`, and `.agents/skills/live-galaxy-{rust-conventions,rust-tests,x4-integration,x4-tests}/SKILL.md` (**HIGH**, repository authority).
