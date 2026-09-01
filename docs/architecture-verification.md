# Observation Data Flow Verification Registry

## Purpose

This is the durable evidence contract for the observation data-flow
architecture. It records what must be demonstrated before a target rule may be
called implemented or production-admitted. It survives individual phases;
phase plans may satisfy entries but do not replace them.

The target system is documented in [ARCHITECTURE.md](ARCHITECTURE.md). Decision
status and rationale live in
[architecture-decisions.md](architecture-decisions.md). Current implementation
gaps, task sequencing, and phase closure remain under `.planning/**`.

## Evidence vocabulary

- `documented`: supported by an authoritative API, protocol, or installed-source
  document for the exact version in scope;
- `observed`: reproduced by a named executable test or disposable runtime probe
  with retained output;
- `inferred`: a reasoned expectation that is not sufficient for admission;
- `unknown`: no adequate evidence exists;
- `passed`: the entry's acceptance criteria are satisfied by retained evidence;
- `blocked`: a required prerequisite is missing and the architecture cannot
  safely claim the affected capability;
- `deferred`: evidence is intentionally postponed until its promotion gate is
  reached.

Repository tests prove repository behavior. They do not prove X4 runtime
semantics. Source inspection proves a call shape or implementation path, not its
normal-time or SETA cost. A disposable probe proves only its recorded X4 build,
mod stack, campaign shape, settings, and workload.

## Evidence retention rules

Every runtime result cited here must retain:

- exact X4 build and DLC state;
- exact Live Galaxy and relevant installed-extension revisions;
- test campaign type and workload identity;
- normal-time or measured SETA mode;
- requested and delivered callback cadence when scheduling is involved;
- raw bounded metrics or logs needed to reproduce the conclusion;
- a stable repository or approved machine-local artifact locator;
- digest and provenance for private retained artifacts;
- explicit limits of what the evidence does not prove.

Do not retain save files, secrets, raw private payloads, or unbounded game logs.

## Verification matrix

<!-- markdownlint-disable MD013 -->

| ID | Contract under proof | Current evidence class | Admission state | Owning decision |
| --- | --- | --- | --- | --- |
| VER-LG-001 | Scheduler callback delivery and lifecycle | observed nearby precedent; Live Galaxy proof incomplete | open | [ADR-LG-004](architecture-decisions.md#adr-lg-004-initial-scheduler-callback-seam) |
| VER-LG-002 | Real-time scheduler budget and SETA degradation | design only | open | [ADR-LG-003](architecture-decisions.md#adr-lg-003-continuous-cooperative-scheduling) |
| VER-LG-003 | Native enumeration cost, identity, and source coverage | partial source and prototype evidence | open | [ADR-LG-006](architecture-decisions.md#adr-lg-006-independent-aggregate-safety-bounds), [ADR-LG-019](architecture-decisions.md#adr-lg-019-coverage-absence-and-future-deltas) |
| VER-LG-004 | Carrier A bounded duplex behavior | installed-source evidence; pressure/lifecycle proof incomplete | open | [ADR-LG-008](architecture-decisions.md#adr-lg-008-carrier-neutral-application-protocol) |
| VER-LG-005 | Whole-message boundary and oversize classification | library and prototype evidence; production path incomplete | open | [ADR-LG-007](architecture-decisions.md#adr-lg-007-semantic-records-and-bounded-messages) |
| VER-LG-006 | Stop-and-wait disposition semantics | architecture contract only | open | [ADR-LG-011](architecture-decisions.md#adr-lg-011-stop-and-wait-receiver-feedback) |
| VER-LG-007 | Exact retry, reconnect, and unknown-outcome recovery | partial Rust unit coverage | open | [ADR-LG-013](architecture-decisions.md#adr-lg-013-identity-exact-retry-and-reconnect) |
| VER-LG-008 | Content-bound completion and per-section atomicity | prototype generation tests; end-to-end path incomplete | open | [ADR-LG-017](architecture-decisions.md#adr-lg-017-content-bound-completion) |
| VER-LG-009 | Durable publication and restart recovery | persistence tests exist; observation transaction incomplete | open | [ADR-LG-018](architecture-decisions.md#adr-lg-018-atomic-durable-publication) |
| VER-LG-010 | Keyed candidate fairness and dependency supersession | architecture contract only | open | [ADR-LG-015](architecture-decisions.md#adr-lg-015-bounded-keyed-rust-candidates), [ADR-LG-016](architecture-decisions.md#adr-lg-016-dependency-handling) |
| VER-LG-011 | Aggregate bounds and sustained stream stability | numerical policy intentionally unset | open | [ADR-LG-006](architecture-decisions.md#adr-lg-006-independent-aggregate-safety-bounds) |
| VER-LG-012 | Heavy faction ship conformance proof | deferred to dedicated phase | deferred | [ADR-LG-021](architecture-decisions.md#adr-lg-021-heavy-detail-grouping), [ADR-LG-022](architecture-decisions.md#adr-lg-022-heavy-ship-proof-scope) |
| VER-LG-013 | Station-specific complete-set and deletion semantics | Phase 05.1 evidence incomplete | deferred | [ADR-LG-019](architecture-decisions.md#adr-lg-019-coverage-absence-and-future-deltas), [ADR-LG-023](architecture-decisions.md#adr-lg-023-successor-phase-boundaries) |
| VER-LG-014 | Authoritative X4 source epoch | no authoritative value found | deferred | [ADR-LG-014](architecture-decisions.md#adr-lg-014-unknown-source-epoch-baseline) |
| VER-LG-015 | Event/delta completeness | no qualifying X4 stream proven | deferred | [ADR-LG-019](architecture-decisions.md#adr-lg-019-coverage-absence-and-future-deltas) |

<!-- markdownlint-enable MD013 -->

## VER-LG-001: Scheduler callback delivery and lifecycle

### VER-LG-001 question

Can one instantiated Mission Director cue safely deliver the shared Lua
scheduler pulse across ordinary play, pause, menus, minimization, save, load,
reload, disconnect, and sustained SETA without reentrancy or unbounded catch-up?

### VER-LG-001 disposable probe

Create the smallest developer-only cue and Lua callback that:

1. requests the initial probe cadence;
2. records bounded real-time and game-time deltas;
3. detects concurrent entry with an `in_callback` guard;
4. performs no game-state collection beyond a fixed synthetic state transition;
5. records save/load/reload and connection lifecycle edges;
6. caps output and removes itself after the test window.

### VER-LG-001 acceptance

- delivered cadence distributions are retained rather than inferred from the
  request;
- no callback reentrancy or proportional catch-up occurs;
- pause, minimize, save, load, reload, and SETA behavior are classified;
- the seam has an explicit registration, teardown, and duplicate-registration
  result;
- any unsafe lifecycle edge invalidates unpublished work rather than silently
  resuming it.

### VER-LG-001 failure consequence

Reject the callback seam and evaluate a narrower alternative. Do not compensate
for missed delivery by increasing work per callback.

## VER-LG-002: Real-time scheduler budget and SETA degradation

### VER-LG-002 question

Does the shared scheduler bound admitted work in real time while using game-time
freshness only for urgency?

### VER-LG-002 required evidence

- frozen-clock and coarse-jump deterministic tests;
- token refill, burst, debt, and step-cap tests;
- one-heavy-permit admission tests;
- same-band fairness when capacity exists;
- core reservation and detail degradation tests;
- normal-time and sustained approximately `6.2x` SETA runtime measurements;
- visible decision blocking when required data becomes stale.

### VER-LG-002 acceptance

SETA does not increase real-time refill or heavy-work admission. A measured
overrun creates debt. Lower-priority detail degrades before core work, and
unsatisfied core freshness blocks dependent decisions rather than hiding
starvation.

## VER-LG-003: Native enumeration, identity, and coverage

### VER-LG-003 question

For each source adapter, what does X4 actually guarantee about membership,
ordering, count/fill mutation, identity stability, object reuse, and deletion?

### VER-LG-003 docs mcp handoff

All API research is routed through the separate Docs MCP repository. Requests
must name exact immutable source snapshot IDs. When enrichment publishes a new
snapshot, preserve the original request as provenance and issue a linked
expanded request that adds only the exact new snapshot IDs.

Required documentation evidence includes exact signatures, ownership and
lifetime rules, count/fill semantics, cursor or paging support, error behavior,
event coverage, ordering, retention, and lifecycle signals. A `complete`
Docs MCP search with zero results proves only corpus absence for the selected
snapshots; it does not prove an X4 API is absent.

### VER-LG-003 disposable probes

Use source-specific minimal probes to establish:

- count, allocation, and fill behavior during growth and shrink;
- ordering and duplicate behavior;
- invalid or deleted object identities;
- identity reuse inside one uninterrupted runtime scope;
- ownership transfer during capture;
- exact cost distributions for every indivisible native stage;
- behavior under representative modded populations, normal time, and SETA.

### VER-LG-003 admission rules

- `observed_count_fill_only` and unknown source consistency cannot certify
  `complete_set` or `known_empty`;
- equal counts or digests before and after capture do not exclude an ABA change;
- transport batching after a whole-array fill is not native paging;
- a visible or simulation-budget-breaking indivisible call rejects production
  admission for that source;
- absence and deletion remain disabled until complete-set and stable-identity
  evidence both pass.

## VER-LG-004: Carrier A bounded duplex behavior

### VER-LG-004 question

Can the installed `sn_mod_support_apis` path provide bounded nonblocking duplex
transport with explicit message, empty-read, oversize, disconnect, reload, and
error behavior from the X4 callback?

### VER-LG-004 required docs mcp evidence

The source set must include an immutable snapshot of the exact installed
`sn_mod_support_apis` version. Inspect and retain evidence for:

- raw read and write signatures and buffer ownership;
- synchronous versus deferred copy behavior;
- message mode and whole-message boundaries;
- empty nonblocking reads;
- `ERROR_MORE_DATA` and other error classification;
- inbound and outbound buffer or quota behavior;
- write-success meaning;
- reconnect, close, reload, and callback ownership;
- save/load lifecycle hooks and available clocks.

### VER-LG-004 disposable pressure probe

Exercise exact inbound boundary, one byte over, malformed control, absent Rust,
slow Rust, Rust restart, disconnect, reconnect, reload, and repeated empty poll.
Measure callback time and retain bounded error classifications. Do not combine
this probe with game-data enumeration.

### VER-LG-004 acceptance

The facade can distinguish success, would-block/no-message, oversize, closed,
and terminal error without blocking or crashing X4. Every callback operation has
a finite input, output, and work bound. If not, record the exact carrier B
promotion trigger rather than weakening the application protocol.

## VER-LG-005: Message boundaries and oversize handling

### VER-LG-005 required tests

- exact `T_batch` and `H_message` boundaries plus one byte over;
- complete-record packing without record splitting;
- record-count and work ceilings independent of bytes;
- sender rejection before write for an oversized record or message;
- receiver quota overflow classified separately from EOF;
- no prefix of an oversized message reaches decoding;
- invalid UTF-8 and malformed framing discard the affected candidate;
- maximum decoded-memory expansion is independently bounded.

### VER-LG-005 acceptance

Sender and receiver agree on one complete-message contract. Oversize and
malformed input cannot leave earlier staged data eligible for a later completion
certificate.

## VER-LG-006: Stop-and-wait disposition semantics

### VER-LG-006 required tests

- Lua retains exactly one immutable unacknowledged batch;
- `received` is emitted only after message-local validation and volatile stage;
- `committed` is emitted only after durable atomic publication;
- capacity-unavailable proves the frame was not consumed and preserves exact
  retry eligibility;
- permanent rejection aborts the section attempt and requires a new revision;
- terminal/control responses remain possible when data capacity is exhausted;
- exact duplicate returns the prior applicable disposition without a second
  mutation;
- missing or ambiguous response does not advance the sender sequence.

### VER-LG-006 acceptance

The sender never labels pipe handoff as Rust acceptance and never rebuilds bytes
under the same identity. Sustained normal-time and SETA evidence must later show
that the slot drains fast enough or that collection slows before any upstream
bound is reached.

## VER-LG-007: Retry, reconnect, and ambiguous outcomes

### VER-LG-007 required tests

- exact immutable retry after would-block or proven non-consumption;
- same identity with different digest is terminal conflict;
- gap, reorder, timeout, and supersession discard private state only;
- transport-epoch change rejects late frames from the former epoch;
- reconnect discards every volatile candidate and retains accepted revisions;
- retry exhaustion enters bounded cooldown or circuit break and prevents an
  immediate expensive recollection loop;
- disconnect before commit keeps the previous accepted revision;
- disconnect after commit recovers the durable terminal receipt;
- an unknown commit outcome reconciles against durable receipt and current
  pointer before any new publication.

## VER-LG-008: Content-bound completion

### VER-LG-008 required tests

- contiguous section-local ordinals independent of global transport sequence;
- missing, duplicate, reordered, and conflicting batches;
- batch count, record count, exact lengths, ordered batch-manifest digest, and
  canonical membership/content digest;
- exact schema, policy, canonicalization, and digest-algorithm versions;
- frozen dependency and expected-current-pointer revalidation;
- capture window, source epoch status, coverage, and quality preservation;
- zero-record known-empty accepted only with qualifying source evidence;
- partial or failed empty attempt never becomes known empty;
- completion commits while its own terminal message is handled.

### VER-LG-008 acceptance

Rust publishes only the exact candidate certified by Lua and never upgrades the
collector's source claim. An identity-only marker cannot pass.

## VER-LG-009: Durable publication and restart recovery

### VER-LG-009 required crash cuts

Crash or inject failure before and after:

1. immutable revision content insertion;
2. terminal receipt insertion;
3. conditional current-pointer update;
4. storage transaction commit;
5. `committed` response emission.

### VER-LG-009 acceptance

Recovery exposes either the previous complete revision or the new complete
revision. It never exposes mixed content, a pointer without content, a committed
revision without its terminal receipt, or a duplicate publication. Exact replay
of an already committed terminal identity returns the durable disposition.

## VER-LG-010: Keyed candidates, fairness, and dependencies

### VER-LG-010 required tests

- one incomplete revision per active section key;
- aggregate candidate count, raw bytes, decoded estimate, work, age, and
  inactivity bounds;
- alternating batches from several sections through one global transport slot;
- a large backpressured section does not prevent an eligible urgent section
  from receiving scheduler service;
- one candidate failure or timeout leaves unrelated candidates unchanged;
- dependency change during capture does not interrupt work;
- exact dependency mismatch at completion discards the stale private candidate;
- bounded cooldown prevents dependency-churn restart storms.

### VER-LG-010 acceptance

Fairness is demonstrated only when capacity exists. Under overload, intentional
starvation is visible as stale data and blocked decisions.

## VER-LG-011: Aggregate bounds and stream stability

### VER-LG-011 measurements required before choosing numbers

1. complete-record UTF-8 size distributions, including Unicode and optional
   availability shapes;
2. section cardinality and total canonical bytes across representative vanilla
   and supported modded workloads;
3. Lua normalization, serialization, concatenation, allocation, and GC cost;
4. each indivisible native call under normal time and SETA;
5. delivered callback cadence and same-frame clock behavior;
6. pipe-write and control round-trip latency with normal, slow, absent, and
   restarting Rust;
7. actual pipe quotas and whole-message behavior;
8. Rust receive allocation, decode, semantic validation, staging, persistence,
   and commit cost;
9. raw and decoded candidate expansion, records, work, duration, and inactivity;
10. retry amplification and repeated native or serialization work;
11. backlog slope and drain time after bounded stalls;
12. capture-to-commit freshness under normal time and sustained SETA;
13. accepted-history, decision-pin, receipt, and diagnostic retention costs.

### VER-LG-011 stability conditions

For cumulative Lua production `P(t)`, semantic Rust staging `S(t)`, durable
publication `C(t)`, and the independently bounded retained state `B_total`:

```text
supremum over t of (P(t) - S(t)) <= B_total
```

Over the longest sustained production window:

```text
Rust staged rate >= Lua produced rate
```

Product admission additionally requires:

```text
p99(collection + batching + transport + staging + commit)
  < section freshness or stale-block deadline
```

If the staged rate is lower, upstream collection must demonstrably slow before
any bound is exhausted. Local pipe-write counters cannot substitute for staged
or committed rates.

### VER-LG-011 policy outputs

Measurements must derive, at minimum:

- callback token rate, burst, debt, step cap, and heavy threshold;
- section freshness, skew, and stale-decision thresholds;
- record, batch target, hard message, and record-count ceilings;
- native allocation, continuation, per-candidate, and aggregate candidate
  limits;
- retry count, cooldown, timeout, and maximum candidate age;
- post-save cooldown and revalidation policy;
- accepted-history, decision-pin, receipt, and diagnostic retention.

The selected value is the minimum independently safe boundary, not the largest
buffer one layer can allocate.

## VER-LG-012: Heavy faction ship conformance proof

### VER-LG-012 ordered gates

1. One deliberately heavy ordinary `mind_candidate` faction exercises the full
   pipeline for diagnosis.
2. Every dynamically discovered `mind_candidate` faction exercises the same
   architecture under aggregate normal-time and sustained approximately `6.2x`
   SETA evidence.

### VER-LG-012 required coverage

- complete core ship index to the strength proven by its source adapter;
- deterministic bounded cargo, crew, and loadout groups when applicable;
- explicit `not_applicable`, `unsupported`, and `unknown` dispositions;
- eventual group coverage without a false simultaneous faction snapshot;
- scheduler fairness, bounded memory, transport feedback, retry, reconnect,
  candidate completion, durable publication, and dependency-aware decision
  snapshots;
- faction eligibility manifest with exactly one of `mind_candidate`,
  `excluded`, or `unknown` for every discovered faction;
- KHK, player, and XEN excluded under their distinct recorded product reasons.

### VER-LG-012 non-claims

Passing the proof does not make ship reading a production Faction Mind feature,
does not validate a global X4 snapshot, and does not establish special XEN
semantics.

## VER-LG-013: Station-specific completeness and deletion

After the generic architecture and heavy ship proof pass, isolate only the
station-specific remainder:

- authoritative faction-station membership semantics;
- count/fill mutation and stable station identity;
- ownership transfer and deletion behavior;
- supported mod-stack population and indivisible fill cost;
- truthful `complete_set` or continued `partial_set` policy;
- two-absence reconciliation only after qualifying complete revisions.

If no station-specific gap remains, the owning phase may shrink or be removed
through GSD rather than inventing work to preserve the phase number.

## VER-LG-014: Authoritative source epoch

### VER-LG-014 current result

No authoritative campaign UUID, save UUID, loaded-world ID, or equivalent
X4-owned epoch has been established from the inspected support API, vanilla
lifecycle signals, Live Galaxy, or X4 Live MCP evidence.

The following remain boundary triggers or local identities, not source epochs:

- game started or loaded events;
- Lua on-load or named-pipe reload callbacks;
- producer and transport generations;
- real, engine, or game time;
- build identity or saved user data;
- local random tokens;
- `UniverseID` or descriptive object metadata.

### VER-LG-014 promotion gate

An authoritative epoch may be adopted only with exact-version Docs MCP evidence
or a source-owned runtime value whose stability, change conditions, and
ownership are proven. Until then, the conservative unknown-epoch baseline is
mandatory.

## VER-LG-015: Event and delta completeness

### VER-LG-015 required proof

A source may enter baseline-plus-delta mode only when it provides:

- a source-owned baseline boundary or version;
- a complete ordered stream for create, remove, ownership, and relevant update
  events;
- epoch and monotonic sequence identity;
- bounded retention and explicit overflow or gap signaling;
- a race-free scan-to-event handoff;
- deletion and identity-reuse semantics;
- a tested full-rebase path after gap, overflow, expired history, or epoch
  change.

### VER-LG-015 current admission

Deferred. Repeated scans plus selected events remain replacement observations,
not a synthetic oplog.

## Observability evidence

Every performance or stability claim must preserve stage boundaries rather than
one aggregate “sent” counter.

### X4 and Lua

- requested and delivered callback cadence;
- real/game deltas and measured SETA ratio;
- steps attempted, permitted, skipped, and failed by class;
- heavy permits and native-stage latency;
- token balance, burst, and debt;
- section due, overdue, stale, and blocked counts;
- capture age, accepted-section age, dependency rejection, and decision blocks;
- bounded Lua memory and GC-pause evidence.

### Batching and transport

- produced record and UTF-8 bytes;
- batch configuration identity, bytes, records, held age, and seal reason;
- pending identity, bytes, age, and retry count;
- local write attempts, failures, cooldowns, and circuit breaks;
- slot or future window high-water mark;
- reconnect, abandoned-candidate, save-pause, and resume outcomes.

### Rust

- whole message, oversize, EOF, and error receive results;
- decode, validation, staging, persistence, and commit latency;
- candidate section, revision, bytes, decoded estimate, work, age, and ordinal;
- exact replay, conflict, rejection, and abort classes;
- accepted revision and completion time;
- produced-to-staged lag and capture-to-commit freshness;
- decision snapshot freshness, skew, compatibility, and block result.

Rolling averages do not replace burst, tail latency, longest consecutive
backpressure, backlog slope, retry amplification, and end-to-end freshness.
Raw IDs, payloads, native error strings, and unbounded per-callback traces are
not acceptable public diagnostics.
