# Changelog

## 2026-09-18

- Clarified TOOL-04 ABI prerequisite preparation on clean runners. Source:
  Actions run `35306856167` failed because the debug DLL was not built before
  ship ABI fixtures; the workflow also lacked compatible host preparation.

- Clarified LOG-02 context ownership before cleanup and rejection propagation.
- Source: Phase 05.5 run `055-normal-b4d6b65-20260918-005155` retained only
  `invalid_fact`; merged field checks and success-only metrics lost its cause.
- Added the ordinary-renderer regression example without changing log policy.

## 2026-09-05

- Added owner-approved common rules for all Live Galaxy engineering code.
- Scoped adoption to new/changed logic and necessary related corrections.
- Centralized detailed developer logging, journal-failure policy, and tooling.
- Preserved architecture authority without treating existing code as exemplar.
