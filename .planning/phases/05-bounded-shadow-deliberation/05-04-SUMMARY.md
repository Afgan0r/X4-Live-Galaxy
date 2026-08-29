---
phase: 05-bounded-shadow-deliberation
plan: 04
subsystem: provider-orchestration
tags: [rust, provider-port, shadow-admission, degradation, replay]
requires:
  - phase: 05-01
    provides: strict shadow admission and atomic checkpoint projection
  - phase: 05-02
    provides: exact cache identity and current-state revalidation
  - phase: 05-03
    provides: bounded scheduler pause and reconciliation semantics
provides:
  - provider-neutral bounded effect port with deterministic fake evidence
  - admission-gated runner with cache parity and redacted degradation records
  - one-CAS persistence contract for replayed provider outcomes
affects: [phase-05-05, phase-06-diagnostics, phase-08-evaluation]
actuals:
  tokens: 3925
  tasks: 3
  commits: 3
tech-stack:
  added: []
  patterns: [provider-neutral-effect-port, admission-only-runner, bounded-degradation]
key-files:
  created:
    - crates/mind-orchestration/src/provider_port.rs
    - crates/mind-orchestration/src/runner.rs
    - crates/mind-orchestration/src/degraded.rs
    - crates/mind-orchestration/tests/provider_contract.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/mind-orchestration/Cargo.toml
    - crates/mind-orchestration/src/lib.rs
key-decisions:
  - Provider adapters return candidate bytes or typed failures and cannot access persistence or X4 state.
  - Cache bytes use the same admission path; checkpoint persistence remains the only CAS owner.
  - Fake and manual evidence classes are explicit and deterministic fake evidence is never a quality marker.
requirements-completed: [MODEL-01, MODEL-03, MODEL-06, MODEL-07]
coverage:
  - id: D1
    description: Deterministic fake provider replay and cache bytes pass through the same bounded admission path.
    requirement: MODEL-01
    verification:
      - kind: integration
        ref: crates/mind-orchestration/tests/provider_contract.rs#deterministic_fake_replays_candidate_through_shared_admission
        status: pass
    human_judgment: false
  - id: D2
    description: Provider timeout preserves accepted state, emits bounded degradation, and requires a newer observation before recovery.
    requirement: MODEL-03
    verification:
      - kind: integration
        ref: crates/mind-orchestration/tests/provider_contract.rs#provider_timeout_pauses_until_newer_reconciled_observation
        status: pass
    human_judgment: false
  - id: D3
    description: Fake, cache, and manual paths cannot create a second checkpoint compare-and-set.
    requirement: MODEL-06
    verification:
      - kind: integration
        ref: crates/mind-orchestration/tests/provider_contract.rs#fake_cache_and_manual_paths_share_admission_but_only_persistence_owns_one_cas
        status: pass
    human_judgment: false
  - id: D4
    description: Deterministic fixture evidence stays distinct from manual harness evidence and cannot become a strategic-quality marker.
    requirement: MODEL-07
    verification:
      - kind: integration
        ref: crates/mind-orchestration/tests/provider_contract.rs#deterministic_fake_replays_candidate_through_shared_admission
        status: pass
    human_judgment: false
duration: 4m
completed: 2026-08-29
status: complete
---

# Phase 05 Plan 04: Provider Orchestration Summary

**A provider-neutral, synchronous Rust effect seam now sends fake or future manual candidate bytes through the existing Shadow admission path while bounded degradation preserves accepted state and blocks stale recovery.**

## Performance

- **Duration:** 4m
- **Started:** 2026-08-29T03:04:52Z
- **Completed:** 2026-08-29T03:08:36Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Added a typed `ShadowProvider` boundary with bounded request identifiers, provider metadata, candidate bytes, typed failures, and explicit evidence classes.
- Added a runner that routes provider and cache candidate bytes through the one existing domain admission function without persistence or X4 authority.
- Added bounded timeout degradation with one recorded attempt and newer-observation reconciliation.
- Proved fake/cache/manual parity and that `mind-persistence` remains the sole checkpoint CAS owner.

## Task Commits

1. **Task 1: RED shared-port fake and degradation contracts** — `f129b3c` (`test`)
2. **Task 2: GREEN the bounded provider port and degradation runner** — `f191028` (`feat`)
3. **Task 3: REFACTOR evidence separation and bounded metadata** — `5e22511` (`refactor`)

## Decisions Made

- The runner returns admission or typed degradation only; it has no persistence, X4, report, tool, credential, or prompt-log capability.
- Provider metadata is limited to bounded provider/model identifiers; raw candidate bytes, prompts, credentials, hidden reasoning, and local paths are not retained in degradation records.
- Deterministic fixtures satisfy replay and contract tests only. Strategic-quality evidence remains manual-harness-only.

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None.

## Verification

- `cargo test -p mind-orchestration --test provider_contract --locked`
- `cargo test -p mind-domain --test shadow_deliberation_evals --locked`
- `cargo test --workspace --locked`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo run -p source-size-lint --locked -- crates`

All commands passed. No automated test uses a network, provider account, credential, API runtime, X4 mutation, or save access.

## Next Phase Readiness

Plan 05-05 can add the isolated manual subscription harness behind the same port. The deterministic kernel remains provider-type-free and no accepted outcome has an X4 effect.

## Self-Check: PASSED

- Verified all eight declared workspace, crate, source, and contract-test artifacts exist.
- Verified RED `f129b3c`, GREEN `f191028`, and REFACTOR `5e22511` exist in Git history.
- Verified focused contracts, workspace tests, formatting, strict Clippy, and source-size lint pass.
