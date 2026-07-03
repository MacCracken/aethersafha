# aethersafha — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**0.4.2** (2026-07-03) — **Bite D (screen capture + recording) complete**: `screen_capture`
(D1 — per-agent permission model + sliding-window rate limiting + full-screen/region/window
framebuffer capture + byte-exact RAW/BMP/PNG encoders; 90 assertions) and `screen_recording`
(D2 — recording sessions + start/capture/pause/resume/stop state machine + frame ring buffer,
built on D1; 72 assertions). Also: mehman `0.3.1` → `1.0.0`; toolchain `6.3.40` → `6.3.42`.

Built on **0.4.1** (foreign guest surface presentation), **0.4.0** (mehman foreign-app
hosting + kavach-sandboxed guest execution — the XWayland-successor path), 0.3.0 (kashi fonts,
B3 desktop wiring), and the 0.2.0 parity milestone. Ported from Rust via `cyrius port`; 27,207
lines preserved at `rust-old/` as the parity oracle.

## Toolchain

- **Cyrius pin**: `6.3.42` (in `cyrius.cyml [package].cyrius`)
- Build: `cyrius lib sync --full && cyrius deps && cyrius build src/main.cyr build/aethersafha`
  (the `lib sync --full` is required — the declared stdlib set exceeds the incremental pin).

## Source

- Rust reference: 27,207 lines at `rust-old/` (frozen, do not edit).
- Cyrius port: **19 modules**; compiles clean + runs on the bhumi seam.
  - **Core (M1/Bite A)** — `geom`, `window`, `compositor`, `render`, `input`, `main`.
    compositor: workspaces + context types + move/switch + secure/agent-aware modes
    + window-at-point hit-test; renderer: alpha blend + damage tracking + window
    **decorations** (close/max/min buttons + `deco_hit`) + **bitmap text** via the
    **kashi** IBM VGA 8×16 console font (`draw_char`/`draw_text`; window titles render);
    input: **window-management shortcuts** (Tab focus-cycle, F4 close, F5 maximize-
    toggle, F6 minimize) via `input_map`/`input_apply`.
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
  - **Foreign (M5, mehman)** — `foreign` hosts a foreign-ABI app as a kavach-sandboxed
    guest, runs + **captures** its stdout (`mehman_sandbox_capture_guest`), and
    **presents** the captured surface as the hosted window's content
    (`render_desktop_foreign` / `render_foreign_content`, painted after `render_desktop`).
  - **Screen capture (M4/Bite D1, standalone)** — `screen_capture`
    (`ScreenCaptureManager`): per-agent permission model + sliding-window rate limiting +
    secure-mode authorization + capture-history ring buffer + full-screen/region/window
    capture off a bhumi framebuffer + byte-exact RAW/BMP/PNG encoders (hand-rolled
    Adler-32/CRC-32/zlib-STORED). Not yet wired into the compositor surface.
  - **Screen recording (M4/Bite D2, standalone)** — `screen_recording`
    (`ScreenRecordingManager`), built on D1: recording sessions (target/format/interval/
    `max_frames`/`max_duration`), a start → capture-frame → pause/resume → stop state
    machine, a per-session frame ring buffer (cap 100; `frame_count`/`total_bytes` count
    all frames ever), one-recording-per-agent, and `capture_frame` → `scap_mgr_capture` →
    `RecordedFrame`. Caps use `-1 == None` (so `Some(0)` is distinct). Not yet wired.
  - **Apps (M4/Bite C1, standalone)** — `apps` framework (`AppError`/`AppType`/`AppWindow`
    + the `DesktopApplications` aggregate: open/close/list windows, live sub-app getters)
    + data-model apps (FileManager, AgentManager, AuditViewer, ModelManager) + 8 WebBrowser
    factory configs + Shruti + the **Terminal** allowlist + path-traversal basename-strip.
    Process-spawn (C2) and fs/net effect bodies (C3) stubbed to clean-env fallback. Not wired.

## Tests

- **17 `.tcyr` files, all green.** Core: `aethersafha` (38), `render` (34 — decoration
  hit-test + shell-panel bars + bitmap text pixel test), `input` (13), `leaf_modules`
  (11), `desktop` (15). mehman: `foreign` (23 — guest spec/surface + host-as-window +
  sandboxed run + capture + **presentation pixel test**). capture: `screen_capture`
  (90 — permissions / rate-limit / secure-mode / region-clamp / window / history +
  RAW/BMP/PNG encoder checksums), `screen_recording` (72 — session lifecycle / state
  machine / frame + duration limits / ring buffer / one-per-agent / queries). apps:
  `apps` (125 — framework / data-model apps / aggregate open-close-list / Terminal
  allowlist + basename security). Behavioral per-module: `theme_bridge`, `gestures`,
  `accessibility`, `ai_features`, `shell`, `security_ui`, `shell_integration`, `plugin_host`.
- Run: `cyrius tests tests/` (or a single `cyrius test tests/<file>.tcyr`).

## Dependencies

- **stdlib** (auto-prepended) — syscalls, string, alloc, atomic, fmt, vec, str,
  slice, hashmap, fnptr, io, fs, process, args, tagged, result, chrono, math,
  assert, bench.
Active (auto-prepended; stdlib declared per each dep's reviewed needs):
- **bhumi** 1.0.0 — platform backend (output/input/seat).
- **agnostik** 1.3.3 — shared domain primitives (errors namespaced `STIK_ERR_*`).
- **agnodrm** 1.4.5 — udev/DRM device model (errors namespaced `DRM_ERR_*`).
- **kashi** 1.0.2 — bitmap console fonts (freestanding `font_data.cyr`, VGA 8×16).
- **mehman** 1.0.0 + **kavach** 3.6.0 — foreign-app "swallow" backend (the XWayland
  successor). Consumed via `src/foreign.cyr` (host → sandboxed run → capture → present);
  we pull only `types`/`surface`/`sandbox` (1.0.0 also ships per-ABI `guest`/`shim`
  modules, not yet consumed). Pulls the full TLS/crypto stdlib
  cascade (net, sandhi, thread_local, random, freelist, sync, async, fdlopen,
  dynlib, mmap, tls, tls_native*, sha1, keccak, sigil, sakshi — all declared in
  `[deps].stdlib`). `[deps.kavach]` is declared explicitly (its `Backend`/`config`/
  `sandbox_*` surface is named directly by mehman's sandbox module).

Deferred:
- **mabda** 4.0.2 — GPU, off the v1.0 path.

Opt-in stdlib: `cyrius build` prepends every `[deps.*]` module, so each dep's
stdlib needs must be declared in `[deps].stdlib` (reviewed from its `dist/*.deps`
sidecar + referenced symbols). That's by design — it keeps the dependency surface
visible, not a bug.

## Consumers

_None yet (top-level application, `publish = false`)._

## Next

**Bite C started** — **C1** (app framework + data-model apps + aggregate + the Terminal
security logic) is ported + tested (125 assertions). Next on the apps track: **C2** (the
process-spawn bodies — Terminal exec, browser/Shruti launch — through the kavach/mehman
sandbox seam, a security-reviewed sub-bite) and **C3** (the fs/net effect bodies —
agent-socket scan, audit-log parse, model gateway). Also complete + awaiting wiring: Bite D
(capture + recording). Other large unported layers: HUD widgets (Bite E), the native
Wayland protocol surface (Bite F, highest-risk). **mehman track (Bite G)**: consume 1.0.0's
per-ABI `guest`/`shim` + real XRGB pixel fidelity (mehman ADR 0004). See
[`roadmap.md`](roadmap.md) / [`parity-plan.md`](parity-plan.md).
