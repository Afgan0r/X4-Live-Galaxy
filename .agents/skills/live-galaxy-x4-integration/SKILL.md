---
name: live-galaxy-x4-integration
description: Safe research, design, implementation, and verification for Live Galaxy X4 integration.
---

# Live Galaxy X4 Integration

Use this skill for X4 XML, Mission Director, Lua, game-data, installed-mod,
compatibility, or in-game verification work. Use the global `lua` skill for Lua
implementation and review.

## Evidence

- Classify claims as documented, observed, inferred, or unknown.
- X4 runtime evidence outranks static assumptions when freshness and test setup
  are known.
- A missing value is not an empty value unless the source contract proves it.
- Trace which system owns a value, event, command, or patch before proposing an
  integration seam.

## Integration Context

Use the Docs MCP lookup and handoff contract in
`.agent-instructions/x4/AGENTS.md` for unknown X4 API, loader, or lifecycle
claims. Preserve source provenance and enough context to assess the seam:

- loader, registration, extension-relative module resolution, and package path;
- Lua runtime and native binding acquisition;
- exact calls, arguments, return shapes, and native-versus-canonical identity;
- lifecycle, game-thread ownership, cadence, and reconnect behavior;
- failure, partial-result, and completeness semantics;
- observed or source-supported volume and performance behavior.

A fake derived from the proposed design proves only its local contract, not an
independent production precedent. Keep source-resolvable uncertainty in the
shared research workflow rather than turning it into an in-game probe.

## Product Regression Coverage

Protect affected loader, native-binding, identity, bounded-work, and
partial-result behavior with focused executable regressions. Invalid package
fixtures must fail for the intended reason; a permissive test-only loader or a
green happy path cannot establish production package correctness.

## Safety

- Treat the game installation and installed mods as read-only.
- Never read or modify save files.
- Use a disposable Creative Custom campaign or an explicit test copy only under
  a written verification plan.
- Start with the smallest probe that can answer one hypothesis. Do not use a
  broad runtime hook when a narrow event or query is sufficient.
- Bound work executed on the game thread and degrade safely when the bridge is
  unavailable.

## Action Boundary

- Export normalized observations with stable identities, timestamps, freshness,
  and quality metadata.
- Commands must carry a unique identity, validation context, budget, and explicit
  outcome.
- Reconnect and retry must not duplicate accepted game actions.
- Unsupported or stale operations must be rejected explicitly, not coerced into
  a nearby command.
- Detailed development diagnostics belong in a separate debug interface, not in
  the public player-facing surface.

## Compatibility and Provenance

- Map file patches, events, hooks, ownership, load order, and behavioral overlap
  before claiming compatibility.
- The first public alpha is incompatible with the Faction Enhancer suite.
- KUDA AI Tweaks and Add More Sectors require explicit compatibility tests.
- More AI Economy Ships compatibility is not supported or a release gate.
  Similar economy-fleet functionality is only a possible later addition to
  Live Galaxy if needed, not committed scope.
- Learn mechanisms from external code, but implement original algorithms. Quote
  minimally and record provenance only for material implementation influence.
- Never copy code whose license or redistribution terms are unresolved.

## Verification

Use extension-relative module paths for every production Lua `require`,
including dependencies loaded by other modules. Compile shipped Lua with the
actual interpreter and load real product modules through normal `require`.
Exercise delayed imports through the native, lifecycle, and transport paths
that use them. Fake only the external X4 environment and verify its calls;
follow `live-galaxy-x4-tests` for test design and evidence requirements.

Provision local Busted once with `tools/provision-lua.ps1 -WithBusted
-CompilerPath <installed clang.exe>`. Normal tests use the installed tools.
Run `extensions/live_galaxy/tests/run_contracts.ps1 -Suite telemetry` (or
`component_discovery`, `x4_discovery`, `scheduler`, `syntax`, `xml`) for focused
checks; `-Filter` forwards a Lua pattern to Busted. Use `-Suite all` once for
the final product regression after review convergence. XML checks cover the
manifest, UI registration, entrypoint existence, and persistence schema.

A local module load proves only the paths executed against the supplied fake
environment. It does not establish all imports or X4 loader/native
compatibility. Do not recreate a source lexer, import-graph analyzer, or
vocabulary guard as a substitute for executable behavior.

Keep static/package, pure Lua, and fake-adapter results distinct from behavior
observed in X4. Residual runtime uncertainty requires the applicable written
disposable verification plan; the user performs all in-game actions under the
shared contract. Record exact X4 version, mod set, scenario and workload,
elapsed real and game time, SETA state when relevant, and independent readback.
Scope conclusions and fallback choices to that evidence and its limitations.
