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
| 05-01-01 | 01 | 1 | MIND-06, MODEL-02 | T-05-01/T-05-02 | Candidate bytes pass every ordered validator before one atomic admission. | unit/integration | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | MIND-07, MODEL-06 | T-05-03 | Trigger coalescing, queue, call, retry, timeout, payload, and dialogue limits fail closed. | unit/property | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-02-01 | 02 | 2 | INST-04, INST-05 | T-05-04 | Executive actions and replacements use the same admission path and preserve causal predecessor evidence. | unit/state machine | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-02-02 | 02 | 2 | INST-06, INST-07 | Agreement performs zero dialogue cycles; exceptional dialogue cannot exceed two complete cycles. | unit/property | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-03-01 | 03 | 2 | MODEL-01, MODEL-07 | Fake and manual harness share one typed port while CI cannot treat fake output as quality evidence or invoke the subscription client. | integration/static contract | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |
| 05-03-02 | 03 | 2 | MODEL-03, MODEL-04 | Failure preserves accepted state; every cache-key component changes identity and every hit is revalidated. | integration/property | `cargo test -p mind-domain --test shadow_deliberation_evals --locked` | ❌ W0 | ⬜ pending |

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

## Validation Sign-Off

- [x] Every planned behavior has an automated command or a Wave 0 dependency.
- [x] Sampling continuity requires automated verification after every task.
- [x] Wave 0 covers every currently missing deterministic fixture and contract.
- [x] No watch-mode flags are used.
- [x] Feedback latency is measured before a numeric bound is recorded.
- [x] `nyquist_compliant: true` is set in frontmatter.

**Approval:** planning contract approved; execution evidence pending
