# X4 Modding Memory Contract

## Authority and Scope

Use the personal MemPalace MCP server namespace `mempalace_personal` and the
single active development wing `wing_x4_modding`.

Repository files, current source snapshots, current documentation, and fresh
runtime evidence remain primary. Memory supplies historical context; it does
not override contradictory current evidence.

The gameplay wing `wing_x4` is outside this contract. Do not store development
decisions there or pull campaign state into mod-development tasks.

## Rooms

Use only these rooms in `wing_x4_modding`:

- `decisions` — accepted product, architecture, and process decisions;
- `contracts` — stable cross-component and cross-repository contracts;
- `conventions` — reusable engineering and evidence rules;
- `operations` — verified repeatable workflows and runbooks;
- `incidents` — serious root causes and proven prevention lessons;
- `migrations` — repository, contract, and workflow transitions.

The `Sources` field must name the originating repository and stable evidence.
Do not add a parallel owner taxonomy or repository-specific wings.

## Recall Lifecycle

The main agent owns one logical recall sequence for the top-level task:

1. Seed recall from the exact repository, task, identifiers, and affected
   contracts.
2. Fetch only the drawers that can change a current decision or verification
   path.
3. Expand the same recall sequence during work when new evidence reveals a new
   identifier, dependency, conflict, or cross-repository impact.
4. Verify stale or consequential memory against primary evidence before using
   it as a premise.
5. Give specialists and subagents only the distilled context they need.

Memory is not restricted to task start and closure. Expansion during work is
required when there is a concrete reason; repeated broad searches without new
evidence are not.

## Capture Lifecycle

Capture semantic outcomes at task closure by default. Do not store raw plans,
GSD artifacts, logs, prompts, status narration, copied source, or generated
indexes.

Mid-task capture is allowed only when waiting until closure risks losing a
verified fact that affects remaining or cross-repository work:

- a completed X4 experiment with retained, digest-identified evidence;
- a proven serious root cause and its prevention boundary;
- a newly accepted contract that changes the remaining task.

Each retained drawer must state the task, outcome, accepted decisions,
validation, limitations, and exact sources. Deduplicate before writing and
re-fetch the retained drawer after writing. Capture the semantic conclusion,
not the entire artifact that led to it.

Corrections, invalidations, and deletions require an exact preview and the
user's explicit approval. Do not create a local memory outbox when the server
is unavailable; warn and continue from primary evidence.

## Legacy Wings

These wings are read-only historical sources:

- `wing_x4_live_galaxy`;
- `wing_x4_ai_workspace`;
- `wing_x4_live_mcp`;
- `wing_live_galaxy`.

Search a legacy wing only when its historical scope is relevant. Distill a
still-useful fact into `wing_x4_modding` only through normal semantic capture;
never bulk-copy drawers. Do not create tunnels between the X4 development
repositories.
