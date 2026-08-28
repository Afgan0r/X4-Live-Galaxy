# Phase 2: Hostile-Faction Research Track - Research

**Researched:** 2026-08-29  
**Domain:** X4 9.00 XEN/KHK observation evidence  
**Confidence:** MEDIUM — static installed-game evidence is strong for configured mechanisms; no disposable runtime observation was performed.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Research scope

- **D-01:** Cover XEN and KHK state, events, identity, visibility, scheduling,
  economy or spawning ownership, and known control limitations in X4 9.00.
- **D-02:** XEN is the primary observed hostile pressure for 0.1; KHK is
  recognized only when authoritative observations contain it.
- **D-03:** Do not assign ordinary government institutions, motives, or
  diplomacy to XEN or KHK by analogy.

### Evidence discipline

- **D-04:** Label every claim documented, observed, inferred, or unknown.
- **D-05:** Use vanilla files and disposable runtime observations as primary
  evidence. Installed mods and third-party code are read-only precedents.
- **D-06:** Record only materially influential provenance; do not copy a raw
  foreign corpus into the repository.

### Independence

- **D-07:** Unresolved hostile-faction questions cannot delay Phases 1 or 3–7
  and cannot silently expand milestone 0.1.
- **D-08:** Phase 8 inventories the research result without treating it as an
  autonomous hostile-mind implementation.

### the agent's Discretion

The researcher owns evidence collection order, artifact organization, and the
exact disposable scenarios, provided the source hierarchy and claim labels are
preserved.

### Deferred Ideas (OUT OF SCOPE)

- Autonomous XEN/KHK minds, replacement behavior, and hostile write primitives
  are later-milestone work.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| RES-01 | Versioned XEN/KHK state, events, identity, visibility, scheduling, and control-limit evidence. | Evidence inventory, runtime-observation gap, and minimum disposable probe below. |
| RES-02 | Research remains independent from the ZYA/ARG critical path. | Explicit non-gating and no-architecture boundary. |
| RES-03 | Claims are classified and materially influential provenance is recorded. | Every finding has an evidence label and source. |
</phase_requirements>

## Summary

The installed X4 build identifies itself as **9.00**. **[OBSERVED: installed `version.dat`]** The installed `08.cat` catalog contains current `libraries/factions.xml`, `libraries/jobs.xml`, `md/khaak_activity.xml`, and `md/crisis_xenon_khaak_combo.xml`, so hostile behavior is represented in vanilla catalog data rather than being a Live Galaxy invention. **[OBSERVED: installed X4 9.00 `08.cat`]**

XEN has configured civilian economy and military job definitions: the inspected jobs include mineral mining, energy transport, and patrol roles with faction, quota, location, owner, and `buildatshipyard` fields. **[OBSERVED: installed X4 9.00 `08.cat` → `libraries/jobs.xml`]** KHK is driven by a dedicated Mission Director activity script that keeps per-sector activity, evaluates every minute, creates KHK-owned hives/outposts, and assigns related defensive or harassment quotas. **[OBSERVED: installed X4 9.00 `08.cat` → `md/khaak_activity.xml`]**

**Primary recommendation:** retain XEN as a normalized pressure source and KHK as an observed-when-present source in 0.1; preserve raw authoritative identities, ownership, visibility, time, and quality, but make no hostile-mind or write-primitive decision from this research.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
| --- | --- | --- | --- |
| Hostile entity, sector, and event capture | X4 / game runtime | Rust bridge | X4 is authoritative; the bridge only normalizes observations. **[DOCUMENTED: `AGENTS.md`; `PROJECT.md`]** |
| Faction-visible hostile pressure | Rust bridge | X4 / game runtime | Visibility is a recorded policy over authoritative observations, not a game-side inference. **[DOCUMENTED: `REQUIREMENTS.md` MIND-02; `PROJECT.md`]** |
| KHK spawn/activity mechanics | X4 Mission Director | — | The installed script owns activity tables, cooldowns, and station creation. **[OBSERVED: installed X4 9.00 `md/khaak_activity.xml`]** |
| Future hostile autonomy | Unknown | — | It is deferred and no architecture is selected. **[DOCUMENTED: `02-CONTEXT.md` D-03, Deferred Ideas]** |

## Evidence Inventory

| Area | XEN | KHK | Planning consequence |
| --- | --- | --- | --- |
| State and identity | **[OBSERVED]** XEN job definitions declare `faction="xenon"`, owner `xenon`, jobs, quotas, locations, and build environment. | **[OBSERVED]** KHK activity creates stations owned by `faction.khaak`; the Finder precedent queries by true owner and named hive/nest macros. | Capture authoritative entity identity, owner/faction, location, macro/class, observation time, and source/version; never key by display name. |
| Events | **[UNKNOWN]** No event-to-telemetry contract was verified in this phase. | **[OBSERVED]** Activity state can create hive/outpost stations; the script signals station initialization after creation. | Phase 1/7 must verify the actual exportable event boundary before relying on it. |
| Visibility | **[UNKNOWN]** No XEN visibility API or player/faction discovery semantics were runtime-verified. | **[OBSERVED]** The installed Finder mod deliberately marks found KHK sector/cluster/station known and scanned, demonstrating those are distinct operations; this is a third-party read-only precedent, not vanilla behavior. | Do not infer visibility from faction ownership or location. Carry visibility and quality explicitly. |
| Scheduling | **[OBSERVED]** Job entries encode quotas and patrol/routine orders. | **[OBSERVED]** `Evaluate` is `checkinterval="1min"`; resource checks use a randomized 5–60 minute refresh window; hives/outposts apply game-time cooldowns. | Treat schedule/cadence as observed state if exported; do not emulate vanilla scheduling in the bridge. |
| Economy / spawning ownership | **[OBSERVED]** Inspected mining and energy jobs use `buildatshipyard="true"`; configured fleets are tied to vanilla job/faction logic. | **[OBSERVED]** `Khaak_Activity` owns activity accumulation and creates KHK hive/outpost stations and ship quotas. | X4 owns all spawning/economy effects. Milestone 0.1 remains observation-only. |
| Control limits | **[DOCUMENTED]** No 0.1 command may mutate fleets, economy, diplomacy, institutions, or X4 state. | **[DOCUMENTED]** Same read-only limit applies; autonomous KHK behavior is deferred. | Reject hostile write primitives; do not invent a control channel. |

## Findings by Hostile Faction

### XEN

1. **[OBSERVED: installed X4 9.00 `08.cat` → `libraries/jobs.xml`]** XEN uses jobs that express a mining routine and an energy trade routine, with `faction="xenon"`, `owner exact="xenon"`, quotas, location constraints, and `buildatshipyard="true"`. This establishes that XEN economy/fleet replenishment is X4-owned job/economy behavior, not a model-owned subsystem.
2. **[OBSERVED: installed X4 9.00 `08.cat` → `libraries/jobs.xml`]** XEN military jobs include patrol roles with faction-logic tags, quotas, location predicates, ship selection, and subordinate job references. This is configured scheduling/force composition evidence, not proof of a runtime combat event contract.
3. **[INFERRED: from the installed XEN job definitions]** Future hostile telemetry should distinguish job/faction/owner/location and observable asset state; a single aggregate "XEN threat" number would discard material cause and freshness data.
4. **[UNKNOWN]** The canonical runtime identifiers, event sequence, discovery/visibility rules, and externally observable scheduling transitions for individual XEN assets require a disposable 9.00 probe.

### KHK

1. **[OBSERVED: installed X4 9.00 `08.cat` → `md/khaak_activity.xml`]** `Khaak_Activity` maintains per-sector mining and KHK-activity tables, watches sectors, and evaluates on a one-minute interval.
2. **[OBSERVED: installed X4 9.00 `08.cat` → `md/khaak_activity.xml`]** The script evaluates ore, silicon, and nividium resource/yield information, activity thresholds, cooldowns, and gate-distance eligibility before creating KHK-owned hive or outpost stations.
3. **[OBSERVED: installed X4 9.00 `08.cat` → `md/khaak_activity.xml`]** The script creates KHK hives/outposts and records/updates defensive and harassment ship quotas; station creation is followed by an X4 station-initialization signal.
4. **[OBSERVED: installed extension `z_ram_khaakfinder` v101]** A read-only third-party precedent finds existing KHK stations by true owner plus known hive/nest macros and treats discovery, scan state, fog-of-war, notification, and Logbook output as separate actions. This validates a useful inspection hypothesis but does not establish the vanilla bridge interface.
5. **[UNKNOWN]** The safe, authoritative way for Live Galaxy to observe KHK activity tables, spawn decisions, or quota changes has not been established; no installed-file conclusion substitutes for a runtime observation contract.

## Architecture Patterns

### Evidence Flow

```text
X4 9.00 authoritative runtime
  → normalized hostile observations (identity, owner, location, time, quality)
  → recorded faction visibility filter
  → ZYA/ARG shared XEN-pressure fact + observed-KHK fact
  → Shadow-only strategic input and diagnostics

X4 jobs / Mission Director scheduling
  └→ never reimplemented or controlled by the bridge in milestone 0.1
```

### Required Observation Shape

Use a source-tagged observation record, not a hostile-mind model:

```text
hostile observation = { entity identity, faction/true owner, location,
  observed-at, source/version, visibility state, freshness/quality }
```

The exact field schema is intentionally **[UNKNOWN]** until Phase 1 establishes the X4 bridge contract; this is a planning shape, not a selected type or protocol.

### Anti-Patterns to Avoid

- **Treating configured jobs as live facts:** static job definitions do not prove an entity currently exists. **[INFERRED: static-versus-runtime evidence boundary]**
- **Treating player discovery as faction visibility:** the Finder precedent performs separate known, scanned, and fog actions. **[OBSERVED: installed `z_ram_khaakfinder`]**
- **Recreating KHK activity or spawn logic:** vanilla Mission Director owns it, and 0.1 cannot mutate X4. **[OBSERVED: installed `md/khaak_activity.xml`; DOCUMENTED: `PROJECT.md`]**
- **Projecting government institutions or diplomacy onto XEN/KHK:** prohibited by D-03. **[DOCUMENTED: `02-CONTEXT.md` D-03]**

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| XEN/KHK spawning or economy | A bridge-side hostile simulator | Authoritative X4 observation only | Vanilla jobs and Mission Director own configured behavior; 0.1 forbids mutation. **[OBSERVED: installed X4 9.00 catalog; DOCUMENTED: `PROJECT.md`]** |
| KHK activity calculation | A replica of mining activity, cooldown, and quota logic | A bounded observation/probe of authoritative outcomes | Static constants and runtime state may change with game/extension versions. **[OBSERVED: installed `md/khaak_activity.xml`]** |
| Visibility assumptions | Name/sector-based inference | Explicit observed visibility metadata | Discovery and scan operations are distinct in the inspected precedent. **[OBSERVED: installed `z_ram_khaakfinder`]** |

## Common Pitfalls

### Static Configuration Is Not Runtime Proof

**What goes wrong:** planners turn jobs, catalog fields, or MD scripts into a promised event/API contract.  
**How to avoid:** label installed-file results as observed static evidence and use a disposable scenario to verify runtime identity, timing, visibility, and readback. **[DOCUMENTED: `live-galaxy-x4-integration/SKILL.md`; `live-galaxy-x4-tests/SKILL.md`]**

### KHK Presence Is Not a Fixed-Map Fact

**What goes wrong:** a fixed sector or hive count becomes a schema invariant.  
**How to avoid:** preserve the observed source/time/quality and discover entities at runtime; the installed script itself uses activity, cooldown, candidate sectors, and game-time evaluation. **[OBSERVED: installed `md/khaak_activity.xml`]**

### Installed Mods Become Compatibility Guarantees

**What goes wrong:** a useful mod's contents are mistaken for a supported Live Galaxy contract.  
**How to avoid:** use the Finder only as a read-only visibility precedent, and test compatibility in a disposable scenario before making any guarantee. **[DOCUMENTED: `AGENTS.md`; OBSERVED: installed extension manifests]**

## Minimum Verification Spike

Run only after Phase 1's read-only adapter contract exists, in a disposable Creative Custom campaign:

1. Capture a bounded snapshot containing a XEN asset and any available KHK asset/station; record X4 version, enabled extensions, game time, and SETA state. **[DOCUMENTED: `live-galaxy-x4-tests/SKILL.md`]**
2. Independently read back identity, faction/true owner, sector/cluster/zone, visibility/discovery state, and source timestamp; classify absent KHK as `unknown/absent observation`, never a claim that KHK cannot exist. **[INFERRED: D-04/D-05 evidence discipline]**
3. Observe one KHK activity-related transition only if safely attributable; compare exported evidence to authoritative game readback, without injecting commands or changing save data. **[DOCUMENTED: `AGENTS.md`; `live-galaxy-x4-integration/SKILL.md`]**

## Open Questions (RESOLVED)

These runtime unknowns are deliberately unresolved by this read-only phase.
Their disposition closes Phase 2 scope only; it does not answer the underlying
fact or block the ZYA/ARG critical path.

| Question | Current status | Future owner | Evidence needed | Non-gating disposition |
| --- | --- | --- | --- | --- |
| Which exact runtime event/API surface exports XEN/KHK identity and lifecycle? | RESOLVED: deferred — **[UNKNOWN]** | Phase 1 and Phase 7 X4 validation | Attributable disposable X4 9.00 observation and independent readback | RESOLVED: deferred; Phases 3–7 proceed with bounded observed facts only. |
| Can KHK activity, cooldown, hive/outpost quota, or spawn cause be observed without inspecting MD internals? | RESOLVED: deferred — **[UNKNOWN]** | Later hostile-design research | Disposable X4 9.00 probe with authoritative readback | RESOLVED: deferred; future hostile-design input only and no 0.1 requirement depends on it. |
| What visibility semantics are available to each faction, rather than the player? | RESOLVED: deferred — **[UNKNOWN]** | Phase 3 information-boundary work | Authoritative observation contract plus recorded visibility-policy tests | RESOLVED: deferred; Phase 3 records a policy over authoritative observations and this phase does not define it. |
| How do installed extensions modify hostile jobs, MD logic, or map scope in a live campaign? | RESOLVED: deferred — **[UNKNOWN]** | Later compatibility research | Versioned extension inventory and disposable compatibility scenario | RESOLVED: deferred; compatibility research is deferred and no guarantee is created. |
| What hostile architecture or primitives should exist after 0.1? | RESOLVED: deferred — **[UNKNOWN]** | A later milestone product discussion | Explicit owner decision backed by then-current evidence | RESOLVED: deferred; D-03/D-07/D-08 defer the decision and no 0.1 requirement depends on it. |

## Validation Architecture

This research phase has no code or package installation. Validation is evidence review plus the later disposable in-game spike; no test framework is introduced. **[DOCUMENTED: `02-CONTEXT.md`; `config.json`]**

| Requirement | Validation | Status |
| --- | --- | --- |
| RES-01 | Inspect this artifact's XEN/KHK coverage and provenance; later run the minimum disposable spike. | Static evidence complete; runtime evidence pending. |
| RES-02 | Confirm no Phase 3–7 dependency or hostile implementation task is created. | Complete by scope. |
| RES-03 | Review every material claim for evidence label and source. | Complete by artifact review. |

## Security Domain

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | No | No network/auth surface is added. **[DOCUMENTED: phase boundary]** |
| V3 Session Management | No | No session surface is added. **[DOCUMENTED: phase boundary]** |
| V4 Access Control | Yes | Do not infer faction-visible data; preserve explicit visibility policy and source quality. **[DOCUMENTED: `PROJECT.md`; OBSERVED: Finder precedent]** |
| V5 Input Validation | Yes | Treat X4 observations as untrusted until normalized and bounded. **[DOCUMENTED: `AGENTS.md`]** |
| V6 Cryptography | No | No cryptographic operation is introduced. **[DOCUMENTED: phase boundary]** |

## Project Constraints (from AGENTS.md)

- X4 owns authoritative state and final effects; models and the bridge must not bypass deterministic validation. **[DOCUMENTED: `AGENTS.md`]**
- Installed vanilla files and extensions are read-only evidence; never inspect or modify saves. **[DOCUMENTED: `AGENTS.md`]**
- Separate documented, observed, inferred, and unknown claims; record only material provenance and do not copy third-party code. **[DOCUMENTED: `AGENTS.md`]**
- XEN/KHK specialized architecture is outside the first autonomous-director slice unless a later milestone admits it. **[DOCUMENTED: `AGENTS.md`]**
- Preserve the observation-only 0.1 boundary; do not claim prototype/public readiness. **[DOCUMENTED: `AGENTS.md`; `PROJECT.md`]**

## Sources

### Primary

- Installed X4 9.00 `version.dat`, `08.cat`, `08.dat` — catalog version and the inspected faction/jobs/KHK activity files. **[OBSERVED]**
- Installed extension `z_ram_khaakfinder` v101 — visibility/discovery precedent only. **[OBSERVED]**
- `AGENTS.md`, `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, and `02-CONTEXT.md` — scope and non-gating contract. **[DOCUMENTED]**

### Secondary

- Installed `kuda_ai_tweaks`, `more_ai_economy_ships`, and `zadd_sectors` manifests — evidence that enabled extensions can alter combat behavior, job populations, and map scope; no compatibility claim is made. **[OBSERVED]**

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | A future bridge can expose hostile identity/visibility/quality in a single normalized record. **[INFERRED]** | Required Observation Shape | The Phase 1 adapter may need a different decomposition. |
| A2 | A disposable campaign can provide the required hostile runtime evidence without save inspection. **[INFERRED]** | Minimum Verification Spike | The exact probe may require a different approved test setup. |

## Metadata

**Confidence breakdown:**

- Installed static behavior: HIGH — directly inspected X4 9.00 catalog entries.
- Runtime behavior and visibility semantics: LOW — no disposable runtime run.
- Scope and safety boundary: HIGH — locked project/phase documents.

**Research date:** 2026-08-29  
**Valid until:** A game update, extension change, or the first attributable disposable runtime observation.
