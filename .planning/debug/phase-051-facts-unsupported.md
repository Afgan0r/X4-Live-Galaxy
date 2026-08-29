---
status: ready_for_runtime
trigger: "Phase 05.1 disposable X4 verification reaches facts_unsupported after native station enumeration."
created: 2026-08-30
updated: 2026-08-30
---

# Phase 05.1 facts unsupported classification

## Symptoms

- **Expected:** The bounded Argon station observation is emitted after all
  candidate metadata and capacity facts validate.
- **Actual:** The runtime reaches discovery and returns `facts_unsupported`.
  The bridge receives health-only frames and no completion marker.
- **Known:** The previous extension-relative import and module-local `ffi.C`
  binding defects are resolved; native enumeration now advances beyond them.

## Current Focus

- hypothesis: One of the post-enumeration owner, sector, or capacity contract
  checks rejects a native value.
- test: Add an opt-in, closed diagnostic class that identifies only the failed
  validation category while preserving the public `facts_unsupported` result.
- constraints: No raw IDs, native text, values, payloads, retries, or wider
  native seams may be added.
- expecting: A disposable X4 run identifies one allowlisted category and keeps
  the bridge protocol health-only on failure.

## Evidence

- timestamp: 2026-08-30
  checked: Disposable attempt `obs-x4-component-discovery-051-03`
  found: X4 loads Live Galaxy, native enumeration completes, and the adapter
    returns `facts_unsupported` without an observation or completion marker.
- timestamp: 2026-08-30
  checked: Focused component, X4 runtime, component package, and disposable
    install-guard contracts
  found: All checks pass with an allowlisted diagnostic class that preserves
    the external `facts_unsupported` result and is emitted only with trace
    enabled.

## Next Action

Deploy the bounded revision, enable trace for one disposable attempt, reload
the already-authorized disposable test save, and record only the emitted closed
diagnostic class.
