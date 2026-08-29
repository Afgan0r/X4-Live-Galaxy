---
status: diagnosed
trigger: "Establish why named-pipe telemetry works in a disposable X4 session but D-09 runtime discovery remains unproven when the accepted observation is sector:live_galaxy. Diagnose only."
created: 2026-08-29T00:00:00+07:00
updated: 2026-08-29T00:00:00+07:00
goal: find_root_cause_only
---

## Current Focus

hypothesis: "D-09 is unproven because the X4-loaded event handler emits a hard-coded probe observation from `runtime.next_payload()` and never invokes the existing adapter-based discovery functions in `live_galaxy_telemetry.lua`."
test: "Trace the X4 event handler's payload source, compare its accepted runtime frame with the ledger, and inspect the only discovery module's call contract."
expecting: "If the hypothesis is true, the runtime source contains fixed observation values and no dependency or call path to `telemetry.observe_runtime_scope()` or `telemetry.produce_observation()`."
next_action: "Route a narrowly scoped Phase 1 gap-closure plan that connects a read-only X4 discovery adapter to the telemetry transport and then earns an isolated OBS-X4-04 ledger row."
bug_class: Bohrbug (deterministic implementation gap)
known_pattern_candidate: none; no debug knowledge base exists.
reasoning_checkpoint:
  hypothesis: "The X4 runtime sends a synthetic observation instead of a discovery-derived observation because `runtime.next_payload()` constructs the literal frame directly and has no call path to the telemetry module's adapter-based discovery API."
  confirming_evidence:
    - "`live_galaxy_runtime.lua` constructs `entity_id=sector:live_galaxy`, `observed_at_unix_millis=1`, `quality=unknown`, and `content=runtime_probe` in the observation payload."
    - "The observed X4 trace reports precisely `sector:live_galaxy`; the ledger explicitly calls it a synthetic fixed observation and retains OBS-X4-04 as pending."
    - "`live_galaxy_telemetry.lua` exposes discovery only through an injected `adapter:list_scope(scope, limit)` and the runtime module neither requires nor calls that module."
  falsification_test: "A source trace showing the registered `live_galaxy_observation` handler invokes a real X4 adapter, and a disposable trace showing accepted entities/timestamps originate from that adapter rather than the fixed literals, would falsify this diagnosis."
  fix_rationale: "Replacing only the observation-production seam with a bounded, read-only X4 adapter makes the existing transport carry discovered data; it does not alter transport framing, bridge admission, or add any return/mutation capability."
  blind_spots: "The exact X4 API calls needed to enumerate sectors, assets, capacity, and ownership have not been demonstrated in X4. That is a remediation dependency, not evidence that discovery is currently wired."
  candidate_causes:
    - "code: the registered runtime handler bypasses `live_galaxy_telemetry.lua` and emits a fixture payload."
    - "data: all observation identity/time/content values in the emitted frame are fixed probe literals, so accepted transport data has no authoritative runtime provenance."
    - "environment: the installed X4 enumeration API and its per-frame cost are still unverified in a disposable session."
  and_gate: "yes — D-09 requires both a code path to a read-only X4 adapter and runtime evidence that the adapter's data is actually accepted; either alone is insufficient."

## Symptoms

expected: "D-09 requires sectors, assets, capacity, and ownership to be discovered from X4 state rather than a fixed map or job count."
actual: "The active disposable X4 session sends accepted named-pipe frames, including `sector:live_galaxy`, but that identity is a synthetic fixed probe value. OBS-X4-04 remains pending."
errors: "No transport error establishes a discovery failure. The evidence gap is deterministic: the emitted payload does not originate from runtime discovery."
reproduction: "Run the registered `live_galaxy_observation` event in the current extension; every observation cycle follows `hello`, `heartbeat`, `runtime_health`, fixed `observation`, and `complete_marker`."
started: "Present in the currently inspected Phase 1 runtime implementation and recorded by the 2026-08-29 disposable transport trace."

## Eliminated

- hypothesis: "Named-pipe transport failure prevents D-09 evidence."
  evidence: "The ledger records accepted hello, heartbeat, runtime-health, observation, and completion-marker frames in a disposable X4 session, plus a compatible bridge restart while X4 remained running."
  timestamp: 2026-08-29T00:00:00+07:00

## Evidence

- timestamp: 2026-08-29T00:00:00+07:00
  checked: "Git freshness"
  found: "The local branch is `codex/0.1-shadow-director`, ahead of origin by three commits and dirty with user/other-agent changes. `git fetch --prune origin` completed after an initial sandbox permission failure. No pull was attempted."
  implication: "This diagnosis uses the current dirty worktree as the active runtime evidence and preserves all pre-existing changes."

- timestamp: 2026-08-29T00:00:00+07:00
  checked: "`tests/x4-disposable/01-probe-evidence.md`, Observed Runtime Transport Trace"
  found: "A disposable X4 9.00 session accepted the bounded transport sequence and compatible bridge reconnect, but recorded the observation as `sector:live_galaxy`; formal OBS-X4-01 through OBS-X4-04 remain pending."
  implication: "Observed-in-X4 transport proof is not proof of scope-complete runtime enumeration."

- timestamp: 2026-08-29T00:00:00+07:00
  checked: "`extensions/live_galaxy/lua/live_galaxy_runtime.lua`, `runtime.next_payload()` and registered event handler"
  found: "The fourth payload step is a literal JSON observation with `entity_id` `sector:live_galaxy`, timestamp `1`, quality `unknown`, and content `runtime_probe`. The handler emits this returned value directly through `runtime.emit()`."
  implication: "The accepted runtime frame is a deterministic fixture, not discovered X4 state."

- timestamp: 2026-08-29T00:00:00+07:00
  checked: "`extensions/live_galaxy/lua/live_galaxy_telemetry.lua` and `live_galaxy_normalize.lua`"
  found: "Discovery is represented separately by `telemetry.observe_runtime_scope(adapter, scope, limit)`, which requires `adapter:list_scope`; serialization requires normalized values from source `x4_runtime`. The runtime module has no require/call path to either function."
  implication: "A testable discovery seam exists, but it is disconnected from the X4-loaded producer."

- timestamp: 2026-08-29T00:00:00+07:00
  checked: "Phase contract: D-09, OBS-06, 01-09 plan, and the evidence ledger"
  found: "D-09 and OBS-06 require discovery of sectors, assets, capacity, and ownership from observed state. Plan 01-09 makes the human OBS-X4-04 attempt the required proof and says Phase 1 cannot close until qualifying observations exist."
  implication: "This is a Phase 1 closure gap, not a transport regression or a later-phase feature."

## Resolution

root_cause: "Confirmed: `extensions/live_galaxy/lua/live_galaxy_runtime.lua` is a transport probe, not a runtime-discovery producer. Its registered X4 event handler emits a literal synthetic observation and bypasses the adapter-based discovery/normalization seam, so named-pipe acceptance cannot establish D-09 or OBS-06."
fix: "Not applied (diagnose-only). Create one bounded Phase 1 gap-closure plan: implement a thin read-only X4 discovery adapter and have the existing telemetry producer emit normalized, bounded sections through the already-proven transport; retain all transport/reconnect behavior and prohibit any return or mutation vocabulary."
verification: "Required acceptance evidence: (1) focused pure-Lua and fake-adapter tests prove bounded enumeration, stable ordering, normalization failures, and no fixed fixture fallback; (2) static/package checks prove the X4 event-to-adapter-to-transport path; (3) one fresh isolated disposable Creative Custom OBS-X4-04 attempt records exact X4/mod/protocol/scenario/time/SETA setup, expected and observed discovered sector, asset, capacity, and ownership identities, bounded health, and correlated diagnostics; (4) evidence remains `pending` or `failed` if any required class is unavailable rather than fabricating it."
files_changed: []

## Root-Cause Report

### Claim classification

| Claim | Classification | Direct evidence |
| --- | --- | --- |
| Named-pipe telemetry and compatible bridge restart work in the disposable X4 session. | Observed | `tests/x4-disposable/01-probe-evidence.md`, Observed Runtime Transport Trace. |
| The accepted `sector:live_galaxy` frame is synthetic. | Observed | The same ledger identifies it as synthetic; `live_galaxy_runtime.lua` constructs the exact literal. |
| The current handler does not perform discovery before sending that frame. | Documented | `extensions/live_galaxy/lua/live_galaxy_runtime.lua`: `runtime.next_payload()` returns the fixed string directly to `runtime.emit()`. |
| A bounded adapter seam exists but is disconnected. | Documented | `extensions/live_galaxy/lua/live_galaxy_telemetry.lua` and `live_galaxy_normalize.lua`. |
| The exact native X4 enumeration calls and runtime cost can produce all D-09 classes. | Unknown | No qualifying OBS-X4-04 attempt exists. |
| Connecting a thin adapter is the smallest remediation that can close D-09. | Inferred | It changes the sole broken source seam while preserving the proven pipe and no-mutation boundary. |

### Smallest GSD-routable remediation

Add a single autonomous Phase 1 gap-closure plan before the existing Plan 01-09 human checkpoint. Its file ownership should be limited to the X4 Lua discovery adapter, the producer integration, focused Lua/fake-adapter tests, static/package checks, and the preflight portion of the existing disposable procedure. Do not modify bridge protocol semantics, strategic code, reports, persistence, saves, or installed X4 files.

The plan must preserve one bounded section per cycle, stable typed identities, explicit `unknown`/`unsupported`/`partial` states, and failure without a synthetic fallback. It must emit only the existing telemetry framing and bounded health diagnostics.

### Dependencies and restart boundary

- **Dependency — documented/unknown:** establish the installed X4 9.00 read-only APIs that can enumerate the required sectors, assets, capacity, and ownership, and bound their game-thread work.
- **Dependency — documented:** implement/verify a thin adapter conforming to `adapter:list_scope` and `adapter:read_observation`; keep pure policy out of X4 globals.
- **Dependency — observed:** reuse the existing named-pipe transport and compatible bridge-restart behavior; no new bridge capability is required.
- **X4 restart:** **yes.** Installing or replacing the Lua extension is a game-facing code change, so X4 must be closed and restarted under D-05. A compatible external bridge restart after that installation can still be tested without restarting X4.

Skill learning: none — the fixed-probe-versus-runtime-discovery distinction is already covered by the integration skill's authority/metadata requirements and the X4-tests skill's separate in-game evidence rule; one confirmed gap does not establish a new reusable rule.
