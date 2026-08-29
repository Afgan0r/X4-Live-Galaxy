---
status: diagnosed
trigger: "Diagnose whether the post-Plan 01-11 failure in strategic-state::mutation_baseline::visibility_availability_capacity_and_canonical_order_remain_observable (assertion zya_foreign_economy.is_some()) is caused by timestamp-authority changes, a flaky/pre-existing condition, or a test/fixture contract migration issue."
created: 2026-08-29T00:00:00+07:00
updated: 2026-08-29T00:00:00+07:00
---

## Current Focus

bug_class: bohrbug
hypothesis: "Confirmed: the Plan 01-11 runtime-fact migration helper leaves the obsolete root `observed_at_unix_millis` field and adds obsolete runtime-fact key `t`, both rejected by `deny_unknown_fields`; every migrated strategic fixture is rejected atomically, yielding an empty snapshot."
test: "Five exact repetitions; static schema comparison; ast-index and LSP navigation of the helper and test call site."
expecting: "All repetitions fail identically; neither obsolete key belongs to the strict v2 schema; no observation enters the projection."
next_action: "Implement only the fixture-helper v2 serialization correction, then require `AdmissionOutcome::Accepted` in every migrated strategic-state fixture test before deriving packets."

## Symptoms

expected: "The full workspace remains green as it was immediately before Plan 01-11 timestamp-authority work."
actual: "Only reported post-change failure: strategic-state::mutation_baseline::visibility_availability_capacity_and_canonical_order_remain_observable asserts zya_foreign_economy.is_some()."
errors: "assertion failed: zya_foreign_economy.is_some()"
reproduction: "cargo test -p strategic-state --test mutation_baseline visibility_availability_capacity_and_canonical_order_remain_observable -- --exact"
started: "After the Plan 01-11 timestamp-authority and v2 runtime-fact fixture migration; the workspace was reportedly green immediately before those changes."

## Eliminated

- hypothesis: "The helper's `faction:fixture` runtime ownership changes `FactOwner` and hides the ARG economy fact."
  evidence: "`derive.rs` assigns `FactOwner` from the observation `entity_id` prefix, not `RuntimeFacts.ownership.owner_id`; the legacy ARG entity ID remains unchanged."
  timestamp: 2026-08-29T00:00:00+07:00

## Evidence

- timestamp: 2026-08-29T00:00:00+07:00
  checked: Workspace and remote freshness
  found: "The checkout is on codex/0.1-shadow-director, ahead 3, with extensive pre-existing uncommitted Phase 1 work; `git fetch --prune origin` completed. No local changes were altered."
  implication: "Use the current dirty worktree as the supplied post-Plan-01-11 evidence; do not infer a clean historical baseline from it."

- timestamp: 2026-08-29T00:00:00+07:00
  checked: Existing timestamp-authority diagnostic and amended 01-11 plan
  found: "Plan 01-11 explicitly migrates observation fixtures from client Unix timestamps to v2 runtime facts with bridge receipt-time authority; the relevant strategic-state test was changed to call `support::runtime_fact_frames`."
  implication: "A test/fixture migration is a direct candidate; timestamp authority itself is not yet shown to affect packet visibility."

- timestamp: 2026-08-29T00:00:00+07:00
  checked: Exact reported test
  found: "`cargo test -p strategic-state --test mutation_baseline visibility_availability_capacity_and_canonical_order_remain_observable -- --exact --nocapture` fails at `mutation_baseline.rs:36` with exactly `assertion failed: zya_foreign_economy.is_some()`."
  implication: "The symptom is deterministic on the current worktree, not yet a flaky or live-X4 condition."

- timestamp: 2026-08-29T00:00:00+07:00
  checked: `crates/strategic-state/tests/support/mod.rs` against `crates/observation-ingest/src/wire.rs` and `src/runtime_facts.rs`
  found: "The helper keeps the legacy root `observed_at_unix_millis` and emits runtime key `t`; `WireObservation` and `RuntimeFacts` are both `deny_unknown_fields` and define neither field."
  implication: "The v2 decoder rejects every migrated helper frame before a projection observation is created. The helper's `faction:fixture` ownership is semantically unrelated because strategic derivation classifies owner from `entity_id`, not runtime ownership."

- timestamp: 2026-08-29T00:00:00+07:00
  checked: `crates/observation-ingest/src/batch.rs:112-137` and `crates/strategic-state/src/derive.rs:44-74`
  found: "`admit_batch` returns the prior projection after rejection; callers use `into_projection()` without asserting `AdmissionOutcome::Accepted`. `derive_packets` accepts a zero-observation snapshot and emits empty packets, so the first visibility assertion reports the downstream symptom."
  implication: "This is a fixture-contract migration defect caused by the Plan 01-11 strict receipt-time schema, not a strategic visibility-policy regression."

- timestamp: 2026-08-29T00:00:00+07:00
  checked: Bounded stability test and package suite
  found: "The exact test failed 5/5 times with the identical assertion. `cargo test -p strategic-state` reaches the same sole reported failure after the preceding `capability_contract` and `doctrine_priority` tests pass."
  implication: "Flakiness is refuted for this reproduction; the visible failure is deterministic and tightly localized to the migrated mutation-baseline fixture path."

- timestamp: 2026-08-29T00:00:00+07:00
  checked: Semantic navigation
  found: "`ast-index symbol runtime_fact_frames` resolves the helper to `crates/strategic-state/tests/support/mod.rs:5`; LSP resolved the test call site and reported no unrelated diagnostics in the helper or v2 wire schema."
  implication: "The failure is a valid compile-time-clean semantic contract violation, not an unresolved-symbol or compiler diagnostic."

## Resolution

root_cause: "Confirmed single cause (AND-gate: no): Plan 01-11's `crates/strategic-state/tests/support/mod.rs::runtime_fact_frames` attempts to adapt legacy observations but serializes two v1-only fields: root `observed_at_unix_millis` and runtime-fact `t`. Strict v2 `WireObservation` and `RuntimeFacts` both reject unknown fields. Atomic admission preserves the empty prior projection; because `mutation_baseline.rs` calls `into_projection()` without first asserting acceptance, `derive_packets` produces empty packets and the later `zya_foreign_economy.is_some()` assertion fails."
fix: "Not applied (diagnosis-only). Smallest exact remediation: rewrite only `runtime_fact_frames` to construct a strict v2 observation from parsed legacy `scope`, `entity_id`, `version`, and `quality`, omitting legacy root `observed_at_unix_millis`, `content`, and runtime key `t`; retain valid `g` only when deliberately needed. In each migrated strategic-state test helper call, assert `AdmissionOutcome::Accepted` before deriving the snapshot, so malformed fixture migrations fail at admission rather than at a downstream visibility assertion."
verification: "Reproduce before remediation: `cargo test -p strategic-state --test mutation_baseline visibility_availability_capacity_and_canonical_order_remain_observable -- --exact`. Verify remediation: run that command, then `cargo test -p strategic-state`; add/retain a helper-level assertion that a generated v2 frame is accepted and that its runtime facts have bridge-stamped receipt time."
files_changed: []

## Diagnosis

| Candidate | Classification | Evidence |
| --- | --- | --- |
| Timestamp-authority regression in strategic visibility logic | Refuted | `derive.rs` still derives `FactOwner` from legacy-compatible `entity_id`; receipt time does not participate in visibility. |
| Flaky or pre-existing test | Refuted for current worktree | Exact test failed identically 5/5 times, after the Plan 01-11 helper was introduced. Historical pre-change execution is reported, not locally reproducible from the dirty worktree. |
| V2 fixture contract migration defect | Confirmed | Helper emits two fields forbidden by strict v2 decoder; rejection is atomic; its caller discards the rejected outcome before the downstream assertion. |

Skill learning: none — reviewed the Rust conventions and testing rules. This defect is covered by their existing boundary-contract principle: fixture adapters must be admitted through the same strict boundary and tests must assert the boundary outcome. No new durable project rule is justified.
