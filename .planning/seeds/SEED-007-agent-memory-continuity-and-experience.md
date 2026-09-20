---
id: SEED-007
status: dormant
planted: 2026-09-20
planted_during: Phase 05.5 — Heavy Faction Ship Conformance
trigger_when: when relevant
scope: unknown
---

# SEED-007: Evaluate agent memory for continuity and accumulated experience

## Why This Matters

The owner wants in-game agents to preserve goals, decisions, and unfinished
work, and to let past successes and failures influence future strategy.
Relevant historical episodes should be available without placing the entire
history into every model request. This concerns game-agent memory, not the
development team's personal MemPalace.

## When to Surface

**Trigger:** when relevant

Surface when revisiting persistent minds, historical retrieval, or strategic
evaluation. Compare the idea with the existing typed-plus-narrative capsule
design before proposing a replacement or an additional storage system.

## Scope Estimate

**Unknown** — retrieval methods, storage, and lesson validation remain open.

## Breadcrumbs

- `.planning/PROJECT.md` — persistence, memory capsules, and the boundary that
  keeps MemPalace out of the required public runtime core.
- `.planning/ROADMAP.md` — Phase 04 persistent minds and Phase 08 continuity
  and causal evaluation.
- `.planning/seeds/SEED-004-replayability-architecture-evaluation.md`
- `.planning/seeds/SEED-006-strategic-summaries-and-detail-requests.md`
- [MemPalace](https://github.com/MemPalace/mempalace) and
  [its storage structure](https://mempalaceofficial.com/concepts/the-palace)
  — memory storage and scoped semantic retrieval.
- [AI Influence feature guide](https://reggie-ai-influence.github.io/AI-Influence/)
  — version 6.0.2 describes recent messages plus retrieved historical chunks,
  local BM25, and semantic embeddings through Player2.

## Notes

Owner-confirmed dialogue outcome on 2026-09-20:

- Both continuity and accumulated experience matter.
- Embeddings represent text for similarity search; RAG retrieves selected
  information into model context. MemPalace is a memory system that can support
  that process, not an alternative to embeddings as a general mechanism.
- No storage product, embedding provider, or MemPalace runtime adoption or
  replacement was selected.
- Current resources and active obligations need explicit authoritative input;
  historical retrieval cannot guarantee that a necessary record will be found.
- The open evaluation problem is distinguishing a useful lesson from an
  incorrect causal explanation of a past success or failure.

Status: future idea and desired behavior, not a revised phase contract.
External sources clarify the concept; they are not evidence of implementation
in Live Galaxy or permission to copy third-party code.
