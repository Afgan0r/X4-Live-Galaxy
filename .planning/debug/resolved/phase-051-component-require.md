---
status: resolved
trigger: "X4 disposable Phase 05.1 probe does not load Live Galaxy runtime."
created: 2026-08-30
updated: 2026-08-30
---

# Phase 05.1 component module resolution

## Symptoms

- **Expected:** X4 loads the Live Galaxy runtime, registers its event handler,
  and sends bounded telemetry to the ready bridge.
- **Actual:** The bridge records only `listener_ready`; no runtime frames arrive.
- **Error:** X4 reports that `live_galaxy_component_discovery` cannot be found
  and searches the flattened `extensions/` package path.
- **Timeline:** First observed after deploying the Phase 05.1 build for its
  disposable runtime verification.
- **Reproduction:** Start X4 with `live_galaxy` and `sn_mod_support_apis`
  enabled, then load the fresh disposable campaign.

## Current Focus

- hypothesis: After the import fix, native enumeration is unavailable because
  the component adapter dereferences `_G.C`; the installed working precedent
  binds `local C = require("ffi").C` instead.
- test: Require a module-local `ffi.C` binding and reject `globals.C` native
  calls in the static binding contract.
- expecting: The contract fails on the current adapter, then passes after the
  native calls use the established local FFI binding.
- next_action: Route the separate post-enumeration `facts_unsupported` UAT gap
    only under a newly authorized bounded diagnostic revision.

## Evidence

- timestamp: 2026-08-30
  checked: X4 `debug.log` and the correlated bridge evidence log
  found: X4 searched `extensions/live_galaxy_component_discovery.lua`; the
    bridge retained only `listener_ready`.
- timestamp: 2026-08-30
  checked: New component package-resolution contract
  found: The focused component suite failed on the existing bare import before
    the production change.
- timestamp: 2026-08-30
  checked: Focused component, runtime, package guard, and install guard suites
  found: All focused checks pass after switching to the extension-relative
    module path; native X4 reload remains pending.
- timestamp: 2026-08-30
  checked: Corrected X4 runtime attempt `obs-x4-component-discovery-051-02`
  found: X4 loaded the module and bridge accepted hello and heartbeat, then
    discovery emitted `enumeration_unavailable` and a health-only frame with no
    completion marker.
- timestamp: 2026-08-30
  checked: Installed X4 9.00 call-shape precedent
  found: The same count/fill calls use a module-local `ffi.C`; the Live Galaxy
    adapter instead dereferences `globals.C`.
- timestamp: 2026-08-30
  checked: Corrected X4 runtime attempt `obs-x4-component-discovery-051-03`
  found: The module loaded, native enumeration advanced past the prior
    `enumeration_unavailable` result, and the closed adapter returned
    `facts_unsupported`; the bridge accepted health only and no completion
    marker.

## Eliminated

- hypothesis: The extension or support dependency is disabled.
  evidence: X4 profile `content.xml` marks both `live_galaxy` and
    `ws_2042901274` enabled, and X4 attempted to execute the runtime script.

## Resolution

- root_cause: Two sequential loader/native binding gaps are confirmed. The bare
    Lua module name prevented initial loading; after that fix, the adapter used
    `_G.C` instead of the module-local `ffi.C` binding required by the installed
    X4 UI-Lua call shape.
- fix: Use the established extension-relative module path and module-local
    `ffi.C`; enforce both bindings in the static contract.
- verification: Focused Lua, package, install, and Rust contracts pass. X4 loads
    the corrected module and advances beyond native enumeration. The separate
    post-enumeration `facts_unsupported` result is retained as UAT gap
    `G-05.1-1` rather than widened inside this debug session.
- files_changed: `extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua`,
    `extensions/live_galaxy/tests/component_discovery_binding_contract.ps1`
