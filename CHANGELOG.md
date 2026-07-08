# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.6.0] - 2026-07-08 — native display protocol (setu server + frame-loop integration)

### Added

- **The setu server — aethersafha serves the native display protocol, wired into
  the real frame loop.** The full compositor side of the sovereign
  dhancha ↔ aethersafha wire (`setu`), built incrementally and proven end-to-end
  on Linux:
  - **`src/setu_server.cyr`** — the setu socket transport (AF_UNIX; fail-closes
    on agnos): `setu_srv_listen` / `accept` / `setu_srv_read_msg`
    (length-from-header framing) / `setu_srv_read_exact` / `setu_srv_write_frame`.
  - **`src/setu_dispatch.cyr`** — message → compositor state: `CREATE_SURFACE` →
    a real `comp_create_window` `Window` + `SURFACE_CREATED` reply; `ATTACH` +
    inline BGRA payload + `COMMIT` received byte-exact and blitted into the bhumi
    framebuffer, the framebuffer region verified against the source
    (`setu_srv_serve_present` / `serve_session` / `serve_accepted`).
  - **`src/main.cyr`** — the real compositor frame loop now stands up a
    **non-blocking** setu listener and, each frame, accepts + composites client
    presents on top of its desktop. `AF_UNIX` fail-closes on agnos, so the setu
    block is skipped there and the loop keeps its bounded agnos form untouched.
  - Adds a dependency on the new **`setu` 0.1.0** contract lib, and blits client
    surfaces into the bhumi framebuffer.
  - Proven on Linux: `build/aethersafha` accepting a real dhancha `DhClient`,
    minting its `Window`, and compositing its rendered widget tree — the display
    slice end-to-end (the final kernel-scanout hop, `bhumi_backend_present` →
    `fbinfo#38`/`blit#39`, is agnos-only).

### Changed

- **Direction: Wayland refused — the native protocol is sovereign, not a port.**
  aethersafha's client↔compositor protocol is redefined as a native,
  first-principles display protocol; the Rust `wayland/` surface (~3360 lines) is
  **retired, not ported**, and "Bite F" is redefined from "reimplement Wayland in
  Cyrius" to "design + build the native protocol." See
  [`docs/adr/0001-native-display-protocol.md`](docs/adr/0001-native-display-protocol.md)
  and the ecosystem pivot in `agnosticos/docs/design-patterns.md`. Docs
  (`README`, `docs/architecture/overview.md`, `roadmap.md`, `parity-plan.md`,
  `state.md`) reconciled to the native direction. No code change.

## [0.5.0] - 2026-07-03 — built-in apps (Bite C1 + C2)

### Added

- **Built-in apps (`src/apps.cyr`, Bite C1 + C2)** — port of the app framework, data-model
  apps, and the process-spawn bodies from Rust `apps.rs` (2986 lines).
  - **C1** — `AppError` / `AppType` / `AppWindow`, the **`DesktopApplications`** aggregate
    (open / close / list windows, live sub-app getters), the data-model apps
    **FileManager** / **AgentManager** / **AuditViewer** / **ModelManager**, the 8
    **WebBrowser** factory configs, **Shruti**, and the **Terminal** security surface — the
    30-program allowlist + `Path::file_name()`-faithful **basename-strip** (path-traversal
    neutralisation) + `split_whitespace` tokenizer.
  - **C2** — the real **process spawn**: `Terminal.execute_command` resolves the allowlisted
    bare name via a PATH search, fork+execve's it (direct + unsandboxed like Rust — the
    allowlist is the security control), captures stdout, and maps the real `WEXITSTATUS` to
    `Ok(stdout)` / `WindowError`; `WebBrowser` / `Shruti` `launch` guard on `is_installed`
    then detached-spawn with a merged (inherited + override) environment.
  The filesystem / network effect bodies (`list_agents`, `get_logs`, the model gateway) remain
  stubbed to their clean-env fallback, deferred to **C3**. `tests/apps.tcyr`
  (**133 assertions** — incl. real `echo` / `true` / `false` execution + launch guards; all
  green). Standalone; compositor wiring is follow-on.

### Changed

- **Toolchain `6.3.42` → `6.3.43`** — matches the installed `cycc`; refreshes the vendored
  `lib/` snapshot.

## [0.4.2] - 2026-07-03 — screen capture + recording (Bite D)

### Added

- **Screen capture (`src/screen_capture.cyr`, Bite D1)** — port of Rust
  `screen_capture.rs`: a `ScreenCaptureManager` with a per-agent **permission model**
  (grant/revoke/list/get, allowed-target kinds, expiry), **sliding-window rate
  limiting** (per-permission `max_captures_per_minute`), **secure-mode** +
  system-vs-agent authorization, a **capture-history ring buffer** (cap 100), and
  full-screen / **region** (clamped, saturating) / **window** capture off a
  caller-supplied bhumi framebuffer. Includes byte-exact **RAW / BMP / PNG encoders**
  (hand-rolled Adler-32 + CRC-32 + zlib STORED deflate). `tests/screen_capture.tcyr`
  mirrors the Rust unit tests (**90 assertions**, all green). Not yet wired into the
  compositor surface (follow-on, like the M2 leaf modules).
- **Screen recording (`src/screen_recording.cyr`, Bite D2)** — port of Rust
  `screen_recording.rs`, built on D1: a `ScreenRecordingManager` with recording
  **sessions** (config: target / format / frame-interval / `max_frames` / `max_duration`),
  a start → capture-frame → pause/resume → stop **state machine**, a per-session frame
  **ring buffer** (cap 100; `frame_count`/`total_bytes` count all frames ever), and
  one-recording-per-agent enforcement. `capture_frame` delegates to D1's
  `scap_mgr_capture` and wraps the result as a `RecordedFrame`; `max_frames` /
  `max_duration` are hard pre-capture limits and caps use `-1 == None` (so `Some(0)` is
  distinct). `tests/screen_recording.tcyr` mirrors the 22 Rust unit tests
  (**72 assertions**, all green). Standalone; **Bite D (capture + recording) complete**.

### Changed

- **mehman `0.3.1` → `1.0.0`** — API-compatible for the consumed
  `types`/`surface`/`sandbox` modules (the 1.0.0 delta only *adds* mehman's per-ABI
  `guest`/`shim` modules). Foreign capture + presentation unchanged and still green.
- **Toolchain `6.3.40` → `6.3.42`** — matches the installed `cycc`; refreshes the
  vendored `lib/` snapshot (sankoch 2.4.9).

## [0.4.1] - 2026-07-03 — foreign guest surface presentation

### Added

- **Foreign windows now show their guest's output** — the mehman "swallow" loop is
  visible end to end. A hosted guest's captured stdout is presented as the window's
  content: new `render_foreign_content` / `render_desktop_foreign` (`src/foreign.cyr`)
  paint the captured surface buffer into the window body via a new line-aware
  `draw_text_lines` (`src/render.cyr`, honors `\n`). The `desktop` aggregate now
  tracks its hosted foreign apps (`DSK_FOREIGN` vec + `desk_foreign` /
  `desk_foreign_count`), and the `main` frame loop presents them each frame
  (`render_desktop_foreign` after `render_desktop`). `main` hosts + runs a live
  `/bin/echo` guest so its window shows real output. Stdout-as-framebuffer MVP
  (mehman ADR 0004); real XRGB pixel fidelity is the next step. `tests/foreign.tcyr`
  gains a presentation group with a pixel-level assertion that the captured text
  paints the window body (**23 assertions**, all green).

### Changed

- **Foreign guests are captured, not just run.** `foreign_run` uses
  `mehman_sandbox_capture_guest(guest, surface, &exit)` — it runs the guest in a
  kavach PROCESS sandbox (real fork+exec+reap) **and captures its stdout into the
  hosted window's surface buffer** (the M2 handoff).
- **mehman `0.2.1` → `0.3.1`** (0.4.0 shipped 0.2.1; `0.3.0` added the capture API,
  `0.3.1` is the current pin); **toolchain `6.3.39` → `6.3.40`.**
- **`scripts/version-bump.sh`** rewritten for the Cyrius layout — bumps `VERSION` +
  `cyrius.cyml [package].version` with a post-write self-check (the stale Rust-era script
  targeted a nonexistent root `Cargo.toml`, ran `cargo check`, and never touched
  `cyrius.cyml`, so it would crash mid-run and leave the manifest un-bumped).

## [0.4.0] - 2026-07-03 — mehman foreign-app hosting

Wires in **mehman** (the XWayland-successor "swallow" backend) as a real dependency —
foreign-app hosting *and* execution, end to end.

### Added

- **mehman foreign-app hosting + execution.** `src/foreign.cyr` — the "swallow" seam
  onto **mehman** 0.2.1 + **kavach** 3.6.0: builds a sandboxed-guest spec (swallow
  caps) + an XRGB8888 foreign-surface descriptor; `desktop_host_foreign` registers a
  compositor window backed by it; and **`foreign_run` executes the guest in a kavach
  PROCESS sandbox** (real fork+exec+reap, returns `MehmanError.OK`). `main` hosts a
  demo foreign `xterm` and runs a benign guest. Required declaring the full TLS/crypto
  stdlib cascade for kavach→sandhi (net/sandhi/thread_local/tls*/sha1/keccak/sigil/
  sakshi/…) + `[deps.kavach]` explicitly. `tests/foreign.tcyr` (11 assertions, incl.
  live guest execution). Toolchain pin → 6.3.39.

## [0.3.0] - 2026-07-03 — kashi fonts + desktop wiring

Adds the `kashi` font dependency (real bitmap text) and completes the B3 desktop
wiring (shell status panel, theme-driven background).

### Added

- **Bitmap text** — `draw_char`/`draw_text`/`text_width` in the renderer, backed by
  the **kashi** font subsystem (`[deps.kashi]` 1.0.2, freestanding `font_data.cyr`
  core — IBM VGA 8×16 glyphs). **Window titles now render** in their titlebars.
  Pixel-level test verifies glyph blitting. (Replaced an initial hand-rolled 5×7 font.)
- **B3 wiring completed** — a **shell status panel** rendered from the desktop shell
  (cpu/mem/battery bar-graphs, net-status dot, notification badge; `render_shell_panel`
  + pure `panel_bar_w`), and **theme → renderer** (`render_desktop` clears to the
  theme's high-contrast background via `desk_bg_color`, then paints windows + panel).
  All 8 leaf subsystems are now wired into the running frame.

## [0.2.0] - 2026-07-03 — parity milestone

Compositor + renderer depth, the full M2 leaf-module set, B3 wiring (desktop
aggregate), Bite A window interaction (decorations + input routing), and the
sovereign-dependency de-collision (agnostik/agnodrm/aegis) landed on top of the
0.1.0 port.

### Added

- Compositor depth: workspaces + context types, move-window-to-workspace,
  switch-workspace, secure + agent-aware modes, window-at-point hit-testing.
- Renderer depth: alpha blend (`rend_blend`) + damage tracking (bounding-box
  `DamageTracker`).
- Ported `shell_integration` + `plugin_host` — completes all 8 M2 leaf modules.
- Behavioral parity test suites for all 8 leaf modules (~670 assertions, all green).
- **B3 wiring**: a `desktop` aggregate owns the compositor + all 8 leaf managers and
  is instantiated by `main`, so the subsystems are reachable + running. First live
  cross-subsystem connection — compositor → accessibility
  (`desktop_sync_accessibility` mirrors the window stack into the a11y tree).
  `tests/desktop.tcyr` (14 assertions).
- **Bite A**: window **decorations** — close/maximize/minimize titlebar buttons +
  `deco_hit` decoration hit-test (body/titlebar/buttons/resize edges). **Input
  routing** — window-management keyboard shortcuts (Tab focus-cycle, F4 close, F5
  maximize-toggle, F6 minimize) via a pure `input_map` + `input_apply`, wired into
  the frame loop. `tests/render.tcyr` (22) + `tests/input.tcyr` (13).

### Changed

- **Toolchain 6.3.36 → 6.3.38; bhumi 0.7.0 → 1.0.0** (API-compatible bump).
- **Dependency de-collision + re-enable.** agnostik + agnodrm namespaced their error
  families (`STIK_ERR_*` / `DRM_ERR_*`) to end the `ERR_*` symbol collision — cut as
  **agnostik 1.3.3** + **agnodrm 1.4.5** — and are now active deps (reviewed stdlib:
  `+trait`, `+ct`). Active deps: **bhumi, agnostik, agnodrm**. The one downstream
  consumer, `aegis`, was migrated to the new names + cut as 1.1.3.
- **mehman deferred to Bite G.** Cyrius stdlib is opt-in (declare what each dep needs);
  reviewing mehman showed its `[deps.kavach]` → sandhi → the full `tls_native` TLS
  stack, too large a surface for a types-only, unused dep. Re-enable when the
  compositor actually hosts guests.

## [0.1.0] - 2026-07-02 — Cyrius port

First release of the Cyrius incarnation of aethersafha. The prior Rust crate is
frozen at `rust-old/` as the parity oracle (see the pre-port baseline below).

### Changed

- **Ported Rust → Cyrius via `cyrius port`.** The 27,207-line Rust tree moved to
  `rust-old/` (frozen parity oracle); the project is now a Cyrius crate pinned to
  toolchain 6.3.35 in `cyrius.cyml`.
- **Replaced the Wayland platform stack with sovereign AGNOS backends**: bhumi
  0.7.0 (platform I/O — DRM/KMS + libinput + logind → `output`/`input`/`seat`),
  mehman 0.1.0 (foreign-surface hosting — the XWayland successor, post-MVP).
  Native Wayland protocol dispatch stays in aethersafha proper.
- **Dependency mapping**: `agnostik` → agnostik 1.3.2 (Cyrius); `agnosys` →
  agnodrm 1.4.4 (agnosys decomposed 2026-06-19, device model → agnodrm);
  GPU (`mabda`) deferred — software renderer is the v1.0 path.

### Added

- Foundational compositor base on the bhumi seam: `geom`, `window`, `compositor`,
  `render` (software renderer over the bhumi XRGB framebuffer), `input`
  (bhumi HID → actions), and a `main` frame loop. Compiles + runs.
- M2 leaf-module parity batch (structural parity vs `rust-old/`, prefixed symbols,
  compiling + smoke-tested): `theme_bridge`, `gestures`, `accessibility`,
  `ai_features`, `shell`, `security_ui`.
- `tests/aethersafha.tcyr` (21 core assertions) + `tests/leaf_modules.tcyr`
  (11 leaf-coexistence assertions) — 32 green.
- Parity roadmap (`docs/development/roadmap.md`) mapping every Rust module to its
  Cyrius target, backend binding, and milestone (M1–M5).
- Toolchain pin advanced to 6.3.36.

### Notes

- Structural parity for the M2 leaf batch (compiles + smoke-tested); deeper
  behavioral parity tests against `rust-old/` are the next increment.
- Known: agnostik + agnodrm both bundle the shared `ERR_*` module → benign
  duplicate-symbol warnings ("last wins"). See roadmap "Known cleanup".

## Pre-port Rust baseline - 2026-04-01

_Not a Cyrius release — the extracted Rust crate that the 0.1.0 Cyrius port
targets for parity. Source frozen at `rust-old/`._

- Initial extraction from `agnosticos/userland/desktop-environment/`
- Wayland compositor with backend abstraction (`WaylandBackend`)
- AI desktop features — context-aware suggestions, agent HUD
- Desktop shell — app launcher, notifications, quick settings
- Renderer — scene graph, damage tracking, decorations, high-contrast
- Accessibility — AccessibilityTree, tab navigation, announcements
- Plugin host — sandboxed plugin loading, capability system
- XWayland manager — surface mapping, property translation
- Shell integration — tray, window management, notification bridge
- Theme bridge — AGNOS→Flutter ThemeData, platform channel
- Desktop applications — web browser, file manager, terminal, model manager
- Screen capture — per-agent permissions, rate limiting, PNG/BMP/raw encoding
- Screen recording — frame-by-frame, poll-based streaming, ring buffer
- Security UI — permission dialogs, threat alerts, agent dashboard
- Gesture recognition system
- HUD overlays — crew status, domain filter, GPU status
- Criterion benchmarks for compositor and screen capture
