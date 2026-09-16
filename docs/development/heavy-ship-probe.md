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
pwsh -NoProfile -File tests/heavy-ship-evidence.ps1 -Preparation
pwsh -NoProfile -File tests/heavy-ship-evidence.ps1 -SelfTest
pwsh -NoProfile -File tests/carrier-b-local.ps1 -SelfTest -Scenario heavy-ship-detail
```

`-Preparation` validates source provenance, declaration/detail snapshot separation,
units and the pending result schema. `-RuntimeAcceptance` rejects pending rows.
The default checker remains strict runtime acceptance. Neither Preparation nor
the local actual-chain harness grants owner approval or phase closure.

## Runtime targets still pending

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
