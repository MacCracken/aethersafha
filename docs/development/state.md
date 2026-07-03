# aethersafha — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**0.1.0** — ported from Rust (2026-07-03) via `cyrius port`. 27,207 lines of Rust
preserved at `rust-old/` as the frozen parity oracle.

## Toolchain

- **Cyrius pin**: `6.3.36` (in `cyrius.cyml [package].cyrius`)
- Build: `cyrius lib sync --full && cyrius deps && cyrius build src/main.cyr build/aethersafha`
  (the `lib sync --full` is required — the declared stdlib set exceeds the incremental pin).

## Source

- Rust reference: 27,207 lines at `rust-old/` (frozen, do not edit).
- Cyrius port: **6,006 lines across 14 modules**; compiles clean + runs on the bhumi seam.
  - **Core (M1)** — `geom`, `window`, `compositor`, `render`, `input`, `main`.
    Deepened this pass: compositor now has **workspaces + context types +
    move/switch + secure/agent-aware modes + window-at-point hit-test**; renderer
    has **alpha blend + damage tracking**. Wired into the running binary.
  - **Leaf (M2)** — `theme_bridge`, `gestures`, `accessibility`, `ai_features`,
    `shell`, `security_ui`, `shell_integration`, `plugin_host` (all 8). Parity vs
    `rust-old/` (heap offset-accessor structs, prefixed symbols); compile
    individually + together (no collisions); behaviorally tested. Not yet wired
    into the compositor (B3).

## Tests

- **11 `.tcyr` files, ~670 assertions, all green.** Core: `aethersafha` (38),
  `render` (13), `leaf_modules` (11). Behavioral per-module: `theme_bridge`,
  `gestures`, `accessibility`, `ai_features`, `shell`, `security_ui`,
  `shell_integration`, `plugin_host`.
- Run: `cyrius tests tests/` (or a single `cyrius test tests/<file>.tcyr`).

## Dependencies

- **stdlib** (auto-prepended) — syscalls, string, alloc, atomic, fmt, vec, str,
  slice, hashmap, fnptr, io, fs, process, args, tagged, result, chrono, math,
  assert, bench.
- **bhumi** 0.7.0 — platform backend (output/input/seat). **The only active dep**
  — clean (no git sub-deps).
- **Deferred (documented in `cyrius.cyml`, NOT auto-prepended until consumed):**
  - **mehman** 0.1.0 — pulls `[deps.kavach]` → sigil/patra/sandhi, dragging in the
    HTTP/`thread_local` surface as reachable-undefined. Types-only, post-MVP (Bite G).
  - **agnostik** 1.3.2 + **agnodrm** 1.4.4 — both bundle the shared `ERR_*` module
    (duplicate symbols); unused today. Re-enable with selective `modules` when consumed.
  - **mabda** 4.0.2 — GPU, off the v1.0 path.

  Rationale: `cyrius build` prepends every `[deps.*]` module; the unused heavy
  bundles only broke the build. The full dependency mapping is preserved in the
  manifest comments.

## Consumers

_None yet (top-level application, `publish = false`)._

## Next

M2-B3: wire the 8 leaf modules into the compositor/shell surface. Continue Bite A
(renderer decorations + bitmap text; input routing). See
[`roadmap.md`](roadmap.md) / [`parity-plan.md`](parity-plan.md).
