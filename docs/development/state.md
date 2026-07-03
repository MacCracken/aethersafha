# aethersafha — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**0.2.0** (2026-07-03) — parity milestone: compositor/renderer depth, all 8 M2 leaf
modules, B3 desktop wiring, Bite A window interaction. Ported from Rust via
`cyrius port`; 27,207 lines preserved at `rust-old/` as the frozen parity oracle.

## Toolchain

- **Cyrius pin**: `6.3.38` (in `cyrius.cyml [package].cyrius`)
- Build: `cyrius lib sync --full && cyrius deps && cyrius build src/main.cyr build/aethersafha`
  (the `lib sync --full` is required — the declared stdlib set exceeds the incremental pin).

## Source

- Rust reference: 27,207 lines at `rust-old/` (frozen, do not edit).
- Cyrius port: **15 modules**; compiles clean + runs on the bhumi seam.
  - **Core (M1/Bite A)** — `geom`, `window`, `compositor`, `render`, `input`, `main`.
    compositor: workspaces + context types + move/switch + secure/agent-aware modes
    + window-at-point hit-test; renderer: alpha blend + damage tracking + window
    **decorations** (close/max/min buttons + `deco_hit` decoration hit-test);
    input: **window-management shortcuts** (Tab focus-cycle, F4 close, F5 maximize-
    toggle, F6 minimize) via `input_map`/`input_apply`, wired into the frame loop.
  - **Leaf (M2)** — `theme_bridge`, `gestures`, `accessibility`, `ai_features`,
    `shell`, `security_ui`, `shell_integration`, `plugin_host` (all 8). Parity vs
    `rust-old/` (heap offset-accessor structs, prefixed symbols); behaviorally tested.
  - **Wiring (B3, complete)** — `desktop` aggregate owns the compositor + all 8 leaf
    managers, created by `main`. `render_desktop` is the unified frame: clear to the
    **theme** background (`desk_bg_color`), paint windows, draw the **shell** status
    panel (`render_shell_panel` — cpu/mem/battery bars, net dot, notification badge).
    Live cross-subsystem links: compositor→accessibility (`desktop_sync_accessibility`),
    theme→renderer, shell→renderer, shell_integration tray. (ai/security/gestures/
    plugin instantiated; deeper feature wiring is follow-on.)

## Tests

- **13 `.tcyr` files, all green.** Core: `aethersafha` (38), `render` (27 — decoration
  hit-test + shell-panel bars), `input` (13 — key→action→window mgmt), `leaf_modules`
  (11), `desktop` (15 — B3 wiring + theme bg). Behavioral per-module: `theme_bridge`, `gestures`,
  `accessibility`, `ai_features`, `shell`, `security_ui`, `shell_integration`,
  `plugin_host`.
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
