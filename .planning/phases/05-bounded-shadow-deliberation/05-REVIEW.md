---
phase: 05-bounded-shadow-deliberation
reviewed: 2026-08-29T00:00:00Z
depth: deep
files_reviewed: 40
files_reviewed_list:
  - Cargo.toml
  - crates/mind-domain/Cargo.toml
  - crates/mind-domain/src/admission.rs
  - crates/mind-domain/src/arbitration.rs
  - crates/mind-domain/src/cache_identity.rs
  - crates/mind-domain/src/deliberation.rs
  - crates/mind-domain/src/ledger.rs
  - crates/mind-domain/src/ledger/preemption.rs
  - crates/mind-domain/src/lib.rs
  - crates/mind-domain/src/mind.rs
  - crates/mind-domain/src/mind_commands.rs
  - crates/mind-domain/src/posture.rs
  - crates/mind-domain/src/preemption_admission.rs
  - crates/mind-domain/src/request_bounds.rs
  - crates/mind-domain/src/scheduler.rs
  - crates/mind-domain/tests/shadow_deliberation_evals.rs
  - crates/mind-domain/tests/shadow_deliberation_evals/dialogue.rs
  - crates/mind-domain/tests/shadow_deliberation_evals/exact_cache.rs
  - crates/mind-domain/tests/shadow_deliberation_evals/posture.rs
  - crates/mind-orchestration/Cargo.toml
  - crates/mind-orchestration/src/degraded.rs
  - crates/mind-orchestration/src/lib.rs
  - crates/mind-orchestration/src/provider_port.rs
  - crates/mind-orchestration/src/runner.rs
  - crates/mind-orchestration/tests/provider_contract.rs
  - crates/mind-persistence/src/checkpoint.rs
  - crates/mind-persistence/src/checkpoint/accessors.rs
  - crates/mind-persistence/src/checkpoint_preemption.rs
  - crates/mind-persistence/src/checkpoint_validation.rs
  - crates/mind-persistence/src/deliberation_checkpoint.rs
  - crates/mind-persistence/src/legacy.rs
  - crates/mind-persistence/src/lib.rs
  - crates/mind-persistence/tests/deliberation_checkpoint.rs
  - shadow-deliberation-evals/v1/manifest.json
  - tools/shadow-harness/Cargo.toml
  - tools/shadow-harness/src/evidence.rs
  - tools/shadow-harness/src/lib.rs
  - tools/shadow-harness/src/main.rs
  - tools/shadow-harness/src/subscription_adapter.rs
  - tools/shadow-harness/tests/manual_contract.rs
findings:
  critical: 5
  warning: 1
  info: 0
  total: 6
status: issues_found
---

# Phase 05: Code Review Report

**Reviewed:** 2026-08-29T00:00:00Z
**Depth:** deep
**Files Reviewed:** 40
**Status:** issues_found

## Summary

The reviewed implementation has several broken Phase 5 control paths despite its passing deterministic fixtures. Cross-module tracing found that the manual provider cannot run, its alleged timeout is not enforced, the scheduler cannot complete successful work, and cache revalidation cannot establish that the frozen snapshot is still current. The review also checked admission ordering, checkpoint CAS/replay behavior, preemption binding, posture isolation, bounds, redaction, test reliability, and the 200-line module cap.

## Critical Issues

### CR-01: Manual subscription adapter is permanently unavailable

**File:** `tools/shadow-harness/src/subscription_adapter.rs:14-16`

**Evidence:** The only public constructor sets `available: false`; the field is private and no other constructor exists. Structural usage/caller analysis finds only the unavailable test instance and no route that can call `explicit_benchmark` successfully. Every real benchmark therefore returns `ProviderFailure::Unavailable`, contradicting the manual-harness requirement.

**Fix:** Replace the boolean with a validated explicit benchmark configuration/constructor (for example `SubscriptionAdapter::for_explicit_benchmark(...) -> Result<Self, ProviderFailure>`), construct it only from the `--benchmark` CLI command, and add a fake process-spawner contract test that proves the configured path reaches the bounded invocation.

### CR-02: The manual provider has no execution timeout or pre-read output limit

**File:** `tools/shadow-harness/src/subscription_adapter.rs:28-36`

**Evidence:** `Command::output()` waits indefinitely and buffers all stdout/stderr before returning. `TIMEOUT_MILLIS` is only compared to zero after process completion, so its `30_000` value never limits execution. The 64 KiB check likewise occurs after the unbounded buffering allocation. A hung or chatty local `codex` process can block the harness or exhaust memory, violating the required time and payload bounds.

**Fix:** Execute via a bounded child-process abstraction that concurrently drains capped stdout/stderr, waits until a real deadline, then kills and reaps the child on expiry. Return `Timeout` on deadline and `Oversized`/a distinct bounded failure before retaining oversized output. Cover hung-child and over-limit-output cases deterministically.

### CR-03: Successful deliberations are never completed, so every faction remains outstanding forever

**File:** `crates/mind-domain/src/scheduler.rs:75-95`

**Evidence:** Eligibility sets `FactionState.outstanding = true` at line 93. The only method that clears it is `timeout` at line 100; there is no success/rejection/cancellation completion transition. Structural callers/usages show the scheduler is exercised only by tests, so no orchestration layer can clear it after an admitted or rejected response. After the first successful request, subsequent requests for that faction are permanently `Coalesced`.

**Fix:** Add a kernel-owned terminal method such as `complete(faction, completed_at)` that clears `outstanding` without pausing, wire it through every provider/admission terminal outcome, and test a successful request followed by a later tick/event that becomes eligible.

### CR-04: Relevant events are suppressed after any strategic tick

**File:** `crates/mind-domain/src/scheduler.rs:86-91, 135-139`

**Evidence:** `tick()` maps every `RelevantEvent` and `Interrupted` trigger to `0`. Once a previous strategic tick is recorded (for example `4`), the cooldown check treats a later relevant event as `0 <= 4 + cooldown` and returns `Cooldown`. This makes event-driven replanning impossible after normal tick scheduling, contradicting MIND-07's relevant-event trigger.

**Fix:** Include the observation/tick identity in `RelevantEvent` and `Interrupted`, or track cooldown separately from event relevance. Compare monotonically increasing observation identities and add tests for an eligible relevant event after a completed strategic-tick request.

### CR-05: Cache revalidation cannot reject a changed strategic snapshot

**File:** `crates/mind-domain/src/admission.rs:130-131, 185-187`

**Evidence:** The sole current-state check compares only faction. `admit` receives neither a current snapshot identity nor a state revision, so an old `DeliberationRequest` and cached bytes remain accepted whenever the faction is unchanged, even after observations, visible facts, or active initiatives changed. This is exactly the stale-cache bypass that D-05/D-12 require revalidation to prevent.

**Fix:** Make the current authoritative snapshot/version a required input to admission/revalidation and reject when it differs from the frozen request identity or when its visible-fact/current-initiative projection differs. Route cache hits through that comparison and add a same-faction, changed-snapshot regression test with zero pending commit/CAS.

## Warnings

### WR-01: Corpus integrity test accepts malformed or incomplete manifests

**File:** `tools/shadow-harness/src/evidence.rs:35-40`

**Evidence:** `validates_manifest` uses substring presence only. It does not parse JSON, enforce unique IDs, associate every case with its own `fixture_hash`, or require all declared SD-010 variants. For example a malformed string containing all `SD-001` through `SD-013`, one `fixture_hash`, and one benchmark marker passes. The test at `manual_contract.rs:24-27` therefore does not prove the claimed corpus integrity.

**Fix:** Deserialize a strict manifest struct, reject duplicate/missing IDs and malformed fields, compare the exact required deterministic and benchmark case set, and validate each fixture hash/version/evidence-class association.

---

_Reviewed: 2026-08-29T00:00:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
