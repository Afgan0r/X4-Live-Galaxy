# Disposable X4 Observation Smoke Procedure

## Purpose

Use this procedure only after the exact X4 9.00 Lua and Mission Director APIs
for the narrow observation hook are documented or observed. It proves the
runtime seam; it does not replace the fake-contract or Rust decoder checks.

## Safety Boundary

- Use a new disposable Creative Custom campaign only.
- Do not read or modify any save file.
- Keep the extension telemetry-only. Do not add effect, report,
  acknowledgement, or generic-action paths while running this procedure.

## Procedure

1. Record X4 version, extension list, scenario, real time, game time, and
   SETA state before enabling the observation cue.
2. Probe one runtime scope at a time: sectors, assets, capacity, then
   ownership. Record the exact adapter API and stable identity evidence.
3. Observe normal-speed and SETA ticks. Confirm each tick samples no more than
   one section and returns `backpressure`, `bridge_unavailable`, or
   `save_suppressed` without waiting when applicable.
4. Force no game-state operation. Capture bridge health and emitted telemetry
   only, then independently read back the observed scope from X4.
5. Record expected and observed behavior, failures, bounded diagnostics, and
   whether the result is documented or observed in X4.

## Evidence Classification

- **Verified locally:** Rust `protocol_contract` and `backpressure_contract`,
  plus XML parsing, validate the fixture and static boundary.
- **Pending game smoke test:** exact embedded-Lua syntax, Mission Director hook,
  runtime APIs, identity stability, cadence, save detection, and SETA bounds.
- **Observed in X4:** only after this procedure records matching in-game
  behavior with the full environment details above.
