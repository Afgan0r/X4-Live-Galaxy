---
phase: 05
phase_name: bounded-shadow-deliberation
audit_date: 2026-08-29
ai_spec_present: true
overall_score: 74.29
verdict: NEEDS WORK
critical_gap_count: 1
---

# EVAL-REVIEW — Phase 05: Bounded Shadow Deliberation

**Audit Date:** 2026-08-29

**AI-SPEC Present:** Yes

**Overall Score:** 74.29/100
**Verdict:** NEEDS WORK

The deterministic Phase 05 evaluation contract is now implemented and executable.
The remaining missing strategic-quality evidence is deliberately manual and non-gating
for Phase 05 code completion, but it blocks any quality, production, public-readiness,
playability, or observed-X4 claim. This is still a 0.x internal prototype.

## Dimension Coverage

| Dimension | Status | Measurement | Finding |
| --- | --- | --- | --- |
| Admission safety and Shadow-only scope | COVERED | Code | Ordered admission, no pending projection after rejection, one-CAS persistence, idempotent retry, and forbidden external-effect paths passed focused tests. |
| Faction knowledge discipline | PARTIAL | Code; human packet-label sampling | Visibility, stale-state preflight, cache revalidation, and frozen-packet posture checks passed. The planned human packet-label sampling has not occurred; deterministic fakes cannot verify real provider context discipline. |
| Replay, cache, and recovery determinism | COVERED | Code | Exact-key component variation and canonical ordering, cache revalidation, no duplicate CAS, checkpoint replay, and timeout/reconciliation paths passed. |
| Bounded orchestration and degradation | COVERED | Code | Required nonzero bounds, one outstanding request, bounded process outcomes, pause/reconcile behavior, two-cycle cap, and no speculative state change after failure are executable and green. |
| Institution and Executive causality | PARTIAL | Code; human benchmark review | Typed preemption/replay and finite dialogue contracts passed. The planned human review of real benchmark trajectories for coherence has not been performed. |
| ZYA/ARG doctrine divergence | MISSING | Subscription benchmark with calibrated human review | No real subscription run, paired ZYA/ARG trajectory, blind human labels, or calibration exists. Posture/fake acceptance does not prove material doctrine divergence. |
| Causal continuity and non-repetition | PARTIAL | Code; subscription benchmark with calibrated human review | Retention/preemption and causal-record invariants are tested, but no real trajectory establishes non-churn or validates the human rubric. |

**Coverage Score:** 4/7 (57.14%)

## Infrastructure Audit

| Component | Status | Finding |
| --- | --- | --- |
| Eval tooling (Rust integration tests + manifests) | OK | Focused corpus, provider-contract, standalone harness, workspace, formatting, Clippy, and source-size gates executed successfully. The AI-SPEC intentionally requires no external eval platform. |
| Reference dataset | OK | `v1` has all 13 required CI cases plus a distinct `SD-010-benchmark` manual subcase, digest-pinned schema/fixtures, closed scenario/disposition validation, and tamper/path tests. SD-012 direct agreement is restored to CI, as the AI-SPEC requires. |
| CI/CD integration | OK | `.github/workflows/phase-05.yml` runs format, strict workspace and harness Clippy, focused corpus/provider tests, workspace tests, standalone harness tests, and source-size lint on pull requests and pushes. |
| Online guardrails | OK | Pure request/candidate admission, cache revalidation, finite Shadow posture effects, bounds, stale preflight, scheduler degradation, and checkpoint CAS are in real runner paths and are tested. |
| Tracing (correlated evidence tuple) | OK | `RedactedEvidence` is emitted for accepted and degraded public runner outcomes. Its fields cover correlation, faction, snapshot/hash, identities, candidate hash/size, validator, usage/latency availability, admitted IDs, evidence class, and recovery; the contract test verifies redaction and bounded output. |

**Infrastructure Score:** 90/100

## Critical Gaps

- **BLOCKER — Strategic-quality evidence is missing for any public or production claim.** No real subscription benchmark or calibrated human review proves doctrine divergence, causal continuity, non-repetition, or institutional coherence. The AI-SPEC explicitly defers this from the Phase 05 deterministic completion gate; it must remain a blocker for quality/public readiness, not be misrepresented as a failed deterministic Phase 05 code contract.

## Remediation Plan

### Deliberately manual / later-phase follow-up

1. Run the explicit developer-controlled subscription harness on representative paired ZYA and ARG trajectories; retain only redacted evidence and do not create Phase 8 thresholds here.
2. Have the owner and a strategic-simulation expert blind-label doctrine divergence, continuity, non-repetition, and institution coherence. Calibrate any future LLM judge against those labels before using it for decisions.
3. Phase 7 owns disposable X4 operational evidence. Phase 8 owns measured quality/reliability baselines and release thresholds; neither is required for Phase 05 deterministic completion.

## Files and Commands Found

**Evidence files:**

- `shadow-deliberation-evals/v1/manifest.json`, schema, and fixtures
- `crates/mind-domain/tests/shadow_deliberation_evals.rs`
- `crates/mind-persistence/tests/deliberation_checkpoint.rs`
- `crates/mind-orchestration/src/{runner,degraded,evidence}.rs`
- `crates/mind-orchestration/tests/provider_contract/{paths,stale}.rs`
- `tools/shadow-harness/src/evidence.rs` and tests
- `.github/workflows/phase-05.yml`

**Executed successfully (offline, deterministic):**

```text
cargo test -p mind-domain --test shadow_deliberation_evals --locked --offline
cargo test -p mind-orchestration --test provider_contract --locked --offline
cargo test --manifest-path tools/shadow-harness/Cargo.toml --locked --offline
cargo test --workspace --locked --offline
cargo fmt --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo clippy --manifest-path tools/shadow-harness/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo run -p source-size-lint --locked --offline -- crates tools
```

The score was calculated by the required deterministic command:

```text
node C:/Users/pavlo/.codex/gsd-core/bin/gsd-tools.cjs query eval.score --covered 4 --total 7 --infra ok,ok,ok,ok,ok --raw
```

Result: `coverage_score: 57.14`, `infra_score: 100`, `overall_score: 74.29`,
`verdict: NEEDS WORK`.
