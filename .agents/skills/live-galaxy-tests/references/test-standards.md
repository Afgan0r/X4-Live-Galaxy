# Test Standards

## TEST-01: Scenarios and Oracles

Test observable contracts with independent expected results. Cover the success
path plus relevant boundaries, rejection paths, preserved state, and forbidden
side effects. Assertions must make the failed contract clear; asserting only
that a call returned does not prove the behavior.

Test each risk the changed contract actually introduces. For example, an
admission rule may need accepted, stale, malformed, over-budget, duplicate, and
out-of-order scenarios, while a display-only change may not. Coverage
percentages help find blind spots but do not replace those scenarios.

## TEST-02: Minimum Capable Level

Test pure policy and state transitions directly. Do not mock the system under
test or expose private production internals merely to make them testable.
Choose a unit, contract, integration, structural, or runtime test according to
the claim:

- Unit tests prove local deterministic behavior.
- Contract tests prove an explicit seam and its rejection behavior.
- Integration tests prove the owned components work together.
- Structural, schema, and package checks prove an actual structural claim.
- Runtime evidence proves behavior that local seams cannot establish.

Source-text matching is not primary behavioral evidence when executable
evidence is practical. It may guard a narrow structural invariant.

## TEST-03: Offline and Real Boundaries

Normal unit and contract suites do not run X4, use the internet, or call a live
model. Controlled local integration services are allowed. At a product
cross-language boundary, exercise the real producer/consumer path and normal
module loading. Do not use a permissive fake loader that masks a missing
production import.

Doubles replace explicit external seams only. Give them realistic contract
behavior and deliberate success, absence, malformed-input, rejected-context,
and failure outputs as relevant. Unexpected calls and invalid inputs must not
magically succeed. A fake establishes the local contract, never the existence
or semantics of a real X4 API.

## TEST-04: Persistence and Recovery

Use actual isolated storage to prove durability. Establish state, persist it,
then read it independently or restart with clean in-memory state. A failure
injection fake can prove error handling, but cannot prove storage durability.
Do not accept a serializer round trip through the same implementation as the
only persistence or serialization evidence.

For recoverable operations, verify the terminal receipt or accepted state after
an interruption and that replay cannot duplicate an accepted effect.

## TEST-05: Fixtures and Isolation

Keep fixtures and builders small and comprehensible. Make essential inputs
visible; avoid hidden defaults and automatic resets that can mask the defect.
Each test starts from fresh isolated state and leaves no ordering dependency.

Control clocks, randomness, identifiers, and scheduling inputs whenever they
affect the result. Do not use sleeps for pure logic. For asynchronous behavior,
wait on a bounded condition and make timeout failure diagnostic.

## TEST-06: Diagnostics

When a changed path emits diagnostics, assert the stable semantic fields that
matter to the contract: event identity, reason or disposition, correlation,
and state. Do not assert every formatted line, timestamp, stack frame, or
incidental field. Test relevant journal or diagnostic-failure branches without
duplicating the logging policy owned by the code-conventions skill.

## TEST-07: Evidence and Regression

State what was executed and what it proves. Separate locally verified behavior,
runtime evidence still pending, and behavior observed in X4. A regression test
must fail for the original defect, not merely execute the edited line.

For a negative API test, attempt the forbidden capability through the current
public API and check the intended rejection reason. Failure caused only by a
removed method name or unrelated import error does not prove that the capability
is inaccessible.

Run focused checks during iteration and one final relevant full regression
after review convergence. Record commands, outcomes, mutation disposition when
applicable, and honest coverage gaps in the existing phase or review artifact.
