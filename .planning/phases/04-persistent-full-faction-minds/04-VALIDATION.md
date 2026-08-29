---
phase: 04
slug: persistent-full-faction-minds
status: verified
nyquist_compliant: true
wave_0_complete: true
---

# Phase 04 — Validation Strategy

## Test Infrastructure

| Property | Value |
| --- | --- |
| Rust framework | Cargo integration tests with deterministic fixtures and in-memory checkpoint-port fake |
| Static X4 framework | PowerShell XML contract test; it proves schema only |
| Focused commands | `cargo test -p mind-domain --test <target>` and `cargo test -p mind-persistence --test <target>` |
| Quality commands | `cargo run -p source-size-lint` and `cargo clippy --workspace --all-targets -- -D warnings` |
| Full suite | `cargo test --workspace` |
| Mutation runner | Existing reviewed `cargo-mutants`, scoped to pure `mind-persistence` policy |
| Runtime proof owner | Phase 7 disposable Creative Custom campaign procedure in `04-X4-PERSISTENCE-EVIDENCE.md` |

## Evidence Levels

| Level | Phase 4 claim |
| --- | --- |
| Documented | Mission Director saved-state restoration, missing variables in older saves, and cue-version patches support the versioned root-cue schema. |
| Verified locally | Pure domain, codec, fake-port, recovery, migration, capsule, and XML schema contracts. |
| Pending-X4 | Payload capacity, write interruption, save/load restoration, actual MD compare-and-set behavior, and reconnect timing. |
| Observed in X4 | None in this phase; Phase 7 owns the recorded disposable proof. |

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirements | Threat Ref | Behavior | Automated command | Evidence level |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 04-01-01 | 01 | 1 | MIND-01, MIND-05 | T-04-01/T-04-03 | Paired packet fixture creates independent doctrine-divergent typed mind aggregates deterministically. | `cargo test -p mind-domain --test mind_tracer` | verified locally |
| 04-01-02 | 01 | 1 | INST-03, INST-08 | T-04-01/T-04-02 | Exactly three capability slots per faction; one active initiative per slot; proposal, objection, disposition, validation, ownership, preemption, and terminal events preserve causal history and idempotency. | `cargo test -p mind-domain --test initiative_lifecycle` | verified locally |
| 04-02-01 | 02 | 1 | STATE-01, STATE-06 | T-04-04/T-04-05 | Static MD root cue and shared manifest contain equal checkpoint schema/protocol/cursor/hash fields and no broadened game-side behavior. | `pwsh -NoProfile -File extensions/live_galaxy/tests/persistence_schema_contract.ps1` | verified locally |
| 04-02-02 | 02 | 1 | STATE-01, STATE-06 | T-04-06 | Phase 7 procedure distinguishes pending X4 observations from static contract evidence. | `pwsh -NoProfile -File extensions/live_galaxy/tests/persistence_schema_contract.ps1` | verified locally |
| 04-03-01 | 03 | 2 | STATE-01, STATE-02 | T-04-07/T-04-09 | Canonical envelope binds typed mind, strategic-tick, replay/admission, and report-reservation state; typed full-mind serde round-trip/replay rejects malformed checkpoint fields; envelope rejects malformed records and matches the shared MD schema manifest. | `cargo test -p mind-domain --test mind_checkpoint && cargo test -p mind-persistence --test checkpoint_tracer && cargo test -p mind-persistence --test schema_contract` | verified locally |
| 04-03-02 | 03 | 2 | STATE-03, STATE-06 | T-04-08/T-04-10 | Fake port proves acknowledged compare-and-set, retry-safe accepted strategic-tick identity, compatible reload without a duplicate tick, and restart-required disposition. | `cargo test -p mind-persistence --test fake_port_contract` | verified locally |
| 04-04-01 | 04 | 3 | STATE-04, STATE-05 | T-04-11 | Last valid acknowledged checkpoint survives corrupt, partial, stale, duplicate, out-of-order, migration, and three deterministic crash-point fixtures without exposing speculative state; restore rejects corrupted core/profile/slot/history/ledger/command fields. | `cargo test -p mind-persistence --test recovery_contract` plus `cargo test -p mind-domain --test mind_checkpoint` | verified locally |
| 04-04-02 | 04 | 3 | MODEL-05 | T-04-12/T-04-13/T-04-14 | Provider-relative capsule preserves typed authority; pure policy mutation baseline is measured and reviewed. | `cargo test -p mind-persistence --test capsule_contract && cargo test -p mind-persistence --test mutation_baseline && cargo mutants -p mind-persistence -- --test mutation_baseline` | verified locally or measured tool failure |

## Sampling Rate

- After every RED and GREEN task cycle: run the focused test, source-size lint, and strict workspace Clippy.
- After Wave 1 and Wave 2: run `cargo test --workspace` plus the static XML contract test.
- Before phase verification: run every focused target, `cargo fmt --check`, `cargo run -p source-size-lint`, strict Clippy, `cargo test --workspace`, and the scoped `cargo mutants -p mind-persistence -- --test mutation_baseline` baseline.
- Do not treat the fake port or XML parsing as X4 runtime proof. Phase 7 runs the prewritten disposable procedure and records the pending evidence.

## Requirement and Decision Coverage

| Source | Covered by | Evidence |
| --- | --- | --- |
| MIND-01, D-01 | 04-01 | Independent ZYA/ARG aggregate fields and typed Executive posture asserted by tracer. |
| MIND-05 | 04-01 | Shared-scenario fixture proves recorded doctrine causes non-cosmetic divergence. |
| INST-03, D-02 | 04-01 | One-active-slot and explicit replacement-predecessor contract. |
| INST-08, D-03 | 04-01 | Immutable typed causal ledger makes conversation prose non-authoritative. |
| STATE-01, D-06/D-07 | 04-02, 04-03 | Stable X4-owned MD schema plus opaque Rust envelope/port; no player-save or external-authority path. |
| STATE-02, D-08 | 04-03 | One complete envelope includes accepted snapshot, mind, initiative, replay, admission, and reserved report state. |
| STATE-03, D-10 | 04-03 | Exact retry and compatible reconnect recovery through acknowledged cursor. |
| STATE-04, STATE-05, D-09 | 04-04 | Adversarial recovery/migration fixtures retain last valid acknowledged state. |
| STATE-06, D-11 | 04-02, 04-03 | MD schema and fake compatibility contract distinguish compatible reconnect from explicit X4 restart. |
| MODEL-05, D-04/D-05 | 04-04 | Versioned typed-plus-narrative capsules use provider-relative measured budget profile identity. |

## Source Coverage Audit

| Source type | Item | Status | Plan |
| --- | --- | --- | --- |
| GOAL | Independent full minds and one-owner initiatives survive compaction, restart, retry, and schema transitions. | covered | 04-01 through 04-04 |
| REQ | MIND-01, MIND-05, INST-03, INST-08, MODEL-05, STATE-01 through STATE-06 | covered | map above |
| RESEARCH | MD root cue, fake checkpoint port, no new crate, canonical envelope, authority firewall, recovery matrix, static/pending-X4 split, mutation policy | covered | 04-01 through 04-04 |
| CONTEXT | D-01 through D-11 | covered | map above |

Deferred public API settings/credentials, mutable institutional power, multiple simultaneous initiatives, model calls, dialogue, report dispatch, general acknowledgement channel, and all X4 game-state mutation are excluded. Phase 6 retains ownership of report delivery and acknowledgement; Phase 7 retains ownership of disposable X4 observations.

## Mutation Strategy

- Scope `cargo-mutants` to pure envelope, recovery, migration, and capsule policy code after focused tests pass.
- Establish an actual baseline, review every survivor, and record each as a new behavioral test, equivalent mutant, unsupported operator, or bounded gap.
- Exclude Mission Director XML and fake-port/adapter code from score claims until the Phase 7 harness demonstrates actionable mutants.
- Do not set a mutation percentage target before measured evidence exists.

## Validation Sign-Off

- [x] All focused Rust and static XML contracts pass.
- [x] Every Rust source stays at or below 200 physical lines and strict Clippy is clean.
- [x] Workspace suite and formatter pass after the phase wave.
- [x] Mutation baseline records the measured reviewed-tool failure (`cargo-mutants` is unavailable); no score is inferred.
- [x] Phase documentation separates documented, verified locally, pending-X4, and observed-in-X4 facts.
- [x] Phase 7 owns all payload, interruption, save/load, and reconnect runtime observations.

**Approval:** verified locally; mutation score pending reviewed runner availability
