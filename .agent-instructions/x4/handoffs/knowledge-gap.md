# Docs MCP Knowledge-Gap Handoff

Read this file only after source research and the Docs MCP lookup gate leave a
load-bearing knowledge gap.

Emit the following structure in chat. Do not write it to a repository, queue,
or proposal store.

```text
X4 Docs MCP knowledge gap

Blocked scope: <full task or exact dependent slice>
Missing entity or claim: <precise unknown>
Why it blocks: <decision or implementation that cannot be made safely>
Exact request: <verbatim server, tool, and structured arguments>
Development context: <consumer repo, feature, expected use, constraints>
Sources already checked: <stable locators and snapshot identities>
Residual evidence needed: <what would unblock the retry>
```

A full blocker stops the task. A local blocker stops only the dependent slice;
unrelated work may continue. Wait for the user to coordinate enrichment. After
the user says data was added, repeat the exact request and resume only if the
required evidence is present.
