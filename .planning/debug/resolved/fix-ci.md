---
status: resolved
trigger: "fix ci"
created: 2026-09-03
updated: 2026-09-03
---

# CI failures

## Symptoms

- Expected: Phase 05 deterministic gates passes on the milestone branch.
- Actual: run 33679329583 fails in workspace tests at revision 2114a8d.
- CI failure: strategic-state packet_determinism permutation fingerprints differ.
- Local reproduction: five x4-bridge named_pipe_contract tests fail.
- Reproduction: cargo test --workspace --locked.
- Timeline: multiple recent runs failed; precise introducing revisions pending.

## Current Focus

- Root causes: wall-clock-dependent fixtures, intentionally RED deferred
  contracts, and oversized test modules.
- Next action: publish the verified repair and check GitHub Actions.
- Scope: repair existing behavior without changing approved phase decisions.

## Evidence

- Remote refs fetched; HEAD and upstream both 2114a8d; ahead/behind 0/0.
- Unrelated instructions/configuration changes predate this task and are preserved.
- CI job 100411806181 passes formatting and Clippy, fails packet_determinism.
- Local workspace run passes packet_determinism but fails named_pipe_contract.
- Packet helper calls SystemReceiptClock; replay fingerprint hashes observed_at.
  A millisecond boundary between permutation fixtures changes their inputs.
  Check a fixed receipt before injecting the existing ReceiptClock test seam.
- Named-pipe failures were committed intentionally RED by dc3b1ef for 05.1-09.
  HANDOFF prohibits resuming that superseded plan. Owner approved restoring
  current-behavior regressions and explicitly ignoring future contracts until
  Phase 05.3 reconciliation; no production transport changes are authorized here.
- Source-size lint also rejects runtime_facts_contract.rs (222 lines) and
  named_pipe_contract.rs (213 lines); split cohesive contracts without changing
  the 200-line limit.

## Resolution

- Inject the existing receipt-clock interface into strategic-state test
  fixtures. Keep production timestamp hashing and all permutation assertions.
- Add an empty/singleton/four-observation fixed-receipt regression. It failed
  with the original helper; the original permutation failure was also reproduced.
- Restore nine current bridge regressions from the pre-RED contracts. Keep all
  five future contracts compiled but explicitly ignored with the owner's approval.
- Separate station facts, pipe fixtures, deferred publication, and accept retry
  contracts into cohesive modules under the unchanged source-size limit.
- Do not resume the superseded Phase 05.1-09 implementation or alter runtime code.

## Review and Verification

- Reviewed test input identity, preserved fingerprint sensitivity, restored
  reconnect/disconnect assertions, exact retained future contracts, module
  discovery, and changes outside the task. No remaining findings.
- Focused strategic-state tests: 12 passed; named-pipe tests: 9 passed and
  5 explicitly ignored; runtime-facts tests: 9 passed.
- Final cargo fmt, workspace Clippy with warnings denied, locked workspace
  tests, shadow-harness tests and Clippy, and source-size lint all passed.
- Mutation testing is not applicable: production safety logic is unchanged.
  The original fixture helper demonstrated the regression's failing oracle.
- No X4 API or runtime behavior claim changed; an in-game probe is unnecessary.
- Unrelated changes to AGENTS.md, .planning/config.json, the deleted researcher
  profile, and machine-local .gsd remain outside this commit.

### Clean Windows Checkout Follow-Up

- Run 33682723335 passed all workspace checks, then exposed a further failure
  in shadow-harness evidence_contract at its baseline corpus validation.
- Reproduced that exact assertion with a disposable Git checkout using
  core.autocrlf=true. The local checkout uses input; Windows conversion changed
  the byte-digested JSON corpus from LF to CRLF.
- Add a corpus-scoped .gitattributes rule for JSON text eol=lf. This follows
  the official Git gitattributes contract and preserves manifest digests and
  production validation; it does not normalize unrelated repository files.
- Verify the same disposable checkout under core.autocrlf=true and the existing
  evidence contracts after the rule, then recheck the full hosted workflow.

## Skill Learning

Skill learning: none. Reviewed the clock-dependent regression, intentional
RED checkpoint, and file-size failures. Existing skills already require
deterministic fixtures, approved phase boundaries, and the 200-line limit;
no additional behavioral rule is justified by this repair.
