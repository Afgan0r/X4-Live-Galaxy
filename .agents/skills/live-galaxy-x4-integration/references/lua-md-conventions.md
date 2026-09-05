# Lua and Mission Director Code

Read the parent integration skill and common code conventions. These rules
specialize that contract; existing implementation is not a quality exemplar.

## XCODE-01 — Scope and runtime

Use local variables/functions by default. Keep required globals and game
registrations at the explicit integration boundary. Load dependencies through
the extension-relative module paths required by the parent skill.

Use syntax, APIs, and native bindings supported by the established target
runtime. Resolve uncertain game API/loader/lifecycle facts through the shared
Docs MCP gate; a standalone interpreter or permissive fake does not establish
X4 compatibility. Do not adopt another Lua version's features from memory.

## XCODE-02 — Tables and contracts

Define module input/output record shapes and meaningful absence/error outcomes.
Callers know whether the callee may mutate or retain a table. Assignment does
not copy a Lua table; a snapshot requires actual independent ownership of the
mutable data it promises to freeze. Do not deep-copy everything by default.

Distinguish a dense sequence from a keyed map. Do not use `#` to infer the
size/completeness of a table with holes or rely on `pairs` order for a
deterministic decision or wire contract. Handle multiple return values
intentionally. In particular, `value or default` is invalid when `false` is a
valid distinct value that must be preserved.

## XCODE-03 — Failure boundaries

Use protected calls at a concrete error-isolation boundary, preserving the
cause, operation outcome, and cleanup behavior. A caught exception is not an
empty successful observation. Expected absence/rejection has an explicit
contract rather than an assertion against external data. Internal failures
must not be swallowed to continue partially applied work.

Metatables, table reuse, and caches require an actual responsibility or measured
need; generic Lua optimization/OOP advice does not mandate them. Preserve
ownership and invalidation guarantees. Pure modules do not acquire game globals,
native bindings, or logging side effects implicitly.

## XCODE-04 — MD and game boundaries

For an MD handler, define trigger, inputs, state scope, repeat/re-entry and
late-event behavior, and completion. Follow accepted responsibility boundaries
between MD, Lua, and Rust; do not duplicate strategic rules across languages.
Use established schemas and correct XML encoding for externally supplied data.

Game-facing callbacks admit bounded work and respect native resource lifetime.
Do not assume batching after an indivisible native call made that call safe.
Preserve architecture-owned scheduler/pump separation and generation fences;
do not invent loader hooks or lifecycle guarantees without source evidence.

## Source notes

- [Lua 5.1 values and references](https://www.lua.org/manual/5.1/manual.html#2.2)
  explain table aliasing; the target-runtime contract determines applicability.
- [Lua 5.1 length](https://www.lua.org/manual/5.1/manual.html#2.5.5) and
  [iteration](https://www.lua.org/manual/5.1/manual.html#pdf-next) explain the
  sequence/map traps above. These language facts are not proof of X4 APIs.
