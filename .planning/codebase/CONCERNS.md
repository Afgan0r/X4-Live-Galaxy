---
last_mapped_commit: 80881814c2b3753c50c9edc7cb79818ee7444dde
---

# Codebase Concerns

**Analysis Date:** 2026-09-14

This is a source and recorded-evidence map at the revision above. No product
tests or game probes were run for this mapping. Planned scope, recorded
limitations, and concrete maintenance issues are distinguished below.

## Tech Debt

**Production observation is still a one-getter prototype:**

- Issue: The registered Lua runtime wires `GetCurRealTime` into a single opaque point measurement. It does not connect faction, ship, or station collection to the active callback.
- Files: `extensions/live_galaxy/lua/live_galaxy_observation.lua:15-79`, `extensions/live_galaxy/lua/live_galaxy_runtime.lua:6-9`, `extensions/live_galaxy/lua/live_galaxy_component_discovery.lua:1-184`
- Impact: The installed production path cannot produce the heavy faction-ship observations required by OBS-06, and its `source_epoch_status`, completeness, consistency, and stable identity remain explicitly unknown.
- Fix approach: Promote the heavy source only after exact X4 API and lifecycle evidence is available; derive limits from measured workload rather than reusing the one-record Carrier B values. This is a deferred scope boundary, not evidence of an unsafe collector.

**The X4 persistence cue is a schema placeholder:**

- Issue: The Mission Director cue initializes a static envelope but does not implement save/load, interruption, or reconnect integration.
- Files: `extensions/live_galaxy/md/live_galaxy_persistence.xml:3-15`, `extensions/live_galaxy/checkpoint_schema.json`
- Impact: Local Rust checkpoint tests cannot establish that X4 saves retain, restore, or reject the envelope across the actual game lifecycle.
- Fix approach: Use the planned disposable X4 persistence probe before claiming automatic resume or cross-restart durability; retain the explicit `x4_restart_required` posture until then.

**An obsolete scheduler contract remains in the tree:**

- Issue: `scheduler_contract.lua` calls `sample_slice`, `enqueue_or_backpressure`, and `save_suppressed`, while the current scheduler exposes `tick` and the aggregate runner maps the scheduler suite to `carrier_b_contract.lua`.
- Files: `extensions/live_galaxy/tests/scheduler_contract.lua:8-39`, `extensions/live_galaxy/lua/live_galaxy_scheduler.lua:19-67`, `extensions/live_galaxy/tests/run_contracts.ps1:55-64`
- Impact: The normal suite does not execute these obsolete expectations, including the retired save-suppression behavior. Their presence can mislead future maintenance; it does not prove that the current Carrier B scheduler lacks equivalent relevant coverage.
- Fix approach: Remove or rewrite the stale contract and make the active runner mapping explicit; keep scheduler behavior covered by the current Carrier B contract.

## Known Bugs

No source-confirmed runtime bug was admitted during this mapping. Open verification rows below are recorded as evidence gaps or deferred scope rather than promoted to defects.

## Security Considerations

**Checkpoint integrity uses a non-cryptographic checksum:**

- Risk: `mind-persistence` labels a 64-bit FNV-1a checksum as `integrity_hash`; it detects ordinary accidental changes but does not provide cryptographic collision resistance against a deliberate crafted record.
- Files: `crates/mind-persistence/src/integrity.rs:1-15`, `crates/mind-persistence/src/checkpoint.rs:159-185`
- Current mitigation: Checkpoints bind canonical envelope fields, reject malformed hash formats, enforce a 32 KiB envelope cap, and have tamper-rejection tests.
- Scope: This checksum is not an authentication guarantee. No exploit or requirement for adversarial tamper resistance was established by this mapping; a stronger digest would require a concrete trust-boundary need and the owning phase's decision.

## Performance Bottlenecks

**Heavy-workload cost is not calibrated:**

- Problem: Current Lua and Carrier B limits admit one record and one batch, while the later heavy scheduler policy requires measured permits, debt, step caps, and aggregate behavior.
- Files: `extensions/live_galaxy/lua/live_galaxy_carrier.lua:9-22`, `config/carrier-b-limits.json:1-25`, `docs/architecture-verification.md:124-152`
- Cause: Phase 05.4 evidence covers a one-getter stream and explicitly does not establish shared scheduler behavior under heavy collection or SETA acceleration.
- Improvement path: Measure indivisible getter stages and representative heavy faction workloads, then freeze bounded values and sustained normal-time/SETA evidence in the owning phase.

## Fragile Areas

**Heavy source semantics are not established by the one-getter proof:**

- Files: `extensions/live_galaxy/lua/live_galaxy_component_discovery.lua:52-76`, `extensions/live_galaxy/lua/live_galaxy_observation.lua:22-34`, `docs/architecture-verification.md:154-201`
- Why fragile: A successful getter call or transported record does not prove count/fill ownership, arbitrary-faction membership, stable identity, deletion, or source epoch semantics. These properties are required before treating absence or deletion as authoritative.
- Safe modification: Keep collection on the game/Lua callback boundary and preserve `unknown` quality for unproven fields. Resolve source semantics through the shared Docs MCP/source-research gate first; use a written disposable probe only for residual runtime uncertainty.
- Test coverage: Only the opaque realtime getter has observed runtime evidence; native enumeration and source coverage remain open under VER-LG-003.

**Verification registry and earned runtime scope must be read together:**

- Files: `crates/observation-ingest/src/scheduler.rs:116-183`, `crates/observation-application/src/lib.rs`, `docs/architecture-verification.md:297-369`
- Why fragile: The registry still marks VER-LG-007/008/009 open with older descriptions such as an incomplete observation transaction. The active bridge now composes SQLite publication, and the Phase 05.4 earned-scope section records successful startup orders, load/reload, bridge restart, and durable revisions. Open generic rows alone must not be read as proof that this implemented path is absent or broken.
- Safe modification: Preserve immutable retry bytes, validate before staging, and keep ambiguous outcomes blocked until durable classification; do not infer X4 recovery semantics from fake-adapter evidence.
- Test coverage: Distinguish the recorded one-getter proof from the broader generic crash-cut, multi-section, source, and workload criteria. Verify current phase evidence when a future task depends on an open registry row; mapping did not rerun those scenarios.

## Scaling Limits

**The shipped Carrier B policy is intentionally tiny:**

- Current capacity: `max_candidate_records=1`, `max_candidate_batches=1`, `max_candidates=2`, `max_aggregate_records=2`, and `max_aggregate_bytes=4096`.
- Files: `config/carrier-b-limits.json:1-25`, `extensions/live_galaxy/lua/live_galaxy_carrier.lua:9-22`
- Limit: These values can demonstrate bounded transport but cannot represent a faction-wide ship/detail workload or prove aggregate stream stability.
- Scaling path: Select evidence-backed per-candidate and aggregate bounds within the approved heavy-conformance phase, then verify them across its admitted observation subjects. The one-record values are not defaults for larger collectors.

## Dependencies at Risk

**Release provenance and licensing are incomplete:**

- Risk: The repository has no selected open-source license, so redistribution permission and third-party provenance are unresolved for a public release.
- Files: `README.md:38-42`
- Impact: Packaging a public alpha without these decisions creates legal and provenance ambiguity even if the runtime contracts pass.
- Migration plan: Select a license and complete the third-party provenance review before the 1.0 public-alpha gate; keep 0.x artifacts clearly internal.

## Missing Critical Features

**Heavy faction and hostile-observation conformance is deferred:**

- Problem: The repository has no verified dynamic faction census, arbitrary-owner ship enumeration, or required XEN/KHK ship observation path through X4, Lua, DLL, transport, Rust, and durable publication.
- Files: `docs/architecture-verification.md:433-478`, `.planning/phases/05.5-heavy-faction-ship-conformance/05.5-RESEARCH.md:496-501`
- Blocks: OBS-06 and the heavy observation completeness gate; no Faction Mind or recipient-visibility expansion should be inferred from transport readiness.

**Station completeness and source-delta semantics remain deferred:**

- Problem: Complete-set/deletion semantics for station membership, authoritative source epoch, and an event/delta stream have not been proven.
- Files: `docs/architecture-verification.md:482-548`, `.planning/phases/05.5-heavy-faction-ship-conformance/05.5-RESEARCH.md:64-78`
- Blocks: Treating empty results, ownership changes, or deletions as authoritative observations.

**Correlated X4 reports are a later phase:**

- Problem: The bounded Mail/Logbook return channel, acknowledgement, deduplication, and recipient-information semantics are still a Phase 6 research/planning item.
- Files: `.planning/ROADMAP.md:373-386`, `.planning/STATE.md:222`
- Blocks: Player-visible reporting and end-to-end correlation from observation through report acknowledgement.

## Test Coverage Gaps

**CI does not run the shipped Lua/XML/package conformance suites:**

- What's not tested: The workflow runs Rust and shadow-harness gates, but not `run_contracts.ps1`, XML/persistence schema validation, or package self-test.
- Files: `.github/workflows/phase-05.yml:1-22`, `extensions/live_galaxy/tests/run_contracts.ps1:42-74`, `tools/package-live-galaxy.ps1`
- Risk: A change can pass required CI while breaking Lua registration, Mission Director shape, persistence schema, or package layout.
- Priority: Medium; add these checks when the workflow owner expands CI beyond the Phase 05 deterministic Rust gates.

**General lifecycle and heavy workload evidence remains incomplete:**

- What's not established: Pause/menu/minimize/save behavior, general callback scheduling, native enumeration, heavy detail groups, XEN/KHK coverage, and representative aggregate limits. The recorded Phase 05.4 proof already covers the one-getter path across startup, load/reload, bridge restart, 10 minutes of normal time, and 20 minutes of SETA; it does not establish these wider claims.
- Files: `docs/architecture-verification.md:105-152`, `docs/architecture-verification.md:433-478`, `.planning/phases/05.5-heavy-faction-ship-conformance/05.5-CONTEXT.md:126-139`
- Risk: Local contracts can pass while an X4-specific lifecycle, source, or workload assumption remains wrong.
- Priority: High before treating observation completeness or operational readiness as proven.

---

*Concerns audit: 2026-09-14*
