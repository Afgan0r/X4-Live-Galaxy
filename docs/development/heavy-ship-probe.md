# Heavy ship production probe

Status: source preparation is integrated into approved Plan 05.5-09.
All 21 required X4 runtime outcomes remain pending. Local fixture success does
not establish in-game semantics, completeness, timing or SETA acceptance.

## Implementation and owner boundary

The retained preparation artifacts came from commits `cab157d/64b68b8`.
Obsolete Plan 02 SUMMARY, STATE, halt metadata and zero-envelope advice are not
part of this execution. Source evidence is input to production implementation,
not an entry gate requiring an already-proven native-call latency upper bound.

Plan 09 Task 1 expands the existing carrier with typed cargo, strict copied ABI
arrays, exact receiver-owned core/member dependencies and the real local
Lua -> owned DLL -> named pipe -> bridge -> independently reopened SQLite path.
The supplied X4 source in that harness is explicitly a local fixture.
Crew, actual installed loadout, shared limits and game-runtime composition
remain subsequent tasks in the same approved plan.

A Lua wall-clock check cannot interrupt a native getter. Initial call latency
is a measurement target for a finite experiment. Task 3 supplies the executable
package/profile proposal; Plan 10 obtains owner approval before any game work.
No agent starts, reloads, controls X4 or reads saves in Plan 09.

## Detail-source enrichment

The lead verified six exact searches at limit=5 in snapshot
`x4-9.00-steam-23660954-ship-detail-source-v1`, root
`blake3:4f04a991bd595ff9493dc025a2f636fa64a6c59ed213c0dc1b19d3598b97b518`.
All 28 inspect pages exhausted to null; 24 claims, no conflicts, partial
coverage. See the phase SOURCE-EVIDENCE record for exact entities, queries,
claim IDs and per-claim epistemic classes. Snapshot is game/x4:vanilla at
X4 9.00 / Steam build 23660954; no new owner run is implied.

- Cargo source uses ware-keyed item amounts. StorageInfo capacity/spaceused
  are volume; storage display is m³. Per-ware capacity counts items.
  Lua sum/product precision and ranges remain unknown.
- PeopleInfo supplies roles/counts/numtiers/canhire, not a qualification
  member. GetRoleTiers supplies raw lower thresholds and counts.
  Combined normalization100 and individual display scale0..15 stay separate.
  UI hireable totals are not all-person totals. Runtime raw bounds are unknown.
- Physical/virtual installed slots use indices1..N; software buffers use
  indices0..returned_count-1. Component0 does not erase macro-only slots.
  Thruster representation remains inferred; templates remain excluded.
- UIWareInfo amount is signed int. Cargo quantity dimension is inferred from
  related paths; direct getter semantics, negatives and completeness are unknown.
  AmmoData missile capacity comparisons may use volume, not item count.
- UnitData supplies macro/category/uint32 amount items. Preserve every raw
  category with onlydrones=false; downstream UI exclusions are separate.
- Vanilla/DLC origin inventory has patch/context provenance. Service/mission
  classifications remain inferred; visitor lifetime and campaign availability
  remain unknown. No-economic tags or fixed relations cannot exclude XEN/KHK.

ABI sizes require target ffi.sizeof; do not sum fields or use packed structs.
Copy pointer strings before retaining results. Per-type slot applicability,
crew pilot/arriving policy and all actual outcomes remain pending.

**Do not perform these steps under E0.** First return the source-enrichment
and envelope decision in chat. After source and capture preflight are complete
and a specific finite experiment is explicitly approved:

1. Owner verifies X4 9.00 / Steam build 23660954 and records enabled DLC IDs/
   versions, Live Galaxy package/version/digest and the selected probe revision.
   Enabled stack is vanilla + installed DLC + Live Galaxy only: no Live MCP,
   Carrier A, KUDA, Add More Sectors or other third-party mods.
2. Owner starts a new disposable Creative Custom campaign. Do not access,
   load, copy or inspect player saves. Record a logical run ID, operator=owner,
   monotonic real-time origin, game-time origin and SETA=off.
3. Execute only the approved resolved source calls. Discover faction IDs once
   for taxonomy evidence, using includehidden=false; retain all IDs, including
   ordinary, small/pirate, XEN/KHK, player, mod-added, service, temporary and
   unknown. Attach reviewed vanilla/DLC definition provenance to each
   disposition. GetFactionData name/shortname/primaryrace and zero ships do
   not prove origin/exclusion. Mind participation remains separate.
4. For approved count/fill attempts, record selector, before count, actual
   allocation type/size/capacity, filled count, lossless IDs, after count and
   capture times. Mismatch, duplicate/invalid identity or changed membership
   discards the whole attempt. Even equal counts do not prove atomicity or ABA
   safety. No implicit retry or full admitted-set acceptance.
5. Owner identifies representative ships with cargo/crew/equipment, plus
   docked/nonlocal, zero-capacity, zero crew, empty cargo/missile/unit lists
   and absent slots where genuinely encountered. Capture core independently
   from each detail interval; bind details to exact core revision/member IDs.
   Missing cases stay pending. Do not manufacture them.
6. Owner performs only approved bounded ownership/location change scenarios
   on disposable objects and records before/after identity, source outcome
   and intervals. If a ship disappears from UI, retain the identity and
   classify addressability; UI disappearance alone is not deletion.
   Terminal unavailable getters are inaccessible/absent evidence, not an
   authoritative deletion certificate. Do not deliberately destroy ships
   without separate explicit scenario authorization.
7. Select a deliberately heavy ordinary eligible faction from approved
   measured ship count **and** representative detail cardinality/bytes.
   Return comparison and rationale; a convenient small faction is insufficient.
   If measurements cannot justify selection, leave candidate pending.
   XEN/KHK are mandatory later observation subjects, not ordinary Gate A
   candidates or new minds. No Gate A/B or SETA soak occurs in this source run.
8. Owner ends the approved probe; the agent interprets retained results and
   normalizes only after evidence is returned. Missing applicable rows stay
   blockers and require source alternatives or a precise handoff.

## Local verification

Run the source-preparation checker without declaring runtime acceptance:

```powershell
pwsh -NoProfile -File tests/heavy-ship-evidence.ps1 -Preparation -EvidenceFile tests/fixtures/heavy-ship-source-contract.json
pwsh -NoProfile -File tests/heavy-ship-evidence.ps1 -SelfTest
pwsh -NoProfile -File tests/carrier-b-local.ps1 -SelfTest -Scenario heavy-ship-detail
```

`-Preparation` validates source provenance, declaration/detail snapshot separation,
units and the pending result schema. `-RuntimeAcceptance` rejects pending rows.
The default checker remains strict runtime acceptance. Neither Preparation nor
the local actual-chain harness grants owner approval or phase closure.

## Prepared finite experiment profile

`config/heavy-ship-experiment.json` is proposed experimental policy, not an
accepted X4 performance envelope. The package materializes the exact validated
JSON into `live_galaxy_config.lua` and includes its SHA-256 identity. The bridge
uses the same JSON; unknown, duplicate, zero, overflowing or inconsistent fields
are refused before collection. The clock-only profile remains separate.

The initial subject remains ordinary faction `argon`, with byte-bounded core
membership and one core-qualified ship per detail group. There is no independent
128-ship or 64-inner-entry quota. Allocation, wire bytes, queue capacity and
deadlines can still refuse a capture honestly; no refusal yields a complete
prefix. This prepared profile does not establish Gate A/B acceptance.

<!-- markdownlint-disable MD013 -->

| Bound | Proposed value | Local arithmetic or purpose |
| --- | --- | --- |
| Admission window | 60,000 ms | One bridge-run window, not renewed on reconnect |
| Core membership | Derived from admitted bytes | UniverseID count × 8 must fit the per-allocation envelope |
| Detail group | 1 ship | Separate cargo, crew and installed-loadout captures |
| Inner collection | Derived from the message byte envelope | Compatibility count guards cannot be independently lowered into world quotas |
| Per allocation | 4 MiB | Count × actual target `ffi.sizeof` must fit before allocation |
| Aggregate allocation | 64 MiB | Cumulative across nested allocations in one capture |
| Native calls / steps | 64 MiB work units each | Finite emergency work envelope; the 30-second age guard remains independent |
| Heavy permits | Historical: 1 per callback | Synchronous-first repair removes this experimental multi-call rejection; control pumping remains first |
| Callback target | 2 ms | Owner-approved next experiment: measurement-only; overrun does not cancel successful capture |
| Rate interval | 50 ms | Existing Rust scheduler; transport-byte work is charged independently |
| Message / control | 1 MiB / 512 bytes | Complete semantic records; the local ABI delivered 9.6–12 KiB detail content |
| Candidate raw / work | 64 MiB / 64 MiB work units | Finite emergency envelope, not a measured safe X4 operating threshold |
| Aggregate decoded / pending / total | 128 MiB / 1 MiB / 256 MiB | One pending slot and bounded staging; not measured safe X4 thresholds |
| Capture age / inactivity | 30,000 / 10,000 ms | Distinct total-age and feedback-stall bounds |
| Parent freshness | 30,000 ms | Receiver receipt age and producer-owned monotonic age |
| Attempts / reconnect attempts | 1 / 1 | Incomplete captures require fresh qualification |
| Retention | Current plus 2 unprotected prior receipts per section | Referenced parents, decision pins and ambiguous replay evidence are protected |

<!-- markdownlint-enable MD013 -->

Local fixtures use PeopleInfo=40, RoleTierData=16, software=16, UIWareInfo=24
and UnitData=24 bytes. These are fixture ABI sizes, not packed-layout promises
for X4.
Production uses target `ffi.sizeof`; every allocation must satisfy both per-call
and cumulative ceilings. A nested role/tier population can exceed the cumulative
ceiling even when individual counts fit, and is then refused before allocation.
The existing flat profile retains generic compatibility count fields because
the current carrier/receiver contracts require them. Candidate records/batches
equal candidate raw bytes, aggregate records/batches equal aggregate bytes,
publication records equal publication content bytes, and inner records equal
message bytes. Each accepted row consumes at least one byte; these conservative
derived guards therefore do not add an independent population restriction.
The actual byte/work safeguards remain enforced by each consumer.

The historical `e661f36` package processes owned identities, cargo copies,
storage validation, crew role/tier validation
and merge ordering advance in at most 32 processing operations per callback.
Membership transfers ownership without another full sort/copy. Native count/fill
and immediate copying of borrowed strings are indivisible: neither this helper
nor the timer can preempt them. The DLL's synchronous record copy/validation
and encode/progress remain measured callback work, not preemptible operations.
The existing scheduler pumps feedback first and makes no source admission when
the one-slot downstream queue is unavailable. A busy record handoff retains the
same pending fact and resumes without repeating its getter.

### Synchronous-first follow-up: implemented and locally verified

The initial owner-approved normal-time game run initialized successfully but
refused `ship_core` revision 1 with `collection_overflow`: four native calls,
7,736 allocated bytes, 570 steps and 29,978 ms last-successful-step duration.
The bridge reconnected after repeated 10-second `peer-inactive` deadlines;
independent revision-1 database readback returned `MissingRevision`. The owner
confirmed immediate unpause and no visible freezes. No exact game population,
continuous-sort CPU duration or callback thread topology was established.

The owner then selected synchronous collection of one requested selection in
one callback without artificial operation-count slices. Keep exact owned data,
deterministic membership and parent binding; retain captured data under delivery
backpressure without repeating getters or waiting for ACKs in Lua. Make 2 ms an
experimental timing observation rather than a cancellation guard. Preserve
byte/allocation/work safeguards and all numeric deadlines. The bridge's first
data wait is 30 seconds from each collection demand; subsequent data progress
uses the existing 10-second inactivity bound. No heartbeat/new protocol.

The scoped implementation and focused RED/GREEN tests are complete. Independent
standard-depth review found no open scoped blockers/warnings. Ordinary runtime
diagnostics expose callback duration/overrun and charged represented source bytes
(not exact Lua heap size). Fresh clock/30-second age refusal remains enforced
before native admission. Final relevant local regression passed; these
facts do not establish X4 timing, responsiveness or runtime headroom.

Post-fix gates: 330 Rust tests (five unchanged ignored), workspace formatting,
all-target Clippy, source-size checks, 86 Lua contracts, 20 syntax checks,
XML/schema and serialized clock/native recovery passed. Actual core/detail and
configured restart/interruption paths passed exact independent readback.
Undelayed synthetic first/restart throughput took 5,286.9603/9,570.4167 ms,
with 12 exact revisions, 129 core records and 80 nested entries; measured
callback maxima were 1.4653/1.8998 ms. These are finite local fixtures, not
whole-faction X4 capacity.

Explicit interleave aging (two 16-second waits outside callback measurement)
passed 19 exact revisions in 57,071.2118 ms. Core replacement resumed unfinished
loadout:g0 and later advanced through g5. The history oracle proves one
session/epoch without reconnect; callback maximum 1.7134 ms excludes waits.
The corrected nonterminal diagnostic is `waiting/peer-inactive`, not a false
disconnect. Deliberate aging/recovery elapsed time is not throughput headroom.

Local validation/review and package regeneration passed. On 2026-09-17 the owner
separately approved revised installation and one 60-second normal-time run.
With X4 closed, revised source `b4d6b65` was installed and all 26 extension file
digests verified; the previous package was retained for rollback. The agent has
not started the bridge or game; the owner must load the disposable campaign
before the admission window begins. Plan 10,
Gate A/B, SETA and numerical acceptance remain pending. Retained initial evidence:
logical ID `055-normal-e661f36-20260917-051641`, locator
`%LOCALAPPDATA%/LiveGalaxy/runs/055-normal-e661f36-20260917-051641/locator.json`.

Build and verify locally before installation of the owned experimental package:

<!-- markdownlint-disable MD013 -->

```powershell
pwsh -NoProfile -File tests/carrier-b-local.ps1 -SelfTest -Scenario heavy-ship-recovery -LimitsFile config/heavy-ship-experiment.json
pwsh -NoProfile -File tools/package-live-galaxy.ps1 -SelfTest -LimitsFile config/heavy-ship-experiment.json
pwsh -NoProfile -File tools/package-live-galaxy.ps1 -OutputDirectory dist/live-galaxy-heavy-experiment -LimitsFile config/heavy-ship-experiment.json
```

<!-- markdownlint-enable MD013 -->

The bundle is labelled `prepared-heavy-experiment`, not ready or accepted. Its
startup procedure invokes the packaged bridge with `--ship-faction argon` and
the packaged limits file. Record manifest/source revision, native SHA-256,
profile SHA-256 and bundle manifest SHA-256 before owner approval. Never mix a
different limits file or DLL into that identity.

An already-entered synchronous X4 call has unknown latency and cannot be
interrupted by this timer. The 2-ms target detects an overrun only after return;
the entire call can still freeze a frame. This residual risk is an owner decision,
not a requirement to supply prior game timings before preparing the package.
For the initial experiment approved on 2026-09-17, the agent installs the exact
owned bundle while X4 is closed, preserves a verified backup, checks installed
digests and prepares the external bridge. Other installed mods and vanilla files
remain read-only. The owner alone starts X4, enables Live Galaxy/disables Protected
UI and other third-party extensions, starts a disposable Creative Custom campaign
and performs the approved normal-time actions. Do not use player saves.
The agent starts the bridge only after the disposable campaign is ready: its
60-second experiment window begins with the bridge run, not after menu navigation.
Keep X4 running at normal speed between bridge-only attempts. Do not pause merely
to inspect bridge output or prepare the next connection. Pause only for an
explicit pause-behavior scenario; close X4 before replacing the installed
extension or native library.
Stop on a source/identity change, frozen or
backward clock, allocation/count/work/byte refusal, feedback stall or overrun.
The agent stops its bridge and the owner disables the experimental extension
between runs; if a call does not return, the owner exits the disposable session.
Fresh producer qualification is required after load/reload or restart.

Record normal-time and separately approved SETA samples: package/profile/run
identity, real/game time and SETA factor, count/fill cardinalities, target ABI
sizes, getter latency, callback duration, cumulative calls/allocations/work,
record/message bytes, queue/pending occupancy, receipt ages, stop reason and
independent earlier/current readback. Expand experimental resource envelopes only
after reviewing those measurements and explicit owner selection. Missing
cases, all 21 runtime rows and numerical acceptance remain pending.

## Local preparation throughput evidence

Owner-approved Plan 10 local repair is committed as `67b9b09`, preceded by
intentional RED `25925ea`: a byte-safe 129-identity census previously returned
`collection_overflow` rather than `sampled`. The regression also bounds per-pulse
identity copying and verifies lossless busy handoff. A separate eighty-ware
regression verifies incremental normalization and retry without loss.

The existing actual-chain harness supports a local-only load switch:

<!-- markdownlint-disable MD013 -->

```powershell
pwsh -NoProfile -File tests/carrier-b-local.ps1 -SelfTest -Scenario heavy-ship-recovery -LimitsFile config/heavy-ship-experiment.json -Throughput
```

<!-- markdownlint-enable MD013 -->

It copies the profile into ignored harness storage and changes only that local
copy to callback20ms/rate25ms. Prepared game settings remain callback2ms/rate50ms;
candidate30s, inactivity10s, parent30s and bridge admission60s are unchanged.
The first above-target attempts stopped on the original callback2ms guard,
including revision6 after five commits and 453 getter calls. These are real
calibration risks, not evidence that 2ms is production-safe. No deadline was
extended to obtain success.

The explicitly synthetic target is 128 core records plus one ship's three detail
records with 64 inner entries, within a 30-second test window: 131/30 = 4.367
semantic records/s. This is not an approved X4 refresh interval or measured game
demand. Above-target input uses 129 core records and 80 inner entries. The actual
Lua host, owned release DLL, native ABI, pipe, production bridge and SQLite ran;
only X4 getters and their ABI-size fixtures are synthetic.

These measurements precede the fair-cursor review repair; they establish the
capacity slice, not verification of the later scheduling changes.

<!-- markdownlint-disable MD013 -->

| Measured local result | First producer | Distinct restarted producer |
| --- | --- | --- |
| Exact committed revisions | 1–8, same connection | 9–12, same durable database |
| Semantic records / elapsed | 136 / 24.518s | 132 / 22.357s |
| End-to-end service | 5.547 records/s | 5.904 records/s |
| Margin over synthetic 4.367 records/s demand | 27.0% | 35.2% |
| Callback samples / p95 / max | 1596 / 0.099 / 1.910ms | 1456 / 0.097 / 0.271ms |
| Native push samples / p95 / max | 136 / 0.110 / 0.213ms | 132 / 0.025 / 0.168ms |
| Native progress samples / p95 / max | 2860 / 0.043 / 0.145ms | 2295 / 0.047 / 0.171ms |
| Seal-to-commit samples / p95 / max | 8 / 47.714 / 47.714ms | 4 / 48.071 / 48.071ms |
| Backpressure-paused pulses / final backlog | 332 / 0 | 617 / 0 |
| Core capture / calls / allocation / steps | 11.109s / 263 / 1032B / 464 | 11.093s / 263 / 1032B / 464 |

<!-- markdownlint-enable MD013 -->

These small seal/commit samples use nearest-rank empirical p95; they do not
establish population tails. QPC measures actual callbacks, not an assumed 50ms
pulse. Cargo80 capture took 0.812–0.814s, seven calls, 24 allocated bytes and
50 steps; crew80 roles took 4.466–4.475s, 87 calls, 4480 bytes and 288 steps;
loadout80 software took 0.613–0.620s, 32 calls, 1328 bytes and 37 steps. Native
begin/finish maxima were 0.026/0.011ms. Pipe transfer, receiver validation and
SQLite commit are exercised but not separately timed; seal-to-commit combines
those stages. Stage-specific sustainable capacity remains unproven.

Independent CLI reads reopened earlier/current core revisions1/9 (129 complete
records each), cargo2/10 (80 wares; 9570/9650 content bytes), crew3/11 (80 roles;
4416 bytes), and loadout4/12 (80 software entries; 11998/11999 bytes). No ABI or
wire fragmentation change was needed: complete semantic records above 8KiB fit
the finite 1MiB frame. First/current workload content totals were 75856/40383
bytes, or approximately 3094/1806 content bytes/s. These are independently read
content bytes, not transport-overhead-inclusive throughput.

The bridge previously waited the 5s availability interval after every commit.
The load exposed parent freshness expiry and a 60s watchdog. Heavy collection now
uses its existing configured rate interval for new demand while preserving
pump/reconcile order; legacy availability behavior is unchanged.

This finite slice ends with zero backlog, but is not a long steady-state or
full-faction proof. At the measured 80-role cost, all 129 ships' three groups
cannot fit one 30-second parent window. The review repair resumes the last
durable member identity and unfinished family after replacement cores, rebinding
the ordinal and exact dependencies to current membership. Before new detail
admission, remaining parent age is compared with the observed family duration
plus one configured rate interval. This estimate is not a worst-case native
duration guarantee; an unknown first family may start against a fresh parent.
The observation cost uses receiver wall time from intent issuance to the
matching terminal committed receipt, including collection and persistence.
SectionStart arrives after capture and cannot measure that whole interval.
Canonical game-time capture timestamps remain source evidence, not receiver
duration: accelerated game time must not inflate the proactive refresh estimate.
Stale wire captures receive the existing superseded disposition, discard only
incomplete staging, refresh core and resume that position on the same peer.
Permanent refusal and ambiguous commits remain terminal.

Native detail admission now makes one allocation-free bounded walk across all
nested arrays and strings before owned decoding. Its shared envelope is eight
times the admitted frame bytes: each node reserves 128 bytes of typed/formatting
overhead and strings reserve twice their actual length. Depth is finite. The
1MiB profile therefore supplies an 8MiB admission-work envelope, not an
independent population quota or a measured safe X4 memory threshold. The actual
ABI fixture's individually valid 16 roles × 16 tiers exceed its 32KiB envelope
and are refused before detail copying/serialization; later valid captures and
independent earlier/current readback pass. This added native regression was
executed after implementation and is GREEN-only evidence, not a native RED
claim. The cursor family-loss regression was independently RED before repair.

The post-repair local `-Throughput -Interleave` run committed revisions1–19 on
one producer and peer in 59.362s: 275 semantic records, two complete 129-ship
cores and 17 detail records with 80 inner entries. Core15 replaced core1 after
cargo:g4 revision14; crew:g4 revision16 and loadout:g4 revision17 resumed that
member against exact core/member revision15, then progressed to member g5.
Independent CLI reads reopened every committed revision and checked current
parent identity and dependencies. This covers only the first six members,
not all 129 ships' details. The remaining 0.638s of the unchanged 60s window
is approximately 1.1%: there is essentially no admission-window headroom.

The actual-chain oracle requires its capture/readback revision sequence to equal
the committed sequence exactly. After replacement core it checks the surviving
member's unfinished family (or the successor if absent/completed), then requires
progress to a later member. Executable mutations reject reset-to-g0/family,
missing capture rows and later-member starvation; ordinal binding alone is not
treated as cursor continuity evidence.

Its 3836 callback samples had p95/max 0.159/2.705ms; the maximum exceeds the
prepared 2ms guard. Native push p95/max was 0.133/0.263ms over 275 records;
progress p95/max was 0.058/1.117ms over 6977 calls. Seal-to-commit p95/max was
48.598/48.598ms over 19 commits; 695 pulses paused under backpressure and
the final backlog was zero. Those observations do not establish steady backlog,
full-faction refresh capacity, separate receiver/SQLite capacity, or X4 safety.

Actual required X4 flow/headroom, full-faction eventual coverage, non-preemptible
fill/copy timing and game responsiveness remain unproven. Gate A and all owner
pauses remain pending; neither capacity ceilings nor the relaxed local callback
profile resolve those obligations.

### Final post-review local verification

The final synthetic restart uses callback20ms/rate25ms, not the prepared
callback2ms/rate50ms profile. Its first 8 commits (136 semantic records)
took 24.722s: 1599 callback samples, p95/max 0.163/0.654ms. Its distinct
producer restart committed 4 more (132 records) in 22.526s: 1455 samples,
p95/max 0.198/10.764ms. The **10.764ms callback** is the latest observed
maximum and exceeds the prepared 2ms guard. A timer detects the violation
after return; it cannot preempt an indivisible callback or native call.
Prepared-profile fixture success does not certify that latency limit in X4.
Seal-to-commit maxima were 47.171/50.292ms; backpressure paused 335/616
pulses and both final backlogs were zero. All 12 complete revisions reopened;
replacement core9 resumed crew:g2/loadout:g2 then progressed to cargo:g3.

The final same-peer interleave committed all 19 revisions in 59.473s,
275 semantic records (two 129-record cores and 17 details). It retained only
0.527s, approximately 0.9%, of the unchanged 60s admission window: required
headroom is not demonstrated. Callback n/p95/max was 3842/0.181/1.411ms;
native push was 275/0.152/0.570ms, progress 6983/0.058/0.814ms, and
seal-to-commit 19/48.493/48.493ms. There were 701 backpressure pauses and
zero final backlog. Complete independent reads verified every revision,
core15 exact dependencies, unfinished crew:g4/loadout:g4 and subsequent g5.
This remains a capacity slice, not full-faction or separate-stage headroom.

After the clock repair converged, formatting, workspace all-target Clippy,
325 executed Rust tests, the 200-line source check, 80 Lua contracts,
20 Lua syntax checks, XML/schema and serial native clock/recovery scenarios
passed. Five existing Phase05.1 station-publication tests remain ignored for
Phase05.3 reconciliation; they are not ABI harness cases. Separate heavy ABI
core/detail checks and the exact prepared2ms/50ms first/restart plus all four
interrupted-family recovery checks passed. Oracle mutations rejected cursor
reset, missing capture and later-member starvation.

An earlier pre-clock full pass was discarded as final evidence. One synthetic
launch overlapped the still-running prepared interruption matrix and refused
pipe opening; that orchestration error is excluded from product measurements.
The measured synthetic rerun and interleave began only after predecessor exit
and process cleanup. No game operation or runtime acceptance occurred.

Retain private run evidence outside Git in the shared contract's durable
machine-local artifact store, with owner-only permissions and a stable locator
containing logical run ID, exact retained paths and verified digests. Public
handoffs name the logical ID and locator, not raw game payloads or local paths.
Verify retained files and digests before ending the checkpoint.

## Pending runtime interpretation

Preserve count/fill observation strength as Partial/Unknown; a committed transfer
is not source completeness or deletion proof. Target checks include actual
empty/absent/inaccessible/unsupported distinctions, cargo storage dimensions,
crew role/pilot selectors, signed tier and missile values, macro-only slots,
virtual thruster applicability, all raw unit categories, owner/source changes,
native-call latency, callback pump ordering, SETA load and durable restart.

No pointer lifetime guarantee is inferred from vanilla copy-before-yield
precedents. Copy returned strings synchronously and record observed/inferred/
unknown facts separately. Exact ABI size and alignment come from target
`ffi.sizeof`, never a packed struct guess. Plans 10/11 record the actual game
outcomes against the retained source contract and finite package identity.

## Repeatable runtime-window and frame-capture repair

The first focused loaded frame run is valid normal-time evidence for package
source `658def3`. Logical runtime ID
`055-gate-a-fps-focused-658def3-20260920-200209` committed 35 revisions with
no rejected or failed outcome. Logical capture ID
`055-fps-loaded-focused-20260920-200209` measured average FPS -0.627%, 1% low
-1.173%, 0.1% low -6.525% and p99 frame time +4.205% against the clean
baseline; no frame was at least 33.333 ms. The owner observed no freeze or
stutter. The separate unfocused control
`055-fps-unfocused-control-20260920-200500` captured 4,878 frames in 14.989
seconds, proving that PresentMon collection does not require X4 focus or owner
interaction. These results do not establish SETA acceptance or numerical Gate A.

Two attempted SETA runs are explicitly invalid: the first never had SETA
enabled, and the second was stopped after 0.887 seconds and four commits. They
must not be used as performance or stability evidence.

The retry failure exposed a lifecycle mismatch. Rust correctly creates one
finite admission window per bridge process, but the Lua module retained its
expired `run_started` value for the lifetime of X4. A transition into a new
bridge compatibility qualification now discards old selection state and lets
the next ready selection start a fresh Lua window. Ordinary polls, transport
retry and reconnect do not renew either window. The 60-second bridge-run limit
is unchanged.

`tools/run-heavy-ship-runtime.ps1` owns the timed measurement lifecycle. In
SETA mode it waits for a measured 4x--8x game-time factor before starting the
bridge and PresentMon, runs the full requested interval independently of chat,
stops only its own child processes and rejects missing frames, missing commits,
failed/rejected outcomes, lost PresentMon events or insufficient SETA evidence.
It publishes machine-readable status and result files under the private
evidence root. This removes chat timing and repeated X4 restarts from the
measurement protocol; it does not operate X4 or enable SETA for the owner.

The rebuilt installed package records source revision
`0869e1aae3afd1cc97cdb8c0b4749b0af01fb01d`; manifest SHA-256:
`bbd9d051a472d57991e7d2f1b33c6d7e2a6cc582d8bfb1d19803cc1fb5e30faa`.
All 29 package entries and all 26 installed extension files were independently
rehashed with zero mismatches while X4 was closed. Rollback artifact ID:
`05.5-0869e1a-20260920-204804-35070262`; its owner-only locator SHA-256 is
`ffe7eb9f8fa1771ea6d6a225cacc3af201265670bc58e85d21ec1d7291cc49d9`.
The next valid measurement may start from SETA in the already-loaded disposable
campaign; restarting X4 between bridge-run windows is no longer part of the
procedure.

## SETA capture and core-change recovery repair

Logical run `055-gate-a-seta-20260920-230703` completed a 65-second capture at
an observed 5.991x game-time factor. It produced 7,362 frames at 113.290 average
FPS, with 54.678 1% low, 41.755 0.1% low, 11.835 ms p95, 15.959 ms p99,
20.403 ms p99.9 and 37.065 ms maximum frame time. Fifty-six frames were at
least 16.667 ms, one was at least 33.333 ms, none was at least 50 ms, and
PresentMon reported no lost events. SETA changes game workload, so these values
must not be compared directly with the normal-time baseline as mod overhead.

The bridge durably committed 25 revisions in 4.282 seconds: one 971-record
`ship_core` revision and cargo, crew and loadout revisions for groups g0 through
g7. Independent reopening verified all 25 revisions and 995 semantic records
with no failed read. The complete event span was 64.633 seconds.

This run is not a Gate-A pass. X4 subsequently reported `core_changed` while
revalidating `ship_cargo:g8` at revision 26. The rejection was correct and no
invalid detail was published, but the native producer stayed paused instead of
accepting a fresh core intent. The original runner also inspected only bridge
history and therefore incorrectly labelled the window successful despite the
Lua diagnostic.

The recovery repair treats `core_changed` like `stale_parent`: it discards the
incomplete section, retains fail-closed publication behavior and permits only a
fresh `ship_core` intent. The runner now captures privacy-safe runtime
`event:detail` diagnostics for the measured interval and fails on every pair
except `transition:sampled` and `initialized:initialized`. Its self-test proves
that `transition:core_changed` fails the verdict. No deadline, retry count,
protocol or collection limit changed.

Runtime locator SHA-256:
`d6faa60c5ce5098b9ec9cc5fbcb60d3192a08bc973eea00849c6069aefb6a468`.
Frame-capture locator SHA-256:
`48055139405a852e58b5608fb02309d4f76e0a7de22a56867aae5a7ee8c0a98b`.
All seven retained runtime files and all three retained frame files were
re-read with zero digest mismatches under owner-only ACLs. A revised installed
package and a fresh SETA run remain required; numerical acceptance, Gate A/B
and Plan 10 remain pending.

The repair is commit `915f9f3`. Its package records source revision
`915f9f35b1ae13c8ab3384c8f842eb28df3a6c0f`; manifest SHA-256:
`19b9a8bc2b4a47c0f5b7a0e37393babcf407aa360ec18725a713bdb055bca439`.
All 29 package entries passed independent digest verification. The final
regression passed the full Rust workspace, strict all-target Clippy, source-size
enforcement, all native ABI tests, all seven actual Carrier B lifecycle
scenarios, 99 Lua contracts, 20 Lua syntax checks, XML validation and
persistence schema validation. Fresh scoped review found no remaining P0--P2
finding.

With X4 closed, all 26 installed extension files were backed up, atomically
replaced and rehashed against the manifest; all 26 backup digests were also
re-read. Rollback artifact ID:
`05.5-915f9f3-20260920-233925-5cc05048`. Locator SHA-256:
`026f7419579dda3ef0684b89560640f8eec516094b256eadaa3023d429313a4a`.
The rollback directory has protected owner-only ACLs. One earlier ACL setup
attempt stopped before copying any file; its empty directory was verified and
removed. The revised installed package is ready for a replacement SETA run;
Gate A remains pending until that run succeeds.

### Full-selection throughput correction

The subsequent clean runtime evidence showed that source collection was not the
bottleneck: a roughly 750-record core completed in 9--13 ms, but the configured
`group_members = 1` path created three stop-and-wait sections per ship and
advanced only 39 ships in 65 seconds. Extending the run window would have hidden
that orchestration defect.

The repaired experimental profile derives detail membership from the full
accepted core rather than imposing a one-ship group. Cargo, crew, and loadout
remain separate dependent sections; the existing one-megabyte wire limit splits
each section into immutable batches as needed. Local actual-DLL/pipe/SQLite
evidence collected 129 ships with 80 nested rows per family. Two complete
core/cargo/crew/loadout cycles committed in 2.015 seconds with zero final
backlog; a restarted producer committed the next cycle in 6.003 seconds. Those
wall-clock durations include synthetic capture, serialization, IPC
acknowledgments, durable commits, and polling. They are correctness evidence,
not a throughput comparison with the 9--13 ms X4 core-only capture. Independent
readback returned 129 records for every detail revision, including 1,547,870
bytes of loadout content in the largest observed section. The largest synthetic
synchronous callback was 143.163 ms for crew. These local measurements do not
establish X4 throughput or frame safety. A rebuilt installed package and one
replacement in-game run remain required; Gate A is still pending.

### Full-selection package installation

Commit `2303865` was pushed and packaged from source revision
`2303865c37ac3a137997eee0f07fbd1b5518383b`. Manifest SHA-256:
`3e71daacc4e5f4f4779af362a9afc80fe790c3848a7206e6d5377c134f2c6c2d`.
With X4 closed, all 26 extension files were staged, verified, atomically
installed, and independently rehashed with zero mismatches.

Rollback artifact ID: `05.5-2303865-20260921-110425-ff6f9931`. Stable
owner-only locator:
`%LOCALAPPDATA%/LiveGalaxy/install-backups/05.5-2303865-20260921-110425-ff6f9931/locator.json`.
Locator SHA-256:
`2d3da4b7bbdfe39f2b9716b4de9d561469b09251c028972a8536ae76cd27bd5c`.
The replacement runtime run must report per-family X4 capture time and separate
end-to-end transport/commit time; synthetic fixture wall clock is not the
throughput baseline.

### Full-snapshot ACK decoupling and churn correction

The replacement normal-time run measured approximately 9 ms for core, 18 ms
for cargo, 76 ms for crew and 196 ms for loadout. Their successful source span
was nevertheless approximately 1.46 seconds because each family waited for a
durable ACK and another telemetry callback. A later loadout attempt discarded
the complete roughly 750-ship batch when one ship at ordinal 643 changed
location, then waited for parent expiry before refreshing core.

The corrected path captures core, cargo, crew and loadout into one owned Lua
snapshot during the core callback. Existing section envelopes are then drained
from that snapshot without another X4 getter and without the bridge's former
rate-interval sleep. Wire byte limits, backpressure, durable acknowledgment,
reconciliation and the finite admission window remain active.

Final revalidation now marks only the affected record as `possibly_stale` with
`location_changed`, `owner_changed`, `missing` or `core_changed`. The batch and
unchanged records remain admissible, while the receiver rejects malformed
consistency/reason pairs.

Local RED/GREEN evidence covers 950 ships in one four-family callback, cached
delivery without repeated getters, per-record churn, actual DLL/pipe/SQLite
publication, restart recovery and independent historical/current readback. A
129-ship, 80-nested-row synthetic run completed two snapshots with zero final
backlog; the first process took 1.910 seconds end to end and its largest source
callback was 207.366 ms. Those figures are synthetic load evidence. The next
installed X4 run must measure the real combined callback and frame impact;
Gate A remains pending.

### Full-snapshot replacement installation

Commit `f6eb065` was pushed and packaged from source revision
`f6eb0650b523a38500c14d4bcb773ac4bcdad09f`. Manifest SHA-256:
`f8220688f6e8a9e0ff106fbf1374ceea48b6308534ebce93cdc67189bd538d8a`.
The package self-test passed before installation.

With X4 closed, all 26 extension files were staged, atomically installed and
rehashed against the manifest with zero mismatches. All 26 files from the prior
installation were retained and re-read in the protected rollback artifact
`05.5-f6eb065-20260921-070127-b4b0f963`. Stable owner-only locator:
`%LOCALAPPDATA%/LiveGalaxy/install-backups/05.5-f6eb065-20260921-070127-b4b0f963/locator.json`.
Locator SHA-256:
`e8c872c697d5cf75cf6a3bf78e8d04d1879319281b3be4e980576067a824fd31`.
The next evidence run uses normal time and SETA in one X4 session; no reinstall
or second restart is required between them.

## Core churn and FrameView SDK CLI correction

Normal-time run `055-gate-a-normal-sdk-20260921-214108` captured 115,423 X4
frames in seven FrameView SDK segments over 70.953 seconds. It measured
278.645 average FPS, 126.081 1% low, 78.323 0.1% low, 6.447 ms p99 and
29.713 ms maximum frame time, with no frame at least 33.333 ms. This trace is
not Gate-A evidence because the bridge committed no revision.

The exact runtime failure was a ship owner change at core ordinal 16, revision
95. Whole-core revalidation discarded the approximately 750-record capture and
retried stale data until exhaustion. Core collection now retains valid fields,
marks only that record `possibly_stale` with an exact reason, and continues the
batch. Initial identity mismatch and malformed facts still fail closed. The
strict consistency pair is preserved through Lua, native ABI, wire content and
receiver validation.

For this machine, use
`C:/Program Files/NVIDIA Corporation/FrameViewSDK/SDK/FvSDKTestClient_Public.exe`
as `-PresentMonPath`. The runner recognizes that executable, launches repeated
`--test_case 6` ten-second segments in the capture directory, merges the
per-frame CSV files, and filters them to `-X4ProcessId`. The legacy PresentMon
arguments remain available for a compatible standalone binary. The SDK CLI
does not require X4 focus or manual overlay interaction.

The 28 private files for this failed acceptance attempt are retained under an
owner-only locator:
`%LOCALAPPDATA%/LiveGalaxy/evidence-locators/055-gate-a-normal-sdk-20260921-214108/locator.json`.
Locator SHA-256:
`4b2d5e5c5fd832104906a055cb36149c469f31108f2bba63b8fac062b7fe5b6a`.
