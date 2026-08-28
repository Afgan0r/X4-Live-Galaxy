---
phase: 03
slug: faction-scoped-strategic-state
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-08-29
---

# Phase 03 — Validation Strategy

## Test Infrastructure

| Property | Value |
| --- | --- |
| Framework | Cargo test harness with pure Rust integration/property fixtures |
| Focused command | `cargo test -p strategic-state --test <target>` |
| Quality command | `cargo run -p source-size-lint` |
| Full suite | `cargo test --workspace` |
| Mutation runner | Existing reviewed `cargo-mutants`, scoped to `strategic-state` |
| Feedback target | Focused checks under 30 seconds; measure actual duration on first execution |

## Sampling Rate

- After every task commit: run that task's focused automated target and the source-size/strict-Clippy command.
- After every plan wave: run `cargo test --workspace`.
- Before phase verification: run all seven strategic-state targets—`tracer_packet`, `visibility_contract`, `capability_contract`, `doctrine_priority`, `shadow_primitive_contract`, `packet_determinism`, and `mutation_baseline`—plus workspace tests, source-size lint, and the scoped mutation baseline.
- No X4 test is required: Phase 3 consumes frozen Phase 1 projections and makes no game-side call. Phase 1 runtime semantics stay an explicit compatibility assumption in local tests.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirements | Threat Ref | Secure behavior | Test type | Automated command | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 03-01-01 | 01 | 1 | OBS-04, OBS-05, MIND-03 | T-03-02 | One four-family accepted projection derives paired ZYA/ARG packets with XEN/KHK semantics and explicit availability | Rust tracer integration | `cargo test -p strategic-state --test tracer_packet` | pending |
| 03-01-02 | 01 | 1 | MIND-02 | T-03-01/T-03-02 | Paired faction policies preserve the same authorized base while making visibility differences explicit | Rust contract | `cargo test -p strategic-state --test visibility_contract` | pending |
| 03-02-01 | 02 | 2 | INST-01, INST-02 | T-03-04 | Exactly three shared capabilities read one shared visible snapshot | Rust contract | `cargo test -p strategic-state --test capability_contract` | pending |
| 03-02-02 | 02 | 2 | MIND-03, INST-01 | T-03-03 | Six internal labels and two doctrine orders are versioned product-policy fixtures | Rust deterministic fixture | `cargo test -p strategic-state --test doctrine_priority` | pending |
| 03-03-01 | 03 | 3 | OBS-04, OBS-05, MIND-03 | T-03-05 | Four typed planning-only primitives are exhaustive, owned, non-mutating, capped at four candidates, priority 1..100, and eight evidence references each | Rust contract | `cargo test -p strategic-state --test shadow_primitive_contract` | pending |
| 03-03-02 | 03 | 3 | MIND-04 | T-03-06 | Primitive/evidence permutations preserve canonical packet/fingerprint; unsupported or ambiguous input fails closed | Rust property-style fixture | `cargo test -p strategic-state --test packet_determinism` | pending |
| 03-03-03 | 03 | 3 | MIND-04, INST-01, INST-02 | T-03-07 | Visibility/order/capability/primitive regressions receive measured mutation evidence | Rust mutation baseline | `cargo test -p strategic-state --test mutation_baseline && cargo mutants -p strategic-state -- --test mutation_baseline` | pending |

## Requirement and Decision Coverage

| Source | Covered by | Evidence |
| --- | --- | --- |
| OBS-04 | 03-01, 03-03 | paired ZYA/ARG derivation covers economic, military, territorial, and threat facts with explicit availability |
| OBS-05 | 03-01, 03-03 | paired fixtures retain XEN shared pressure and KHK only when observed; primitives reference the typed threat evidence |
| MIND-02 | 03-01 | versioned pre-derivation visibility policy |
| MIND-03 | 03-01, 03-02, 03-03 | bounded strategic inputs, institution views, and finite typed Shadow primitive candidates |
| MIND-04 | 03-03 | canonical ordering and replay fingerprint include primitive/evidence keys |
| INST-01 | 03-02 | exactly three shared capability contracts and six locked labels |
| INST-02 | 03-02 | all three views reference one faction-visible snapshot |
| D-01, D-02, D-03 | 03-01 | authorized own/static facts; no omniscience; explicit unavailable state |
| D-04, D-05, D-06 | 03-02 | exact roster, differentiated doctrine profiles, shared knowledge |
| D-07, D-08 | 03-03 | Executive-only bilateral posture primitive, no negotiation or X4 relation change |
| D-09 | 03-03 | canonicalized facts, priorities, primitive/evidence keys, admission inputs, and fingerprint |

## Source Coverage Audit

| Source type | Item | Status | Plan |
| --- | --- | --- | --- |
| GOAL | Deterministic, replayable ZYA and ARG packets with institutional views and Executive Shadow posture inputs | covered | 03-01 through 03-03 |
| REQ | OBS-04, OBS-05, MIND-02, MIND-03, MIND-04, INST-01, INST-02 | covered | map above |
| RESEARCH | pure crate; availability; paired faction visibility; four fact families; six labels; policy-priority inference; finite primitive allowlist; Executive-only diplomacy; canonical replay; bounds; mutation baseline | covered | 03-01 through 03-03 |
| CONTEXT | D-01 through D-09 | covered | map above |

Deferred private institutional knowledge, mutable influence, sabotage, internal politics, a diplomacy institution, and inter-faction negotiation are excluded as required by Phase 3 context.

## Validation Sign-Off

- [ ] Focused targets pass after each task.
- [ ] Source-size lint confirms every Rust source file is no more than 200 physical lines and strict Clippy is green.
- [ ] Workspace suite passes after every wave.
- [ ] Current Phase 1 runtime-semantics assumption remains visible and compatibility-tested.
- [ ] Mutation baseline reports measured survivor dispositions or an explicit tool-availability failure.
- [ ] Local verification is reported separately from pending Phase 1 X4 runtime evidence.

**Approval:** pending execution
