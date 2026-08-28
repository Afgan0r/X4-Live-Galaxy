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
- KUDA AI Tweaks, More AI Economy Ships, and Add More Sectors require explicit
  compatibility tests.
- Learn mechanisms from external code, but implement original algorithms. Quote
  minimally and record provenance only for material implementation influence.
- Never copy code whose license or redistribution terms are unresolved.

## Verification

Validate static schemas and contracts first, then run focused adapter tests, and
only then use a disposable in-game probe. Capture exact game version, mod list,
scenario, elapsed game time, logs, and expected versus observed results.
