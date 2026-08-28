# Memory Recall (MemPalace)

_Wing: wing_x4_live_galaxy · Mode: augment · Transport: MCP_

## Prior decisions

- Keep the X4 Lua/Mission Director adapter thin and stable; compatible Rust
  releases restart and reconnect without restarting X4. Incompatible
  game-facing protocol combinations fail closed with an explicit X4 restart
  condition. — MemPalace decision sourced from `.planning/REQUIREMENTS.md`,
  filed 2026-08-28
- Milestone 0.1 is an observation-only AFK/SETA test stand. Structured external
  snapshots, diagnostics, and decision traces are primary evidence; in-game
  reporting is only an integration sanity check. — MemPalace decision filed
  2026-08-27
- XEN/KHK research runs independently and must not delay the ZYA/ARG observation
  path. — MemPalace decision filed 2026-08-27

## Patterns

- Validate Rust behavior through focused and full tests, then establish a
  measured mutation baseline at phase/release gates. Keep static, pure Lua,
  fake-adapter, and observed-in-X4 evidence distinct. — diary entry
  `SESSION:2026-08-28|LiveGalaxy.testing.policy.persisted`

## Surprises / gotchas

- Rust-only reconnect and game-facing incompatibility are different restart
  classes and must not collapse into one failure state.
- No current temporal-KG facts were returned for the `Live Galaxy` entity;
  native planning artifacts remain the primary phase contract in augment mode.
