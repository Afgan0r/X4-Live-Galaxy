---
phase: 05
slug: bounded-shadow-deliberation
status: verified
threats_open: 0
asvs_level: 1
created: 2026-08-29
---

# Phase 05 — Security

> Threat verification for the bounded Shadow deliberation boundary.

## Trust Boundaries

| Boundary | Description | Data Crossing |
| --- | --- | --- |
| Provider bytes → strict proposal | Untrusted model output enters deterministic admission. | Bounded JSON candidate bytes |
| Admission → checkpoint | Only an accepted immutable pending commit may request persistence. | Typed commit and causal metadata |
| Cache → admission | Cached bytes remain untrusted and must pass current-state admission. | Candidate bytes and exact identity |
| Local Codex CLI → manual adapter | An explicit developer command invokes a bounded local process. | Canonical corpus request and candidate bytes |
| Benchmark evidence → evaluation | Manual evidence is descriptive and non-authoritative. | Redacted identities and disposition |

## Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation | Status |
| --- | --- | --- | --- | --- | --- | --- |
| T-05-01 | Tampering | Proposal decoding and admission | high | mitigate | Strict direct decode, unknown-field denial, and ordered admission gates | closed |
| T-05-02 | Elevation | Accepted proposal projection | high | mitigate | Rejections cannot project; accepted state uses one checkpoint CAS | closed |
| T-05-03 | Information disclosure | Rejection evidence | medium | mitigate | Retain bounded typed metadata and hashes, never raw candidates | closed |
| T-05-04 | Tampering | Cache identity | high | mitigate | Versioned length-framed identity binds every authority component | closed |
| T-05-05 | Elevation | Cache hit | high | mitigate | Cached bytes re-enter normal admission with authoritative current state | closed |
| T-05-06 | Denial of service | Request resources | high | mitigate | Required nonzero caps cover bytes, calls, retries, timeout, history, and dialogue | closed |
| T-05-07 | Denial of service | Faction scheduler | high | mitigate | One outstanding request, coalescing, cooldown, pause, and newer reconciliation | closed |
| T-05-08 | Elevation | Preemption | high | mitigate | Normal admission precedes typed initiative transition; provider owns no persistence | closed |
| T-05-09 | Tampering | Dialogue and causal replay | high | mitigate | Kernel cycle cap and checkpoint-bound preemption validation | closed |
| T-05-10 | Spoofing | Provider metadata | medium | mitigate | Bounded provider/model identifiers remain non-authoritative | closed |
| T-05-11 | Denial of service | Provider execution | high | mitigate | Stale preflight, scheduler degradation, bounded process deadline and output | closed |
| T-05-12 | Repudiation | Evidence classification | medium | mitigate | Deterministic and manual evidence classes are distinct and manifest-pinned | closed |
| T-05-13 | Information disclosure | Harness evidence | high | mitigate | Redacted bounded records and confined corpus-relative paths | closed |
| T-05-14 | Elevation | CLI harness | high | mitigate | Standalone manual workspace has no X4, report, credential, or persistence port | closed |
| T-05-15 | Repudiation | Benchmark corpus | medium | mitigate | Versioned schema, fixture digests, closed mappings, and negative integrity tests | closed |
| T-05-16 | Denial of service | Local process | medium | mitigate | Deadline, byte cap, tree termination, child reap, and bounded drain cleanup | closed |
| T-05-SC | Tampering | Package installs | high | accept | No package installation; workspace and harness use committed pinned lockfiles | closed |

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
| --- | --- | --- | --- | --- |
| AR-05-01 | T-05-SC | Phase 05 deliberately adds no package installation path and relies only on committed pinned dependency graphs. | Owner-approved Phase 05 plan contract | 2026-08-29 |

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
| --- | --- | --- | --- | --- |
| 2026-08-29 | 17 | 17 | 0 | gsd-security-auditor |

## Evidence

- `cargo test -p mind-domain --test shadow_deliberation_evals --locked --offline`
- `cargo test -p mind-orchestration --test provider_contract --locked --offline`
- `cargo test --manifest-path tools/shadow-harness/Cargo.toml --test manual_contract --locked --offline`
- `05-SECURITY-REVIEW.md` records the supporting ASVS L1 V2–V5 review.

## Sign-Off

- [x] All threats have a disposition.
- [x] Accepted risks are documented.
- [x] `threats_open: 0` is confirmed.
- [x] `status: verified` is set in frontmatter.

**Approval:** verified 2026-08-29
