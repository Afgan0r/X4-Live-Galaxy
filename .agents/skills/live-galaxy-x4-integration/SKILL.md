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

## Integration Admission

Do not design or implement an X4 seam from an isolated call signature. Before
planning it, produce a phase-owned integration dossier backed by both the
primary source and a verified working production precedent. Cover:

- loader, registration, extension-relative module resolution, and package path;
- Lua runtime and native binding acquisition;
- exact calls, arguments, return shapes, and native-versus-canonical identity;
- lifecycle, game-thread ownership, cadence, and reconnect behavior;
- failure, partial-result, and completeness semantics;
- observed or source-supported volume and performance behavior.

Inspect adjacent production integration before test helpers. Expand research to
installed mods and available public mod sources until no load-bearing question
that sources can answer remains unknown. A fake derived from the proposed design
is not an independent precedent. Do not send a source-resolvable uncertainty to
an in-game probe.

Source-collection agents may report evidence, conflicts, and unknowns, but may
not declare the dossier sufficient. An independent planning or verification
gate owns PASS or BLOCK.

## Known-Failure Gate

Before any X4 seam is planned, implemented, packaged, or tested in game, read
the project-owned Known X4 Failure Registry and produce a coverage matrix. For
every registered failure class, name the independent dossier evidence and
executable check that exclude it, or justify why it is not applicable. Absence
of the registry or a matrix row is blocking.

The gate must cover the demonstrated classes of loader mismatch, native-binding
assumptions, native-versus-canonical identity mismatch, invented unmeasured
bounds, partial results represented as complete, permissive local harnesses, and
isolated call-shape research without its integration context. Test the gate by
reintroducing each registered defect in a negative fixture and proving that the
gate fails. A green happy path alone is insufficient.

There are no automatic exceptions for small changes. Only an explicit owner
override may bypass the gate, and the decision ledger must record its scope,
reason, and remaining risk. When a new X4 runtime defect is discovered, add its
generalized class and a reproducing regression fixture before treating the fix
as complete.

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

Use extension-relative module paths for every production Lua `require`,
including dependencies loaded by other modules. Before deployment, statically
verify the complete production import graph rather than only the registered
entry point.

Run packaged conformance with production module-resolution behavior; a more
permissive test-only package path cannot prove that X4 can load the extension.
Validate the dossier and Known X4 Failure coverage first, then static schemas,
pure Lua behavior, focused adapter contracts, and packaged conformance. Only
residual native uncertainty may proceed to a disposable in-game experiment.

For residual uncertainty, prepare multiple source-derived candidates in one
developer-only build when they can coexist safely. Execute them serially and in
isolation so one failure does not suppress the remaining candidates. Separate
each candidate's verdict into execution, contract, and effect; a successful call
that returns a valid but unexpected answer is not a passing candidate. Log every
stage with candidate identity, source, expected and actual result, completeness,
elapsed real and game time, SETA state when relevant, and the exact classified
failure point. Retain the full structured log as machine-local test evidence and
record its run identity and digest in the sanitized phase decision ledger.

Select production and fallback behavior only from the recorded evidence. Keep
losing candidates in the dossier and negative fixtures when they establish a
reusable incompatibility; remove disposable experiment code that provides no
useful fallback. Capture exact game version, mod list, scenario, and independent
readback used to judge the expected effect.
