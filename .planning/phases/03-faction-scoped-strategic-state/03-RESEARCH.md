# Phase 3: Faction-Scoped Strategic State - Research

**Researched:** 2026-08-29  
**Domain:** Pure Rust strategic-packet derivation from frozen observation projections  
**Confidence:** HIGH for local boundary and locked internal profiles; MEDIUM for documented community-wiki canon context

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Faction knowledge

- **D-01:** Each faction receives authoritative own-state facts plus only the
  operational information available to that faction under a recorded
  visibility policy.
- **D-02:** Complete static resource-map potential may be known, but changing
  foreign production, fleet, movement, and station facts are not omniscient.
- **D-03:** Missing, stale, inaccessible, or unsupported facts remain explicit
  and cannot be filled by model inference.

### Institution roster

- **D-04:** Each faction has exactly three primitive institutions mapped to:
  defense and military strategy; economy and logistics; territorial
  development and infrastructure.
- **D-05:** ZYA and ARG use canon-grounded institution identities and different
  doctrine-conditioned fixed priorities while sharing those three engine
  capability contracts.
- **D-06:** All institutions see the same authoritative faction-visible
  snapshot. Private institutional knowledge and false beliefs are excluded.

### Executive diplomatic posture

- **D-07:** Diplomacy is not a fourth institution. The strategic packet exposes
  the typed facts and allowed dispositions needed for the Executive to preserve
  relations, de-escalate, increase pressure, or seek limited coordination
  against a shared threat.
- **D-08:** This posture concerns only Shadow planning; it cannot negotiate with
  another faction or alter X4 relations.

### Determinism

- **D-09:** Equivalent frozen snapshots and policies yield canonically ordered
  facts, priorities, allowed primitives, and admission inputs.

### the agent's Discretion

Exact schemas, scoring formulas, priority weights, and canon-grounded
institution names are owned by research and planning. Names and profiles must
be supported by X4 evidence rather than invented LLM stereotypes.

### Deferred Ideas (OUT OF SCOPE)

- Private institutional knowledge, mutable influence, sabotage, and internal
  political simulation are post-alpha work.
- A separate diplomacy institution and inter-faction negotiation are outside
  milestone 0.1.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| OBS-04 | Provide supported economic, military, territorial, and threat facts. | Derive a typed fact inventory from frozen projection sections without inventing values. |
| OBS-05 | Represent shared XEN pressure and observed KHK. | Use a typed threat subject plus explicit fact availability, not hostile-mind behavior. |
| MIND-02 | Apply and record faction visibility. | Freeze a versioned visibility-policy identifier into each packet. |
| MIND-03 | Derive bounded strategic inputs. | Pure projection-to-packet compiler owns fact filtering, scoring inputs, and primitives. |
| MIND-04 | Reproduce equivalent inputs. | Ordered maps/vectors, total sort keys, and pure tests establish canonical output. |
| INST-01 | Exactly three doctrine-conditioned institutions. | One shared capability enum and two versioned faction profiles. |
| INST-02 | Institutions share one faction-visible snapshot. | Packet owns one filtered fact set; institutions receive views, never source projections. |
</phase_requirements>

## Summary

Implement Phase 3 as a new pure `strategic-state` domain crate. Its only input is a frozen `observation_ingest::ProjectionSnapshot`; it must neither read transport data nor invoke models or storage. The compiler validates section quality and visibility, emits an immutable faction-visible fact set, then derives the three institution capability views, Executive-only diplomatic inputs, deterministic priority scores, allowed Shadow primitive candidates, and admission-input fingerprint. [VERIFIED: crates/observation-ingest/src/snapshot.rs:1-67] [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md]

The existing input boundary is deliberately narrow. `ProjectionSnapshot` retains observations in a `BTreeMap`, while every observation carries a typed identity, source, time, version, and a quality state; malformed or stale incoming batches preserve the last accepted projection. The Phase 3 compiler must preserve—not reinterpret—these distinctions. [VERIFIED: crates/observation-ingest/src/snapshot.rs:18-67] [VERIFIED: crates/observation-domain/src/section.rs:3-43] [VERIFIED: crates/observation-ingest/tests/atomic_rejection.rs]

**Primary recommendation:** Build a pure, total, fail-closed packet compiler with one shared capability contract, two evidence-gated faction profiles, and explicit unavailable facts; defer mind persistence to Phase 4 and all proposal/admission execution to Phase 5.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
| --- | --- | --- | --- |
| Frozen projection ingestion | API / Backend | X4 adapter | Phase 1 has already admitted the authoritative normalized projection. [VERIFIED: crates/observation-ingest/src/lib.rs:1-67] |
| Visibility filtering and fact availability | API / Backend | — | It is deterministic domain policy, before any model context. [VERIFIED: .agents/skills/live-galaxy-rust-conventions/SKILL.md] |
| Institution capability views and priorities | API / Backend | — | They are pure typed inputs, not UI or model-owned data. [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md] |
| ZYA–ARG Shadow diplomatic inputs | API / Backend | — | The Executive owns posture inputs; diplomacy is explicitly not a fourth institution. [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md] |
| Mind/initiative history | Database / Storage | API / Backend | Explicitly deferred to Phase 4. [VERIFIED: .planning/ROADMAP.md] |
| Model deliberation and proposal admission | API / Backend | — | Explicitly deferred to Phase 5. [VERIFIED: .planning/ROADMAP.md] |

## Project Constraints (from AGENTS.md)

- X4 remains authoritative; model output must never bypass deterministic validation. [VERIFIED: AGENTS.md]
- Preserve deterministic replay inputs, stable ordering, typed domain values, bounded work, no unsafe Rust, and no recoverable-boundary `unwrap` or `expect`. [VERIFIED: AGENTS.md] [VERIFIED: .agents/skills/live-galaxy-rust-conventions/SKILL.md]
- Treat installed X4, mods, TALKER, and X4 Live MCP as read-only evidence; do not inspect saves. [VERIFIED: AGENTS.md]
- Separate documented, observed, inferred, and unknown X4 claims; local/fake and in-game evidence must remain distinct. [VERIFIED: AGENTS.md] [VERIFIED: .agents/skills/live-galaxy-x4-integration/SKILL.md]
- Keep Rust modules cohesive and at most 200 physical lines; use focused unit/contract/property tests, then measured mutation testing for pure high-risk logic. [VERIFIED: .agents/skills/live-galaxy-rust-conventions/SKILL.md] [VERIFIED: .agents/skills/live-galaxy-rust-tests/SKILL.md]

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| Workspace Rust | pinned in repository | Pure domain compiler and tests | Existing workspace and project conventions already establish Rust as the deterministic host. [VERIFIED: Cargo.toml] [VERIFIED: .agents/skills/live-galaxy-rust-conventions/SKILL.md] |
| `observation-domain` | workspace-local | Typed identity, time, version, and section quality | Reuse the Phase 1 public domain instead of duplicating stringly state. [VERIFIED: crates/observation-domain/src/lib.rs] |
| `observation-ingest` | workspace-local | Frozen accepted `ProjectionSnapshot` input | It is the accepted, reconciled boundary. [VERIFIED: crates/observation-ingest/src/lib.rs:1-67] |

### Supporting

No new external package is recommended. Standard-library ordered collections are sufficient for this phase; adding serialization, persistence, provider, or game crates would cross Phase 4/5 boundaries. [VERIFIED: .planning/ROADMAP.md]

## Architecture Patterns

### System Architecture Diagram

```text
accepted ProjectionSnapshot
          |
          v
section-quality gate ---- unavailable fact (explicit reason)
          |
          v
visibility-policy filter ---- excluded fact (recorded policy decision)
          |
          v
canonically ordered FactionVisibleSnapshot
          |
          +--> three CapabilityViews (defense | economy | territorial)
          |
          +--> ExecutiveDiplomacyInputs (ZYA <-> ARG, shared threat only)
          |
          v
StrategicPacket { scores, allowed primitives, admission inputs, replay fingerprint }
```

### Recommended Project Structure

```text
crates/strategic-state/
├── src/lib.rs                 # re-exports and crate composition only
├── src/faction.rs             # typed faction/profile identifiers
├── src/fact.rs                # availability-preserving visible facts
├── src/policy.rs              # versioned visibility and doctrine policies
├── src/packet.rs              # immutable packet and canonical ordering
├── src/derive.rs              # pure projection-to-packet compiler
└── tests/
    ├── visibility_contract.rs
    ├── packet_determinism.rs
    └── capability_contract.rs
```

### Pattern 1: Availability is data, never a default

Use a fact wrapper that distinguishes `Available`, `Unknown`, `Stale`, `Inaccessible`, and `Unsupported`; no caller receives an empty numeric or collection as a substitute for one of these states. This extends the accepted quality contract rather than replacing it. [VERIFIED: crates/observation-domain/src/section.rs:3-43]

Verbatim current source values: `"Fresh, KnownEmpty, Unknown, Partial, Stale, Unsupported"`. [VERIFIED: crates/observation-domain/src/section.rs:5-12]

### Pattern 2: Filter first, derive once, share by reference

The compiler must construct exactly one `FactionVisibleSnapshot` per faction and derive all three capability views from it. An institution gets its capability projection plus the same packet fact identities; it cannot access the raw cross-faction `ProjectionSnapshot`, add private claims, or alter visibility. [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md]

### Pattern 3: Canonical input is an explicit product contract

Sort all facts by typed subject and stable fact kind, all capability views by the fixed capability order, and all primitives by a typed primitive key before constructing the packet fingerprint. Phase 1 already uses ordered maps for frozen observations, but Phase 3 must not depend only on that implementation detail because derived vectors introduce new ordering. [VERIFIED: crates/observation-ingest/src/snapshot.rs:1-67] [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md]

### Canon profiles and doctrine

Lock these six **Live Galaxy internal identities**: `ZYA Defense & Military Strategy`, `ZYA Economy & Logistics`, `ZYA Territorial Development & Infrastructure`, `ARG Defense & Military Strategy`, `ARG Economy & Logistics`, and `ARG Territorial Development & Infrastructure`. They are descriptive capability labels, not claimed X4 ministries, councils, or official institutions. The source decision is explicit: the targeted installed-file follow-up found no official institution/ministry names; use the documented faction identities only to ground the `ZYA`/Patriarchy and `ARG`/Federation prefixes. [CITED: https://wiki.egosoft.com/X4%20Foundations%20Wiki/Manual%20and%20Guides/Objects%20in%20the%20Game%20Universe/Factions/Zyarth%20Patriarchy/?rev=47.1] [CITED: https://wiki.egosoft.com/X4%20Foundations%20Wiki/Manual%20and%20Guides/Objects%20in%20the%20Game%20Universe/Factions/Argon%20Federation/?rev=35.1] [VERIFIED: targeted installed-file follow-up]

Lock versioned fixture priorities as **INFERRED Live Galaxy product policy**, not canon statistics: ZYA starts defense first, territorial second, economy third because the documented context describes hardened-warrior leadership, self-reliance, unification, Argon/Xenon conflict, and border exposure; ARG starts economy first, defense second, territorial third because its documented context describes an industrial/cultural center, mutually beneficial relations, and recurring Xenon/war pressure. These are deterministic seed priorities for shared-scenario divergence and must remain revisable only by a versioned profile change and Phase 8 evaluation evidence. [INFERRED: documented community-wiki faction context + D-05 product policy] [CITED: https://wiki.egosoft.com/X4%20Foundations%20Wiki/Manual%20and%20Guides/Objects%20in%20the%20Game%20Universe/Factions/Zyarth%20Patriarchy/?rev=47.1] [CITED: https://wiki.egosoft.com/X4%20Foundations%20Wiki/Manual%20and%20Guides/Objects%20in%20the%20Game%20Universe/Factions/Argon%20Federation/?rev=35.1]

The three shared contracts are fixed and exhaustive: defense and military strategy; economy and logistics; territorial development and infrastructure. ZYA and ARG differ only through versioned profile priorities and evidence-backed doctrine inputs, never through different engine capabilities. [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md]

### Executive-only diplomatic posture inputs

Represent a typed bilateral relationship subject, visible supporting facts, shared-threat facts, and the four allowed Shadow dispositions: preserve relations, de-escalate, increase pressure, and seek limited coordination. Expose these only in the Executive slice of the packet; institutions may receive no diplomatic disposition authority. This is input data for Phase 5, not negotiation or an X4 command. [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Identity/order | Display-name keys or hash-map iteration | Existing `EntityId`, `ObservationVersion`, and ordered collections | Phase 1 already provides stable typed identity/version and ordered storage. [VERIFIED: crates/observation-domain/src/identity.rs:3-71] [VERIFIED: crates/observation-ingest/src/snapshot.rs:1-67] |
| Missing data | `Option`/empty defaults that erase cause | Availability-bearing visible fact | Required for OBS-03/OBS-04 and D-03. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md] |
| Institution variation | Three separate strategy engines | One capability contract plus `FactionProfile` data | Keeps D-04/D-05 finite and testable. [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md] |
| Diplomacy | A fourth agent/institution or free-text negotiation | Executive-only typed inputs | D-07/D-08 prohibit both. [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md] |

## Common Pitfalls

### Treating `KnownEmpty` as global ignorance or an empty value

`KnownEmpty` is a meaningful observed state, while `Unknown`, `Partial`, `Stale`, and `Unsupported` have different decision consequences. Preserve the quoted source values and their provenance through filtering; never score an unavailable fact as zero. [VERIFIED: crates/observation-domain/src/section.rs:5-12]

### Visibility filtering after scoring

If a shared projection is scored before per-faction filtering, both faction scores and later model inputs can leak changing foreign information. Filter first, then derive every score and primitive from the filtered immutable view. [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md]

### Canonical iteration without canonical tie breakers

`BTreeMap` helps only at the observation map boundary. Derived facts, priorities, and primitives need total typed sort keys and deterministic tie behavior; otherwise equal input can produce unequal replay packets. [VERIFIED: crates/observation-ingest/src/snapshot.rs:18-67] [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md]

### Premature model/persistence coupling

Do not include prompts, model requests, initiative lifecycle, caches, databases, or report intents in Phase 3. Phase 4 owns persistence/full mind state; Phase 5 owns provider deliberation and admission. [VERIFIED: .planning/ROADMAP.md]

## Code Examples

The following is an implementation skeleton, not a claim about existing symbols; every proposed identifier is deliberately `[ASSUMED]` until Phase 3 tests define its public contract.

```rust
// [ASSUMED] Pure, total compiler shape.
pub fn derive_packet(
    frozen: &ProjectionSnapshot,
    profile: &FactionProfile,
    visibility: &VisibilityPolicy,
) -> StrategicPacket {
    let visible = visibility.filter(frozen, profile.faction());
    StrategicPacket::from_visible(visible, profile)
}
```

## State of the Art

| Old Approach | Current Approach | Impact |
| --- | --- | --- |
| Phase 1 accepted projection | Faction-filtered immutable strategic packet | Adds no new source of truth; it derives bounded, replayable inputs. [VERIFIED: crates/observation-ingest/src/snapshot.rs:18-67] |
| One shared strategic world view | Recorded faction-specific visibility policy | Required information discipline before Phase 5 provider work. [VERIFIED: .planning/REQUIREMENTS.md] |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | The exact fact taxonomy and numeric score weights can be selected after OBS-04 coverage is finalized. | Architecture Patterns | Premature schema could omit real X4 facts. |
| A2 | The proposed Rust public identifiers are suitable. | Code Examples | Tests may need a different minimal API. |

## Open Questions (RESOLVED)

1. **What exact installed-X4 catalog/localization evidence names the ZYA and ARG institution identities and doctrine?**
   - Resolved: no official institution/ministry name is currently evidenced. The six locked descriptive labels are Live Galaxy internal identities; the X Community Wiki grounds the faction prefixes and product-policy priority rationale only. [CITED: https://wiki.egosoft.com/X4%20Foundations%20Wiki/Manual%20and%20Guides/Objects%20in%20the%20Game%20Universe/Factions/Zyarth%20Patriarchy/?rev=47.1] [CITED: https://wiki.egosoft.com/X4%20Foundations%20Wiki/Manual%20and%20Guides/Objects%20in%20the%20Game%20Universe/Factions/Argon%20Federation/?rev=35.1]
2. **Which supported observation scopes satisfy OBS-04 in a disposable X4 9.00 run?**
   - Resolved owner: Phase 1/7 runtime evidence. The harness is not yet runnable; explicit availability facts make this non-gating for the pure compiler and do not establish runtime semantics. [VERIFIED: .planning/phases/01-read-only-observation-spine/01-RUNTIME-GAP-RESEARCH.md]
3. **What numeric priorities make the two profiles meaningfully divergent?**
   - Resolved owner: lock the qualitative fixture order in this phase as explicit INFERRED product policy, then let Phase 8 evaluate and revise numeric weights rather than treating stereotypes as evidence. [INFERRED: documented community-wiki faction context + D-05 product policy] [VERIFIED: .planning/ROADMAP.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| Rust workspace | pure Phase 3 crate/tests | ✓ | project-pinned toolchain | — [VERIFIED: Cargo.toml] |
| Phase 1 accepted projection | compiler input | ✓ | workspace-local API | — [VERIFIED: crates/observation-ingest/src/lib.rs:1-67] |
| Runnable X4 observation harness | runtime OBS-04/OBS-05 confirmation | ✗ | — | Explicit unavailable facts; human X4 validation remains pending. [VERIFIED: .planning/phases/01-read-only-observation-spine/01-RUNTIME-GAP-RESEARCH.md] |

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Cargo test harness. [VERIFIED: .planning/phases/01-read-only-observation-spine/01-VALIDATION.md] |
| Quick run command | `cargo test -p strategic-state --test visibility_contract` [ASSUMED until crate exists] |
| Full suite command | `cargo test --workspace` [VERIFIED: .planning/phases/01-read-only-observation-spine/01-VALIDATION.md] |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| OBS-04 | Each fact family retains availability/provenance. | unit + table fixtures | `cargo test -p strategic-state --test fact_availability` | ❌ Wave 0 |
| OBS-05 | XEN pressure and KHK-if-observed survive filtering. | unit + property | `cargo test -p strategic-state --test threat_visibility` | ❌ Wave 0 |
| MIND-02 | Policy permits/denies facts deterministically. | property + negative authorization | `cargo test -p strategic-state --test visibility_contract` | ❌ Wave 0 |
| MIND-03 | Packet contains bounded facts/scores/primitives. | unit | `cargo test -p strategic-state --test packet_derivation` | ❌ Wave 0 |
| MIND-04 | Equivalent frozen input yields byte/equality-equivalent packet. | property | `cargo test -p strategic-state --test packet_determinism` | ❌ Wave 0 |
| INST-01/02 | Exactly three shared contracts; one shared visible snapshot. | unit + mutation | `cargo test -p strategic-state --test capability_contract` | ❌ Wave 0 |

### Wave 0 Gaps

- [ ] `crates/strategic-state` public contract and focused test targets.
- [ ] Deterministic fixture builder for accepted projections with all six quoted quality values.
- [ ] Versioned ZYA/ARG profile fixture using the six locked Live Galaxy internal labels and the documented-context policy rationale; assert it never claims an official X4 institution name.
- [ ] Property tests that permute input order and visibility membership.

### Mutation Strategy

Run `cargo-mutants` only after the pure compiler exists and a normal test baseline is measured. Target visibility predicates, availability transitions, ordering keys, score saturation, and capability-count guards; review every survivor rather than imposing an invented score. [VERIFIED: .agents/skills/live-galaxy-rust-tests/SKILL.md]

## Security Domain

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V4 Access Control | Yes | Allow/deny visibility policy before any score or packet. [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md] |
| V5 Input Validation | Yes | Only accepted frozen projection enters derivation; quality/availability remain explicit. [VERIFIED: crates/observation-ingest/tests/atomic_rejection.rs] |
| V6 Cryptography | No | Phase 3 has no credential or cryptographic operation. [VERIFIED: .planning/ROADMAP.md] |

| Threat | STRIDE | Mitigation |
| --- | --- | --- |
| Foreign fact leakage | Information disclosure | Filter before derivation; negative tests deny excluded fact IDs. [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md] |
| Missing fact converted to zero | Tampering | Availability-bearing fact states and invariant tests. [VERIFIED: crates/observation-domain/src/section.rs:5-43] |
| Nondeterministic replay | Tampering | Total ordering, pure compiler, permutation/property tests. [VERIFIED: .planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md] |

## Sources

### Primary (HIGH confidence)

- `AGENTS.md` — authority, safety, testing, evidence, and scope constraints.
- `.planning/phases/03-faction-scoped-strategic-state/03-CONTEXT.md` — D-01 through D-09 and phase boundary.
- `.planning/REQUIREMENTS.md` and `.planning/ROADMAP.md` — required behaviors and Phase 4/5 ownership.
- `crates/observation-domain/src/identity.rs`, `section.rs`, and `crates/observation-ingest/src/snapshot.rs` — actual Phase 1 public input contract.
- `.planning/phases/01-read-only-observation-spine/01-RUNTIME-GAP-RESEARCH.md` — Phase 1 evidence boundary.

### Secondary (MEDIUM confidence)

- `F:/Agent Projects/TALKER/src/talker/domain/facts.py` and `claims.py` were identified structurally as typed-domain precedent, but the sandbox could not establish Git freshness because both reference repositories report dubious ownership. They informed no required Live Galaxy runtime fact.

### Tertiary (LOW confidence)

- Exact official X4 institution/ministry names: the targeted installed-file follow-up established none; they are intentionally not fabricated.

### Documented community-wiki context (MEDIUM confidence)

- [Zyarth Patriarchy, revision 47.1](https://wiki.egosoft.com/X4%20Foundations%20Wiki/Manual%20and%20Guides/Objects%20in%20the%20Game%20Universe/Factions/Zyarth%20Patriarchy/?rev=47.1) — `ZYA`/Patriarchy identity, self-reliance, unification, Argon/Xenon conflict, and territorial-border context; not installed 9.00 runtime proof.
- [Argon Federation, revision 35.1](https://wiki.egosoft.com/X4%20Foundations%20Wiki/Manual%20and%20Guides/Objects%20in%20the%20Game%20Universe/Factions/Argon%20Federation/?rev=35.1) — `ARG`/Federation identity, industrial/cultural center, mutually beneficial relations, and Xenon/war context; not installed 9.00 runtime proof.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — no new package or runtime choice is needed.
- Architecture: HIGH — constrained by locked Phase 3 decisions and actual Phase 1 types.
- Canon profiles: MEDIUM — documented community-wiki context grounds faction prefixes and product-policy priorities; the labels are explicitly Live Galaxy internal, not official X4 institutions.
- Runtime observation semantics: MEDIUM — local contracts exist; X4 9.00 evidence remains pending.

**Research date:** 2026-08-29  
**Valid until:** Revisit immediately after Phase 1 runtime harness or a readable installed-X4 canon source is available.
