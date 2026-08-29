---
phase: 05
slug: bounded-shadow-deliberation
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-08-29
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust built-in test harness |
| **Config file** | `Cargo.toml` workspace |
| **Quick run command** | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` |
| **Full suite command** | `cargo test --workspace --locked` |
| **Estimated runtime** | Measure during Wave 0; do not invent a threshold |

## Sampling Rate

- **After every task commit:** Run the focused corpus plus targeted crate tests.
- **After every plan wave:** Run the full workspace suite, strict Clippy,
  formatter check, and source-size lint.
- **Before `$gsd-verify-work`:** The full deterministic suite must be green.
- **Max feedback latency:** Measure the focused corpus during Wave 0 and record
  the observed bound before execution expands it.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 05-01-01 | 01 | 1 | MIND-06, MODEL-02 | T-05-01/T-05-02 | RED tracer proves strict candidate admission then one atomic checkpoint CAS. | integration | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | MIND-06, MODEL-02, MODEL-03 | T-05-01/T-05-02/T-05-03 | Ordered admission accepts typed Shadow state and rejects without side effects. | unit/integration | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-01-03 | 01 | 1 | MODEL-02, MODEL-03 | T-05-01/T-05-02 | Idempotent acceptance has one CAS and redacted bounded evidence. | integration | `cargo test --workspace --locked` | ❌ W0 | ⬜ pending |
| 05-02-01 | 02 | 2 | MODEL-04, MODEL-06 | T-05-04/T-05-06 | Exact-key mutations, ordering, and absent resource limits fail deterministically. | unit/property | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-02-02 | 02 | 2 | MODEL-02, MODEL-04, MODEL-06 | T-05-04/T-05-05/T-05-06 | Cache hits revalidate every gate and all explicit bounds are enforced. | unit | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-02-03 | 02 | 2 | MODEL-04 | T-05-04/T-05-05 | Stale cache replay has redacted deterministic identity diagnostics and no state change. | property | `cargo test --workspace --locked` | ❌ W0 | ⬜ pending |
| 05-03-01 | 03 | 3 | MIND-07, MODEL-06 | T-05-07 | Trigger coalescing, interruption, cooldown, and pause/reconcile paths remain bounded. | unit/property | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-03-02 | 03 | 3 | INST-04, INST-05, INST-06, INST-07 | T-05-08/T-05-09 | Executive action remains admission-gated; causal preemption has one owner and dialogue is zero or at most two cycles. | state machine | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-03-03 | 03 | 3 | MIND-07, INST-05, INST-07 | T-05-07/T-05-09 | Replay/interruption preserves causal predecessor and bounded diagnostics. | property | `cargo test --workspace --locked` | ❌ W0 | ⬜ pending |
| 05-04-01 | 04 | 4 | MODEL-01, MODEL-03, MODEL-07 | T-05-10/T-05-11/T-05-12 | Fake port and timeout/degradation contracts are deterministic and evidence-separated. | integration | `cargo test -p mind-orchestration --test provider_contract --locked` | ❌ W0 | ⬜ pending |
| 05-04-02 | 04 | 4 | MODEL-01, MODEL-03, MODEL-06 | T-05-10/T-05-11 | Shared provider port has bounded same-identity retry and requires reconciliation after failure. | integration | `cargo test -p mind-orchestration --test provider_contract --locked` | ❌ W0 | ⬜ pending |
| 05-04-03 | 04 | 4 | MODEL-03, MODEL-07 | T-05-11/T-05-12 | Fake/cache/provider paths cannot duplicate CAS or leak sensitive evidence. | integration | `cargo test --workspace --locked` | ❌ W0 | ⬜ pending |
| 05-05-01 | 05 | 5 | MODEL-01, MODEL-07 | T-05-13/T-05-14/T-05-15 | Manual harness isolation and all SD corpus cases are testable without a subscription. | contract | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-05-02 | 05 | 5 | MODEL-01, MODEL-03, MODEL-06 | T-05-13/T-05-14/T-05-16 | Explicit local CLI adapter is timeout/byte bounded and returns typed unavailable/failure outcomes. | contract | `cargo test --manifest-path tools/shadow-harness/Cargo.toml --test manual_contract --locked` | ❌ W0 | ⬜ pending |
| 05-05-03 | 05 | 5 | MIND-06, MODEL-01, MODEL-04, MODEL-06, MODEL-07 | T-05-13/T-05-15/T-05-16 | Corpus integrity, monitoring fields, and evidence classes remain complete and non-thresholded. | integration | `cargo test --workspace --locked` | ❌ W0 | ⬜ pending |

_Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky_

## Wave 0 Requirements

- [ ] `crates/mind-domain/tests/shadow_deliberation_evals.rs` — executable
  coverage for AI-SPEC cases SD-001 through SD-013.
- [ ] `shadow-deliberation-evals/v1/manifest.*` — fixture hashes,
  schema/policy/prompt versions, expected deterministic outcomes, and evidence
  class.
- [ ] Controlled fake-provider fixtures for malformed, timeout, cache, replay,
  dialogue, and recovery paths.
- [ ] Manual-only harness contract test proving normal CI never invokes the
  subscription client and never accepts fake trajectories as quality evidence.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Subscription-backed trajectory and harness capability record | MODEL-01, MODEL-07 | Account/model availability, latency, usage metadata, and strategic quality cannot be proven by a fake or normal CI. | Run the explicit developer harness against the versioned corpus; retain redacted provider/model/configuration and trajectory evidence. Do not derive acceptance thresholds in Phase 5. |

## ASVS L1 Completion Gate

Plan 05 Task 3 creates `05-SECURITY-REVIEW.md` from the focused corpus, provider-contract, harness-contract, workspace lint, and source-size evidence. It must evaluate V2 local-client authentication and no-secret handling, V3 non-authoritative session metadata, V4 frozen faction access and admission authority, and V5 strict bounded candidate validation. Every finding records an ID, severity, disposition, evidence command, and remediation. Any unresolved high-severity V2–V5 finding fails Phase 5 completion; this artifact supplements and does not replace the normal `$gsd-secure-phase 5` review.

## Validation Sign-Off

- [x] Every planned behavior has an automated command or a Wave 0 dependency.
- [x] Sampling continuity requires automated verification after every task.
- [x] Wave 0 covers every currently missing deterministic fixture and contract.
- [x] No watch-mode flags are used.
- [x] Feedback latency is measured before a numeric bound is recorded.
- [x] `nyquist_compliant: true` is set in frontmatter.
- [ ] V2–V5 security review is complete with no unresolved high-severity finding.

**Approval:** planning contract approved; execution evidence pending
