# aethersafha — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**0.1.0** — ported from Rust (2026-07-03) via `cyrius port`. 27,207 lines of Rust
preserved at `rust-old/` as the frozen parity oracle.

## Toolchain

- **Cyrius pin**: `6.3.37` (in `cyrius.cyml [package].cyrius`)
- Build: `cyrius lib sync --full && cyrius deps && cyrius build src/main.cyr build/aethersafha`
  (the `lib sync --full` is required — the declared stdlib set exceeds the incremental pin).

## Source

- Rust reference: 27,207 lines at `rust-old/` (frozen, do not edit).
- Cyrius port: **15 modules**; compiles clean + runs on the bhumi seam.
  - **Core (M1)** — `geom`, `window`, `compositor`, `render`, `input`, `main`.
    compositor has workspaces + context types + move/switch + secure/agent-aware
    modes + window-at-point hit-test; renderer has alpha blend + damage tracking.
  - **Leaf (M2)** — `theme_bridge`, `gestures`, `accessibility`, `ai_features`,
    `shell`, `security_ui`, `shell_integration`, `plugin_host` (all 8). Parity vs
    `rust-old/` (heap offset-accessor structs, prefixed symbols); behaviorally tested.
  - **Wiring (B3)** — `desktop` aggregate owns the compositor + all 8 leaf managers
    and is created by `main`, so they're reachable + running. First live cross-subsystem
    connection: **compositor → accessibility** (`desktop_sync_accessibility` mirrors the
    window stack into the a11y tree). Deeper wiring (shell panel, notifications,
    input→gestures) is next.

## Tests

- **12 `.tcyr` files, all green.** Core: `aethersafha` (38), `render` (13),
  `leaf_modules` (11), `desktop` (14 — B3 wiring). Behavioral per-module:
  `theme_bridge`, `gestures`, `accessibility`, `ai_features`, `shell`,
  `security_ui`, `shell_integration`, `plugin_host`.
- Run: `cyrius tests tests/` (or a single `cyrius test tests/<file>.tcyr`).

## Dependencies

- **stdlib** (auto-prepended) — syscalls, string, alloc, atomic, fmt, vec, str,
  slice, hashmap, fnptr, io, fs, process, args, tagged, result, chrono, math,
  assert, bench.
Active (auto-prepended; stdlib declared per each dep's reviewed needs):
- **bhumi** 1.0.0 — platform backend (output/input/seat).
- **agnostik** 1.3.3 — shared domain primitives (errors namespaced `STIK_ERR_*`).
- **agnodrm** 1.4.5 — udev/DRM device model (errors namespaced `DRM_ERR_*`).

Deferred (mapping kept in `cyrius.cyml`, re-enable per the opt-in review):
- **mehman** 0.2.1 — now ships `sandbox`/`surface` modules but still declares
  `[deps.kavach]` → sandhi → the full `tls_native` TLS stack; too heavy until the
  compositor actually hosts guests. Wire at Bite G.
- **mabda** 4.0.2 — GPU, off the v1.0 path.

Opt-in stdlib: `cyrius build` prepends every `[deps.*]` module, so each dep's
stdlib needs must be declared in `[deps].stdlib` (reviewed from its `dist/*.deps`
sidecar + referenced symbols). That's by design — it keeps the dependency surface
visible, not a bug.

## Consumers

_None yet (top-level application, `publish = false`)._

## Next

B3 first increment landed (desktop aggregate + a11y sync). Deepen the wiring
(shell panel render, notifications surface, input→gestures, theme→renderer) and
continue Bite A (renderer decorations + bitmap text; input routing to focused
surface). See [`roadmap.md`](roadmap.md) / [`parity-plan.md`](parity-plan.md).
