---
name: live-galaxy-x4-tests
description: Test Live Galaxy Lua, Mission Director XML, X4 adapters, packages, and runtime evidence. / Тесты Lua, Mission Director XML, адаптеров, пакетов и X4 runtime Live Galaxy.
---

# Live Galaxy X4 Tests

Use this skill for Lua, Mission Director XML, X4 adapter, package, game-command,
or runtime-integration tests. Read
[`live-galaxy-tests`](../live-galaxy-tests/SKILL.md),
[`live-galaxy-x4-integration`](../live-galaxy-x4-integration/SKILL.md), and the
global `lua` skill first. The common skill owns general test sufficiency,
oracles, doubles, fixtures, diagnostics assertions, and evidence reporting.

## X4-Specific Test Layers

- **XT-01 — Real package paths:** Parse XML and check the manifest, UI
  registration, Mission Director structure, identifiers, entrypoints, and
  generated package claims. Compile shipped Lua with the actual interpreter and
  load real product modules through normal extension-relative `require` paths.
- **XT-02 — Pure Lua:** Keep policy, serialization, normalization, scheduling,
  batching, budgeting, and diff logic independent from X4 globals. Run it with
  deterministic fixtures in the compatible, pinned standalone Lua runner
  selected by current X4 evidence.
- **XT-03 — Adapter contract:** Fake only explicit X4 seams. Exercise
  successful observations plus absent or malformed identities, rejected
  context, and thrown native failures. For commands, check identity, preview or
  state-version validation, idempotency, bounded retry, rejection without
  partial mutation, and independent readback semantics.
- **XT-04 — Cross-language path:** When Lua/MD changes affect Rust boundaries,
  run the actual owned producer/consumer path. A fake adapter proves the local
  seam; it cannot establish a real X4 API or game behavior. When durable
  revision identity must survive process restart, require multiple accepted
  collections before restart and a distinct producer process against the same
  durable database afterward. Assert the exact ordered revision history and
  absence of permanent rejection; a single first-revision sample cannot detect
  process-local version reuse. For recurring bridge-controlled collection,
  also sustain enough same-process cycles to cross the demand interval and
  assert that commits continue on one connection without peer-inactivity
  reconnects. After multiple commits, exercise offline readback for both an
  earlier retained revision and the current revision; checking only current
  cannot prove independently readable history.
  Serialize suites that share a fixed native pipe endpoint and wait for the
  preceding scenario to release it. A `PipeCreationFailed` caused by another
  owned test is an isolation failure, not production evidence; establish the
  result with a clean serial run before claiming regression success.
  For streamed numeric ordinals encoded as string record IDs, cross the first
  decimal-width boundary (9 to 10) in the real producer/receiver path and
  verify the committed IDs in independent readback. A producer-only test does
  not prove receiver completion or durable acceptance.
- **XT-05 — In-game evidence:** Use a disposable Creative Custom campaign or
  approved test copy under a written plan. The user performs all X4 actions.
  Report a scenario as `observed in X4` only after expected behavior and its
  health surface succeed with exact version, mod set, setup, elapsed real/game
  time, SETA state when relevant, and independent readback recorded.

## Runtime and Diagnostic Evidence

- Keep static, pure-Lua, fake-adapter, locally integrated, pending-game, and
  observed-in-X4 evidence separate. No fake or source assertion substitutes for
  a required probe.
- Select X4 probe and SETA soak gates from the project risk matrix: apply a
  probe when local evidence cannot establish runtime behavior and a SETA soak
  when timing, scheduling, lifecycle, recovery, or accelerated-time load
  changes. They are not automatic for every Lua or XML edit.
- Test the relevant semantic diagnostic event identity, reason, correlation,
  and state. Significant-operation history and diagnostic failure policy are
  defined by
  [`code-conventions logging`](../live-galaxy-code-conventions/references/logging.md):
  a runtime log failure must be independently visible, must not stop X4, and
  does not waive durability or idempotency evidence. Do not assert formatted
  lines or unbounded per-frame traces.
- When source collection is fast but end-to-end coverage is slow, measure the
  number of sections, wire batches, acknowledgments, and scheduler pulses before
  changing timeouts or collection deadlines. A per-entity stop-and-wait loop can
  dominate throughput even when every getter is fast. Cross-language load tests
  must assert requested-selection coverage, transaction multiplicity, zero
  final backlog, and independent readback of every member.
- For a logical snapshot with several dependent sections, inject delayed
  acknowledgments and prove that they do not gate later X4 getters. Capture all
  selected source families into owned memory first, then assert that delivery
  retries publish the retained values without repeating source calls.
- Inject one moved, transferred, and disappeared member during final
  revalidation. When the contract permits partial per-record freshness, assert
  that the batch completes, only that record carries the exact stale reason,
  and unchanged members remain authoritative. Apply this oracle to the core
  census as well as dependent detail sections: a whole-batch rejection is not
  an acceptable substitute when the retained record still has a valid identity
  and fields that can be marked stale.
- Do not compare an in-game source-capture duration with the wall-clock duration
  of a synthetic end-to-end fixture. Report capture, serialization, transport,
  durable commit, polling/waits, payload bytes, and cycle count separately. A
  fixture wall clock is correctness evidence unless its component timings and
  workload are comparable to the runtime path.
- FrameView SDK per-frame streams can repeat an identical event several times.
  Preserve raw row counts, but deduplicate metric input by process, swap chain,
  and QPC timestamp. Verify duplicate groups have identical metric fields before
  treating this as transport duplication rather than distinct presents.
- A configured callback-time budget is not evidence of bounded game work by
  itself. Drive a workload that advances the monotonic clock during both source
  capture and retained delivery, assert that `collecting` resumes on later
  callbacks, and bound the measured callback maximum. Also prove that yielding
  does not repeat getters or expose a partially published section. One already
  entered native call remains indivisible and may report a measured overrun.
- For the local heavy-ship performance gate, derive the complete frame budget
  from `target_fps`; never encode its millisecond result as an unexplained
  constant. Time every synchronous source/carrier stage and the complete
  callback. The production callback budget remains the cooperative-yield
  target, while no measured indivisible stage or callback may exceed the full
  target-frame budget. Gate callback-normalized throughput against the recorded
  multi-process baseline and require zero final backlog. This local gate catches
  obvious frame killers but does not replace X4 plus FrameView acceptance.
- Runtime evidence oracles must distinguish a configured terminal safety event
  from an early or unexplained copy of the same diagnostic. For a finite
  admission window, allow `admission_window_exhausted` only after measured
  elapsed time reaches that configured window, at least one revision committed,
  and the bridge recorded no rejected or failed outcome. Keep the same event a
  failure before those conditions are proven.
- Runtime commit and failure counts must aggregate every retained operational
  history segment, including rotated `operational-history.jsonl.*` files. A
  high-throughput SETA run can rotate the journal while remaining healthy; the
  current segment alone is not a complete oracle.

## Lua Mutation Testing

- Mutate only executable pure Lua modules initially. Native X4 adapters and
  Mission Director XML remain outside mutation scoring until a useful harness
  is demonstrated.
- Run a bounded `Universal Mutator` spike against one representative pure module
  and its compatible standalone tests. Pin evaluated tool versions and keep the
  spike out of public runtime dependencies.
- Measure invalid, trivial, equivalent, killed, and surviving mutants before
  making it a required gate. Universal Mutator does not list Lua as supported;
  generic rewriting alone is not acceptance. Do not build a replacement mutation
  engine. If the spike is noisy or incompatible, defer the gate and retain the
  selected unit, contract, and runtime evidence.

Follow the focused suite and final regression commands in
`live-galaxy-x4-integration`; use its tool provisioning path rather than
inventing another runner workflow.
