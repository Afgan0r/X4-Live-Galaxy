# X4 Modding Memory Contract

## Authority and Scope

Resolve the exact MCP server and authorized scopes from the current client's
X4 project-memory binding before memory I/O. Do not infer a transport from a
wing name. If the binding cannot be resolved, report the gap and continue from
primary evidence under the outage policy below.
The game family is `x4`; `wing_x4_modding` remains the primary write owner for
engineering knowledge across the registered development repositories.

Use the scope registry supplied by that client binding for known related and
historical sources; it is not a mandatory search checklist. This contract owns
X4 engineering write routing, rooms, recall access, capture timing, and outage
behavior. Other subjects and memory sources follow their own contracts; this
contract does not prohibit independently governed reads from them.

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
Sharing a physical store does not merge logical project ownership or source
restrictions. Each configured source retains its own access boundaries.

Repository files, current source snapshots, current documentation, and fresh
runtime evidence remain primary. Memory supplies historical context; it does
not override contradictory current evidence.

Related game or tool knowledge may inform a task through its own source
contract. Finding a requirement in another context does not make it binding
on an X4 development project.

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

1. Resolve the X4 project-memory binding and its scope registry; do not
   maintain a second sibling list here or in consumer repositories.
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

X4-specific engineering conclusions belong in `wing_x4_modding`. Route other
subjects through their own contracts. Use relevant retrieved context when
deduplicating: an equivalent historical or sibling record is not a reason to
create a copy. Project records need stable evidence accessible to their
intended readers.

Report each verified write with its owner, returned drawer ID, and readback
result. Cite only relevant retrieved evidence for the current task. A memory
write alone requires no repository edit.

Corrections, invalidations, and deletions require an exact preview and the
user's explicit approval. Do not create a local memory outbox when the server
is unavailable; warn and continue from primary evidence.

## Legacy Wings

The configured scope registry marks earlier X4 project wings as read-only
history. They
remain available as read-only context when relevant; their status restricts
new writes, not recall. Verify applicability against current evidence and
reuse a relevant historical drawer in place. Moving, correcting,
or deleting it requires the exact-preview approval flow above. Do not bulk-copy
drawers or create tunnels as a substitute for the configured scope registry.
