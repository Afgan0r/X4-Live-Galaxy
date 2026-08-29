---
status: resolved
trigger: "Phase 05.1 disposable X4 verification reaches facts_unsupported after native station enumeration."
created: 2026-08-30
updated: 2026-08-30
---

# Phase 05.1 faction token diagnosis

## Symptoms

- **Expected:** The bounded Argon station observation is emitted after all
  candidate metadata and capacity facts validate.
- **Actual:** The original faction token was rejected before enumeration. The
  corrected token reaches enumeration, which then rejects the complete scope at
  the obsolete 16-station bound.
- **Known:** The previous extension-relative import and module-local `ffi.C`
  binding defects are resolved; native enumeration now advances beyond them.

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
- timestamp: 2026-08-30
  checked: Disposable attempt `obs-x4-component-discovery-051-04`
  found: The native station function rejects the configured ownership token
    before candidate enumeration. The owner scope was not empty because of the
    fresh save; the token has the wrong native representation.
- timestamp: 2026-08-30
  checked: Disposable attempt `obs-x4-component-discovery-051-05`
  found: The corrected native token reaches the count/fill seam, but the
    complete owner scope exceeds the fixed 16-station bound. The bridge remains
    health-only with no observation or completion marker.

## Resolution

The native faction-token diagnosis is resolved. The successor is the separately
planned 64-station bounded-scope revision in `05.1-04-PLAN.md`; it retains the
pre-allocation overflow boundary and requires a new disposable X4 smoke test.
