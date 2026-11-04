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

hypothesis: Frequent 2 ms cooperative yields resume on a 50 ms configured MD cue; the same cue clocks stop-and-wait sends and ACK consumption, adding delay to already captured detail delivery.
test: Trace real selection/budget/scheduler and native send/feedback paths against retained Argon endpoint differences and receiver timestamps.
expecting: Separate pre-core-publication continuation gaps from cached-detail delivery gaps; do not equate configured cue spacing with exact observed idle time.
next_action: Use verified package dist/live-galaxy-callback-7a60c7c at the owner installation/runtime checkpoint, then obtain normal/SETA/load/reload evidence. Exact runtime cue-versus-scheduler cost, complete engine callback cost and true per-call CPU remain unmeasured.
package_checkpoint: Diagnostic source `3b6d3de` installed and measured in retained run `055-gate-b-normal-20261010-110531`; full Gate B remains open.

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

## Continuation and delivery attribution (2026-10-10)

Read-only diagnosis used fresh, clean `e7e2d42`, synchronized with origin.
The actual package JSON and generated Lua configuration both set
`callback_budget_millis=2` (not the separate `1000 / 144` complete-frame
ceiling). `ship_budget.should_yield` returns true at this elapsed threshold;
the core/detail tick loops return without finishing the requested selection.
`md/live_galaxy_observation.xml` schedules the next runtime event through its
configured `checkinterval="50ms"` cue. There is no local sleep, but returning
to this infrequent event creates continuation latency. The September 22
fallback intentionally introduced cooperative yielding after a failed
synchronous performance gate; removing safety yielding is not the diagnosis.

Retained same-origin endpoint differences separate the measured gaps:

| Endpoint | Additional measured selection-active ms | Additional between-selection ms |
| --- | --- | --- |
| Core publication, after all families captured | 1000 | 18101 |
| Cached cargo delivery | 10 | 918 |
| Cached crew delivery | 31 | 2466 |
| Cached loadout delivery | 33 | 2452 |

Core publication reached 322 selection callbacks (321 gaps, mean 56.39 ms).
All four endpoints reached 354 callbacks (353 gaps, mean 67.81 ms).
The 50 ms value is configured cue cadence, not a measured real-time sleep:
do not multiply it by callback count and claim an exact causal time split.
The pre-core 18101 ms includes capture continuations and initial core delivery;
the terminal aggregate does not isolate the capture-finish instant.

Cached detail requests reuse owned records and the same budget: native source
call count stays 55854 across all four endpoints. Their extra 5836 ms gaps
are therefore not repeated X4 getter work. Receiver history records core,
cargo, crew and loadout reception spans of 532, 691, 2274 and 2255 ms,
respectively; received-message outcome counts are 4, 5, 15 and 15. These
receiver timestamps use Unix ms, not the producer monotonic clock; no absolute
cross-clock subtraction was used.

The shared transport cadence explains a concrete avoidable latency:
`live_galaxy_scheduler.tick` calls `carrier.progress(1)` before
`carrier.poll_control()`, once per event. Native `progress(0)` used by the
84216 budget samples only snapshots status; it never sends or consumes ACKs.
Native `producer_feedback.advance_message` clears an acknowledged pending
message and seals its successor; there is no subsequent send in that scheduler
tick. The successor waits for the next event's `progress(1)`. While awaiting
final ACK/new demand, occupied capacity also skips selection.advance entirely;
these events are absent from selection callback counts but remain inside the
between-selection elapsed intervals. No synchronous wait for ACK was found
in Lua; native control polling uses try_lock/try_recv rather than waiting.

The clock itself is a worker-refreshed atomic snapshot (`transport.snapshot`,
`transport_worker.refresh_clock`, with a worker-loop 1 ms sleep), not a fresh
high-resolution clock read on each Lua sample. Therefore active/gap boundaries
and the 32 ms outlier remain sampled observations, not exclusive CPU proof.
Status-sampling cost is included in sampled active intervals where observed;
it must not be added again as a separate explanation for all 23937 ms gaps.

Executed attribution contract again: 1 success / 0 failures / 0 errors,
exit 0. No production code, installed package, game state or cadence changed.
Recommendation: preserve bounded yielding and stop-and-wait authority, but
avoid tying source continuation and transport progress to the same slow cue;
first resolve the supported higher-frequency runtime seam through Docs MCP.
Changing pump/continuation scheduling requires the owner-approved scope and
focused ordering/backpressure tests plus a new runtime comparison. Exact
exclusive scheduler/feedback/idle durations cannot be reconstructed from the
existing terminal-only aggregates.

## Approved remedy checkpoint (2026-10-10)

The owner approved trying more frequent bounded continuations and same-tick
ACK successor handoff. The independent transport slice now performs one extra
`progress(1)` after `poll_control()` returns Received (4), unless the initial
progress was terminal. It does not poll again or wait for another ACK. Fresh
progress/status still controls feedback and source admission; errors discard
the selection. Successor failures preserve operation and owned correlation;
the ordinary renderer exposes only a validated negative signed-32 numeric
code, not native text. The prior status is never used to admit source work
after a failed operation.

Independent full-depth review of scheduler/runtime/contract tests resolved
WR-ACK-01 (lost successor-failure diagnostics) with no open findings.
The final `pwsh -NoProfile -File extensions/live_galaxy/tests/run_contracts.ps1
-Suite all` exited 0 in 71.226 seconds: 139 Lua tests, 21 syntax checks,
XML/schema, actual native bridge lifecycle/full-set/recovery and performance
gate. Synthetic callback maxima were 3.625 ms before restart and 3.2914 ms
afterward, below `1000 / 144` ms; both final backlogs were zero. These are
offline regression measurements, not an X4 FPS verdict or a causal speedup
comparison. No runtime package was installed or game experiment started.

The source-continuation slice remains blocked on supported global callback
registration/lifecycle evidence. Docs MCP searches for `onUpdate`, `SetScript`,
`RegisterEvent` and `checkinterval` exhausted to null in snapshots
`x4-9.00-steam-23660954`,
`x4-9.00-steam-23660954-faction-ship-observation-v1` and
`x4-9.00-steam-23660954-ship-detail-source-v1`, with no matching items or
conflicts. This is a corpus limit, not proof that the API is absent.

Snapshot `x4-probe-md-pulse-a70af158-v1` returned entity
`entity:47eea368f480d4b1c105f886f4018eb6fb0d9bf8d6eb035cd5a0c87711c46b08`;
all four inspect pages were exhausted. Its cadence/lifecycle claims observe
a 50 ms MD pulse, pause suspension and accelerated real-time delivery under
SETA; they do not establish a safe global per-frame Lua callback. Narrow
read-only vanilla evidence also shows menu-owned `menu.onUpdate` and a
`Check_Fadeout` cinematic cue using `checkinterval="1ms"`; neither proves
global registration, removal/reload ownership or once-per-frame admission.
No timer/callback change or runtime experiment is authorized by that evidence
alone. Request enrichment through the shared chat-only knowledge-gap handoff.

## Callback evidence retry (2026-11-04)

Fresh clean checkout `74c2e12` was fast-forwarded from `03ea3b4` after fetching
origin. The registered `x4-docs` server was queried directly. The original
`search` request (`query="onUpdate"`, `limit=20`) with snapshots
`x4-9.00-steam-23660954`,
`x4-9.00-steam-23660954-faction-ship-observation-v1`, and
`x4-9.00-steam-23660954-ship-detail-source-v1` still returned zero items,
complete coverage and null cursor, with no conflicts.

The same search plus `SetScript`, `RemoveScript`,
`Register_NewFrame_Callback`, `Unregister_NewFrame_Callback` and
`Register_Require_With_Init` used this new snapshot set, unchanged for every
inspection:

- `x4-9.00-steam-23660954-lua-update-source-v1`, root
  `blake3:b6899beafb6296c67d5ce5553cde59c2e98fc08253029ff4b84c31f309340930`;
- `sn-mod-support-apis-1.95-lua-update-source-v1`, root
  `blake3:58d7a86bfcef8e7731cb23bb3434f7487129499cd37359855994fed2f7aba1d4`;
- `x4-probe-lua-update-20261101-02-v1`, root
  `blake3:3377dea4a906fd9408aaa96e543edf7798f6aca4b9bcb81c8a502d89c56927cc`.

All searches used limit 20 and ended with null cursor. Seven entities were
selected; `inspect` limit 20 exhausted 50 pages to null, without conflicts:

| Entity | Stable entity ID | Pages | Coverage |
| --- | --- | --- | --- |
| onUpdate | entity:6e37157eb0482fba481743d63bc779b60cedb7fb6632246076ae75dcdbf15be0 | 10 | partial |
| Helper.onUpdate | entity:c6211a9431b00e308f6e9776e14ea985eba30e8cb54a1c1a6570975636c87dd9 | 2 | complete |
| SetScript | entity:ecb05e111049f571c02c97f1c370d1d0093dc13d325b74af6b9ccafcd62377d4 | 7 | partial |
| RemoveScript | entity:e0766b54e6c0c3b7ea8f324db1ef451c0f5591e91ca788698655c0f7a461d3d5 | 7 | partial |
| Register_NewFrame_Callback | entity:06db683cdf90c5c5d5ec08b41210ff1bed4cb8c7dfb5af3a2a4364f2ea2ee630 | 16 | partial |
| Unregister_NewFrame_Callback | entity:7818723452f39074b7ba09a52591c9a6e2392525852c2986e4dfb1c40bc13db8 | 7 | partial |
| Register_Require_With_Init | entity:1370d452b6df4c227e868b9ed16fd8609beb69e8c29235197c1d206716e484fd | 1 | complete |

Source claims `claim:lua-update:blake3:c2b137e878926c7b06db02cf643dd6c8501315d9a3933d01890d4e1ee075cdd6`
and `claim:lua-update:blake3:d3051b0e7bf21c46476fd6453f0f9295fa580db3a72ac8e32e17edb639304288`
establish direct `SetScript("onUpdate", callback)` and
`RemoveScript("onUpdate", same_callback)` with exact function identity.
The global list is separate from the current menu handler. The shared
dispatcher protects individual calls with pcall; list mutation during dispatch
is not established as supported.

Source claims `claim:lua-update:blake3:3e2d79851ed95a9dd27ea284f50a7ae8752290970be3c4cf0137f1c1a5ea0a63`
and `claim:lua-update:blake3:458b3a7d21c0055d772ce87c7bd0d63979a7f2533b7d26667151af7f85ead6d1`
establish the SirNukes same-reference wrapper with no period argument. Neither
wrapper name nor its MD fallback implies every-frame or millisecond delivery.
Loader claim `claim:lua-update:blake3:af0a23d34cc1224b1d7563acb1150a1354cc87776f55379e4153d7092420cf9b`
does not establish immediate late init or deduplicated init queueing.

Runtime evidence is one instrumented serial session, not a controlled
performance comparison. Both candidates delivered during owner-reported
closed menus, clock-corroborated pause/SETA, removal/re-registration and
reconstructed module states. This does not prove native VM recreation.
Claim `claim:lua-update-runtime:aad5a687d77585e11487465bda573a85efa0f9b6fbd4605349d173f76d2e72dd`
retains the support post-reload 109.1339 ms gap and an unmatched interior
observer; no universal faster-than-50-ms or every-frame guarantee follows.
Direct maximum latency is also unbounded by this evidence. Logging overhead,
unsynchronized external intervals and uncalibrated engine clock precision
remain limitations; `GetCurRealTime()` is not a proven subframe budget clock.
Claim `claim:lua-update-runtime:40b1b0da557a7d7fd91364d389b8dc29628c8042804c0b49ae5b8f2997bcbd1a`
preserves unknown support callback-error runtime behavior. Native reentrancy,
general native safety and the original lua_rawgeti fault cause remain unknown.

Assessment: registration/lifecycle source knowledge is sufficient to prepare a
bounded consumer adaptation within the approved Plan 11 remedy. It is not a
performance or safety acceptance. Prefer evaluating the direct API without
adding a support-wrapper dependency; final wiring must follow actual consumer
owners. Preserve the existing time budget and ACK authority, avoid two periodic
collection pumps, protect duplicate init/reentry/stale callbacks, and test
failure recovery plus ordinary diagnostics. Do not mutate callback lists from
within dispatch based on an unproved assumption. Consumer normal/SETA,
reload/load and complete-callback timing evidence remain required.

Execution checkpoint: project-local GSD generation changed definitions at
03:48:24 UTC; the actual owning codex.exe ancestor started at 03:07:14 UTC.
Restart Codex before typed dispatch; no inline implementation bypass, new X4
launch, package installation or runtime acceptance occurred. Memory search
worked, but fetching the previous ACK checkpoint drawer failed with
`TypeError | fetch failed`; historical coverage is partial and primary
evidence is used under the X4 outage policy. No memory write was attempted.

## Global callback consumer remedy (2026-11-04)

Resumed from fresh `9fd11cc` after Codex restart; project-local GSD routing
was current. The GSD base check selected sequential execution in the current
milestone checkout. No branch, model, budget, native ABI or protocol setting
was changed.

Runtime now retains one direct update callback and exact-reference teardown.
MD delivers GameStart/GameLoaded boundaries only, not a periodic second pump.
Fresh state performs no initialization, source or transport work before a
game-scope boundary. Live same-state module replacement may carry that scope
with uncertain lua_reload evidence; an explicitly stopped owner cannot revive
it. Reentry, reconstruction/teardown during dispatch, stale callbacks and
registration/removal errors are covered without assuming native nonreentrancy
or changing the callback list during dispatch. Existing scheduler/ACK logic
and the 2 ms cooperative target remain unchanged.

Two assertion REDs established absent update registration and unintended
pre-game source work. Focused GREEN: scheduler 68, ship detail 37, loader 1,
syntax 21. The real runtime fake-clock test exercises source and retained
delivery yielding, no premature publication and exactly five core/revalidation
reads for its single member, with no repeated getters. Its complete test
callback ceiling is derived from the configured target_fps, not engine-clock
precision or a claimed real X4 measurement.

Independent full-depth review of all eight implementation/procedure files
resolved WR-CB-01: package assertions formerly accepted missing lifecycle
conditions. Assertion RED confirmed that gap; the corrected checker rejects
missing, replaced, extra and malformed triggers in 18 negative fixtures.
Full-task re-review converged with no open BLOCKER/WARNING. The review record
preserves the original finding and disposition.

Final verification after convergence:

- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: exit 0
  after repeating with Cargo-cache access following a sandbox unpack denial.
- `cargo test --locked --workspace -- --test-threads=1`: exit 0. Five
  pre-existing deferred 05.1 named-pipe contracts remain ignored; no new test
  was ignored to obtain green, and those old cases are not claimed as passed.
- `cargo run --locked -p source-size-lint -- crates`: exit 0.
- `pwsh -NoProfile -File extensions/live_galaxy/tests/run_contracts.ps1
  -Suite all`: exit 0, 69.251 seconds; 149 Lua tests, 21 syntax checks,
  XML/schema, actual native lifecycle/full-set/recovery and performance gate.
  Local heavy-gate callback maxima were 5.5091 and 3.6363 ms against
  `1000 / 144` ms, with zero final backlog in both processes.
- Install-guard PowerShell parsing and diff checks passed; a real packaged
  guard invocation follows packaging. Lua mutation is not claimed: no qualified
  existing runner was introduced, and no Rust safety/domain logic changed.

These are offline results, not global engine-callback timing, a causal
throughput comparison, FPS evidence or normal/SETA acceptance. Existing
security/mutation/goal-verification phase gates are not silently closed by
this scoped review. The old MD cadence has been removed from production, but
actual runtime improvement and affected Gate A lifecycle/timing still need
owner evidence. No X4 launch, control, install or save access occurred.

Package checkpoint: source `7a60c7c42fedc5fd758b1e53fc69ac20dbf3af28`, heavy
candidate `dist/live-galaxy-callback-7a60c7c`, manifest SHA-256
`4ec5cf114fb114ed8e1ad87d7a18e20750804864a4b84f209cdff85cfef898cd`.
All 31 manifest files and packaged XML/schema assertions passed. Approved
limits, DLL and bridge hashes remain unchanged; exact identities are in the
current Gate B procedure. The separately rebuilt base clock package passed
`01-install-guard.ps1 -VerifyPackageOnly`. No installation or game run occurred.

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
