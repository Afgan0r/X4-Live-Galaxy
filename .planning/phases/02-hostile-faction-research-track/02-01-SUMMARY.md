---
phase: 02-hostile-faction-research-track
plan: "01"
subsystem: research-validation
tags: [x4, xen, khaak, evidence, powershell, pester]
requires:
  - phase: 01-read-only-observation-spine
    provides: Future read-only X4 observation contract and disposable runtime probes.
provides:
  - Versioned XEN/KHK static claim register with source-scoped conclusions.
  - Read-only PowerShell validation and deterministic malformed-source fixtures.
  - Non-gating ownership of the remaining hostile runtime unknowns.
affects: [phase-03-information-boundary, phase-07-x4-validation, phase-08-inventory]
actuals:
  tokens: 6693
  tasks: 3
  commits: 4
tech-stack:
  added: [PowerShell, Pester]
  patterns: [source-scoped claim registry, structured non-gating deferral]
key-files:
  created:
    - .planning/phases/02-hostile-faction-research-track/02-XEN-KHK-EVIDENCE.md
    - tools/verify_xen_khk_evidence.ps1
    - tools/verify_xen_khk_evidence.Tests.ps1
  modified:
    - .planning/phases/02-hostile-faction-research-track/02-VALIDATION.md
key-decisions:
  - "The validator consumes only the named JSON register; explanatory prose cannot elevate a claim."
  - "XEN/KHK runtime unknowns remain explicit, owned, non-gating deferrals."
patterns-established:
  - "Every material research claim resolves to an allowlisted source and permitted conclusion."
requirements-completed: [RES-01, RES-02, RES-03]
coverage:
  - id: D1
    description: Versioned XEN/KHK register covers all required faction-area pairs.
    requirement: RES-01
    verification:
      - kind: unit
        ref: tools/verify_xen_khk_evidence.ps1 -Stage full
        status: pass
    human_judgment: false
  - id: D2
    description: Source integrity and scope-boundary rejection cases are deterministic.
    requirement: RES-03
    verification:
      - kind: unit
        ref: tools/verify_xen_khk_evidence.Tests.ps1
        status: pass
    human_judgment: false
  - id: D3
    description: Disposable X4 runtime evidence remains intentionally deferred and non-gating.
    requirement: RES-02
    verification:
      - kind: other
        ref: tools/verify_xen_khk_evidence.ps1 -Stage full
        status: pass
    human_judgment: false
duration: 5 min
completed: 2026-08-29
status: complete
---

# Phase 2 Plan 01: Hostile Evidence Register Summary

**Versioned XEN/KHK static evidence with source-scoped claims, deterministic validation, and explicit non-gating runtime deferrals.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-08-29T05:08:19+07:00
- **Completed:** 2026-08-29T05:12:50+07:00
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Added a single versioned register for all required XEN and KHK research areas.
- Enforced source kind, path, boundary, and permitted-conclusion provenance.
- Recorded hostile autonomy, architecture, writes, control, and critical-path status as prohibited structured scope flags.
- Mapped every Phase 2 requirement and locked decision to repeatable checks and runtime deferrals.

## Task Commits

1. **Task 1 RED: failing verifier fixtures** — `a7e1854` (test)
2. **Task 1 GREEN: structured parser and skeleton** — `95c379d` (feat)
3. **Task 2: full XEN/KHK static coverage** — `14c77f6` (docs)
4. **Task 3: validation matrix and boundary audit** — `6b49386` (docs)

## Files Created/Modified

- `02-XEN-KHK-EVIDENCE.md` — machine-readable source registry and claim register.
- `verify_xen_khk_evidence.ps1` — read-only parser and full invariant checker.
- `verify_xen_khk_evidence.Tests.ps1` — valid and malformed registry fixtures.
- `02-VALIDATION.md` — requirement/decision matrix, evidence levels, and sampling cadence.

## Decisions Made

- The validator accepts only structured JSON, so narrative text cannot create an unapproved hostile implementation contract.
- Static installed evidence remains distinct from runtime observations; all unresolved runtime claims have a future owner and evidence need.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Used the installed Pester 3.4-compatible invocation.**

- **Found during:** Task 1 verification.
- **Issue:** The planned `Invoke-Pester -CI` switch is unavailable in Pester 3.4.
- **Fix:** Ran the same fixtures with `Invoke-Pester -Path`; all pass/fail semantics remain deterministic.
- **Files modified:** `02-VALIDATION.md`
- **Verification:** Five fixtures pass, including four malformed-source failures.
- **Committed in:** `6b49386`

**Total deviations:** 1 auto-fixed (1 blocking compatibility issue).
**Impact on plan:** No scope or runtime behavior changed.

## Issues Encountered

Pester 3.4 does not expose `-CI`; the compatible command is documented in the validation matrix.

## Known Stubs

None. Runtime unknowns are intentional structured deferrals and do not prevent the research-track goal.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 8 can inventory the evidence only. Phase 1, Phase 3, and Phase 7 retain ownership of the listed disposable-runtime probes; Phase 2 does not gate their work.

## Self-Check: PASSED

- Required evidence, validator, fixture, and validation-matrix files exist.
- All four task commits are present in Git history.

---

*Phase: 02-hostile-faction-research-track*
*Completed: 2026-08-29*
