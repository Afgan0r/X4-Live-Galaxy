# X4 Modding Memory Contract

## Authority and Scope

Use the personal MemPalace MCP server namespace `mempalace_personal`.
The game family is `x4`; `wing_x4_modding` remains the primary write owner for
engineering knowledge across the registered development repositories.

Before recall or capture, read the global `game-repo-standard` skill and its
`references/profiles-and-registry.md` and `references/memory-lifecycle.md` under
`~/.agents/skills/game-repo-standard/`. The shared family registry owns the
complete recall set; this contract keeps the engineering write owner, rooms,
capture timing, and outage behavior below.

Repository files, current source snapshots, current documentation, and fresh
runtime evidence remain primary. Memory supplies historical context; it does
not override contradictory current evidence.

The gameplay wing `wing_x4` participates in family recall and owns general
game knowledge and player requirements. It does not own engineering decisions.
Active campaign state remains in its repository; finding a campaign choice
does not make it a requirement for a development project.

## Rooms

For new engineering writes in `wing_x4_modding`, use only these rooms:

- `decisions` — accepted product, architecture, and process decisions;
- `contracts` — stable cross-component and cross-repository contracts;
- `conventions` — reusable engineering and evidence rules;
- `operations` — verified repeatable workflows and runbooks;
- `incidents` — serious root causes and proven prevention lessons;
- `migrations` — repository, contract, and workflow transitions.

The `Sources` field must name the originating repository and stable evidence.
Do not add a parallel owner taxonomy or repository-specific wings.

## Recall Lifecycle

The main agent owns recall for the top-level task:

1. Resolve the `x4` family in the shared registry; do not maintain a second
   sibling list here or in consumer repositories.
2. At startup and every new substantive topic, run a short thematic query in
   every registered family wing, including gameplay and historical wings,
   even when the current development wing has relevant results. Seed queries
   with the repository, task, identifiers, and affected contracts.
3. Fetch only relevant candidates. Keep game/mod/tool scope, version,
   provenance, and any spoiler boundary attached to the evidence. A sibling
   project's decision is context, not authority over the current project.
4. Expand recall when new identifiers, dependencies, conflicts, or context gaps
   require it. Report unavailable wings as partial coverage; never infer that
   memory is absent from a failed query.
5. Verify stale or consequential memory against primary evidence before using
   it as a premise; give specialists only the distilled context they need.

Reuse applicable context within the same topic. A new substantive topic
requires the family search even if earlier queries found useful context.
Elapsed time or message count alone does not require another search.

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

Choose the write owner by content through the shared registry. Engineering
conclusions remain in `wing_x4_modding`; general game conclusions use the game
owner and its memory lifecycle. Use family recall when deduplicating: an
equivalent historical or sibling record is not a reason to create a copy.

After a verified new write outside the current repository's primary wing,
leave a short local locator in its declared memory-link index or a `Memory
links` section of the consumer-owned root `AGENTS.md`. Include only a safe topic,
exact `wing/room`, and stable `drawer_id`; do not mirror the conclusion or invent
a browser URL. Verify the local entry against the fetched drawer. Do not put
locators in the generated companion bundle. If the pointer write fails, report
it separately and retry the pointer using the existing drawer ID.

Corrections, invalidations, and deletions require an exact preview and the
user's explicit approval. Do not create a local memory outbox when the server
is unavailable; warn and continue from primary evidence.

## Legacy Wings

The shared registry marks earlier X4 project wings as read-only history. They
are part of every new-topic family search; their status restricts new writes,
not recall. Reuse a relevant historical drawer in place. Moving, correcting,
or deleting it requires the exact-preview approval flow above. Do not bulk-copy
drawers or create tunnels as a substitute for the shared family map.
