# Observation Data Flow Decision Ledger

## Purpose

This ledger records the decisions that shape Live Galaxy observation data flow.
It owns decision status, context, rationale, consequences, rejected shortcuts,
and deferred promotion gates. It does not describe implementation progress or
serve as the system walkthrough.

The target system is documented in [ARCHITECTURE.md](ARCHITECTURE.md). Evidence,
measurements, unresolved X4 facts, and disposable probes are tracked in
[architecture-verification.md](architecture-verification.md). Phase delivery and
current implementation gaps remain under `.planning/**`.

## Status vocabulary

- `accepted`: owner-approved baseline or durable architecture rule;
- `deferred`: a known extension with an explicit promotion gate;
- `evidence-gated`: direction is accepted, but production admission awaits
  named evidence;
- `superseded`: retained for traceability but no longer authoritative.

Decision IDs are stable. Amend an entry or add a superseding entry instead of
renumbering the ledger.

## ADR-LG-001: Architecture family

- **Status:** accepted
- **Context:** Live Galaxy must observe large, mutable X4 data sets without
  exposing partial replacements or creating periodic game-thread load spikes.
- **Accepted rule:** use bounded per-section observation snapshot transfer with
  private assembly and atomic install. X4 remains authoritative; Rust stores
  immutable observed projections rather than a writable replica.
- **Rationale:** snapshot-install and initial-sync systems provide the closest
  complete lifecycle: build privately, prove completeness to the strength of
  the source claim, then publish atomically.
- **Consequences:** repeated scans are replacement attempts, not a mutation log.
  No component may claim a globally simultaneous galaxy snapshot. Baseline
  mechanisms are source-agnostic; stations and ships are conformance examples.
- **Supersedes:** the implicit design of unrelated scans, queues, and completion
  markers without one end-to-end state-transfer model.

## ADR-LG-002: Authority and language boundary

- **Status:** accepted
- **Context:** an external process cannot independently call the inspected X4
  getter surface, while complex processing inside X4 increases hitch and crash
  risk.
- **Accepted rule:** keep the in-X4 layer minimal: X4 getters, bounded resumable
  continuations, X4-specific normalization, local frame-safety admission, and a
  carrier facade. Rust owns global collection policy, candidate assembly,
  validation, persistence, publication, diagnostics aggregation, and model
  logic.
- **Rationale:** this minimizes dependence on work performed inside X4 without
  pretending that changing the transport removes Lua from source collection.
- **Consequences:** Rust sends logical collection intent and bounded control,
  not remote calls for individual getters. Lua retains final authority over
  whether another native step is safe in the current callback.
- **Supersedes:** designs that place the complete scheduler, snapshot database,
  or LLM pipeline inside X4, and designs that treat getters as remote RPC.

## ADR-LG-003: Continuous cooperative scheduling

- **Status:** accepted
- **Context:** periodic full scans create load spikes and make large sources
  impossible to refresh predictably. Lua execution is cooperative and cannot
  preempt an indivisible native call.
- **Accepted rule:** one source-agnostic global scheduler advances persistent
  collector state machines in small bounded steps on a frequent delivered
  pulse. Freshness determines urgency; a shared real-time reservation, burst
  cap, step cap, heavy-operation permit, and debt bound determine admission.
- **Rationale:** this is an EDF/CBS-derived soft-deadline model suited to a
  cooperative runtime. It separates data urgency from safe game-thread work.
- **Consequences:** SETA may raise urgency but never multiplies the real-time
  work budget. No proportional catch-up runs after a callback gap. A visible or
  simulation-budget-breaking indivisible call fails production admission even
  if only one such call ran in the callback.
- **Supersedes:** complete scans every `N` seconds and loops that run until a
  sampled clock changes.

## ADR-LG-004: Initial scheduler callback seam

- **Status:** evidence-gated
- **Context:** a true per-frame callback is not yet proven necessary or safer.
  The closest installed precedent uses one Mission Director cue to raise one Lua
  event at a short requested cadence.
- **Accepted rule:** begin with one instantiated Mission Director cue feeding
  one global Lua scheduler. A requested `50 ms` interval is a disposable-probe
  setting only, not a delivered-cadence guarantee or production threshold.
- **Rationale:** the seam is simple and already has nearby X4 precedent. Earlier
  evidence observed about `100 ms` delivery from a `50 ms` request, so measured
  delivery rather than requested cadence controls admission.
- **Consequences:** registration must be idempotent and non-reentrant. A true
  per-frame or native callback is considered only if normal-time and sustained
  approximately `6.2x` SETA evidence shows the simple seam cannot meet the
  freshness and drain-rate contract.
- **Supersedes:** assuming `checkinterval` is exact or adding independent
  high-frequency loops per collector.

## ADR-LG-005: Separate collection scheduler and transport pump

- **Status:** accepted
- **Context:** coupling collection with writes lets production continue while
  older data is already blocked and makes transport failures repeat native work.
- **Accepted rule:** scheduler and pump are separate state machines even when
  invoked by the same callback. The pump runs first. Collection may produce
  only after the necessary downstream reservation exists.
- **Rationale:** backlog must receive service before more work is admitted, and
  transport retry must never invoke getters or rebuild records.
- **Consequences:** collectors never write to the pipe; the pump never calls X4
  getters. Backpressure pauses expensive production before retained memory is
  exhausted.
- **Supersedes:** fill-before-pump FIFO behavior and collector-local writes.

## ADR-LG-006: Independent aggregate safety bounds

- **Status:** accepted
- **Context:** a FIFO bound, record count, or message ceiling protects only one
  owner of memory or work. The legacy `64`, `128`, `129`, `1,800`-byte, and
  `2,048`-byte values were evidence or implementation artifacts rather than a
  coherent safety model.
- **Accepted rule:** every source call, continuation, semantic record, batch,
  transport slot, pipe receive, Rust candidate, accepted-history store, and
  diagnostic stream has independent finite byte, work, count, memory, and age
  bounds as applicable. Per-candidate bounds coexist with aggregate bounds.
- **Rationale:** one finite queue can still hide an unsafe native allocation,
  one huge message, decoded-memory expansion, or unbounded accepted history.
- **Consequences:** work is admitted only after downstream capacity is reserved;
  terminal and control traffic retains capacity that data cannot consume.
  Numerical values are configurable and evidence-derived, beginning generous
  rather than inheriting old fixtures.
- **Supersedes:** the 129-component native ceiling as durable policy and any
  interpretation of “remove the old cap” as “allow unbounded state.”

## ADR-LG-007: Semantic records and bounded messages

- **Status:** accepted
- **Context:** a section may contain many MiB while one serialization, write,
  receive, or decode burst must remain bounded.
- **Accepted rule:** messages pack one or more complete semantic records. A
  record is never split arbitrarily across messages. `T_batch` is a performance
  target; `H_message` is a separately enforced hard ceiling over the complete
  wire message. Record count and decode work are bounded independently of bytes.
- **Rationale:** a queue limit does not control the cost of its largest message,
  and a message limit does not need to cap the total size of a streamed section.
- **Consequences:** an oversized record requires semantic decomposition into
  independently meaningful records. `256 KiB` from X4 Live MCP and every legacy
  fixture size are explicitly non-authoritative.
- **Supersedes:** treating FIFO capacity as the message ceiling or Windows pipe
  capacity as the application payload policy.

## ADR-LG-008: Carrier-neutral application protocol

- **Status:** accepted
- **Context:** `sn_mod_support_apis` is the only locally proven X4 named-pipe
  carrier, but its wrapper and lifecycle limitations must not define the whole
  application protocol.
- **Accepted rule:** start with `sn_mod_support_apis` behind Live Galaxy-owned
  `try_send`, `poll_control`, connection-status, message-identity, and bounded
  error semantics. Collector and Rust snapshot state machines remain unaware of
  carrier-specific error names and buffer assumptions.
- **Rationale:** this yields the fastest real duplex proof while keeping a
  deliberate migration path to a minimal owned native carrier.
- **Consequences:** carrier A is implemented first. A Live Galaxy-owned Rust/C
  Lua module is the expected carrier B only after executable or measured
  evidence proves an A-side correctness, reconnect, callback-time, binary-copy,
  payload, or packaging limitation.
- **Supersedes:** making `sn_mod_support_apis` part of semantic snapshot identity
  and implementing a custom DLL speculatively.

## ADR-LG-009: Protected UI and online features

- **Status:** accepted
- **Context:** preserving protected UI and X4 online features would constrain
  transport choices without advancing the Live Galaxy product goal.
- **Accepted rule:** neither protected UI compatibility nor online-feature
  preservation is a transport admission requirement.
- **Rationale:** the owner explicitly accepts these losses in exchange for a
  simpler and more controllable local integration.
- **Consequences:** installation documentation must still disclose the effects.
  Crash containment, nonblocking behavior, packaging, provenance, compatibility,
  and measured performance remain required.
- **Supersedes:** carrier comparisons that treat these two capabilities as vetoes.

## ADR-LG-010: Small inbound control plane

- **Status:** accepted
- **Context:** observation data is primarily X4-to-Rust. The return path is
  needed for feedback and validated commands, not bulk data transfer.
- **Accepted rule:** Rust-to-Lua messages contain bounded handshake, demand,
  disposition, collection intent, health, session reset, and later small typed
  action primitives. They never carry snapshots, entity inventories, raw model
  output, prompts, large plans, schemas, or bulk diagnostics.
- **Rationale:** this makes a small carrier receive buffer an error-handling
  boundary rather than an expected throughput bottleneck.
- **Consequences:** the current `2,048`-byte support-module receive buffer is not
  copied into protocol policy. The protocol selects a smaller complete-message
  ceiling with headroom and tests exact-boundary and oversize behavior. A future
  large inbound feature requires a separate action/data-plane decision or
  carrier B.
- **Supersedes:** symmetric bulk duplex transport as a baseline requirement.

## ADR-LG-011: Stop-and-wait receiver feedback

- **Status:** accepted
- **Context:** a successful pipe write proves only an OS-level handoff, not Rust
  decoding, staging, durable commit, or publication.
- **Accepted rule:** the baseline has one globally unacknowledged immutable
  application batch. `received` confirms batch-local validation and volatile RAM
  staging; `committed` confirms certificate validation, durable persistence, and
  atomic publication. Negative dispositions preserve retryability distinctions.
- **Rationale:** demand `1` is the smallest complete backpressure contract and
  bounds sender ambiguity while the duplex seam is being proven.
- **Consequences:** Lua releases a batch only after its exact `received` or
  terminal disposition. Rust loss or reconnect discards incomplete candidates
  and requires recollection under a new identity. Control and terminal traffic
  has reserved capacity.
- **Supersedes:** calling write success acceptance and starting with a sliding
  window merely because the OS pipe can buffer more.

## ADR-LG-012: Transport expansion ladder

- **Status:** deferred
- **Context:** stop-and-wait may eventually limit throughput, but increasing
  concurrency before identifying the bottleneck expands retry and recovery
  state unnecessarily.
- **Accepted rule:** promote first to a measured fixed bounded window only when
  end-to-end evidence isolates stop-and-wait as the cause of a drain-rate or
  freshness failure. Consider dynamic receiver credit only after an implemented
  fixed window is proven insufficient for observed receiver-capacity variation.
- **Rationale:** each rung adds independent identities, reserved memory,
  reconnect behavior, and idempotency cases.
- **Consequences:** the extension is recorded and must not be rediscovered, but
  no window size is chosen in advance.
- **Supersedes:** arbitrary window growth during planning.

## ADR-LG-013: Identity, exact retry, and reconnect

- **Status:** accepted
- **Context:** transport order, section completeness, source lifetime, and
  decision replay are different identity domains.
- **Accepted rule:** use separate source epoch, producer incarnation, transport
  epoch, global transport sequence, section key, section revision,
  section-local batch ordinal, exact message digest, record identity/content
  digest, and decision snapshot identity. Exact identity plus exact digest is
  idempotent; the same identity with different bytes is a conflict.
- **Rationale:** overloading one sequence causes interleaved sections and
  lifecycle boundaries to corrupt one another.
- **Consequences:** retries retain immutable bytes. Reconnect changes the
  transport epoch and discards incomplete candidates. Ambiguous outcomes are
  not automatically retryable.
- **Supersedes:** using one global session sequence as both transport order and
  section-local completeness.

## ADR-LG-014: Unknown source epoch baseline

- **Status:** accepted
- **Context:** no authoritative X4 campaign/load UUID has been established.
  Local nonces, clocks, transport generations, `UniverseID`, and lifecycle
  events do not prove source identity.
- **Accepted rule:** operate only inside one uninterrupted, unambiguous runtime
  scope when source epoch is unknown. Fence work with a local producer
  incarnation. Any unproven game start, load, Lua reload, or similar boundary
  ends the scope, discards unpublished work, excludes prior accepted revisions
  from current decisions, and requires a fresh baseline.
- **Rationale:** an explicit unknown is safer than manufacturing continuity.
- **Consequences:** cross-boundary absence, deletion, and entity-continuity
  claims remain disabled. Discovering an authoritative X4-owned epoch is a
  deferred capability rather than a blocker for partial observations.
- **Supersedes:** treating a connection epoch or local nonce as a campaign ID.

## ADR-LG-015: Bounded keyed Rust candidates

- **Status:** accepted
- **Context:** multiple logical sections must progress without one large
  section blocking every smaller urgent section.
- **Accepted rule:** Rust holds a bounded map with at most one volatile
  incomplete revision per active section key. Batches for different candidates
  may alternate through the one global stop-and-wait slot; each completed
  section publishes independently.
- **Rationale:** transport order and section-local assembly are separate. A
  keyed map provides logical interleaving without pretending there are parallel
  wire streams.
- **Consequences:** every candidate has independent byte, work, record, age,
  inactivity, sequence, dependency, and failure state, plus aggregate candidate
  count and memory bounds. One candidate failure does not discard another.
- **Supersedes:** one global candidate and one faction-wide all-or-nothing
  generation.

## ADR-LG-016: Dependency handling

- **Status:** accepted
- **Context:** cancelling heavy work on every dependency update may cause
  perpetual restart, while committing it against changed dependencies produces
  inconsistent decisions.
- **Accepted rule:** freeze exact dependency revisions at candidate start,
  finish collection privately, and revalidate before commit. A mismatch discards
  the private candidate and schedules recollection after bounded cooldown.
- **Rationale:** optimistic finish-then-validate is the simplest general policy
  that never publishes stale dependency combinations.
- **Consequences:** stale candidates are neither published nor kept as accepted
  history. Selective carry-forward requires stable identity, a compatibility
  contract, measurements showing the baseline fails, and new verification.
- **Supersedes:** reactive cancellation as the default and silent cross-revision
  merge.

## ADR-LG-017: Content-bound completion

- **Status:** accepted
- **Context:** a marker-shaped message proves only that a marker arrived. It
  cannot prove ordered batch membership, record counts, content, dependencies,
  capture time, or source coverage.
- **Accepted rule:** use a lightweight start envelope and a content-bound end
  certificate. Rust recomputes ordered batch and canonical content evidence,
  verifies contiguous ordinals, versions, frozen dependencies, capture window,
  coverage, and quality, then commits or rejects the candidate atomically.
- **Rationale:** section publication needs an explicit semantic completion
  boundary independent of pipe writes.
- **Consequences:** an exact upfront membership manifest is optional and
  source-dependent. A correct certificate can preserve but never strengthen a
  weak source claim such as `partial_set`.
- **Supersedes:** identity-only completion markers and commit triggered by an
  unrelated later frame.

## ADR-LG-018: Atomic durable publication

- **Status:** accepted
- **Context:** a crash between accepted content, terminal receipt, and current
  pointer updates can cause duplicate publication or an unrecoverable mixed
  state.
- **Accepted rule:** one storage transaction covers immutable accepted revision
  content, the terminal idempotency receipt, and the conditional current-section
  pointer update. `committed` is emitted only after commit. Decision snapshots
  pin exact compatible accepted revisions.
- **Rationale:** readers must see either the previous complete revision or the
  next complete revision, never partial content.
- **Consequences:** incomplete candidates remain volatile and are recollected
  after Rust loss. Durable per-batch staging is deferred unless recollection is
  measured to be materially harmful.
- **Supersedes:** publication before durable receipt and resumable volatile
  candidates across restart.

## ADR-LG-019: Coverage, absence, and future deltas

- **Status:** accepted
- **Context:** transport completeness does not prove X4 source completeness.
  Missing data, known empty data, and entity deletion have different meanings.
- **Accepted rule:** every section separately records source-membership,
  producer-assembly, transfer, and publication completeness. Partial sets and
  point measurements may publish honestly. Absence semantics require an
  authoritative same-scope complete set, stable identity, and two consecutive
  absences; an explicit authoritative deletion may remove immediately.
- **Rationale:** downstream validation cannot manufacture a stronger upstream
  source claim.
- **Consequences:** optional field failures become unknown or stale rather than
  entity deletion. Baseline-plus-delta mode is admitted only after a source-owned
  boundary and a proven contiguous ordered event stream with gaps, overflow,
  retention, and deletion semantics. Any gap forces a full rebase.
- **Supersedes:** one-absence tombstones, empty-on-error, and treating repeated
  polling as a synthetic event log.

## ADR-LG-020: Exact-version admission

- **Status:** accepted
- **Context:** silently reinterpreting accepted content after schema or policy
  changes makes replay and validation ambiguous.
- **Accepted rule:** current projection admission requires exact schema, policy,
  canonicalization, and digest-algorithm versions. Older revisions retain their
  original identities but are ineligible for new decisions unless a compatible
  reader is deliberately implemented.
- **Rationale:** recollection is simpler and safer than implicit cross-version
  migration in the baseline.
- **Consequences:** upgrades invalidate incompatible current projections and
  recollect. Migration and additive compatibility remain deferred until a real
  retention need or measured recollection cost justifies them.
- **Supersedes:** implicit additive compatibility.

## ADR-LG-021: Heavy detail grouping

- **Status:** accepted
- **Context:** one faction-wide cargo, crew, or loadout snapshot is too large and
  too easy to invalidate, while one section per ship creates excessive metadata
  and scheduling overhead.
- **Accepted rule:** core identity data is separate from heavy details. Cargo,
  crew, and loadout publish as deterministic versioned bounded groups tied to an
  exact core revision and exact member identities.
- **Rationale:** bounded groups provide source-agnostic incremental progress
  while preserving atomic publication at a useful semantic level.
- **Consequences:** groups have independent coverage, capture windows, and
  freshness. The full proof tracks eventual coverage without claiming one
  simultaneous faction-wide detail snapshot. Carry-forward across a core
  revision remains deferred pending stable identity and compatibility evidence.
- **Supersedes:** one all-or-nothing heavy faction revision and one section per
  ship.

## ADR-LG-022: Heavy ship proof scope

- **Status:** accepted
- **Context:** the architecture needs a deliberately heavy cross-layer proof,
  but the proof must not become the definition of the generic system or silently
  admit every special faction.
- **Accepted rule:** first exercise one heavy ordinary eligible faction, then all
  dynamically discovered `mind_candidate` factions. Core ships are required;
  cargo, crew, and loadout use a source-backed capability matrix. KHK, the player
  faction, and XEN are excluded for distinct owner-approved reasons.
- **Rationale:** this validates the architecture under realistic scale for
  future faction minds while isolating special product and source semantics.
- **Consequences:** each discovered faction is classified as `mind_candidate`,
  `excluded`, or `unknown`; unknown blocks proof closure. The proof spans X4,
  Lua, transport, Rust staging, publication, and decision snapshots under normal
  time and approximately `6.2x` SETA, but does not by itself make ship reading a
  production Faction Mind feature.
- **Supersedes:** a station-only architectural proof and an implicit vanilla
  faction allowlist.

## ADR-LG-023: Successor phase boundaries

- **Status:** accepted
- **Context:** Phase 05.1 and Phase 05.2 accumulated useful foundations but also
  mixed generic architecture, station-specific gaps, and verification work.
- **Accepted rule:** Phase 05.3 owns the generic observation data-flow
  foundation and feedback baseline. Phase 05.4 owns the real heavy faction ship
  proof across X4 and Rust. Phase 05.5 owns only station-specific source or
  completeness gaps that remain afterward.
- **Rationale:** generic architecture must exist before another source-specific
  closure attempt; the heavy proof then identifies which station gaps are truly
  unique.
- **Consequences:** `.planning/**` must persist final scope and may shrink or
  remove Phase 05.5 when Phase 05.4 closes the shared seam. This ADR does not
  retroactively expand Phase 05.1.
- **Supersedes:** closing the generic architecture opportunistically inside the
  remaining station work.

## ADR-LG-024: Documentation authority split

- **Status:** accepted
- **Context:** the former architecture file mixed the target system, rationale,
  open questions, current defects, phase scope, verification plans, and agent
  instructions.
- **Accepted rule:** `ARCHITECTURE.md` documents the target system and links to
  ADR IDs. This ledger owns decisions. `architecture-verification.md` owns
  durable evidence gates and probes. `.planning/**` owns implementation gaps,
  phase scope, and delivery state. Repository instructions own agent behavior.
- **Rationale:** each question must have one clear authority so future agents do
  not treat a current defect, rejected option, or unverified number as target
  architecture.
- **Consequences:** Git history preserves the former mixed document; no archive
  copy is maintained as a second source of truth.
- **Supersedes:** the monolithic observation architecture notebook.

## Precedent map

Precedents constrain contract shape but do not donate numerical limits or
guarantees that depend on infrastructure X4 lacks.

<!-- markdownlint-disable MD013 -->

| Concern | Precedent | Adopted shape | Non-transferable assumption |
| --- | --- | --- | --- |
| Backpressure | [Reactive Streams](https://github.com/reactive-streams/reactive-streams-jvm), [gRPC flow control](https://grpc.io/docs/guides/flow-control/), [RabbitMQ confirms](https://www.rabbitmq.com/docs/confirms) | Explicit bounded demand and distinct capacity, volatile receipt, durable commit, and rejection | Pipe write success is not peer processing; Live Galaxy has no broker durability |
| Retry and deduplication | [Kafka design](https://kafka.apache.org/40/design/design/) | Epochs, sequences, immutable retry, exact identity-plus-digest deduplication | No durable replicated log or broker producer state |
| Section assembly | [S3 multipart upload](https://docs.aws.amazon.com/AmazonS3/latest/userguide/mpuoverview.html) | Attempt identity, ordered parts, part digests, explicit complete and abort | Parts are volatile; S3 does not prove X4 coverage or dependencies |
| Atomic publication | [Apache Iceberg](https://iceberg.apache.org/spec/), [PostgreSQL MVCC](https://www.postgresql.org/docs/current/mvcc-intro.html) | Private immutable revision, conditional pointer switch, pinned reader versions | No transactional engine spans X4 and Rust |
| Initial scan and deltas | [Kubernetes list/watch](https://kubernetes.io/docs/reference/using-api/api-concepts/), [Debezium snapshots](https://debezium.io/documentation/reference/stable/connectors/postgresql.html) | Baseline boundary, contiguous ordered changes, rebase on gap | No proven X4 resource version, transaction log, or retention contract |
| Cross-section progress | [SCTP interleaving](https://www.rfc-editor.org/info/rfc8260/) | Separate transport order from section-local identity and alternate ready work | Live Galaxy does not use SCTP streams or wire fragmentation |
| Scheduler admission | [Linux SCHED_DEADLINE](https://docs.kernel.org/scheduler/sched-deadline.html) | Deadline urgency plus independent runtime reservation | Cooperative Lua has no preemption or hard WCET guarantee |
| Stability evidence | [Apache Flink backpressure](https://nightlies.apache.org/flink/flink-docs-stable/docs/ops/monitoring/back_pressure/) | Produced, staged, committed, busy, idle, backpressured, and end-to-end freshness metrics | Flink thresholds and runtime topology do not exist in X4 |

<!-- markdownlint-enable MD013 -->

The architecture is compositional. No precedent above grants Live Galaxy
exactly-once delivery, a durable source log, a global X4 snapshot, hard real-time
scheduling, or safe production limits by analogy.
