# Changelog

## 2026-09-22

- Required FrameView SDK metric deduplication by process, swap chain, and QPC
  after a Phase 05.5 capture repeated identical per-frame events 8--10 times.
- Required an executable continuation oracle for configured callback budgets
  after Phase 05.5 proved that the former 2 ms setting only measured a 277 ms
  callback after completion and allowed a large normal-time frame regression.
- Added a no-X4 heavy performance gate derived from the 144 FPS target after the
  same regression showed that correctness and coarse watchdogs could not reject
  an obvious frame-budget violation before installation.
- Required runtime oracles to conditionally classify terminal safety events
  after a valid 60-second capture was rejected solely because its configured
  admission window closed after eight successful commits.
- Required aggregation of rotated operational history after the SETA runner
  reported only 2 current-file commits while 24 durable commits existed across
  two retained journal segments.

## 2026-09-21

- Extended the per-record churn oracle to the core census after an in-game
  Phase 05.5 run discarded roughly 750 otherwise valid ships because one ship
  changed owner during core revalidation.
- Added dependent-snapshot ACK-decoupling and per-record churn oracles after a
  Phase 05.5 runtime run exposed both artificial inter-section callback waits
  and whole-batch rejection when one ship changed location.
- Added the orchestration-multiplicity diagnostic after Phase 05.5 measured
  millisecond source collection but only 39 per-ship detail cycles in 65
  seconds. Cross-language load evidence now checks full requested-selection
  coverage, transaction count, backlog, and independent member readback before
  anyone proposes longer runtime windows. It also separates in-game capture
  time from synthetic fixture wall clock and its transport, commit, polling,
  payload, and cycle components.

## 2026-09-17

- Clarified fixed-pipe test isolation after Phase 05.5 Plan 09's workspace
  regression collided with its running heavy recovery scenario. The clean
  serial regression passed after the owned endpoint was released; see
  `05.5-09-SUMMARY.md` and implementation commit `5e3f45a`.

## 2026-09-10

- Required offline readback of both earlier and current revisions after the
  Phase 05.4 X4 checkpoint exposed a current-pointer-only diagnostic path.
- Added a sustained same-process collection oracle that crosses recurring
  demand intervals and forbids reconnect-driven progress after the Phase 05.4
  demand/inactivity deadline race reproduced locally.

## 2026-09-09

- Required a distinct producer process against the same durable database for
  restart-sensitive revision tests after the Carrier B fixed-version defect
  survived a single-collection local oracle.

## 2026-09-05

- Linked shared test sufficiency to `live-galaxy-tests` and retained X4-specific
  package, Lua, adapter, cross-language, runtime, and mutation obligations.
- Replaced the former normal-operation diagnostic restriction with the shared
  code-conventions logging policy and semantic diagnostic assertions.

## 2026-08-29

- Required bounded opt-in correlated developer diagnostics for multi-hop X4
  runtime probes, including safe cross-hop evidence and first-failure analysis.

## 2026-08-28

- Created the layered Lua, Mission Director, adapter, in-game, and mutation test
  strategy.
- Selected `Busted` for pure Lua tests and `Universal Mutator` for an isolated
  compatibility and mutant-quality spike; Lua mutation remains gated on its
  measured result.
