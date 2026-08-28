# Phase 01 Disposable X4 Evidence Ledger

## Ledger Contract

Ledger version: `1`

This ledger records evidence without promoting a weaker source to a stronger
level. Add an attempt row only after the matching procedure has been run or
explicitly deferred. `observed-in-X4` requires all setup, expected readback,
observed readback, and health fields from the procedure.

<!-- markdownlint-disable MD013 -->

| Evidence ID | Hypothesis | Baseline evidence | Game status | Evidence level |
| --- | --- | --- | --- | --- |
| OBS-X4-01 | Typed runtime identity and explicit section state | `documented-static`; `fake-local` | pending | `pending-X4` |
| OBS-X4-02 | Bounded producer and backpressure at normal speed and SETA | `documented-static`; `fake-local` | pending | `pending-X4` |
| OBS-X4-03 | Capability handshake, compatible reconnect, and X4-restart mismatch | `documented-static`; `fake-local` | pending | `pending-X4` |
| OBS-X4-04 | Scope-complete runtime enumeration | `documented-static`; `fake-local` | pending | `pending-X4` |

<!-- markdownlint-enable MD013 -->

## Attempt Template

Copy this section once per attempt. Keep failed and pending entries; do not
replace them with a later stronger result.

### Evidence ID: `OBS-X4-XX`

| Field | Recorded value |
| --- | --- |
| Attempt number | pending |
| Exact X4 version | pending |
| Active mod list | pending |
| Extension build identity | pending |
| Protocol identity | pending |
| Creative Custom scenario | pending |
| Isolated hypothesis | pending |
| Real elapsed time | pending |
| Game elapsed time | pending |
| SETA state | pending |
| Expected readback | pending |
| Observed readback | pending |
| Bounded health | pending |
| Evidence level | `pending-X4` |
| Game result | pending |
| Failure diagnostics | pending |
| Save access or modification | `none` |
| Game-state effect, report, or acknowledgement path | `none` |

## Classification Rules

- `observed-in-X4`: qualifying disposable game evidence includes all required
  fields and confirms the single hypothesis.
- `failed`: the attempt has a concrete mismatch, safety stop, or missing
  required runtime signal; preserve its diagnostics.
- `pending`: the attempt has not yet been run or did not collect enough data to
  classify a runtime outcome.
- `documented-static` and `fake-local`: baseline-only evidence; neither can be
  rewritten as an X4 observation.
