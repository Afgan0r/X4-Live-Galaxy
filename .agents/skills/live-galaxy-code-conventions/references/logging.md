# Detailed Development Logging

This reference owns logging policy across Rust, Lua/MD, and tools. Test/review
skills consume it without defining a competing verbosity or failure policy.
Diagnostic history is distinct from required durable recovery state/receipts.

## LOG-01 — Default history

Ordinary developer runs record operation starts, significant steps, decisions
and reasons, transitions, retries, rejections, degradation, and outcomes
without a higher-verbosity rerun. Extra low-level per-frame/byte tracing may
be opt-in and bounded, but cannot be the sole record of significant history.
Logging every function call is not a completeness criterion.

Pure decisions return enough typed outcome/reason information for their
execution owner to record this history. Do not add hidden I/O to pure logic or
invent a tracing framework for every function.

## LOG-02 — Events and correlation

Use stable event names and named fields: component, time/clock meaning,
operation/attempt identity, outcome, and applicable context such as
section/revision, generation, reason, duration, size, or retry count. Reuse
meaningful identities across Lua/MD, transport, and Rust. Timestamps from
different clocks alone do not prove causal order.

A rejection records its condition and safe diagnostic values, such as expected
and observed dependency revisions. Generic `failed` without identity or reason
is insufficient. Rendering can vary while event meaning stays stable.

## LOG-03 — Error and level ownership

Levels reflect consequences: a routine domain rejection is not automatically
an error. Distinguish normal decisions, degradation, and failed work. The normal
developer profile enables detailed steps regardless of library defaults.

One boundary owns the final error record. Preserve cause and operation context;
intermediate layers enrich context or record distinct transitions without
duplicating the exception at each layer. Inspect callers/handlers before
concluding that a failure branch lacks diagnostics.

## LOG-04 — Bounds and disclosure

Bound record size, buffering, retention, rotation, formatting work, and emission
rate as needed. Aggregated repetitions preserve count and time window; never
merge distinct significant operations into indistinguishable noise. Expose
suppression/lost-record status. Avoid expensive formatting for disabled detail
and blocking diagnostic work on the game thread.

Never log secrets, private prompts, or hidden reasoning. Keep detailed developer
history separate from public player diagnostics. Use bounded safe metadata
rather than public raw payloads, native identifiers/errors, or machine-local
paths. Redaction retains the safe reason and correlation needed for diagnosis.

## LOG-05 — Journal failure

For developer bridge/tool startup, verify that the journal can actually accept
a record; path existence alone is insufficient. Refuse startup if it cannot.
Do not start or stop X4 to enforce this.

If diagnostic writing fails during operation, continue otherwise valid work
with explicit degraded status through an available independent channel.
Keep emergency reporting bounded and avoid recursive logging failure. Do not
claim lost history was retained. Clear the failed-sink status only after a
successful write and retain an honest indication of the gap.

Failure to persist accepted state or required idempotency receipts still
follows the correctness contract. Diagnostic continuation never authorizes
acknowledgment without required persistence.
