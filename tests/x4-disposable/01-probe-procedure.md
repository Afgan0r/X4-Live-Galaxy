# Phase 01 Disposable X4 Observation Procedure

## Purpose

This procedure is the only accepted route for upgrading a Phase 01 runtime
claim to `observed-in-X4`. It uses a disposable Creative Custom X4 9.00
campaign and tests one hypothesis at a time. It never reads or modifies a save
file.

## Evidence Taxonomy

<!-- markdownlint-disable MD013 -->

| Evidence level | Meaning | May be upgraded to |
| --- | --- | --- |
| `documented-static` | A versioned source, manifest, or XML check supports the claim. | `pending-X4` or `observed-in-X4` after a separate run. |
| `fake-local` | A deterministic fake, fixture, or local Rust test supports the contract. | `pending-X4` or `observed-in-X4` after a separate run. |
| `pending-X4` | The exact X4 9.00 behavior has not yet been observed in a qualifying run. | `observed-in-X4` only. |
| `observed-in-X4` | A disposable game run recorded all required setup, readback, and health data. | Not applicable. |
| `failed` | The attempted game probe did not meet its expected result or safety bounds. | `pending-X4` only after recording a new attempt. |

<!-- markdownlint-enable MD013 -->

Static checks, local tests, and fake adapters never earn `observed-in-X4`.
Each game attempt needs an independent row in
[`01-probe-evidence.md`](01-probe-evidence.md) with the same Evidence ID.

## Safety Boundary

- Use a new disposable Creative Custom campaign; do not inspect, copy, load,
  edit, or otherwise access any save file.
- Keep the extension telemetry-only. Do not enable or add fleet, economy,
  diplomacy, institution, report, acknowledgement, model, public API, or any
  other game-state effect path.
- Stop an attempt immediately if it requires an effect path, blocks observable
  game scheduling, exceeds the configured bounded work, or cannot produce
  correlated setup, readback, and health evidence.
- Treat initial frame, queue, cadence, native Lua/Mission Director, and
  named-pipe details as assumptions until a row earns `observed-in-X4`.

## Required Fields for Every Attempt

Record the following in the matching evidence row before classifying the
result:

- Evidence ID and attempt number.
- Exact X4 version and active mod list.
- Extension build identity and protocol identity.
- Creative Custom scenario and the isolated hypothesis.
- Real elapsed time, game elapsed time, and SETA state.
- Expected readback, observed readback, and bounded health result.
- Evidence level, result (`observed-in-X4`, `failed`, or `pending`), and
  failure diagnostics when applicable.

## Automated Prerequisite Checks

Run these before opening X4 and record their outcome as `documented-static` or
`fake-local`; they are prerequisites, not runtime proof.

```powershell
cargo test --workspace
cargo lint
<!-- markdownlint-disable-next-line MD013 -->
powershell -NoProfile -Command "[xml](Get-Content -Raw 'extensions/live_galaxy/content.xml') | Out-Null; [xml](Get-Content -Raw 'extensions/live_galaxy/md/live_galaxy_observation.xml') | Out-Null"
```

## Probe Sequence

### OBS-X4-01 — Typed Runtime Identity and Section State

1. Start the disposable Creative Custom scenario with only the intended
   observation extension and required support dependencies active.
2. Identify one runtime-discovered sector or asset and one section state
   (`unknown`, `partial`, `stale`, `unsupported`, or known coverage) without
   assuming an empty value means known-empty.
3. Observe the proposed telemetry/readback surface once at normal speed.
4. Record identity, source, observation time, monotonic version, section
   freshness, coverage, quality, expected readback, observed readback, and
   bounded health in the matching ledger row.
5. Mark `observed-in-X4` only if the identity and explicit section state are
   visible in the same qualifying attempt. Otherwise mark `failed` or
   `pending` with diagnostics.

### OBS-X4-02 — Bounded Producer and Backpressure

1. Use a fresh disposable Creative Custom scenario and do not combine this
   attempt with identity, handshake, or enumeration confirmation.
2. Run the observation path at normal speed for the recorded time window, then
   repeat the same bounded window under the recorded SETA state.
3. Observe producer/readback timing, queue or backpressure disposition, and
   health without introducing additional game work or an effect path.
4. Record both timing windows and state whether scheduling remained observable.
5. Mark `failed` and stop if the game visibly blocks, a bound is exceeded, or
   health is missing. Do not infer numeric limits from a successful run; record
   only the observed values.

### OBS-X4-03 — Capability Handshake and Reconnect Boundary

1. Use a fresh disposable Creative Custom scenario and record the extension
   build and protocol identity before connecting.
2. Exercise a compatible Rust-side reconnect without changing the game-facing
   extension. Record the new bridge/session generation and health readback.
3. Separately exercise an incompatible protocol, capability, or game-facing
   build combination using the explicitly configured test condition.
4. Record whether the compatible case reconnects without restarting X4 and
   whether the incompatible case fails closed with an explicit X4 restart
   requirement.
5. Never substitute a local fake outcome for either result. If the test setup
   cannot safely produce both conditions, mark the missing condition `pending`.

### OBS-X4-04 — Scope-Complete Runtime Enumeration

1. Use a fresh disposable Creative Custom scenario and select one discovery
   scope for sectors, assets, capacity, or ownership.
2. Trigger only the bounded read-only enumeration for that scope.
3. Record the scope identity, completion marker/readback, discovered member
   count, section quality, and bounded health.
4. Confirm that a missing or incomplete marker does not become known-empty and
   does not remove prior membership. Record the actual readback rather than an
   expected topology.
5. Mark `observed-in-X4` only when the same attempt shows a validated
   scope-complete result. Otherwise retain `pending` or record `failed`.

## Completion Rule

The ledger is complete only when every attempted probe has all required fields
and is classified at the level it earned. It is valid for any row to remain
`pending-X4`; a local automated pass does not close that gap.
