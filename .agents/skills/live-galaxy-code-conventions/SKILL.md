---
name: live-galaxy-code-conventions
description: >-
  Engineering rules for Live Galaxy Rust, Lua/MD, and repository tools. Use for
  code design, implementation, refactoring, or review (конвенции кода,
  написание и ревью кода).
---

# Live Galaxy Code Conventions

## Authority and loading

Read root `AGENTS.md` and applicable accepted decisions in
`docs/ARCHITECTURE.md` and `docs/architecture-decisions.md`. Architecture is a
target, not evidence of implementation. Existing code is compatibility
evidence, not a quality exemplar.

This skill owns common code rules. Also read these sources when applicable:

- [Logging](references/logging.md): diagnostics, decisions, errors, transitions,
  and developer startup behavior.
- [Tooling](references/tooling.md): scripts, CLI programs, subprocesses, files,
  provisioning, and check runners in any language.
- [Rust](../live-galaxy-rust-conventions/SKILL.md): Rust-specific obligations.
- [X4 integration](../live-galaxy-x4-integration/SKILL.md) and its Lua/MD
  reference: game-facing code and Lua/MD modules.
- [Tests](../live-galaxy-tests/SKILL.md): test design, implementation, or review.

For delegation, enumerate applicable `SKILL.md` and reference paths explicitly
in the child prompt. A directory or transitive link alone is insufficient.

## C-01 — Change scope

Require conformity from new and changed logic and related corrections needed
for the task. Do not require cleanup of an entire touched file, module, or
repository merely because legacy code differs. A pre-existing defect newly
activated by the change is in scope; broader audits are separate work.

## C-02 — Responsibility and authority

Give modules cohesive responsibilities, explicit dependencies, and clear state
owners. Split independent rules, lifecycles, or external systems when they
become entangled. Moving arbitrary chunks into `utils` is not a split. Lua/MD
and tools use responsibility criteria, not new numeric size limits; Rust's
existing limit remains binding.

Separate pure decisions from execution. Domain code consumes normalized typed
data and returns decisions with reasons; adapters own external shapes and
effects, and orchestration owns their sequence. Preserve X4 authority and all
deterministic validation of untrusted model proposals. Do not duplicate
strategic policy in game adapters or permit unchecked model-directed mutation.

## C-03 — Module contracts

Expose only the surface required by consumers; do not expose mutable internals
for callers or tests. Pass dependencies explicitly rather than locating them
through global services. Pass cohesive inputs, not a giant context for two
fields. Names disclose domain meaning and effects: reads and validation must
not perform hidden business mutations.

Replace ambiguous flags or interchangeable arguments with meaningful contracts.
`set_enabled(enabled)` does not need an enum solely because it accepts a bool;
`execute(command, true, false)` needs a clearer interface.

## C-04 — Types and data meaning

Distinguish identifiers, units, clocks, quantities, bounds, and states whose
interchange changes meaning. Use Rust domain types and explicit Lua/MD record
contracts with centralized boundary construction/validation. Local arithmetic
does not require wrappers for every primitive.

Constructors, conversions, decoding, and mutation all preserve invariants.
External representations stay outside validated domain state until accepted.
Use explicit alternatives instead of accidental boolean combinations or magic
strings. Distinguish absent, false, zero, empty, unknown, stale, and unsupported
where required; partial observations cannot establish authoritative absence.
Check ranges, narrowing, finiteness, overflow, rounding, and unit changes where
they affect the contract. Specify intentional saturation instead of silently
coercing invalid input.

## C-05 — Errors and retries

Distinguish expected absence, domain rejection, transient failure, internal
contract failure, and unknown external outcome. Preserve causes and actionable
context. Do not swallow errors, substitute zero/empty data, or report success
unless that fallback is an explicit contract outcome. Validate at boundaries,
rely on established internal invariants, and revalidate mutable conditions at
the required application point.

A timeout or stopped wait does not prove an external effect did not happen.
Retry only under a safe contract with an aggregate attempt/time bound. Nested
layers must not multiply attempts or renew budgets indefinitely. Follow the
architecture's exact retry/reconciliation rules for its protocol.

## C-06 — State and persistence

Mutate state through its owner's invariant-preserving operations. Define
initialization, valid transitions, failure, and recovery. Derived data and
caches have explicit validity/invalidation rules; avoid independent mutable
copies of the same authority. Keep incomplete work separate from accepted
state, and retain the previous accepted result after a failed replacement.
Preserve the architecture's finish-then-validate dependency policy where it
applies; a general cancellation guideline does not replace it.

Distinguish queued, sent, validated, staged, and committed outcomes. An
acknowledgment cannot promise an unreached stage or durability guarantee.
Persist atomic changes together and make recovery idempotent. A local database
transaction does not establish atomicity with X4 or another service.

For wire/persisted-format changes, identify affected consumers and retained
state. Provide compatible reading, migration, or explicit rejection under the
accepted contract; never hide incompatibility through defaults or deletion of
recovery state. Internal API changes may update their actual callers together;
speculative compatibility layers are not required. Preserve current and
decision-pinned data during retention cleanup.

## C-07 — Resources and concurrency

Identify who creates, uses, supervises, and releases resources/background work.
Cover success, failure, cancellation, and shutdown; cleanup errors must not
hide the primary failure. Ordinary construction must not silently start a
background loop. Define work that completes, is abandoned, or is recovered
on shutdown, with bounded waiting and honest interruption limits.

Make shared-state access order and synchronization explicit. Revalidate
relevant mutable conditions after waits. Old-generation callbacks/results must
not mutate a new generation. Holding a shared lock across slow external work
requires a concrete atomicity/lifecycle justification.

## C-08 — Bounds and reproducibility

Bound externally driven and retained work by applicable count, bytes, age,
steps, retries, and time limits. Per-item bounds do not replace aggregate
bounds. Define capacity behavior: pause, reject, or evict under an explicit
policy. Do not silently lose significant state or effects.

Supply replay-relevant time, randomness, and external inputs explicitly.
Specify order where it affects decisions, identity, or serialized contracts;
do not sort everything indiscriminately. Game-time urgency and real-time work
budgets keep distinct meanings. Beyond explicit safety/resource budgets,
optimize only a measured bottleneck while preserving validity and determinism.

## C-09 — Readability and abstraction

Keep the normal flow locally traceable; separate invalid-input handling where
that clarifies it. Split mixed levels of work into meaningful operations, not
line-count fragments. Document non-obvious units, ownership, effects, states,
errors, and reasons for constraints rather than restating signatures/code.

Each domain rule has one owner. Superficially similar code may stay separate
when its contracts and reasons to change are independent. Every layer or
abstraction owns a real boundary, transformation, policy, resource, or
sequence. Do not add pass-through symmetry, an interface per entity, or a
flag-heavy engine for hypothetical reuse.

## C-10 — Configuration and dependencies

Validate configuration before use. Define source precedence and defaults;
reject invalid explicit values and unknown owned parameters unless an explicit
compatibility contract allows them. Read configuration at composition
boundaries and pass relevant validated values. If runtime reload exists,
validate a replacement before publishing it coherently.

A dependency solves a current need after considering the language, existing
dependencies, target compatibility, and maintenance cost. Current official
documentation and task research own library/runtime selection; conventions
alone do not choose a crate, async runtime, or new scripting stack. Respect
repository manifests, lockfiles, and toolchain policy.

## Applying the rules

Use repository formatters and lint settings. Do not weaken rules/checks to
obtain a green result; use only narrow exceptions permitted by their owning
policy. Test skills own verification selection; the review skill owns findings
and verdicts. Cite a rule and concrete violation, not legacy style as authority.
