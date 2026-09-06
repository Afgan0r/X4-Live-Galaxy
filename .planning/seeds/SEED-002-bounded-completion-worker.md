---
id: SEED-002
status: dormant
planted: 2026-09-06
planted_during: Phase 05.4 Plan 01 execution
trigger_when: when Carrier B transport lifetime architecture is reconsidered
scope: unknown
---

# SEED-002: Consider a bounded carrier-owned completion worker

## Why This Matters

Workerless overlapped I/O cannot safely return from Lua-state finalization while
pending kernel operations still reference DLL-owned storage. A bounded worker
may preserve nonblocking callbacks, cancellation completion, storage lifetime,
and eventual DLL unload, but adopting it changes the approved architecture and
requires owner approval.

## When to Surface

**Trigger:** when Carrier B transport lifetime architecture is reconsidered

Surface during the current Phase 05.4 checkpoint or any later milestone that
revisits native transport shutdown and reload behavior.

## Scope Estimate

**Unknown** — resolve only after the owner chooses the transport architecture.

## Breadcrumbs

- `.planning/phases/05.4-owned-carrier-b-and-production-observation-path/05.4-01-PLAN.md`
- `.planning/phases/05.4-owned-carrier-b-and-production-observation-path/05.4-CONTEXT.md`
- `.planning/phases/05.4-owned-carrier-b-and-production-observation-path/05.4-RESEARCH.md`

## Notes

Captured at the blocking Task 3 checkpoint. This seed records an unapproved
option; it does not modify the accepted phase scope or architecture.
