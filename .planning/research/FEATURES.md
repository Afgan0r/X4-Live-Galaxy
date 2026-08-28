# Feature Landscape

**Domain:** Internal X4: Foundations observation-only strategic-director prototype  
**Milestone:** 0.1 — Shadow Director  
**Researched:** 2026-08-28  
**Overall confidence:** HIGH for product scope; MEDIUM for X4 runtime details

## Scope and Evidence Boundary

Milestone 0.1 is an internal prototype, not a playable or public-ready release. Its product question is whether persistent faction minds can observe a live X4 galaxy, form bounded and explainable strategy, and remain reliable during unattended AFK/SETA operation. X4 remains authoritative; the model proposes typed intent only, while the Rust bridge owns normalization, validation, persistence, recovery, orchestration, caching, evaluation, and diagnostics.

The scope below is derived from the active requirements and decisions in [`PROJECT.md`](../PROJECT.md). Claims about adapter surfaces are informed by the read-only X4 Live MCP precedent at `F:\Agent Projects\X4\extensions`, `F:\Agent Projects\X4\tools`, and `F:\Agent Projects\X4\tests`; that checkout was dirty and could not refresh `FETCH_HEAD`, so those details remain medium-confidence until re-verified against a clean revision. No saves were read.

## Table Stakes

These are the minimum capabilities needed to answer the 0.1 product question. Missing one makes the prototype’s evidence uninterpretable rather than merely less polished.

| Feature | Why Expected | Complexity | Acceptance implications |
| --- | --- | --- | --- |
| Authoritative world-state observation | Minds need current, normalized facts about ZYA, ARG, XEN pressure, and observed KHK activity without mutating X4. | High | Capture stable identities, timestamps, freshness, quality, and source ownership; prove no game-state writes. |
| Full ZYA and ARG Faction Minds | The experiment compares two persistent faction-specific executives under shared hostile pressure. | High | Both factions must maintain independent doctrine, motives, goals, short/long-term plans, and bounded historical context. |
| Primitive faction institutions | The revised 0.1 scope must show that distinct internal roles can propose and own strategy without turning the prototype into political simulation. | High | Give each faction fixed-priority institutions, at most one active Shadow initiative per institution, Executive arbitration, bounded exceptional dialogue, and persistent lifecycle evidence. |
| XEN-pressure and KHK recognition | XEN is the primary pressure in scope; KHK must be recognized when observed, without requiring a complete hostile-faction simulation. | Medium | Separate observed facts from inferred threat assessments; unknown/unseen KHK state must not become an empty or negative fact. |
| Typed strategic decisions and plans | The kernel must evaluate strategy deterministically and keep model output outside the trust boundary. | High | Validate schema, semantics, safety, information boundaries, budgets, and current-state freshness; reject invalid output without partial effects. |
| Explanations without hidden reasoning | Operators need useful rationale and players need concise reports, but private chain-of-thought and raw prompts must never leak. | Medium | Emit bounded public rationale fields derived from accepted decisions; diagnostics must redact secrets and hidden reasoning. |
| Persistent state and restart recovery | Long-lived minds are the core hypothesis, and an AFK run is invalid if restart or transient failure loses identity or duplicates work. | High | Persist compact authoritative runtime state with X4-owned save integration boundaries; make accepted work idempotent and replayable. |
| Deterministic caching and model/cost budgets | The prototype must measure model value under bounded spend and avoid duplicate calls during retries or unchanged observations. | High | Exact cache keys, token/time/call limits, context compaction, retry bounds, and usage accounting are observable in diagnostics. |
| Reliability degradation paths | Provider, bridge, malformed payload, stale state, and transport failures are normal test conditions. | High | Queue and payload bounds, explicit degraded states, safe retry, correlation IDs, and recovery evidence; no simulation stall. |
| Structured external diagnostics | Detailed evidence belongs outside the player-facing surface and must support evaluation and debugging. | High | Correlate observation → decision → validation → report → recovery; export snapshots, traces, health, costs, and rejection reasons without private data or machine paths. |
| Concise Mail/Logbook sanity output | 0.1 needs a low-cost in-game confirmation channel, not a custom dossier UI. | Medium | Reports are short, attributable to ZYA/ARG, rate-bounded, and safe when the bridge is unavailable; detailed evidence remains external. |
| AFK/SETA soak behavior | Unattended operation is the primary acceptance path, exposing scheduling, throughput, memory, and reconnect faults. | High | Run normal-speed and SETA scenarios with exact game version/mod list/time; show bounded health, no vanilla stall, no duplicate reports, and recoverable interruption. |
| Automated strategic-quality evaluation | “It ran” is insufficient; the prototype must assess coherence, persistence, information discipline, and cost/reliability. | High | Use recorded typed fixtures/fakes rather than live models in normal tests; compare decisions against deterministic invariants and evaluator evidence. |

## Differentiators

These are the features that make the prototype useful as a research instrument rather than a generic LLM wrapper.

| Feature | Value Proposition | Complexity | Notes |
| --- | --- | --- | --- |
| Shared pressure, distinct faction doctrines | Demonstrates that ZYA and ARG produce meaningfully different strategies from common facts, rather than two copies of one prompt. | High | Evaluate divergence alongside consistency; avoid rewarding arbitrary personality prose. |
| Auditable initiative ownership | Makes internal disagreement and responsibility measurable instead of hiding every choice inside one faction prompt. | High | Persist proposal, owner, disposition, preemption reason, validation result, and outcome without introducing majority rule or political sabotage. |
| Information-bounded faction viewpoints | Preserves strategic credibility by limiting each mind to authoritative facts available to that faction. | High | Test hidden-information leakage and distinguish observed, derived, and unknown values. |
| Replayable decision packets | Makes model behavior auditable and enables deterministic comparison across providers, prompts, and cache hits. | High | Preserve normalized snapshot, policy inputs, model metadata, validation result, and accepted report; exclude secrets and hidden reasoning. |
| Cost/reliability/model benchmark matrix | Turns provider selection (likely Ollama initially, but not assumed) into measured evidence. | High | Compare latency, token usage, cache effectiveness, failure/retry behavior, and strategic quality under fixed fixtures. |
| Parallel XEN/KHK research track | De-risks future hostile-faction architecture without delaying the ZYA/ARG Shadow Director slice. | Medium | Keep research and observation interfaces separate from future autonomous XEN/KHK control. |
| Operator-readable health and evidence summaries | Lets a developer distinguish stale observation, model degradation, validation rejection, and X4 transport failure during a long run. | Medium | Provide bounded summaries and correlation IDs; do not turn diagnostics into a custom player interface. |

## Anti-Features

Explicitly do not build these in milestone 0.1.

| Anti-Feature | Why Avoid | What to Do Instead |
| --- | --- | --- |
| Fleet, economy, diplomacy, or institution-initiative mutation | It would conflate strategic-quality and integration risks before observation, validation, persistence, and recovery are proven. | Model institutions and typed Shadow initiatives, but reserve command application for a later milestone. |
| Open-ended council debate or internal politics | It would add unbounded model calls and political simulation before the initiative lifecycle is proven. | Use direct agreement by default and at most two Executive–institution dialogue cycles on material disagreement. |
| Direct model access to X4 or arbitrary commands | Breaks the authority and trust-boundary invariants and makes replay/recovery unsafe. | Route all proposals through deterministic typed validation and a future application layer. |
| Player missions or Player Influence | Product scope explicitly follows the autonomous faction core. | Keep interfaces open for a later milestone; do not add player-facing control now. |
| Custom dossier, chronicle, or institution UI | High UI cost would obscure the research question and duplicate diagnostics. | Use concise Mail/Logbook sanity reports plus external structured diagnostics. |
| Full vanilla/DLC or mod-added faction roster | 0.1 is deliberately ZYA/ARG-focused; broad coverage belongs to later private gameplay validation. | Discover and normalize only the supported 0.1 factions and shared pressure. |
| Full autonomous XEN/KHK minds | Parallel research is required, but it must not delay or expand the Shadow Director slice. | Recognize observed KHK and record XEN pressure; research future architecture separately. |
| Faction Enhancer compatibility | First public alpha is explicitly incompatible, and 0.1 is earlier than that gate. | Record incompatibility; test only the scoped observation environment. |
| Reading or modifying player saves | Prohibited and unnecessary for proving the observation loop. | Use X4-owned persistence boundaries and disposable Creative Custom campaigns for probes. |
| Unbounded chat, prose, or hidden chain-of-thought output | Creates privacy, cost, and evaluation problems. | Typed decisions, concise explanations, bounded reports, and redacted diagnostics. |

## Explicitly Deferred Later-Milestone Capabilities

The following are product capabilities, not 0.1 acceptance criteria:

- Deterministic application of fleet, economy, institution, and eventually diplomacy effects after Shadow Director evidence is sufficient.
- Private institutional knowledge, changing influence, refusal, sabotage, and power struggles.
- First-alpha diplomacy beyond bilateral war declaration and termination.
- Broader vanilla-plus-DLC faction coverage and compatibility gates for KUDA AI Tweaks, More AI Economy Ships, and Add More Sectors.
- Gameplay-ready autonomous faction core and later public-alpha packaging/release gates.
- Missions, Player Influence, deeper diplomacy, historical simulations, and a custom in-game dossier/chronicle/institution interface.
- Specialized XEN/KHK architecture beyond observation and parallel research.

## Feature Dependencies

```text
X4 observation adapter
  → normalized snapshots and freshness/quality metadata
  → faction information boundaries
  → ZYA/ARG mind state and primitive institution priorities
  → typed institution proposals + Executive initiative disposition
  → typed decision + explanation validation
  → deterministic cache/budget accounting
  → concise Mail/Logbook report

normalized snapshots + persistent state
  → replayable decision packets
  → restart recovery and idempotency

structured correlation IDs + health states
  → external diagnostics
  → model/cost/reliability evaluation

bounded scheduling + failure isolation
  → AFK/SETA soak acceptance

observed XEN/KHK signals
  → parallel hostile-faction research (does not gate the ZYA/ARG slice)
```

## MVP Recommendation

Prioritize:

1. A read-only X4 observation path producing normalized, timestamped, quality-marked snapshots for ZYA, ARG, XEN pressure, and observed KHK.
2. Persistent ZYA/ARG minds and primitive institutions with typed goals and initiatives, bounded Executive arbitration, deterministic validation, exact caching, and restart-safe recovery.
3. External correlated diagnostics and an AFK/SETA harness, with concise Mail/Logbook sanity reports as the only in-game output.

Defer mutation, player-facing control, custom interfaces, broad faction coverage, and autonomous XEN/KHK behavior until the prototype has repeatable strategic-quality, cost, and reliability evidence.

## Complexity and Acceptance Summary

The high-complexity work is not the report text; it is the end-to-end evidence chain: authoritative observation, typed boundaries, institution initiative state, Executive arbitration, persistence/recovery, bounded model orchestration, and unattended SETA stability. Mail/Logbook output is medium complexity because it depends on a safe existing surface but must remain rate-bounded and non-authoritative. XEN/KHK parallel research is medium complexity and should be isolated so it cannot become a hidden dependency of ZYA/ARG acceptance.

Acceptance must report three statuses separately: **verified locally**, **pending game smoke test**, and **observed in X4**. A passing unit or fake-adapter test cannot establish real X4 behavior; a successful short run cannot establish AFK/SETA durability; and a coherent report cannot establish model reliability or cost bounds.

## Confidence Assessment

| Area | Confidence | Basis |
| --- | --- | --- |
| Table stakes | HIGH | Directly specified by active milestone requirements and constraints in `PROJECT.md`. |
| Differentiators | HIGH | Product-specific synthesis of Faction Minds, deterministic kernel, diagnostics, and evaluation goals in `PROJECT.md`; implementation details remain to be planned. |
| Anti-features and deferrals | HIGH | Explicit out-of-scope and authority boundaries in `PROJECT.md`. |
| X4 surface assumptions | MEDIUM | Informed by the read-only X4 Live MCP precedent; its checkout is dirty and remote refresh was blocked, so runtime claims need clean-revision or in-game confirmation. |

## Sources

- [`PROJECT.md`](../PROJECT.md) — active milestone 0.1 scope, requirements, out-of-scope capabilities, authority, trust-boundary, reliability, observability, and testing constraints.
- MemPalace `drawer_wing_x4_live_galaxy_decisions_a376ced07a211aa8271352e6` — confirmed 0.1 primitive-institution scope extension.
- MemPalace `drawer_wing_dialogue_sessions_dd1780a21bd9ded3e9c4e997` — confirmed small-milestone workflow and public-alpha boundary.
- MemPalace `drawer_wing_bannerlord_operations_7675741e0bb9147f4d2ed3f1` — 103-item reference catalogue; candidates are not automatically product scope.
- [`AGENTS.md`](../../AGENTS.md) — Live Galaxy evidence boundaries, compatibility policy, X4 research routing, and release maturity rules.
- `F:\Agent Projects\X4\extensions`, `F:\Agent Projects\X4\tools`, and `F:\Agent Projects\X4\tests` — read-only X4 Live MCP integration precedent; dirty working tree and unavailable `FETCH_HEAD` make exact revision freshness unresolved.
- [Egosoft X4: Foundations](https://www.egosoft.com/games/x4/info_en.php) — official product context and mod-supported single-player simulation context; version-specific runtime behavior is not inferred from this page.
