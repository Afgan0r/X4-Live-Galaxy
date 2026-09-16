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

The initial cohort is ordinary faction `argon`, with complete bounded core
membership and one core-qualified ship per detail group. A whole faction array
over 128 ships is refused, never represented as a complete prefix. This small
cohort does not establish the later heavy-faction Gate A/B cohort.

<!-- markdownlint-disable MD013 -->

| Bound | Proposed value | Local arithmetic or purpose |
| --- | --- | --- |
| Admission window | 60,000 ms | One bridge-run window, not renewed on reconnect |
| Core membership | 128 ships | 128 × 8-byte UniverseID = 1,024 bytes |
| Detail group | 1 ship | Separate cargo, crew and installed-loadout captures |
| Inner collection | 64 entries | Count checked before allocating or filling |
| Per allocation | 8,192 bytes | Count × actual target `ffi.sizeof` must fit |
| Aggregate allocation | 65,536 bytes | Cumulative across nested allocations in one capture |
| Native calls / steps | 384 / 512 | Core's two reads per member need 256 calls; all stages remain bounded |
| Heavy permits | 1 per callback | Getter or allocation, with control pumping first |
| Callback target | 2 ms | Measured after synchronous return; an overrun stops capture |
| Rate interval | 50 ms | Existing Rust scheduler; transport-byte work is charged independently |
| Message / control | 8,192 / 512 bytes | Whole immutable records, no byte-chunk collection |
| Candidate raw / work | 131,072 / 131,072 | Cumulative native and receiver capture ceilings |
| Aggregate decoded / pending / total | 262,144 / 8,192 / 524,288 bytes | One pending slot and bounded staging |
| Capture age / inactivity | 30,000 / 10,000 ms | Distinct total-age and feedback-stall bounds |
| Parent freshness | 30,000 ms | Receiver receipt age and producer-owned monotonic age |
| Attempts / reconnect attempts | 1 / 1 | Incomplete captures require fresh qualification |
| Retention | Current plus 2 unprotected prior receipts per section | Referenced parents, decision pins and ambiguous replay evidence are protected |

<!-- markdownlint-enable MD013 -->

Local fixtures use PeopleInfo=40, RoleTierData=16, software=16, UIWareInfo=24
and UnitData=24 bytes: 64 entries need respectively 2,560, 1,024, 1,024, 1,536
and 1,536 bytes. These are fixture ABI sizes, not packed-layout promises for X4.
Production uses target `ffi.sizeof`; every allocation must satisfy both per-call
and cumulative ceilings. A nested role/tier population can exceed the cumulative
ceiling even when individual counts fit, and is then refused before allocation.

Build and verify locally before any owner-operated installation:

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
After explicit Plan 10 approval, the owner alone installs the exact bundle,
starts a disposable Creative Custom campaign, and runs the finite normal-time
cohort. Do not use player saves. Stop on a source/identity change, frozen or
backward clock, allocation/count/work/byte refusal, feedback stall or overrun.
The owner stops the bridge and disables the experimental extension between
runs; if a call does not return, the owner exits the disposable game session.
Fresh producer qualification is required after load/reload or restart.

Record normal-time and separately approved SETA samples: package/profile/run
identity, real/game time and SETA factor, count/fill cardinalities, target ABI
sizes, getter latency, callback duration, cumulative calls/allocations/work,
record/message bytes, queue/pending occupancy, receipt ages, stop reason and
independent earlier/current readback. Expand faction/cohort/count ceilings only
after reviewing those measurements and explicit owner selection. Missing
cases, all 21 runtime rows and numerical acceptance remain pending.

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
