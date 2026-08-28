# Phase 2 Validation Matrix

## Purpose

This matrix defines deterministic, read-only validation for the Phase 2 XEN/KHK evidence artifact. It proves record structure and scope boundaries from repository artifacts; it does not claim runtime X4 behavior.

## Evidence Levels

| Level | Meaning | Phase 2 handling |
| --- | --- | --- |
| Verified static installation evidence | Versioned installed X4 or extension source is recorded in a structured claim. | Required for the completed artifact. |
| Pending disposable-runtime evidence | An X4 runtime fact needs an attributable disposable campaign probe and independent readback. | Explicitly non-gating; tracked in the claim register and research deferral table. |

## Requirement and Decision Mapping

| Source | Invariant ID | Exact verifier assertion | Automated command | Evidence level | Runtime/manual deferral | Sampling cadence |
| --- | --- | --- | --- | --- | --- | --- |
| RES-01 | INV-RES-01-CATEGORY-COVERAGE | `claims` contains every allowed area for both `XEN` and `KHK`. | `powershell -NoProfile -File tools/verify_xen_khk_evidence.ps1 -EvidencePath .planning/phases/02-hostile-faction-research-track/02-XEN-KHK-EVIDENCE.md -Stage full` | Verified static installation evidence plus pending runtime evidence | Runtime identity, event, and visibility semantics remain non-gating pending claims. | Every artifact change and before Phase 8 inventory. |
| RES-02 | INV-RES-02-NON-GATING | Forbidden positive scope booleans are false, `phase8_inventory_only` is true, and deferred claims carry an owner and non-gating value. | Same command; assertions `scope.*` and deferred claim fields. | Verified repository-scope evidence | No runtime action is required. | Every artifact change and before Phase 8 inventory. |
| RES-03 | INV-RES-03-CLAIM-INTEGRITY | Each row has exactly one enum classification, a valid `source_id`, material source boundary, and nonempty permitted conclusion within that source's declared scope. | Same command; assertions `classification`, `source_id`, source registry, and required fields. | Verified repository-structure evidence | Manual source review occurs when an installed source changes. | Every artifact change; resample after X4 or extension update. |
| D-01 | INV-D-01-RESEARCH-SURFACE | Both factions cover the seven allowed areas. | Same command; faction-area coverage assertion. | Verified repository-structure evidence | Runtime rows stay pending where static source does not prove behavior. | Every artifact change. |
| D-02 | INV-D-02-OBSERVATION-ROLE | `scope.xen_primary_pressure` and `scope.khk_observed_when_present` are true. | Same command; exact scope assertions. | Documented scope evidence | No runtime action is required. | Every artifact change. |
| D-03 | INV-D-03-NO-HOSTILE-GOVERNANCE | All forbidden hostile autonomy/government/motive/diplomacy fields are false. | Same command; exact forbidden scope-flag assertions. | Documented scope evidence | Later milestone product decision only. | Every artifact change and before Phase 8 inventory. |
| D-04 | INV-D-04-ONE-CLASSIFICATION | Every `classification` equals one and only one allowlisted enum member. | Same command; classification-enum assertion. | Verified repository-structure evidence | No runtime action is required. | Every artifact change. |
| D-05 | INV-D-05-SOURCE-BOUNDARY | Every claim resolves to one allowlisted registry source with an approved kind, path, and version boundary. | Same command; source-registry assertion rejects unknown source IDs and duplicate or unallowlisted registry entries. | Verified static installation evidence | Disposable runtime observation is separately labeled pending. | Every artifact change; resample after X4 or extension update. |
| D-06 | INV-D-06-MATERIAL-PROVENANCE | Every claim's conclusion is within its source registry scope and no unregistered source is accepted. | Same command; source-scope assertion plus manual paraphrase review. | Verified repository-structure evidence | Manual reviewer checks paraphrase and source relevance. | Every artifact change. |
| D-07 | INV-D-07-DEFERRAL-OWNERSHIP | Every unknown claim has non-gating value plus future owner and evidence need; critical-path dependency is false. | Same command; deferred-claim and scope assertions. | Verified repository-scope evidence | No runtime blocker is created. | Every artifact change and before Phase 8 inventory. |
| D-08 | INV-D-08-INVENTORY-ONLY | `scope.phase8_inventory_only` is true and all autonomous implementation flags are false. | Same command; exact scope assertions. | Documented scope evidence | No runtime action is required. | Before Phase 8 inventory. |

## Runtime Deferrals

The first attributable disposable X4 9.00 probe may update claim rows with observed runtime evidence only after Phase 1 provides the read-only adapter contract. This is not a Phase 2 checkpoint and does not delay Phases 1 or 3–7.

## Deterministic Fixture Checks

Run the parser fixtures with the installed Pester version:

```powershell
Invoke-Pester -Path tools/verify_xen_khk_evidence.Tests.ps1
```

The fixtures prove that the parser accepts a valid skeleton and rejects duplicate
source IDs, unknown claim sources, unallowlisted source descriptors, and
conclusions outside a source's permitted scope. The local Pester 3.4 installation
does not provide the newer `-CI` switch; this command is the compatible equivalent
and still reports pass/fail deterministically.

## Evidence Boundary Audit

| Audit question | Result | Evidence level | Action if invalidated |
| --- | --- | --- | --- |
| Is every installed source material to a claim? | Yes: X4 jobs support XEN configuration, KHK activity supports KHK configuration, and the Finder is limited to a visibility precedent. | Verified static installation evidence | Rebuild the registry after a catalog or extension version change. |
| Does each runtime unknown have an owner, evidence need, and non-gating disposition? | Yes: event export, faction visibility, KHK quota/activity observability, and extension interaction are structured unknown claims. | Repository-structure evidence | Keep the claim unknown until an attributable disposable probe succeeds. |
| Do scope fields prohibit hostile implementation? | Yes: autonomy, government institutions, motives, diplomacy, architecture, writes, controls, and critical-path dependency are false. | Documented scope evidence | Reject the artifact until a later milestone explicitly changes the product contract. |
| Can narrative prose bypass the validator? | No: the parser consumes only the named fenced JSON payload. | Verified parser behavior | Correct the structured register; narrative remains explanatory. |

## Required Final Commands

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/verify_xen_khk_evidence.ps1 -EvidencePath .planning/phases/02-hostile-faction-research-track/02-XEN-KHK-EVIDENCE.md -Stage full
Invoke-Pester -Path tools/verify_xen_khk_evidence.Tests.ps1
git diff --check
```

These checks never launch X4, read saves, modify the installed game, contact a
network service, or install a package. They validate only repository-owned
structured evidence and its read-only scope boundary.
