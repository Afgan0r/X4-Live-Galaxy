---
id: SEED-001
status: dormant
planted: 2026-09-06
planted_during: Milestone 0.1 / Phase 05.4 discussion
trigger_when: when relevant
scope: separate future milestone
---

# SEED-001: Bridge Interface, Desktop Options, and Agent Access

## Why This Matters

The owner proposed a web interface for the Live Galaxy bridge while discussing
Phase 05.4 diagnostics, then explicitly placed it in a separate later
milestone. Phase 05.4 uses concise console status and the existing structured
developer logging contract. The idea should remain discoverable after its
originating phase is complete or archived.

## When to Surface

**Trigger:** when relevant

Review this seed during future milestone selection, especially when bridge
usability, diagnostics, or public operation becomes a milestone goal. The owner
decides when to admit the work; this seed does not add it to milestone 0.1 or
promise a delivery date.

## Scope Estimate

**Separate future milestone** — owner-selected delivery boundary; effort is
not estimated. Concrete features, interface design, framework, API shape,
transport, access model, packaging, milestone number, and release placement
remain unselected.

## Desktop and Agent Access Exploration — 2026-11-02

The owner wants to retain a possible lightweight standalone desktop version of
Live Galaxy, without a browser or WebView, targeting Windows, macOS, and Linux.
This extends the earlier interface idea; it does not select a delivery approach
or admit work into the current milestone.

The motivation is to operate Live Galaxy conveniently without the resource
overhead associated with a browser-based desktop shell, and eventually let an
agent operate the application through MCP. Low resource use is a goal to
measure, not an established property of a selected framework. Desktop-shell
portability does not establish portability of the existing X4 integration.

### Confirmed Design Direction for Future Evaluation

- The human GUI and application MCP must call the same application operations,
  with the same validation, persistence, error behavior, and authoritative state.
  Agent changes must become visible in the GUI; do not create parallel business
  logic or a second state/ingestion pipeline.
- Separate the application MCP, which exposes meaningful typed operations and
  queries, from GUI automation, which verifies actual presentation and user
  interactions. The owner endorsed both roles; exact tool contracts remain open.
- Agent access must preserve the application's authority boundaries. This seed
  does not authorize new game-state mutations or change the observation-only
  scope of milestone 0.1.

### Stack Comparison Remains Open

Compare Slint and Iced directly; retain Qt as another viable candidate. Slint
interests the owner, while Iced's maintainer perspective is also appealing;
neither preference selects a framework. Qt's language and integration choices,
including the cost of a possible C++ boundary, need evaluation rather than an
assumption that all application logic must move to C++.

The application will be authored by an agent. Do not prioritize a familiar web
stack merely to reduce the owner's initial learning effort. Evaluate correctness,
test automation, platform integration, resource usage, dependency stability, and
the cost of maintaining and verifying the application. Rust is a candidate
starting point, not a newly approved rewrite mandate.

When this work is admitted, compare representative prototypes with the same
screen, data, and update workload in release builds. Measure startup, total
process memory, idle and active CPU/GPU use, and minimized/background behavior.
Check the controls and interactions actually needed, keyboard behavior,
accessibility, packaging, and behavior on all three operating systems. Define
numeric acceptance budgets during that discussion; none have been selected.

Investigate Slint's documented testing/MCP facilities as a development and GUI
verification option. Their existence does not establish a stable production
application API, license entitlement, or measured performance advantage.

### Scope and Resurfacing

Keep the original separate-future-milestone boundary. Surface this enrichment
when considering the bridge interface, desktop packaging, user-facing
diagnostics, or agent integration. Research, framework selection, implementation,
and any migration require that future milestone's own discussion and approval.
The owner requested idea capture only on 2026-11-02; no comparison experiment or
implementation was started by that request.

## Breadcrumbs

- `.planning/phases/05.4-owned-carrier-b-and-production-observation-path/05.4-CONTEXT.md`
  — originating discussion and the current phase boundary.
- `.planning/PROJECT.md` — public mod direction and internal prototype scope.
- `.planning/ROADMAP.md` — active delivery ownership.
- `.agent-instructions/x4/AGENTS.md` and `.agent-instructions/x4/MEMORY.md`
  — shared authority, workflow, and memory boundaries.
- Owner discussion on 2026-11-02 — desktop/MCP idea, endorsement of shared
  application operations and the two automation roles, and explicit deferral.
- Framework research leads, accessed 2026-11-02; recheck before selection:
  [Slint platforms](https://slint.dev/faqs),
  [Slint testing and embedded MCP](https://docs.slint.dev/latest/docs/rust/i_slint_backend_testing/#embedded-mcp-server),
  [Iced](https://github.com/iced-rs/iced), and
  [Qt Bridge for Rust](https://www.qt.io/development/qt-framework/qt-bridges/rust-gui).

## Notes

User-proposed on 2026-09-06; explicit approval to create this seed followed in
the same discussion. Enriched on 2026-11-02 with the desktop candidate and the
explicitly endorsed GUI/MCP design direction above. Other potential interface
capabilities suggested in chat are not accepted requirements. Keep this seed
independent from the separately deferred joint X4/bridge launcher idea and
reconcile any future MCP design with the project's earlier Debug MCP decisions.
