---
name: live-galaxy-x4-tests
description: Test and mutation strategy for Live Galaxy Lua, Mission Director XML, X4 adapters, and in-game behavior.
---

# Live Galaxy X4 Tests

Use this skill when designing, implementing, reviewing, or running tests for
Lua, Mission Director XML, X4 adapters, game commands, or runtime integration.

Read `.agents/skills/live-galaxy-x4-integration/SKILL.md` first. Use the global
`lua` skill for Lua implementation and review.

## Design for Testability

- Separate pure Lua policy, serialization, normalization, scheduling, batching,
  budgeting, and diff logic from X4 globals, `ffi.C`, UI callbacks, Mission
  Director events, and transport I/O.
- Inject a narrow X4 adapter into testable modules. Do not let pure modules read
  game globals directly.
- Keep the game-facing adapter thin. Test its contract outside X4 and its real
  behavior inside X4.
- Confirm the embedded Lua runtime and language constraints from current X4
  evidence before selecting a standalone test runner.

## Test Layers

### Static and Schema Checks

- Parse every XML file and validate the extension manifest, UI registration,
  Mission Director structure, identifiers, and generated package contents.
- Static source checks may protect a narrow integration invariant, but matching
  a string in Lua or XML is not primary evidence that behavior works.

### Pure Lua Unit Tests

- Use `Busted` as the standalone Lua test runner after confirming and pinning a
  version compatible with the Lua runtime and syntax supported by current X4.
- Execute pure modules outside X4 with deterministic fixtures.
- Cover malformed, missing, oversized, duplicate, stale, and out-of-order data;
  deterministic ordering; payload budgets; retry state; and error isolation.
- Test serialization output through an independent decoder or schema checker,
  not by repeating the serializer's implementation.

### Adapter Contract Tests

- Replace X4 globals and native calls with explicit fakes that model success,
  absence, malformed identities, permission or context failure, and thrown
  errors.
- Trace fake X4 observations through Lua envelopes and the Rust consumer.
- For commands, verify identity, preview and state-version checks, idempotency,
  rejection without partial mutation, bounded retries, and independent
  readback semantics.
- A fake proves the adapter contract, not that the real X4 API exists or behaves
  identically.

### In-Game Black-Box Tests

- Use a separate developer-only test/debug extension and Debug MCP surface over
  the same authoritative runtime state. Do not place this interface in the
  public package.
- Run scenarios in a disposable Creative Custom campaign or an approved copy of
  the owner's campaign. Never mutate the original live save as disposable test
  state.
- Capture exact X4 version, mod list, scenario setup, elapsed real and game time,
  SETA state, expected events/actions, observed readback, runtime health, and
  bounded diagnostics.
- Do not assume X4 has a supported headless test runner. Treat automation level
  as a research question until current evidence proves it.
- A test is `observed in X4` only when the game produced the expected behavior
  and the success/failure health surface remained valid.

### Correlated Developer Diagnostics

- A multi-hop X4 runtime probe must use an opt-in developer trace, separate
  from player-facing UI: every retained event identifies the probe attempt,
  hop, result, and bounded timing or size metadata.
- Correlate the X4 debuglog and bridge trace with the same attempt identity;
  include safe envelope metadata such as generation, sequence, and type when
  available, then diagnose the first missing or rejected hop.
- Keep normal operation to lifecycle and error diagnostics. Enable per-frame
  trace only for a bounded disposable probe, with rate and size limits.
- Never write raw payload content, saves, prompts, credentials, or hidden model
  reasoning into either trace. Record a sanitized, bounded failure reason.
- A runtime ledger entry must cite the correlated artifacts and distinguish
  their observations from static or fake-adapter evidence.

## Lua Mutation Testing

- Mutate only executable pure Lua modules initially. The native X4 adapter and
  Mission Director XML remain outside mutation scoring until a useful harness
  is demonstrated.
- Run mutation testing after the relevant phase and before release, not on every
  commit.
- Establish a baseline before choosing a required score. Review every survivor
  as a missing behavioral test, equivalent mutant, unsupported operator, or
  explicit gap.
- Run the first Lua mutation spike with `Universal Mutator` against one
  representative pure module and its `Busted` tests. Pin the evaluated tool
  versions and keep the spike isolated from public runtime dependencies.
- Measure invalid, trivial, equivalent, killed, and surviving mutants before
  admitting the tool as a required gate. Lua is not an officially listed
  Universal Mutator language, so generic rewriting alone is not acceptance.
- Project-specific Lua mutation rules are allowed; writing a replacement
  mutation engine is not. If the spike remains noisy or incompatible, defer the
  Lua mutation gate and retain unit, contract, and in-game tests instead.

## Required Evidence by Change

- Pure logic change: focused unit tests and applicable mutation evidence.
- X4 adapter change: unit tests, fake contract tests, and pending or completed
  in-game probe.
- Mission Director change: XML/static checks, event contract test, and in-game
  scenario.
- Game command change: Shadow result, canary-faction scenario, idempotent retry,
  independent readback, recovery, and no-partial-mutation evidence.
- Runtime scheduling or performance change: normal-speed and SETA soak with
  bounded health and timing evidence.

Report `verified locally`, `pending game smoke test`, and `observed in X4`
separately. Never collapse them into one passing status.
