---
phase: 01
slug: read-only-observation-spine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-08-28
---

# Phase 01 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Cargo test harness; pure Lua runner deferred until X4 runtime compatibility is observed |
| **Config file** | `none — Wave 0 creates the Cargo workspace` |
| **Quick run command** | `cargo test -p observation-domain` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | Measure during Wave 0; keep focused feedback under 30 seconds |

## Sampling Rate

- **After every task commit:** Run the focused affected-crate test command.
- **After every plan wave:** Run `cargo test --workspace`.
- **Before `$gsd-verify-work`:** Full local suite must be green and manual X4
  evidence must be classified separately.
- **Max feedback latency:** 30 seconds for focused automated checks.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 01-02-01 | 02 | 2 | OBS-01 | T-01 | Bounded negotiated telemetry session | Rust integration + fake adapter | `cargo test -p x4-bridge protocol_contract && cargo test -p observation-ingest tracer_ingest` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 2 | OBS-02 | T-02 | Typed identity and monotonic version admission | Rust unit/property | `cargo test -p observation-domain identity_section_contract` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 2 | OBS-03 | T-02 | Section quality and freshness survive snapshot freezing | Rust unit | `cargo test -p observation-domain identity_section_contract` | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 2 | OBS-06 | T-02 | Reconcile only validated scope-complete observations | Rust unit + fake adapter | `cargo test -p observation-domain reconciliation_policy` | ❌ W0 | ⬜ pending |
| 01-04-01 | 04 | 4 | OBS-07 | T-01/T-02 | Invalid traffic preserves the last accepted snapshot | Rust integration | `cargo test -p observation-ingest atomic_rejection` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 2 | OBS-08 | T-03 | Protocol contains no mutation vocabulary | Schema/contract | `cargo test -p x4-bridge protocol_contract` | ❌ W0 | ⬜ pending |
| 01-07-01 | 07 | 5 | VAL-06 | T-04 | Runtime claims remain pending until disposable X4 evidence exists | Static + manual X4 probe | `cargo test --workspace` plus recorded probe | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## Wave 0 Requirements

- [ ] Cargo workspace and pure `observation-domain` test crate.
- [ ] Adversarial envelope fixtures and a fake X4 adapter.
- [ ] XML/static package validation for the first extension.
- [ ] Runtime Lua syntax probe before selecting a pure Lua runner.
- [ ] Disposable Creative Custom probe procedure and evidence template.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Exact X4 9.00 Lua/MD APIs, identity stability, scheduling, and bounded normal-speed/SETA behavior | VAL-06 | Static evidence and fakes cannot prove runtime semantics | Use a disposable Creative Custom campaign; record exact X4 version, extension list, scenario, real/game time, SETA state, expected observations, readback, health, and diagnostics. Never inspect or modify saves. |

## Validation Sign-Off

- [ ] All tasks have automated verification or explicit Wave 0 dependencies.
- [ ] Sampling continuity: no three consecutive tasks without automated proof.
- [ ] Wave 0 covers every missing test reference.
- [ ] No watch-mode flags.
- [ ] Focused feedback latency is under 30 seconds.
- [ ] Local, pending game smoke, and observed-in-X4 evidence remain separate.
- [ ] `nyquist_compliant: true` is set only after validation succeeds.

**Approval:** pending
