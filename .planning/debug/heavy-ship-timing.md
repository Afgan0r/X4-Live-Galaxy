---
status: investigating
trigger: "Continue Phase 05.5 heavy ship collection performance diagnosis"
created: 2026-10-10
updated: 2026-10-10
---

# Heavy Ship Timing

## Symptoms

Expected: Complete the dynamically discovered faction set with owned core and detail snapshots, bounded callbacks, and accepted normal/SETA performance.
Actual: Normal Gate B run `055-gate-b-normal-20261001-005056` committed 27 sections across 8 core factions, then exhausted its 60-second admission window during freesplit core revision 28. No rejected/failed bridge outcomes were recorded.
Errors: Coverage failed despite active FPS 172.552 and control FPS 195.193 in sequential windows. These windows do not isolate causal overhead.
Timeline: Earlier single-selection source timings were faster; current cumulative duration includes resumed capture, delivery, and callback waits and cannot be compared as getter cost.
Reproduction: Existing dynamic full-set Lua producer and Carrier B bridge; retained runtime evidence and procedure are identified in `tests/x4-disposable/05.5-heavy-ship-evidence.md`.

## Current Focus

hypothesis: Runtime throughput cause remains unknown. Offline diagnostic preparation is complete; observed source/processing/delivery intervals and callback gaps can now distinguish candidate causes.
test: Main performs independent scoped review, final regression and package preparation before the approved finite X4 diagnostic experiment.
expecting: Runtime evidence separates observed active selection intervals from between-selection-callback waits without treating cumulative age as getter CPU.
next_action: Main reviews the bounded Lua diff and prepares the exact package/procedure for the authorized timing experiment; retain Gate B as failed/pending.
package_checkpoint: Source `3b6d3deb307e536eb9c83d0d975c4851c5dee5d7` packaged and installed with X4 closed; all 27 extension hashes match manifest `42e4eba243dfb441f8620fe43d6eff3d2864ad9951afcb78b654286403a1ab2e`. Awaiting owner normal-time readiness; no actual diagnostic run yet.

Preparation checkpoint: Independent scoped GSD review found no blocking findings. Final `run_contracts.ps1 -Suite all` passed in 79.804 seconds: 116 Lua tests, 21 syntax checks, XML/schema, actual native bridge lifecycle/full-set/recovery paths and performance gate. Measured local callback maxima were 3.329 ms before restart and 3.494 ms afterward against `1000 / 144` ms; final backlog was zero. These synthetic results are not X4 FPS evidence. No Rust/native/ABI/configuration source changed.

reasoning_checkpoint:
  hypothesis: Cumulative-only budget diagnostics omit attribution because sample_callback retains only age and callback maxima.
  confirming_evidence:
    - The real budget module returns no timing table and the authored RED fails its attribution assertion.
    - Collector continuations already sample carrier monotonic milliseconds after steps; source and delivery share one retained budget.
  falsification_test: Existing production-path metrics already expose disjoint interval categories with unchanged sampling count.
  fix_rationale: Aggregate existing sample-to-sample intervals by fixed observed operation classes; count entered adapter operations and keep active callback intervals separate from waits.
  blind_spots: Exclusive getter CPU, status sampling overhead, quantization and the complete runtime callback remain unmeasured; runtime throughput cause remains unknown.
  candidate_causes:
    - code: cumulative-only diagnostics omit required attribution.
    - environment: coarse or frozen carrier clock limits measurement precision.
  and_gate: No for the missing diagnostic table; runtime throughput may need several contributing conditions and is not diagnosed.

tdd_checkpoint:
  test_file: extensions/live_galaxy/tests/heavy_profile_contract.lua
  test_name: attributes observed step intervals and callback waits without extra status reads
  status: green
  failure_output: component attribution must accompany cumulative age; passed nil, expected table

bug_class: bohrbug (missing deterministic diagnostic attribution); runtime throughput classification remains unknown.
oracle_type: specified (owner-approved observed intervals, operation counts, no added status calls).
candidate_causes: code (cumulative-only diagnostics), environment (coarse carrier clock), config (delivered pulse cadence), data (full-roster workload).
and_gate: Runtime throughput may require several conditions; no runtime root cause is confirmed.

## Scope

This is a Plan 05.5-11 in-scope diagnostic repair, not phase completion or a new research wave. Preserve existing callback, age, admission, allocation, safety, and transport contracts. Do not change cadence before its X4 source contract and the timing evidence are established. No game start/control, save access, sibling edits, or raw captures in Git.

## Evidence

- Source revision: `e4ee1d6`, clean and synchronized with origin at task start.
- Runtime package source: `b9a16c15b6f211bc65e9964944d9dc4f2dd36ce7`.
- Argon: 943 ships; cumulative capture/delivery duration reached 23052 ms; maximum observed callback 4 ms. This is not 23052 ms of continuous CPU work.
- Existing source: MD tick checkinterval 50ms; cooperative source budget 2ms; budget checks currently invoke carrier progress/status sampling after each collector step.
- The budget wraps source calls but records only cumulative age and callback maxima. Selection captures all families before delivery and reuses the owned budget for retained detail delivery.
- The existing source `clock_getter` is game time. Main-agent Docs MCP searches returned no `GetCurRealTime` unit evidence. The approved feasible instrumentation uses existing carrier monotonic snapshots with no extra status reads.
- Observed step intervals include source-adapter copying, owned processing and the status sample. Exclusive getter CPU, status cost and the full runtime callback cannot be isolated with this clock seam; those costs remain explicitly unmeasured.
- SBFL skipped: no per-test coverage spectrum and no existing failing attribution test. Pure fake-clock regression uses the real budget module; no property framework is needed for this finite interval model.
- RED executed: `pwsh -NoProfile -File extensions/live_galaxy/tests/run_contracts.ps1 -Suite ship_detail -Filter 'attributes observed step intervals'` returned exit 1, 0 successes / 1 failure / 0 errors. The original budget completes both callbacks but supplies no component timing table; assertion fails for the missing attribution contract.
- GREEN target executed with the same command: exit 0, 1 success / 0 failures / 0 errors / 0 pending. Existing two status reads remain two.
- Held-out production-path assertions observe active source and retained delivery intervals, one origin identity across cached detail requests, independent copied metric snapshots, and equal summed interval/callback active durations. The first ship-detail run had 35 successes / 1 failure: its new expected delivery count omitted four intentionally busy begin attempts; the observed 956 equals 950 records plus begin/finish plus four busy reservations. The oracle was corrected to that independently derived count.
- Final focused checks: ship-detail 37 successes / 0 failures / 0 errors / 0 pending; scheduler (ordinary runtime renderer) 35 successes / 0 failures / 0 errors / 0 pending; syntax 21 successes / 0 failures / 0 errors / 0 pending. All runner exits were 0. An initial runtime assertion incorrectly expected an unmeasured tail for returned rejections; the existing final sample measures those rejections, while thrown charging errors retain a pending tail. Its oracle now distinguishes both paths.
- Fix-acceptance signal 5 temporarily replaced only the task-owned budget edits through apply_patch with the exact HEAD baseline. The same target returned exit 1, 0 successes / 1 failure / 0 errors / 0 pending at line 166 with missing attribution. Reapplying the exact instrumentation restored the complete ship-detail result of 37 successes. No Git state mutation was used.

## Instrumentation interpretation

The fixed classes are source, allocation, clock, processing, delivery and mixed.
An interval is the time between existing carrier samples, including the entered
adapter call, immediate owned processing, instrumentation and unmeasured status
sampling overhead. Multiple operation classes before one sample are mixed,
never apportioned into invented exclusive times. Operation counts represent
entered wrapped adapter calls or delivery attempts, including busy attempts;
they are not a count of individual native calls inside an adapter.

Selection callback count/total/max and between-selection-callback count/total/max
share the retained budget lifetime. The origin section/revision/incarnation
remains the core capture even when the current diagnostic belongs to cached
detail delivery. Emitted metric tables are independent snapshots. Nested
existing samples advance one interval boundary, preventing duration duplication.
No extra progress/status calls, guessed X4 clock conversion or per-step logs
were added. Unsampled callbacks, pending operation tails and intra-callback
clock regressions remain explicit. Coarse/frozen clocks can produce zero
observed intervals; zero is not evidence of zero CPU work.

Exclusive getter CPU, status/clock overhead, final diagnostic rendering and the
complete runtime callback remain unmeasured. In particular, callback gaps
include scheduler/feedback/other work outside the selected budget and are not
automatically idle time. The new instrumentation itself has unmeasured runtime
cost. Approved cadence, budgets, source-before-delivery, lossless retained
delivery, safety refusals, native ABI and protocol remain unchanged.

## Eliminated

Continuous selection work does not explain most observed wall time in the
2026-10-10 normal diagnostic: Argon finished all four sections in 25011 ms,
with 1074 ms measured selection-active intervals and 23937 ms between them.
This does not eliminate scheduler/feedback cost, native-call stalls,
instrumentation overhead or clock/preemption effects.

## Runtime diagnostic (2026-10-10)

Source `3b6d3de`, run `055-gate-b-normal-20261010-110531`, retained evidence
and locator are recorded in `tests/x4-disposable/05.5-heavy-ship-evidence.md`.
28 durable sections passed exact independent readback; Argon has 945 ships.
Its retained budget recorded 354 selection callbacks, 84216 status samples,
840 ms source-class intervals and a 32 ms maximum callback interval.
The run stopped intentionally on that ceiling violation; full Gate B was
not attempted to completion. Exclusive CPU and full callback remain unknown.

The final FrameView CSV row was truncated and conflicted with complete rows
for the same frame identity. FPS is invalid, not deduplicated into a pass.
Next: narrowly inspect between-selection scheduling/feedback and sampling
cost. Cadence changes still require the applicable source contract; do not
expand budgets to compensate. SDK service and owned processes are stopped.

## Resolution

root_cause: The offline attribution defect was cumulative-only diagnostics in ship_budget sample_callback. The X4 throughput incident root cause remains unknown.
fix: Fixed bounded observed interval and operation aggregates, selection active/wait aggregates, copied origin-bound snapshots and terminal/error runtime rendering. Offline preparation completed only.
oracle_type: specified
verification:
  target_test: { result: pass, successes: 1, failures: 0 }
  mutation_check: { result: skipped, reason_if_skipped: No configured Stryker/Lua mutation runner; no replacement framework introduced. }
  no_op_deletion: { result: pass, deletion_justified_by_rca: false, evidence: Additive diagnostics and labels; collection and yield decisions retained. }
  adjacent_tests: { result: pass, suites_run: [ship_detail, scheduler, syntax], successes: [37, 35, 21] }
  revert_and_reconfirm: { result: pass, bug_returned_on_revert: true, fixed_on_reapply: true }
  guardrail_verdict: accepted
  scope: Offline instrumentation preparation only; independent review/full regression/package are main-owned; actual X4 diagnosis and normal/SETA Gate B remain pending.
files_changed:
  - extensions/live_galaxy/lua/live_galaxy_ship_budget.lua
  - extensions/live_galaxy/lua/live_galaxy_ship_selection.lua
  - extensions/live_galaxy/lua/live_galaxy_runtime.lua
  - extensions/live_galaxy/lua/live_galaxy_ship_collection.lua
  - extensions/live_galaxy/lua/live_galaxy_ship_details.lua
  - extensions/live_galaxy/tests/heavy_profile_contract.lua
  - extensions/live_galaxy/tests/ship_synchronous_contract.lua
  - extensions/live_galaxy/tests/carrier_b_contract.lua
