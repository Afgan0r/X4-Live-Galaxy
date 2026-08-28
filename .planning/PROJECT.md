# Live Galaxy

## What This Is

Live Galaxy is a public X4: Foundations mod that gives factions persistent,
LLM-backed strategic minds. Those minds interpret the galaxy, maintain goals
and plans, and eventually influence fleets, economies, institutions, and
diplomacy through deterministic, bounded game actions.

The project is separate from the owner's personal X4 Live MCP campaign tooling.
That repository is a technical precedent, not a runtime dependency.

## Core Value

Factions must pursue coherent, distinct, long-lived strategies while X4 remains
authoritative and every proposed effect stays observable, recoverable, and
bounded by deterministic validation.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Observe the supported X4 world state without mutating the game.
- [ ] Maintain persistent short- and long-term plans for full ZYA and ARG
  Faction Minds while treating XEN as the primary hostile pressure.
- [ ] Exercise primitive ZYA and ARG institutions that propose and own typed
  Shadow initiatives under bounded Executive arbitration.
- [ ] Produce typed strategic decisions, explanations, and concise in-game
  reports without exposing hidden model reasoning.
- [ ] Validate model routing, strategic quality, exact caching, token budgets,
  context compaction, persistence, and restart recovery.
- [ ] Provide structured external diagnostics, captured world-state snapshots,
  correlated decision traces, and automated evaluation evidence.
- [ ] Survive unattended AFK/SETA runs without corrupting X4 state or stalling
  the vanilla simulation.
- [ ] Research XEN and KHK behavior in parallel without delaying the ZYA/ARG
  Shadow Director slice.

### Out of Scope

- Game-state mutation in milestone 0.1 — fleet, economy, diplomacy, and
  institution initiatives remain Shadow state until the observation and
  decision evidence is sufficient.
- Player missions and the Player Influence system — deferred until after the
  autonomous faction core is proven and published.
- A custom in-game dossier, chronicle, or institution interface — a separate
  post-alpha milestone; milestone 0.1 uses low-cost Mail or Logbook surfaces.
- Full vanilla and DLC faction coverage — required for the later private
  gameplay-ready build, not the initial ZYA/ARG observation prototype.
- Mod-added factions — not implied by the vanilla-plus-DLC public-alpha target.
- Faction Enhancer compatibility — the first public alpha is explicitly
  incompatible with that suite.
- XRSGE compatibility — uncommitted and subject to a later evidence-based
  research spike.
- Reading or modifying player save files — prohibited; only X4-owned save
  integration and disposable test campaigns may be designed later.
- A public licence or public-ready stability claim before the provenance and
  release gates are complete.

## Context

X4 9.00 is the current research and integration target. The game remains the
authority for live world state and for the final application of any future
action. The Rust bridge owns normalized state, validation, persistence,
recovery, model orchestration, caching, evaluation inputs, and structured
diagnostics. Lua and Mission Director adapters expose bounded X4 integration
surfaces; model output remains outside the trust boundary.

The product core is **Faction Minds**: faction-specific executives and advisers
with motives, doctrine, goals, plans, explanations, and persistent historical
context. The deterministic kernel supplies current facts and allowed strategic
primitives, enforces information boundaries, and rejects invalid or stale
proposals. Model providers are pluggable; Ollama is a likely early backend, but
provider and model choices are benchmarked rather than assumed.

Milestone **0.1 — Shadow Director** is an internal observation-only prototype.
It runs full ZYA and ARG minds against shared XEN pressure, recognizes KHK when
observed, emits concise strategic reports through existing X4 surfaces, and
keeps detailed evidence in external diagnostics. Its primary acceptance path is
an unattended AFK/SETA test stand, not normal play.

Each 0.1 faction also has primitive institutions. They consume the same
authoritative faction-visible snapshot, apply fixed faction-conditioned
priorities, and propose typed Shadow initiatives. An institution owns at most
one active initiative. The Executive Brain remains the final allocator and may
originate, approve, revise, preempt, or reject an initiative, while an explicit
bounded dialogue records material disagreement. Ownership, lifecycle, and
outcome are retained for causal evaluation; no initiative mutates X4 in 0.1.

The initial GSD roadmap describes only milestone 0.1. Later `0.x` milestones are
internal prototypes that progressively validate fleet, economy, and institution
autonomy. A private gameplay-ready build must cover the supported vanilla and
DLC faction roster and pass compatibility gates for KUDA AI Tweaks, More AI
Economy Ships, and Add More Sectors. Version `1.0.0` is the first public alpha,
not a stable release. It ships the autonomous faction core before missions;
the custom interface and Player Influence follow as separate milestones.

The first public alpha intentionally remains primitive. Its factions react to
real economic, military, and territorial changes using a finite validated
vocabulary of strategic primitives. Institutions execute bounded initiatives,
and diplomacy is limited to declaring and ending bilateral wars. Rich treaties,
private institutional knowledge, political resistance, misinformation, and
mutable institutional power remain post-alpha. Publication requires a short
normal-play campaign after automated and AFK/SETA gates pass.

Development advances through small, visible, verified milestones. The project
implements and tests one milestone in game before selecting and discussing the
next; the Bannerlord-derived catalogue remains a reference source rather than a
precommitted product backlog.

Durable product decisions and historical planning evidence live in personal
MemPalace wing `wing_x4_live_galaxy`. The repository owns active GSD state,
source, tests, release artifacts, and concise conclusions needed by the current
milestone. Installed X4 data, installed mods, TALKER, and the X4 Live MCP
repository are read-only evidence sources.

## Constraints

- **Authority**: X4 owns authoritative game state and applies final effects;
  models never mutate the game directly.
- **Trust boundary**: Every model proposal must pass schema, semantic, safety,
  budget, information, and current-state validation.
- **Reliability**: Queues, payloads, time, memory, retries, provider calls, and
  game-side work must be explicitly bounded.
- **Recovery**: Accepted work is idempotent; restart and retry cannot duplicate
  an action or partially mutate game state.
- **Persistence**: Compact authoritative runtime state belongs with the X4 save;
  external caches, diagnostics, and prose are non-authoritative.
- **Observability**: Correlation IDs connect observation, decision, validation,
  dispatch, acknowledgement, and outcome evidence.
- **Information discipline**: A faction may reason only from authoritative
  truth and information available to that faction.
- **Compatibility**: Discover runtime sectors, assets, capacity, and ownership;
  do not assume a fixed vanilla map or job count.
- **Performance**: Preserve X4 simulation stability during long SETA runs;
  optimization beyond safety bounds requires a measured bottleneck.
- **Privacy**: Never expose credentials, private prompts, hidden reasoning, or
  machine-local paths in public diagnostics.
- **Testing**: Prefer executable layered tests over source-text assertions;
  distinguish local verification, pending game smoke, and observed-in-X4
  evidence.
- **Release maturity**: Every `0.x` release is an internal prototype;
  `1.0.0` is the first public alpha after private gameplay validation.
- **Platform**: Windows is the first supported runtime platform.
- **Licence**: No project licence is selected; provenance and licensing review
  are release gates.

## Key Decisions

| Decision | Rationale | Outcome |
| --- | --- | --- |
| Build a public Live Galaxy mod independently of X4 Live MCP | Personal campaign tooling must not become a public runtime dependency | — Pending |
| Use Faction Minds under a deterministic kernel | Preserve faction agency without giving models arbitrary mutation authority | — Pending |
| Make milestone 0.1 observation-only | Isolate strategy, ingestion, persistence, cost, and reliability before command integration | — Pending |
| Run full ZYA and ARG minds with XEN pressure | Exercise contrasting factions and a shared existential threat without whole-galaxy scope | — Pending |
| Include primitive institutions in milestone 0.1 | Test multi-role strategic disagreement and initiative ownership before any game mutation | — Pending |
| Bound each institution to one active Shadow initiative | Keep concurrency, preemption, causality, and evaluation understandable in the first prototype | — Pending |
| Keep Executive–institution dialogue exceptional and capped | Preserve useful disagreement without open-ended model loops, latency, or cost | — Pending |
| Research XEN and KHK in parallel | Discover hostile-faction telemetry and architecture gaps while change is still cheap | — Pending |
| Use AFK/SETA as the milestone 0.1 acceptance environment | Long-session diagnostics matter more than player-facing utility before mutations exist | — Pending |
| Keep detailed evidence external and in-game reports concise | Use low-cost X4 integration while retaining full developer observability | — Pending |
| Prefer a Rust bridge, subject to phase evidence | Typed state, recovery, and bounded orchestration need a strong deterministic host | — Pending |
| Keep compact runtime state save-authoritative | The campaign must remain coherent when external caches or services disappear | — Pending |
| Combine strategic ticks with event triggers and cooldowns | Avoid both polling-only latency and event-storm instability | — Pending |
| Use exact, versioned cache keys only | Approximate reuse can erase faction identity or replay stale strategy | — Pending |
| Compact conversations by model-relative token budgets | Different models need different safety margins and context limits | — Pending |
| Validate hybrid typed-plus-narrative memory capsules | Preserve role-specific meaning without making summaries authoritative | — Pending |
| Gate strategic quality behind an independent reliability floor | Attractive plans cannot compensate for invalid schemas or information leakage | — Pending |
| Derive evaluation thresholds from measured baselines | Avoid invented quality, cost, latency, and mutation-score targets | — Pending |
| Detail only milestone 0.1 in the initial roadmap | Prevent later prototypes and alpha scope from diluting the first executable slice | — Pending |
| Advance through one verified milestone at a time | Prevent the large idea catalogue from becoming premature scope and design debt | — Pending |
| Treat all `0.x` versions as internal prototypes | Keep release claims aligned with actual gameplay evidence | — Pending |
| Publish `1.0.0` as the first public alpha | Public release follows private closed-loop gameplay validation, not prototype completion | — Pending |
| Keep first-alpha diplomacy primitive | Declare-war and end-war behavior is enough to prove dynamic political change before richer treaty systems | — Pending |
| Ship the autonomous core before missions | The faction simulation is the product value and must work without player tasks | — Pending |
| Keep MemPalace out of the public runtime core | Project memory aids development but is not a required mod dependency | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition:**

1. Move invalidated requirements to Out of Scope with a reason.
2. Move shipped and confirmed requirements to Validated with a phase reference.
3. Add newly discovered requirements to Active.
4. Record significant decisions and their outcomes.
5. Update What This Is if the implemented product has drifted.

**After each milestone:**

1. Review every section against implementation and verification evidence.
2. Reconfirm that the Core Value is still the right priority.
3. Audit Out of Scope and its reasons.
4. Update Context with the current product and release state.

---

*Last updated: 2026-08-28 after project initialization*
