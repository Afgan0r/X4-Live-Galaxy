---
phase: 03-faction-scoped-strategic-state
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 03: Code Review Fix Report

### CR-01: Replay identity

`ObservationRecord` now produces an explicit, length-framed canonical
fingerprint over source identity, observation time/version, quality, and
content. `FactReference` retains that bounded fingerprint and replay bytes
include its fixed-width value. Regressions prove same-family entity
replacement, content change, and version change alter replay identity while
permuted equivalent input remains equal.

### WR-01: Primitive budget

Configured budgets below four now return `PrimitiveLimitExceeded`; 4 and 5 are
accepted while the fixed four-primitive allowlist remains unchanged.

## Verification

- `cargo test -p observation-domain -p strategic-state` — passed.
- `cargo clippy -p observation-domain -p strategic-state --all-targets -- -D warnings` — passed.
- Changed Rust sources remain below the 200-line source-size limit.
- `cargo test --workspace` — blocked by the same unrelated missing exports.
- Scoped formatting and `git diff --check` — passed.
