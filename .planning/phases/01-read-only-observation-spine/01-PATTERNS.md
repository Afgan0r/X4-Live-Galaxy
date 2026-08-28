# Phase 1: Read-Only Observation Spine - Pattern Map

**Mapped:** 2026-08-28  
**Files analyzed:** 7 proposed implementation/test areas  
**Analogs found:** 2 external precedents / 7 areas  

This repository has no Live Galaxy source implementation yet (`ast-index stats` reports 0 files and 0 symbols). The X4 repository precedents below are read-only observations, not Live Galaxy authority or code to copy. Their checkouts were dirty and `git fetch --prune origin` could not update `.git/FETCH_HEAD`; treat their details as versioned local precedent and re-verify during implementation.

## File Classification

| New/Modified File or Area | Role | Data Flow | Closest Precedent | Match Quality |
| --- | --- | --- | --- | --- |
| `crates/observation-domain/src/lib.rs` | model/domain | transform + validation | `F:/Agent Projects/X4/tools/x4-live-protocol.md:63-105,132-148` | conceptual role-match |
| `crates/observation-ingest/src/lib.rs` | service/ingest | request-response batch admission | `F:/Agent Projects/X4/tests/test_x4_live.py:1851-1885` | behavior-match |
| `crates/x4-bridge/src/lib.rs` | service/transport/session | streaming request-response | `F:/Agent Projects/X4/tools/x4-live-protocol.md:3-5,107-125` | role/data-flow match |
| `extensions/live_galaxy/` Lua/MD runtime | adapter/scheduler | streaming producer | `F:/Agent Projects/X4/tools/x4-live-protocol.md:63-89,107-114` | role/data-flow match |
| `tests/fixtures/` | test fixtures | batch/replay | `F:/Agent Projects/X4/tests/test_x4_live.py:1851-1885` | behavior-match |
| `tests/x4-disposable/` | integration evidence | black-box streaming | `.agents/skills/live-galaxy-x4-tests/SKILL.md` (in-game layer) | no code analog |
| Cargo workspace/package manifests and extension manifest | config/package | build/compatibility | `F:/Agent Projects/X4/tools/x4-live-protocol.md:56-61,121-130` | conceptual role-match |

## Pattern Assignments

### `crates/observation-domain/src/lib.rs` (domain, transform/validation)

**Analog:** `F:/Agent Projects/X4/tools/x4-live-protocol.md:63-105,132-148` (precedent only).

Create typed identities, source, capture/observation time, monotonic state/event versions, section freshness/coverage/quality, and explicit complete-marker scope. Keep raw transport formats out of domain types, following `live-galaxy-rust-conventions/SKILL.md` (domain consumes normalized typed state, not wire messages).

**Concrete precedent to preserve:** the X4 protocol describes independently fresh sections and says a heartbeat or neighboring section does not refresh selected detail (`tools/x4-live-protocol.md:65-69`). It also requires complete markers to represent a successful index cycle and forbids false deletion when detail production fails (`:83-89`). Implement equivalent semantics with original types and names; do not copy the foreign protocol.

**Required symbols/decisions:** typed `EntityId`/`EventId` (or equivalent newtypes), `ObservationVersion`, `SectionQuality`/coverage state, `SectionDescriptor`, `CompleteMarker`, and deterministic validation helpers. Preserve `unknown`, `partial`, `stale`, and `unsupported`; never infer known-empty from null/empty.

### `crates/observation-ingest/src/lib.rs` (ingest service, request-response batch admission)

**Analog:** `F:/Agent Projects/X4/tests/test_x4_live.py:1851-1885` (test behavior, not implementation).

The named tests cover invalid complete markers rejecting the entire batch and atomic batch idempotency/state-version preservation. The actionable pattern is validate every envelope and marker before mutating the projection; commit all accepted sections and reconciliation together; on failure retain the last accepted snapshot and emit bounded rejection evidence. This directly follows `live-galaxy-rust-tests/SKILL.md` requirements for malformed, oversized, duplicate, stale, and out-of-order input without partial state.

**Required symbols:** bounded envelope decoder, `validate_batch`, admission/rejection outcome, immutable accepted snapshot/projection, reconciliation transaction boundary, and diagnostic metadata (without raw rejected payloads).

**Tests to create:** malformed/oversized, duplicate, stale, out-of-order, invalid completion, atomic rollback, and idempotent replay fixtures. The X4 precedent’s test names are useful search anchors: `test_invalid_complete_marker_rejects_entire_pipe_batch` and `test_atomic_batch_preserves_idempotency_and_state_version`.

### `crates/x4-bridge/src/lib.rs` (transport/session service, streaming request-response)

**Analog:** `F:/Agent Projects/X4/tools/x4-live-protocol.md:3-5,107-125` (precedent only).

Define a telemetry-only session state machine and capability handshake. A compatible Rust restart may reconnect; incompatible protocol or game-facing build/capability mismatch transitions to an explicit terminal degraded/restart-required state. Keep this separate from user authentication. The precedent records an immutable raw frame, bounded same-handle retry, retry exhaustion, and alignment metadata (`:107-125`); Phase 1 should use these as design constraints only after measuring/probing the actual X4 runtime.

**Required symbols:** versioned hello/capability envelope, `CapabilityDecision` with compatible/degraded-restart/rejected outcomes, bounded frame limits, session generation/sequence tracking, and telemetry-only frame enum. There must be no command/mutation variant, report return path, or generic action dispatcher.

### `extensions/live_galaxy/` (thin X4 adapter, streaming producer)

**Analog:** `F:/Agent Projects/X4/tools/x4-live-protocol.md:63-89,107-114` (precedent only).

Separate pure normalization/scheduling from X4 globals, native calls, and pipe I/O as required by `live-galaxy-x4-tests/SKILL.md`. The adapter may enumerate runtime sectors/assets/capacity/ownership in bounded slices, attach source/time/version/quality, serialize telemetry, and enqueue bounded frames. It must not mutate game state or assume a fixed map/job count. Use cooperative scheduling, backpressure, save suppression, and explicit unsupported/partial states; do not promise hot reload or exact cadence before a disposable probe.

**Required symbols/files:** extension manifest, thin Lua producer, Mission Director scheduler hook (if runtime evidence confirms it), pure serializer/normalizer boundary, and runtime-health/diagnostic projection. Keep any native/FFI seam minimal and explicitly tested.

### `tests/fixtures/` (fixtures, batch/replay)

**Analog:** `F:/Agent Projects/X4/tests/test_x4_live.py:1851-1885`.

Store small valid and adversarial telemetry envelopes, handshake combinations, identity/version sequences, and reconciliation markers. Fixtures must be deterministic and independent of live models, network, sleeps, save files, or mutable X4 state. Test public outcomes and state transitions, not private helper implementation.

### `tests/x4-disposable/` (integration evidence, black-box streaming)

**Analog:** no existing Live Galaxy or directly reusable code analog.

Follow `live-galaxy-x4-integration/SKILL.md` and `live-galaxy-x4-tests/SKILL.md`: use a disposable Creative Custom campaign, capture exact X4 version/mod list/scenario/real and game time/SETA/health/expected versus observed results, and keep static, pure-Lua, fake-adapter, and observed-in-X4 statuses separate. Never inspect saves and never treat source-text assertions as runtime proof. This area is evidence/procedure, not a public runtime dependency.

### Workspace/package manifests (config/package)

**Analog:** `F:/Agent Projects/X4/tools/x4-live-protocol.md:56-61,121-130` (release identity/alignment precedent).

Create only the minimum Cargo workspace/package and extension identity needed for Phase 1. Version protocol and capabilities explicitly; ensure release identity/hash/alignment cannot silently report compatibility when extension, bridge, schema, or live heartbeat disagree. Candidate crates from research remain unverified: do not install dependencies until registry/source legitimacy is human-verified.

## Shared Patterns

### Trust and authority

**Sources:** `AGENTS.md`; `live-galaxy-rust-conventions/SKILL.md`; `live-galaxy-x4-integration/SKILL.md`  
**Apply to:** all adapter, bridge, and domain files.

X4 owns authoritative state. Rust owns normalization, validation, projection, recovery, and bounded diagnostics. Model/command/mutation surfaces are out of scope. External input must be typed, bounded, recoverable, and unable to panic or partially mutate accepted state.

### Quality and reconciliation

**Sources:** `F:/Agent Projects/X4/tools/x4-live-protocol.md:65-69,83-89,132-148`; `live-galaxy-rust-tests/SKILL.md`  
**Apply to:** domain, ingest, adapter, and fixtures.

Section freshness is independent. Reconcile removals only after a validated complete marker for the same runtime scope. Invalid or incomplete input records bounded evidence and leaves the last known-good snapshot unchanged.

### Bounded streaming and recovery

**Sources:** `F:/Agent Projects/X4/tools/x4-live-protocol.md:71-89,107-119`; `live-galaxy-x4-integration/SKILL.md`  
**Apply to:** adapter and bridge.

Use cooperative bounded work, hard payload/queue/retry ceilings, backpressure, save suppression, and explicit terminal health. Exact numeric limits in the external precedent are hypotheses, not defaults for Live Galaxy; measure them in disposable normal-speed and SETA probes.

### Verification layering

**Sources:** `live-galaxy-x4-tests/SKILL.md`; `live-galaxy-x4-integration/SKILL.md`  
**Apply to:** all test/evidence areas.

Run static/schema checks, pure Rust/Lua tests, fake adapter contract tests, then minimal disposable X4 probes. Report `verified locally`, `pending game smoke test`, and `observed in X4` separately.

## Anti-Patterns

- Monolithic snapshot or unbounded game-thread callback; use sectioned cooperative production.
- Treating empty/null as known-empty without coverage completion.
- Committing a section or tombstone before full batch validation.
- Accepting stale, duplicate, or out-of-order traffic as a newer snapshot.
- Requiring an X4 restart for a compatible Rust-only bridge reconnect.
- Hiding protocol/build mismatch while continuing to consume telemetry.
- Introducing any fleet/economy/diplomacy/institution or generic mutation command in the Phase 1 vocabulary.
- Copying X4 Live code or protocol names without license/provenance review.
- Using save files, broad runtime hooks, sleeps, live models, or source-text-only assertions as proof.

## No Existing Live Galaxy Analog

All listed areas have no in-repository implementation analog. The repository is an empty implementation baseline. In particular, there is no existing Cargo workspace, observation domain, ingest service, X4 extension, fixture corpus, or disposable probe harness. Planners must use the project skills and canonical references before implementation and treat X4/TALKER material as external precedent only.

## Canonical Sources to Read Before Implementation

- `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/research/ARCHITECTURE.md`, `.planning/research/SUMMARY.md`.
- `AGENTS.md` and the four Phase 1 skills named in the task.
- `F:/Agent Projects/X4/AGENTS.md`, `F:/Agent Projects/X4/tools/x4-live-protocol.md`, and the relevant range of `F:/Agent Projects/X4/tests/test_x4_live.py`.
- Installed X4 9.00 and `extensions/sn_mod_support_apis/content.xml` only through the written disposable-probe plan; never modify installed files or read saves.

## Metadata

**Analog search scope:** Live Galaxy structural index (empty), named X4 protocol/test precedent, TALKER process contract, and required project skills.  
**Files scanned:** 7 required project files plus game-repo-standard skill and four routed references; external `X4/AGENTS.md`, `X4/tools/x4-live-protocol.md`, `X4/tests/test_x4_live.py`, and `TALKER/AGENTS.md`.  
**Freshness:** X4 and TALKER status checked 2026-08-28; both dirty; remote fetch failed to update `FETCH_HEAD`.  
**Provenance:** External excerpts are paraphrased/minimal signatures only; no foreign implementation code is copied.
