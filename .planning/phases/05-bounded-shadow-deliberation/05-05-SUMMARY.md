---
phase: 05-bounded-shadow-deliberation
plan: 05
subsystem: manual-provider-harness
tags: [rust, codex-cli, provider-port, corpus, asvs]
requires:
  - phase: 05-04
    provides: Provider-neutral port, admission runner, and degraded recovery.
provides:
  - Isolated manual-only harness contract with a standalone lockfile.
  - Redacted evidence metadata and complete SD corpus manifest check.
  - ASVS L1 V2–V5 completion review with no unresolved high findings.
affects: [phase-06-diagnostics, phase-08-evaluation]
actuals:
  tokens: 4109
  tasks: 3
  commits: 3
tech-stack:
  added: []
  patterns: [manual-only-provider-adapter, redacted-benchmark-evidence, corpus-integrity-gate]
key-files:
  created: [tools/shadow-harness/Cargo.toml, tools/shadow-harness/src/subscription_adapter.rs, tools/shadow-harness/src/evidence.rs, .planning/phases/05-bounded-shadow-deliberation/05-SECURITY-REVIEW.md]
  modified: [.planning/phases/05-bounded-shadow-deliberation/05-VALIDATION.md]
key-decisions:
  - The subscription route is manual-only and must return unavailable rather than using an API fallback.
  - Benchmark evidence is bounded and redacted, never authoritative state or a quality threshold.
requirements-completed: [MIND-06, MODEL-01, MODEL-03, MODEL-04, MODEL-06, MODEL-07]
coverage:
  - id: D1
    description: Manual adapter shares the typed provider boundary while remaining outside workspace runtime and CI.
    requirement: MODEL-01
    verification:
      - kind: integration
        ref: tools/shadow-harness/tests/manual_contract.rs#manual_adapter_uses_the_shared_port_and_is_not_quality_evidence
        status: pass
    human_judgment: false
  - id: D2
    description: SD-001 through SD-013 manifest identity and benchmark evidence classification remain complete.
    requirement: MODEL-07
    verification:
      - kind: integration
        ref: tools/shadow-harness/tests/manual_contract.rs#manifest_pins_every_sd_case_and_manual_evidence_class
        status: pass
    human_judgment: false
  - id: D3
    description: A real subscription-backed benchmark can only be collected through an explicit developer command.
    requirement: MODEL-01
    verification: []
    human_judgment: true
    rationale: Local authentication, configured model availability, and real trajectory quality are intentionally outside deterministic CI.
duration: resumed execution
completed: 2026-08-29
status: complete
---

# Phase 05 Plan 05: Manual Harness and Corpus Gate Summary

**Manual-only Codex CLI provider boundary with redacted evidence, complete deterministic SD corpus validation, and a blocking ASVS V2–V5 review.**

## Accomplishments

- Added a standalone `shadow-harness` package with explicit path dependencies and its own lockfile.
- Proved manual-provider boundary and benchmark evidence class contracts without a subscription request.
- Added manifest completeness validation and recorded ASVS V2–V5 findings with no unresolved high severity.

## Task Commits

1. **Task 1: RED manual-only harness isolation and evidence cases** — `c36a14a` (`test`)
2. **Task 2: GREEN the explicit local Codex CLI adapter and redacted evidence writer** — `4889b5e` (`feat`)
3. **Task 3: REFACTOR corpus completeness, monitoring fields, and validation mapping** — `946e882` (`refactor`)

## Verification

- `cargo test --manifest-path tools/shadow-harness/Cargo.toml --test manual_contract --locked --offline`
- `cargo test -p mind-domain --test shadow_deliberation_evals --locked --offline`
- `cargo test --workspace --locked --offline`
- `cargo fmt --check`
- Strict workspace and harness Clippy commands
- `cargo run -p source-size-lint --locked --offline -- crates tools`

All deterministic commands passed. A direct lockfile generation attempt could not reach crates.io because the sandbox rejected the system TLS credentials; the existing local Cargo cache resolved the pinned graph offline.

## Manual Benchmark Status

No real `codex exec` subscription benchmark was run. It is explicitly manual-only, non-CI evidence and cannot establish strategic quality, observed X4 behavior, playability, public readiness, or any Phase 8 threshold.

## Deviations from Plan

### Auto-fixed Issues

1. **[Rule 3 - Blocking] Split the standalone tool library from its binary entrypoint.**
   - **Found during:** Task 2
   - **Issue:** Sharing `main.rs` between Cargo library and binary targets emitted a duplicate-target warning.
   - **Fix:** Added cohesive `src/lib.rs` for the provider contract exports.
   - **Verification:** Standalone strict Clippy and harness tests pass.

## Self-Check: PASSED

- Verified RED `c36a14a`, GREEN `4889b5e`, and REFACTOR `946e882` exist in history.
- Verified all harness, validation, and security-review artifacts exist.
