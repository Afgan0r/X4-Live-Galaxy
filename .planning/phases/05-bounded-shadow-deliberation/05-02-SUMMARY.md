---
phase: 05-bounded-shadow-deliberation
plan: 02
subsystem: strategic-admission
tags: [rust, cache-identity, replay, bounds, validation]
requires:
  - phase: 05-01
    provides: Pure ordered shadow-admission tracer and one-CAS pending commit invariant.
provides:
  - Versioned length-framed D-12 exact cache identity with canonical collection ordering.
  - Explicit bounded request profile and cache-hit revalidation through normal admission.
affects: [05-03, phase-06-diagnostics, replay]
actuals:
  tokens: 4018
  tasks: 3
  commits: 3
tech-stack:
  added: []
  patterns: [length-framed exact identities, fail-closed resource bounds, cache bytes as untrusted input]
key-files:
  created: [crates/mind-domain/src/cache_identity.rs, crates/mind-domain/src/request_bounds.rs, crates/mind-domain/tests/shadow_deliberation_evals/exact_cache.rs]
  modified: [crates/mind-domain/src/admission.rs, crates/mind-domain/src/lib.rs, crates/mind-domain/tests/shadow_deliberation_evals.rs, shadow-deliberation-evals/v1/manifest.json]
key-decisions:
  - Cache entries supply candidate bytes only and always re-enter the same pure admission chain.
  - Request-resource fields are explicit non-zero checked values; no production default is admitted.
requirements-completed: [MIND-06, MODEL-02, MODEL-04, MODEL-06]
coverage:
  - id: D1
    description: Exact D-12 cache identities differentiate a one-component mutation and canonicalize unordered components.
    requirement: MODEL-04
    verification:
      - kind: integration
        ref: crates/mind-domain/tests/shadow_deliberation_evals/exact_cache.rs#sd_006_exact_key_changes_for_each_authority_component
        status: pass
    human_judgment: false
  - id: D2
    description: Cache-hit bytes use current admission validation and resource limits reject missing bounds.
    requirement: MIND-06
    verification:
      - kind: integration
        ref: crates/mind-domain/tests/shadow_deliberation_evals/exact_cache.rs#sd_006_cached_bytes_revalidate_through_current_state_admission
        status: pass
    human_judgment: false
duration: 25m
completed: 2026-08-29
status: complete
---

# Phase 05 Plan 02: Exact Cache Identity and Bounds Summary

**Canonical D-12 cache identity and explicit request bounds now fail closed, while cached bytes must pass the unchanged admission path against current state.**

## Performance

- **Duration:** 25m
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Added a versioned, length-framed exact identity for faction, snapshot, policy, prompt, schema, provider, model, generation, vocabulary, compaction, and bounds.
- Proved component mutation, collection ordering, absent bounds, and stale cache hits through SD-005/SD-006 corpus coverage.
- Kept cache revalidation pure, redacted, and free of persistence or X4 mutation.

## Task Commits

1. **Task 1: RED exact-key component, ordering, and bound cases** — `72659a0` (`test`)
2. **Task 2: GREEN canonical identity and complete revalidation** — `51bcd4e` (`feat`)
3. **Task 3: REFACTOR identity diagnostics and replay assertions** — `de1c429` (`refactor`)

## Decisions Made

- Cache bytes are not authority: a cache hit executes the existing ordered admission function with the current frozen request.
- All request-resource limits use checked typed fields; zero, absent, and excessive values fail before admission.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Source-size] Split exact cache bounds and corpus tests into cohesive modules.**
- **Found during:** Task 3
- **Issue:** `cache_identity.rs` and the corpus test file exceeded the repository's 200-line source limit.
- **Fix:** Extracted `request_bounds.rs` and the exact-cache corpus module without changing the public behavior.
- **Files modified:** `cache_identity.rs`, `request_bounds.rs`, and the shadow deliberation eval modules.
- **Verification:** Focused corpus, full workspace tests, strict Clippy, and source-size lint pass.
- **Committed in:** `de1c429`

## Issues Encountered

The required remote freshness fetch could not update `.git/FETCH_HEAD` because the sandbox denied that write. Local committed repository evidence was used; no remote-state claim is made.

## Next Phase Readiness

05-03 can consume the exact identity and bounded revalidation seam. No cache storage backend or X4 mutation path was introduced.

## Self-Check: PASSED

- Verified all seven implementation and corpus files exist.
- Verified `72659a0`, `51bcd4e`, and `de1c429` exist in Git history.
- Verified focused corpus, workspace tests, formatting, strict Clippy, source-size lint, and diff check pass.
