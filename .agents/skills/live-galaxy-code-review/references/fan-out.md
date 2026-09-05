# Bounded Review Fan-Out

This is a workflow reference, not a second set of code conventions. Use it as
the review lead. A specialist does not dispatch children, fix code, or issue
the final verdict.

## Select useful lenses

Use automatic fan-out for substantive reviews when independent lenses can
materially improve the pass: nontrivial control flow, state/effects, trust or
data boundaries, recovery, or several interacting components. Select only
applicable lenses, up to three. A small mechanical or isolated low-risk change
usually stays with the lead. Do not invent numeric diff-size thresholds or
dispatch agents only to occupy all slots.

- **Bugs:** Incorrect conditions, data/range handling, state transitions,
  lifetime, retries, recovery, and partial effects. Avoid hypothetical
  redesigns or unrelated legacy cleanup.
- **Tests:** Missing mandatory scenarios, weak or circular oracles, hidden
  fixture state, misleading doubles, and false-green checks. Do not demand one
  test per function or a universal coverage percentage.
- **Logging:** Significant decisions/failures without reason or correlation,
  false outcomes, lost history, and broken journal-failure behavior. Do not
  demand logging every helper or duplicating errors already recorded by the
  owner.

The lead retains the complete convention and cross-component review.

## Routing and scope

Request **GPT-5.6 Luna / high** for these narrow read-only specialists. Name the
requested model, effort, and bounded/no-history fork before dispatch. Use
runtime controls explicitly when exposed. A full-history fork inherits its
parent allocation and therefore is not an implicit Luna dispatch. Prefer
filtered context and the exact review scope over copying unrelated history.

Use a matching available agent role only if its configured model and authority
fit this assignment. Do not use a GSD role outside its workflow or silently
override project GSD routing. If Luna/high or delegation is unavailable, do
the lens in the lead and report that limitation rather than silently choosing
a more expensive model. Keep GSD or other mandated specialist gates separate.

Give each child the same exact reviewed revision/diff identity and relevant
full files, callers, tests, accepted contracts, and task decisions. Record
pre-existing dirty work and the read-only boundary. Do not let workers fetch,
switch branches, modify files, run the game, access secrets, or operate memory.
The lead owns freshness, permitted live work, and final synthesis.

Explicitly enumerate every applicable skill and reference, including:

- common code `SKILL.md` and relevant `logging.md` / `tooling.md`;
- common tests `SKILL.md` and `references/test-standards.md`;
- Rust conventions/tests or X4 integration, its Lua/MD reference, X4 tests,
  and the Lua skill when those languages are in scope;
- this review `SKILL.md` and `references/fan-out.md`;
- additional actually relevant references those entrypoints carry.

Do not substitute skill names/directories for exact file paths. Require the
child to list what it read and its coverage limitations. Changelogs are needed
for rule-history disputes, not every code review. For structural discovery
follow the repository's ast-index readiness and narrow-query contract; known
files/diffs and literal searches retain their normal routing.

## Assignment shape

Adapt this text to the actual lens; do not send unresolved placeholders:

```text
Act as a read-only specialist in the [named] lens of this review.
Review [exact change/revision] in [explicit files/contracts].
Do not re-review every convention, edit files, dispatch children, or verdict.
Read these full instruction/reference files: [explicit applicable paths].
The lead has established: [accepted behavior, freshness, scoped user decisions].
Seek grounded candidates, not a quota. Check surrounding owners and evidence.
Return for each candidate: exact location, trigger, violated contract/rule,
expected versus actual behavior, evidence, consequence, and a minimal remedy.
Separate unverified questions from established candidates. Return none if none
are grounded. List files read and material limits. The lead adjudicates all.
```

For the logging lens, explicitly require looking for the caller/handler that
owns the final event. For the tests lens, require the specific mandatory
scenario or concrete defective implementation that could pass. For the bugs
lens, trace the changed path and relevant state rather than listing hazards.

## Join and disposition

The lead continues its full review while children run. Wait through normal
runtime coordination rather than busy polling. Receive each selected result;
verify it against current code, deduplicate by cause, reject unsupported
candidates, and grade the remaining findings. A failed child is either
replaced by explicit lead coverage or left as a declared unresolved limit.

Stop fan-out when the selected questions are answered. After fixes, dispatch
only a focused follow-up justified by changed evidence; avoid automatic
multi-round swarms or repeating unchanged full reviews. Keep findings and
dispositions in the existing review output, not a new review platform.
