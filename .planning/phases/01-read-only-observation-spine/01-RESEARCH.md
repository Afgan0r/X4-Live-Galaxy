# Phase 1: Read-Only Observation Spine - Research

**Researched:** 2026-08-28
**Domain:** X4 9.00 read-only telemetry, session protocol, bounded Rust ingestion, and disposable verification
**Confidence:** MEDIUM

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Authority and transport boundary

- **D-01:** X4 remains authoritative. The game-facing adapter is thin and may
  only emit bounded observations during this phase.
- **D-02:** The long-term integration is asymmetric and bidirectional, but
  Phase 1 implements only telemetry plus the minimum session/capability
  negotiation needed to accept it. The report return path belongs to Phase 6.
- **D-03:** No fleet, economy, diplomacy, institution, or generic mutation
  command may exist in the 0.1 protocol vocabulary.

#### Compatibility and restart behavior

- **D-04:** Compatible Rust bridge releases must be able to restart, update,
  and reconnect without restarting X4.
- **D-05:** A game-facing code change or incompatible protocol combination
  fails closed and explicitly identifies that X4 must restart.
- **D-06:** Unsupported, malformed, stale, duplicate, oversized, or
  out-of-order traffic cannot replace the last accepted snapshot.

#### Observation semantics

- **D-07:** Every strategic entity and event carries stable typed identity,
  source, observation time, and monotonic state or event version.
- **D-08:** Snapshot sections represent freshness, coverage, quality, unknown,
  partial, stale, and unsupported states explicitly.
- **D-09:** Runtime sectors, assets, capacity, and ownership are discovered
  from X4 rather than assumed from a vanilla map or fixed job count.

### the agent's Discretion

Transport topology, framing, buffering, acknowledgement mechanics, handshake
schema, observation cadence, and section partitioning are technical decisions.
They must preserve the locked bounds, restart behavior, and game-thread safety.

### Deferred Ideas (OUT OF SCOPE)

- Report delivery and acknowledgements are deferred to Phase 6.
- Faction reasoning is deferred to Phases 3–5.
- Every game-state mutation path is deferred beyond milestone 0.1.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| OBS-01 | Bounded, versioned X4 envelopes and negotiated transport/session capabilities without game-thread blocking. | Cooperative producer, bounded-frame, single-direction session design, and contract tests. |
| OBS-02 | Stable typed identity, source, observation time, and monotonic versions. | Typed identity/version split plus atomic per-envelope acceptance. |
| OBS-03 | Explicit freshness, coverage, quality, unknown, and unsupported section state. | Section descriptors travel with every snapshot projection. |
| OBS-06 | Discover sectors, assets, capacity, and ownership from observed runtime state. | Full-scan completion marker and reconciliation replace fixed-map assumptions. |
| OBS-07 | Reject/reconcile malformed, oversized, duplicate, stale, and out-of-order input without corruption. | Validate-before-commit, immutable accepted snapshot, duplicate/reorder fixtures. |
| OBS-08 | No game-state mutation command exists. | Telemetry-only protocol vocabulary and compile-time-separated inbound frame family. |
| VAL-06 | Exact X4 9.00 semantics are supported by documentation or disposable observed evidence. | Evidence ledger separates installed static evidence, local precedent, and required in-game probes. |
</phase_requirements>

## Project Constraints (from AGENTS.md)

- X4 owns authoritative state; the Rust bridge owns normalized state, validation, persistence, recovery, and diagnostics; model output cannot bypass deterministic validation. [VERIFIED: AGENTS.md]
- Treat the installed game and extensions as read-only, never read saves, and use only a disposable Creative Custom campaign or explicit test copy after a written plan. [VERIFIED: AGENTS.md]
- Classify X4 integration claims as documented, observed, inferred, or unknown; runtime evidence outranks static assumptions when test setup and freshness are known. [VERIFIED: .agents/skills/live-galaxy-x4-integration/SKILL.md]
- Preserve explicit unknown/unsupported data rather than fabricating absence; bound game-thread work and degrade safely when the bridge is unavailable. [VERIFIED: .agents/skills/live-galaxy-x4-integration/SKILL.md]
- Keep pure Lua/normalization/scheduling separate from X4 globals and verify through static, pure Lua, fake adapter, Rust, and disposable in-game layers. [VERIFIED: .agents/skills/live-galaxy-x4-tests/SKILL.md]
- Rust boundaries must not panic on external input; use typed values, deterministic ordering, replay inputs, explicit state machines, and bounded resources. [VERIFIED: .agents/skills/live-galaxy-rust-conventions/SKILL.md]
- Planning artifacts are English. `.planning/**` is intentionally ignored by repository Markdownlint, so no alternate formatter is forced. [VERIFIED: AGENTS.md]

## Summary

Phase 1 should establish one thin X4 producer boundary and one Rust ingest boundary, with no return-command family. The bridge must admit a telemetry session only after a typed compatibility decision, then validate each bounded frame before atomically advancing the accepted projection. A rejection is evidence, not an update. [VERIFIED: .planning/phases/01-read-only-observation-spine/01-CONTEXT.md:21-45]

Installed `X4.exe` reports `FileVersion: 9.0.0.0`; installed SirNukes Mod Support APIs declares Windows named-pipe support. This proves local target/version and an available support dependency, but does **not** prove Live Galaxy's eventual embedded-Lua, MD, scheduling, or pipe behavior. Those remain disposable-probe gates. [OBSERVED: F:/SteamLibrary/steamapps/common/X4 Foundations/X4.exe VersionInfo; DOCUMENTED: F:/SteamLibrary/steamapps/common/X4 Foundations/extensions/sn_mod_support_apis/content.xml]

The local X4 Live repository demonstrates a useful, non-authoritative precedent: single-or-batched event envelopes over `\\.\pipe\x4_live_mcp`, section rotation, explicit quality, atomic batch rejection, and bounded same-handle write retry. Its checkout was dirty and `git fetch --prune origin` could not write `.git/FETCH_HEAD`; therefore these are observed local precedent, not current-X4 guarantees. [OBSERVED: F:/Agent Projects/X4/tools/x4-live-protocol.md:3-11,63-146; OBSERVED: F:/Agent Projects/X4/tests/test_x4_live.py:1477-1885]

**Primary recommendation:** Plan a telemetry-only protocol with an explicit `Compatible | DegradedRequiresX4Restart | RejectedSession` admission outcome; keep the current accepted snapshot immutable until a complete, validated reconciliation succeeds. [ASSUMED]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
| --- | --- | --- |
| Runtime enumeration and observation time | X4 adapter | Mission Director scheduler | X4 is authoritative for live world state and must emit only bounded observations. [VERIFIED: CONTEXT.md:21-25] |
| Cooperative sampling and frame production | X4 adapter | Transport/session | Game-side work must remain bounded before any bridge I/O. [VERIFIED: live-galaxy-x4-integration/SKILL.md] |
| Frame/session validation and compatibility decision | Rust bridge | X4 adapter | Rust owns untrusted transport normalization; X4 only exposes capabilities. [VERIFIED: AGENTS.md] |
| Current snapshot, rejection evidence, reconciliation | Rust bridge | Storage | Atomic acceptance preserves last valid projection. [VERIFIED: live-galaxy-rust-tests/SKILL.md] |
| Strategy/model/report/mutation | — | — | These are explicitly deferred; no Phase 1 protocol operation owns them. [VERIFIED: CONTEXT.md:23-27,103-105] |

## Standard Stack

### Core

| Library or runtime | Version | Purpose | Why Standard |
| --- | --- | --- |
| Rust/Cargo workspace | `rustc 1.97.1`, `cargo 1.97.1` available locally | Typed ingest, state machine, fixtures, and test runner. | No workspace exists yet; begin with a small pure domain crate plus adapter crate. [OBSERVED: local environment probe] |
| X4 9.00 + installed `sn_mod_support_apis` | X4 `9.0.0.0`; extension `version="195"` | Thin embedded-Lua/MD observer and Windows named-pipe dependency. | Installed local evidence; direct API behavior still needs probe. [OBSERVED: X4.exe VersionInfo; DOCUMENTED: installed `content.xml`] |
| `serde`, `serde_json`, `thiserror` | Resolve from crates.io when the workspace is created | Typed wire fixture decoding and recoverable errors. | Registry metadata identifies established upstream repositories and no legitimacy warnings. [VERIFIED: crates.io package-legitimacy check, 2026-08-29] |

### Supporting

| Library or runtime | Version | Purpose | When to Use |
| --- | --- | --- |
| `tokio` | Resolve from crates.io when needed | Bounded bridge I/O, cancellation, and task ownership. | Only at the transport seam; keep deterministic normalization synchronous. [VERIFIED: crates.io package-legitimacy check, 2026-08-29] |
| `tracing` | Resolve from crates.io when needed | Structured, bounded diagnostics correlated with frame/session IDs. | Use after a redaction contract exists. [VERIFIED: crates.io package-legitimacy check, 2026-08-29] |
| Busted | Not installed | Pure Lua tests outside X4. | Add only after an X4 runtime syntax probe selects a compatible version. [VERIFIED: live-galaxy-x4-tests/SKILL.md] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Versioned local named-pipe telemetry | In-process FFI or direct database scraping | Both widen the game-side trust/failure boundary and do not meet the thin adapter/reconnect objective. [ASSUMED] |
| Explicit section quality | Implicit null/empty conventions | Null cannot distinguish known-empty, unknown, partial, stale, or unsupported. [VERIFIED: F:/Agent Projects/X4/AGENTS.md] |

**Installation:** No package installation belongs in this research output. Wave 0 pins versions through the Rust dependency manifest.

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
| --- | --- | --- | --- | --- | --- | --- |
| `serde`, `serde_json`, `thiserror`, `tokio`, `tracing`, `cargo-mutants` | crates.io | Earliest releases 2014–2021 | 15K–26M weekly | Established upstream repositories returned by crates.io | OK | Eligible for planned use; pin exact versions in repository manifests/tooling. |

**Packages removed due to [SLOP] verdict:** none.

**Packages flagged as suspicious [SUS]:** none after the registry check was rerun with network access. `cargo-mutants` remains unavailable locally until Phase 1 tooling installs its pinned version. [VERIFIED: crates.io package-legitimacy check, 2026-08-29]

## Architecture Patterns

### System Architecture Diagram

```text
X4 9.00 runtime state
  -> thin Lua/MD producer (bounded enumeration; no mutation vocabulary)
  -> telemetry-only frame queue
  -> session hello/capability negotiation
       -> incompatible / missing required capability
          -> DegradedRequiresX4Restart + bounded health evidence
       -> compatible
          -> Rust frame size/schema/identity/version validation
             -> reject: append bounded rejection evidence; retain accepted snapshot
             -> accept: atomically replace touched sections
                -> complete reconciliation marker?
                   -> yes: reconcile tombstones and freeze new snapshot version
                   -> no: preserve section coverage/quality as partial
```

### Recommended Project Structure

```text
crates/
├── observation-domain/     # pure typed identities, quality, versions, reconciliation policy
├── observation-ingest/     # decoding, validation, atomic projection port
└── x4-bridge/              # transport/session adapter and diagnostics port
extensions/
└── live_galaxy/            # thin X4 Lua + MD scheduler only
tests/
├── fixtures/               # valid and adversarial envelopes
└── x4-disposable/          # written Creative Custom probe procedures/results
```

### Pattern 1: Validate, Then Atomically Admit

**What:** Decode to a bounded envelope, reject before projection mutation, then atomically replace only declared sections and advance snapshot state. [VERIFIED: live-galaxy-rust-tests/SKILL.md]

**When to use:** Every observation frame, completion marker, protocol hello, and reconnect attempt.

**Evidence:** The X4 Live precedent contains `test_invalid_complete_marker_rejects_entire_pipe_batch` and `test_atomic_batch_preserves_idempotency_and_state_version`. [OBSERVED: F:/Agent Projects/X4/tests/test_x4_live.py:1851-1885]

### Pattern 2: Sectioned Freshness and Reconciliation

**What:** Publish a compact index plus independently fresh sections; accept removal only from an explicit successful complete marker for that scope. [OBSERVED: F:/Agent Projects/X4/tools/x4-live-protocol.md:63-146]

**When to use:** Runtime asset discovery, slower capacity/ownership detail, and high-churn event tails.

### Pattern 3: Sticky Fail-Closed Session State

**What:** Treat invalid protocol major, missing mandatory read-only capability, or game-facing build mismatch as terminal for that X4 runtime. State the exact `X4 restart required` reason; compatible Rust-only reconnects create a new session without changing the game adapter. [VERIFIED: CONTEXT.md:31-36]

**When to use:** Handshake and reconnect only. It is an implementation policy derived from the locked restart boundary, not an observed X4 native behavior. [INFERRED: CONTEXT.md:31-36]

### Anti-Patterns to Avoid

- **Monolithic world snapshot:** one payload makes slow optional scans stall or erase fast sections. Use independent section metadata. [OBSERVED: F:/Agent Projects/X4/tools/x4-live-protocol.md:63-87]
- **Implicit absence:** do not map a missing property or empty list to known-empty without scope completion. [VERIFIED: F:/Agent Projects/X4/AGENTS.md]
- **Transport-driven mutation:** no inbound command enum, command dispatcher, or generic action payload exists in this phase. [VERIFIED: CONTEXT.md:26-27]
- **Hot-reload assumption:** do not promise Lua/MD hot reload; X4 restart behavior needs probe evidence. [UNKNOWN]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Cross-boundary JSON typing | Stringly typed maps and ad-hoc parsing | Versioned Rust data types plus fixture decoder | Enables deterministic invalid-input tests. [VERIFIED: live-galaxy-rust-conventions/SKILL.md] |
| Transactional reconciliation | Per-row best-effort mutation | One atomic projection transaction behind a storage port | Prevents a malformed completion marker from erasing accepted state. [OBSERVED: F:/Agent Projects/X4/tests/test_x4_live.py:1851-1885] |
| Retry policy | Lua reconnect/hot-reload loop | Explicit bounded session state and a tiny queued raw frame policy | Keeps native interaction and failure behavior auditable. [OBSERVED: F:/Agent Projects/X4/tools/x4-live-protocol.md:108-113] |
| Test game world | Save-file fixtures | Pure fixtures, fake X4 adapter, disposable Creative Custom probe | Saves are prohibited. [VERIFIED: AGENTS.md] |

**Key insight:** phase correctness depends on preserving the last known-good projection and its evidence, not maximizing the number of fields sampled. [INFERRED: OBS-03 and OBS-07]

## Common Pitfalls

### Pitfall 1: Treating a fresh heartbeat as fresh detail

**What goes wrong:** Strategy receives stale capacity or ownership data as if it were newly observed.

**How to avoid:** Carry independent section capture time, coverage, quality, and scope completion; do not let an unrelated heartbeat refresh them. [OBSERVED: F:/Agent Projects/X4/tools/x4-live-protocol.md:63-87]

### Pitfall 2: Reconciliation deletes after incomplete collection

**What goes wrong:** A timeout or optional-section failure is converted into fabricated entity deletion.

**How to avoid:** Reconcile only from a validated complete marker of the same scope and retain a partial/unknown coverage state otherwise. [OBSERVED: F:/Agent Projects/X4/tools/x4-live-protocol.md:83-87,146]

### Pitfall 3: Bridge restart is confused with game-facing incompatibility

**What goes wrong:** An operator restarts X4 unnecessarily, or a changed adapter continues against an incompatible bridge.

**How to avoid:** Handshake must declare a compatibility disposition and restart owner; test Rust-only reconnect independently from game-facing mismatch. [VERIFIED: CONTEXT.md:31-34]

### Pitfall 4: Source-text assertions stand in for X4 proof

**What goes wrong:** Lua/MD looks correct yet performs unbounded or unsupported native calls at runtime.

**How to avoid:** Record static, pure Lua, fake-adapter, and observed-in-X4 results separately. [VERIFIED: live-galaxy-x4-tests/SKILL.md]

## Code Examples

Verified contract shape, with all field names intentionally proposed rather than copied from a foreign protocol:

```text
TelemetrySession -> CapabilityDecision -> FrameValidation -> AtomicAdmission
                                      -> RejectionEvidence (no projection change)
```

The three terminal labels above are design names, not X4 values. [ASSUMED]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| One all-or-nothing background asset snapshot | Rotating independently fresh sections with bounded per-tick work | X4 Live v046 precedent | Use as a design hypothesis only; prove it in Live Galaxy's disposable run. [OBSERVED: F:/Agent Projects/X4/tools/x4-live-protocol.md:63-87] |
| Permanent halt on first pipe write failure | Bounded same-handle raw-frame retry, then terminal halt | X4 Live v051 precedent | Do not copy the policy blindly; Live Galaxy must choose and test its own session contract. [OBSERVED: F:/Agent Projects/X4/tools/x4-live-protocol.md:108-113] |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | A Rust async runtime is needed at the transport seam. | Standard Stack | Could overcomplicate the first bridge. |
| A2 | Named pipes are the selected Live Galaxy topology. | Standard Stack | A different X4-safe transport may be required by a probe. |
| A3 | Session disposition names and exact hello schema can be designed as proposed. | Architecture Patterns | Planner may need a protocol-design checkpoint before coding. |
| A4 | The candidate Rust crates are acceptable dependencies. | Package Legitimacy Audit | Installation is blocked until legitimate, reachable registry evidence exists. |

## Open Questions

1. **Which exact X4 APIs expose sector, asset, capacity, and ownership facts in the installed 9.00 runtime?**
   - What we know: X4 9.00 and Windows named-pipe support are installed. [OBSERVED: local install metadata]
   - What's unclear: Lua/MD call shapes, identity stability, and permission/context edge cases.
   - Recommendation: one disposable Creative Custom probe per source family, with a captured envelope and independent readback.
2. **Does X4's embedded Lua accept the syntax/tooling proposed for standalone pure tests?**
   - What we know: Busted is absent locally. [OBSERVED: environment probe]
   - Recommendation: probe runtime version/syntax before pinning Busted or Lua mutation tooling.
3. **What initial numeric cadence/frame/queue limits are safe?**
   - What we know: X4 Live's documented limits are precedent only. [OBSERVED: F:/Agent Projects/X4/tools/x4-live-protocol.md:72-113]
   - Recommendation: make each limit configuration, collect normal-speed and SETA baselines, then lock measured values.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| X4 executable | Disposable X4 observation probe | ✓ | 9.0.0.0 | None |
| SirNukes Mod Support APIs | Windows named-pipe research | ✓ | extension `195` | Treat alternative transport as a new design/probe decision |
| Rust/Cargo | Rust workspace and tests | ✓ | 1.97.1 | None |
| `cargo-mutants` | Later pure-Rust mutation baseline | ✗ | — | Defer until a workspace and verified package install |
| Lua interpreter | Pure Lua local tests | ✗ | — | Use only static/fake contracts until probe and runner setup |
| Busted | Pure Lua local tests | ✗ | — | Same as Lua interpreter |
| crates.io registry | Dependency verification | ✗ | TLS credential failure | Human-verified network/credential repair before install |

**Missing dependencies with no fallback:** a verified Rust dependency source is required before the first Cargo implementation plan can install crates.

**Missing dependencies with fallback:** standalone Lua/Busted can be deferred while Rust fixtures and static XML checks establish Wave 0.

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | None in Live Galaxy yet — Wave 0 creates Cargo test harness; pure Lua runner waits for runtime confirmation. [OBSERVED: repository root inventory] |
| Config file | `none — see Wave 0` |
| Quick run command | `cargo test -p observation-domain` [ASSUMED until workspace exists] |
| Full suite command | `cargo test --workspace` [ASSUMED until workspace exists] |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- |
| OBS-01 | Session negotiation and bounded producer contract | Rust unit + fake adapter | `cargo test -p observation-ingest session` | ❌ Wave 0 |
| OBS-02 | Stable typed identity/version admission | Rust unit/property | `cargo test -p observation-domain identity` | ❌ Wave 0 |
| OBS-03 | Quality/freshness/coverage survive snapshot freezing | Rust unit | `cargo test -p observation-domain section_quality` | ❌ Wave 0 |
| OBS-06 | Complete markers reconcile runtime-discovered assets only | Rust integration + fake adapter | `cargo test -p observation-ingest reconciliation` | ❌ Wave 0 |
| OBS-07 | Bad/duplicate/stale/out-of-order frames preserve last accepted snapshot | Rust integration | `cargo test -p observation-ingest rejection` | ❌ Wave 0 |
| OBS-08 | Protocol has telemetry frames only | Schema/contract test | `cargo test -p x4-bridge protocol_vocabulary` | ❌ Wave 0 |
| VAL-06 | Exact runtime semantics are separately evidenced | Static + disposable in-game procedure | Manual recorded probe | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** focused affected crate tests once they exist. [ASSUMED]
- **Per wave merge:** `cargo test --workspace` once the workspace exists. [ASSUMED]
- **Phase gate:** full local suite plus an explicitly recorded disposable X4 9.00 probe; neither substitutes for the other. [VERIFIED: live-galaxy-x4-tests/SKILL.md]

### Wave 0 Gaps

- [ ] Cargo workspace and pure `observation-domain` test crate.
- [ ] Adversarial envelope fixture corpus and fake X4 adapter.
- [ ] XML/static package validation for the first extension.
- [ ] Runtime Lua version/syntax probe, then pinned pure Lua runner if supported.
- [ ] Disposable Creative Custom probe script and evidence template recording X4 version, extension list, real/game time, SETA, health, expected readback, and failure diagnostics.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | No | Local bridge session identity is compatibility metadata, not user authentication. [ASSUMED] |
| V3 Session Management | Yes | Explicit session lifecycle, capability admission, bounded reconnect, and terminal degraded state. [VERIFIED: CONTEXT.md:31-36] |
| V4 Access Control | Yes | Telemetry-only protocol vocabulary; no mutation dispatcher. [VERIFIED: CONTEXT.md:26-27] |
| V5 Input Validation | Yes | Size/schema/identity/version validation before atomic admission. [VERIFIED: live-galaxy-rust-conventions/SKILL.md] |
| V6 Cryptography | No | No remote credential or encryption contract is in Phase 1 scope. [ASSUMED] |

### Known Threat Patterns for the Observation Spine

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Malformed or oversized frame | Tampering / DoS | Hard size ceiling, schema decode before projection mutation, bounded diagnostic. [VERIFIED: live-galaxy-rust-tests/SKILL.md] |
| Replay, duplicate, or reordered frame | Tampering | Stable event/state identity plus idempotent acceptance and ordering checks. [VERIFIED: live-galaxy-rust-tests/SKILL.md] |
| Capability downgrade or incompatible adapter | Spoofing / Tampering | Require explicit capability match; fail closed and name X4 restart condition. [VERIFIED: CONTEXT.md:31-36] |
| Game-thread overload | DoS | Cooperative bounded stages, backpressure, and normal-speed/SETA probes. [VERIFIED: live-galaxy-x4-tests/SKILL.md] |
| Accidental mutation surface | Elevation of privilege | Separate telemetry schema from later report/command phases; assert no mutation variants. [VERIFIED: CONTEXT.md:26-27] |

## Sources

### Primary (HIGH confidence)

- [Phase 1 context](01-CONTEXT.md) — locked authority, scope, rejection, observation, and restart decisions.
- [Live Galaxy AGENTS](../../../AGENTS.md) and project Rust/X4 skills — project authority, trust, safety, and verification rules.
- Installed `F:/SteamLibrary/steamapps/common/X4 Foundations/X4.exe` — local executable version observation.
- Installed `F:/SteamLibrary/steamapps/common/X4 Foundations/extensions/sn_mod_support_apis/content.xml` — local extension declaration.

### Secondary (MEDIUM confidence)

- `F:/Agent Projects/X4/tools/x4-live-protocol.md` — local observed pipe, quality, bounded-scheduling, and reconciliation precedent.
- `F:/Agent Projects/X4/tests/test_x4_live.py` — local observed regression coverage for caps, rejection, idempotency, and reconciliation.
- `F:/Agent Projects/TALKER/AGENTS.md` — process precedent only; its current dirty checkout supplied no Phase 1 runtime fact.

### Tertiary (LOW confidence)

- Rust crate/runtime candidates and the exact Live Galaxy session framing — marked `[ASSUMED]` pending registry/documentation and disposable runtime probes.

## Metadata

**Confidence breakdown:**

- Standard stack: LOW — runtime is present but crate registry and Lua runner verification are blocked.
- Architecture: MEDIUM — locked product boundaries and strong local precedent, but no Live Galaxy X4 runtime probe.
- Pitfalls: MEDIUM — backed by local X4 precedent and project skills; cadence/limit values remain unmeasured.

**Research date:** 2026-08-28
**Valid until:** 2026-09-04 for X4/runtime findings; revisit after the first disposable probe or dependency install.
