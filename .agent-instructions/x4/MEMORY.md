# X4 Modding Memory Contract

## Authority and Scope

Use the personal MemPalace MCP registration `mempalace-personal`, exposed as
`mcp__mempalace_personal__*`.
The game family is `x4`; `wing_x4_modding` remains the primary write owner for
engineering knowledge across the registered development repositories.

Before recall or capture, read the global `game-repo-standard` skill and its
`references/profiles-and-registry.md` and `references/memory-lifecycle.md` under
`~/.agents/skills/game-repo-standard/`. The shared family registry owns the
write routing and known scope refinements; it is not a mandatory search
checklist. This contract keeps the engineering write owner, rooms, recall
access, capture timing, and outage behavior below.

The canonical external entrypoint is `shared/MEMORY.md` in
`x4/agent-instructions`; resolve its checkout locally. Registered consumers
read its materialized `.agent-instructions/x4/MEMORY.md` instead. Relevant X4
memory must also be consulted for external or cross-project questions when its
content can help, regardless of the current directory, chat, or result owner.
Reading this memory contract does not apply X4 engineering standards or a
consumer's workflow to another project.

Keep the read source, knowledge owner, and applicability of a rule separate.
Each source's own contract governs its access, subagents, outages, and capture;
neither this contract nor a shared retrieval rule overrides another owner.
Sharing one physical personal palace does not merge logical project ownership,
privacy, game, spoiler, or source restrictions.

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
2. Before substantive analysis, check the topic's coverage and reuse intact,
   applicable evidence. For uncovered needs, run bounded thematic searches
   across the selected physical palace by default, using task identifiers and
   affected contracts. Use a larger bounded result limit when exposed (20 is
   a reasonable starting point), then adapt queries and limits to useful
   coverage rather than accepting the first top-k results.
3. Fetch only relevant candidates. Keep game/mod/tool scope, version,
   provenance, and any spoiler boundary attached to the evidence. A sibling
   project's decision is context, not authority over the current project.
4. Use wing or room filters to refine noise or omissions, not as a compulsory
   search of every family member. If a semantic query cannot safely bound
   protected content, apply safe scope filters before searching. Follow each
   source's access and spoiler rules before fetching; do not dump the corpus.
   For missing context, try evidence-seeded alternate wording and bounded
   listings. A miss does not prove absence. Report unavailable required scopes
   as partial coverage, never as empty memory.
5. Verify stale or consequential memory against primary evidence before using
   it as a premise; give specialists only the distilled context they need.

Check coverage for each new substantive topic and after lost context, using
retained findings when they still cover the need. Do not repeat the same query
per message; elapsed time or message count alone does not require a search.
Only the main agent queries or mutates X4 project memory. Specialists and
subagents receive filtered, provenance-bearing context from the main agent.

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

Choose the write owner by content through the shared registry. X4-specific
engineering conclusions remain in `wing_x4_modding`; general game conclusions
use the game owner and its memory lifecycle. Use relevant retrieved context when
deduplicating: an equivalent historical or sibling record is not a reason to
create a copy.

Project or skill work may also establish a distinct reusable cross-project
conclusion owned by personal memory. Capture it only under that owner's own
rules; X4 facts and paraphrases remain project-owned. Personal records may cite
project memory. Project records must never cite or depend on personal memory,
even when both owners share the same physical palace.

Follow `game-repo-standard` for reporting verified writes, thematic retrieval,
and task-specific citations. A memory write alone requires no repository edit.

Corrections, invalidations, and deletions require an exact preview and the
user's explicit approval. Do not create a local memory outbox when the server
is unavailable; warn and continue from primary evidence.

## Legacy Wings

The shared registry marks earlier X4 project wings as read-only history. They
remain available as read-only context when relevant; their status restricts
new writes, not recall. Verify applicability against current evidence and
reuse a relevant historical drawer in place. Moving, correcting,
or deleting it requires the exact-preview approval flow above. Do not bulk-copy
drawers or create tunnels as a substitute for the shared family map.
