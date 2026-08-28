# X4 Live Galaxy

Live Galaxy is a pre-alpha prototype for an X4: Foundations mod in which an
LLM-backed strategic director can influence faction economies, fleets, and
institutions through validated game actions.

## Status

The repository currently contains only the project bootstrap. All `0.x`
versions are internal prototypes. Version `1.0.0` is reserved for the first
public alpha.

The initial product discovery and roadmap will be created through GSD after the
project-local agent routing is generated and Codex is restarted.

## Direction

- X4 remains authoritative for observed game state and applied actions.
- A deterministic Rust bridge validates state, plans, and strategic actions.
- Models propose typed plans; they never mutate the game directly.
- Reliability, recovery, structured logging, caching, and bounded token usage
  are product requirements.
- Installed game and mod files are read-only research sources.

Implementation details, dependencies, and protocol choices are intentionally
deferred to milestone and phase research.

## Development

Development follows the repository's `AGENTS.md`, project-local skills, and GSD
workflow. Product decisions belong to the project owner; technical design and
implementation may be delegated to agents within those decisions.

## License

No open-source license has been selected yet. Until one is added, no permission
is granted to copy, modify, or redistribute this repository. License and
third-party provenance review are required before the public alpha.
