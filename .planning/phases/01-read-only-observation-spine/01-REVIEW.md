---
phase: 01-read-only-observation-spine
reviewed: 2026-08-29T10:32:12Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua
  - extensions/live_galaxy/lua/live_galaxy_runtime.lua
  - extensions/live_galaxy/lua/live_galaxy_telemetry.lua
  - extensions/live_galaxy/tests/x4_discovery_contract.lua
  - extensions/live_galaxy/tests/run_contracts.ps1
  - tests/x4-disposable/01-install-guard.ps1
findings:
  critical: 3
  warning: 1
  info: 0
  total: 4
status: issues_found
---

# Phase 1: Code Review Report

**Reviewed:** 2026-08-29T10:32:12Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

The replacement does remove the literal probe frame, but it does not retain the
established failure/reconnect behavior or deliver the required discovered asset,
capacity, and ownership facts. The fake suite is also currently blocked by the
known missing standalone Lua runner, so its assertions are not executable local
evidence.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Discovery failure pins the runtime permanently on the observation slot

**Classification:** BLOCKER (P1)

**File:** `extensions/live_galaxy/lua/live_galaxy_runtime.lua:91-95`

**Issue:** After hello, heartbeat, and runtime-health, `sequence` is `2`, so
`step` is always `3`. If discovery fails, this branch returns before `payload()`
increments `sequence`. Each later scheduler event repeats the failed discovery
attempt at the same sequence and never emits a completion marker, heartbeat, or
fresh health disposition. This contradicts the required "suppress the observation
for that cycle" behavior and creates unbounded retry work during an unavailable
X4 API.

**Fix:** Model the failed observation as a completed scheduler transition: consume
or otherwise advance the observation slot exactly once, emit only a bounded
existing health/diagnostic disposition for that cycle, and continue to the next
heartbeat/complete-marker slot. Add a fake-runtime contract that calls
`runtime.next_payload()` repeatedly after a discovery failure and proves bounded
progress without an observation frame.

### CR-02: The emitted observation discards asset, capacity, and ownership facts

**Classification:** BLOCKER (P1)

**File:** `extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua:79-116`

**Issue:** The adapter chooses one sector, reads metadata and one capacity value,
then returns only `entity_id`, timestamp, version, source, and quality. It never
enumerates an asset and does not preserve owner, cargo/ware capacity, or people
capacity in the returned observation. `serialize_telemetry` consequently emits
only those same fields. A `partial` frame can therefore be accepted even though
no sector asset, capacity, or ownership information reaches the telemetry
consumer, so it cannot meet D-09/OBS-06's required observed facts.

**Fix:** Do not report this path as a successful partial runtime discovery until
the existing normalized observation contract can carry a bounded representation
of sectors, assets, capacity, and ownership. Implement a bounded original
enumeration for each required class, preserve the validated values through the
existing approved schema, and return an explicit unsupported/incomplete result
when any class cannot be represented. Add contracts that assert each required
fact is present in the normalized emitted observation, not merely read by a fake.

### CR-03: The new pipe wrapper breaks the proven reconnect outcome after a retry

**Classification:** BLOCKER (P1)

**File:** `extensions/live_galaxy/lua/live_galaxy_runtime.lua:55-66`

**Issue:** `runtime.emit` treats the second result of
`pcall(pipes.Write_Pipe, ...)` as the final write success. In the installed
`sn_mod_support_apis` implementation, `Write_Pipe` retries after an initial
failure but shadows `call_success` in that retry; it returns the original `false`
even when the retry delivered the frame. This code then returns `pipe_unavailable`
and resets `connected` at line 127, causing a new hello/generation after a frame
the bridge may already have accepted. That changes the previously proven reconnect
semantics and can produce an avoidable duplicate semantic session.

**Fix:** Use the installed API path whose return contract can distinguish a
retry-delivered frame from a failed frame, or make the adapter's retry/result
handling explicit around the documented raw call without adding a second semantic
send. Preserve the same generation when the retried write succeeded. Add a fake
contract for first-write failure followed by retry success and assert no reset or
extra hello is emitted.

## Warnings

### WR-01: Sector selection performs unbounded work before applying the one-section ceiling

**Classification:** WARNING (P2)

**File:** `extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua:59-68`

**Issue:** The one-section result limit is applied only after iterating every
sector returned by the first cluster, calling `stable_id` for each, retaining each
valid candidate, and sorting the whole collection. A large cluster therefore
turns one scheduled observation into unbounded game-thread work and memory,
despite the D-09 one-section ceiling.

**Fix:** Establish an explicit, evidenced input scan bound before calling
`stable_id` (or use an API-provided bounded/paginated enumeration). Stop and
return an explicit partial/unsupported disposition when the bound is reached;
then test the maximum scan and over-limit cases.

## Residual Verification Gaps

- `extensions/live_galaxy/tests/run_contracts.ps1:7-9` still depends solely on a
  machine-global `lua`, which is unavailable here. The D-09 fake-adapter suite
  was attempted and did not execute a case. Per the existing runner-gap report,
  this is recorded as a residual verification gap rather than a new finding;
  no code in scope falsely reports the suite as passed.
- `tests/x4-disposable/01-install-guard.ps1 -VerifyPackageOnly` passed. It is a
  static source/package guard only and cannot establish native X4 discovery or
  the reconnect behavior above.
- Lua LSP reported only the expected X4 runtime globals (`DebugError` and
  `RegisterEvent`) as undefined in the standalone workspace; no semantic Lua
  diagnostic disproved the call paths reviewed here.

## Review Evidence

- Scoped source was read in full, with `ast-index stats/update`, outline, and
  caller queries for the runtime/discovery path.
- Runtime and discovery Lua diagnostics were requested through the Lua LSP.
- Installed `sn_mod_support_apis` source was inspected at its documented
  `Write_Pipe` and `_Write_Pipe_Raw` implementation to verify the retry result
  contract; no installed files were modified.
- `git diff --check` reported no whitespace errors in scope. Remote freshness
  could not be refreshed because the sandbox denied access to `.git/FETCH_HEAD`;
  this review therefore uses the current dirty working tree and preserves all
  unrelated changes.

## Skill Learning

Skill learning: none — the findings are direct instances of existing
`live-galaxy-x4-integration` requirements for bounded game-thread work and
reconnect safety, plus the existing `live-galaxy-x4-tests` requirements for
failure-path contracts. The reviewed evidence does not establish a missing,
reusable project rule.

---

_Reviewed: 2026-08-29T10:32:12Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
