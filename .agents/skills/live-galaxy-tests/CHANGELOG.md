# Changelog

## 2026-09-17

- Clarified performance evidence after the initial Phase 05.5 experiment:
  artificial callback waits were confused with continuous sorting cost and
  synchronous execution was overclaimed as whole-game blocking. Distinguish
  measured callback/CPU time, end-to-end time and simulated cadence, without
  assuming X4 thread topology or observed frame impact. Source:
  `.planning/phases/05.5-heavy-faction-ship-conformance/05.5-EXECUTION-REVISION.md`,
  initial experiment checkpoint `644d9e4` and owner discussion.

- Clarified cross-parent continuity and complete readback trace oracles after
  Phase 05.5 harness review: exact current-parent bindings alone allowed a
  reset-to-first-member caller regression, and omitted capture entries could
  skip independent readback. Source:
  `.planning/phases/05.5-heavy-faction-ship-conformance/05.5-REVIEW.md`.

## 2026-09-05

- Created the shared cross-language test sufficiency and evidence rules.
- Clarified negative API regression oracles after Phase 05.3 review P1-01:
  failing on an obsolete method name did not prove that validated authority
  was inaccessible through the replacement public constructor. Source:
  `.planning/phases/05.3-generic-observation-contracts-and-durable-publication/05.3-REVIEW.md`.
