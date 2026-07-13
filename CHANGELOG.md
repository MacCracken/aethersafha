# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.9.4] - 2026-07-12 — theme tokens extracted to the shared rupa lib

The desktop theme system gained its second consumer (dhancha), so the tokens move out of
the compositor into a shared leaf lib — **rupa** (रूप, "form / appearance") — that the
compositor chrome, the widget toolkit, and apps all read. A toolkit/app cannot depend on
the compositor, so the single source of truth has to live below both. Same look, same
MUDRA · Carbon default; only the token *home* moved.

### Changed

- **`src/theme.cyr` slimmed to the compositor-specific packer.** The `AeTheme` struct, the
  four MUDRA / SHANTA grounds, the accessors, the by-name registry, and the single active
  theme all moved to **rupa 0.1.0**. This file now holds only `ae_bhumi(c)` — packing a
  logical rupa `0xRRGGBB` token into the bhumi framebuffer colour via `rupa_color_r/g/b`.
- **`src/render.cyr`** now reads the tokens through `rupa_theme_*` (`rupa_theme_active` /
  `_widget` / `_panel` / `_accent` / `_ink` / `_mute` / `_faint` / `_alert` / `_held` /
  `_bg`); `ae_bhumi` still does the pack. No visual change — MUDRA · Carbon stays the
  out-of-box look.
- **`tests/theme.tcyr`** is now an *integration* test (rupa tokens reachable from the
  compositor + `ae_bhumi` packs correctly, 6 assertions); the 39 token-value assertions
  moved to rupa's own `tests/theme.tcyr`.

### Added

- **`[deps.rupa]`** (`0.1.0`) — the shared theme-token core. Both targets build identically
  (host 15,066,680 B / agnos 14,999,608 B) and the full suite stays green (20 files, 1071
  assertions, 0 failed).

## [0.9.3] - 2026-07-12 — repin to cyrius 6.4.61 (setu-listener accept loop is now leak-free)

The compositor's frame loop polls a **non-blocking `setu_srv_accept`** every frame
(`main.cyr`), and `net.cyr`'s `sock_accept` used to allocate on every would-block poll —
a bump-heap leak that would grow over a long desktop session. **cyrius 6.4.61** fixes it
(`accept(NULL, NULL)` for the never-read peer address + a shared `_net_eagain()` `Err`
singleton for the would-block path, regression-gated by cyrius). This patch repins to
receive the fix; the setu listener's accept loop no longer drips.

### Changed

- **cyrius pin 6.4.34 → 6.4.61** — picks up the `net.cyr` `sock_accept` per-poll
  alloc-leak fix (consumer-filed, fixed upstream). Both targets rebuild identically
  (host 15,066,680 B / agnos 14,999,608 B) and the full test suite stays green
  (20 files, 1104 assertions, 0 failed).
- **Formatting baseline refreshed to 6.4.61.** The newer formatter reformats a handful
  of files that were clean under 6.4.34 (`ai_features`, `gestures`, `security_ui`,
  `shell`, `shell_integration`, `main` + three test files) — mechanical whitespace only,
  no logic change, so CI's fmt gate stays green on the new pin.

## [0.9.2] - 2026-07-12 — the sovereign desktop theme system (MUDRA / SHANTA)

Six divergent aesthetic explorations were consolidated to **two desktop themes**, unified by the
question every one of them was really answering — *how do you make trust visible?* **MUDRA**
(मुद्रा, "the seal") is the sovereign default (signed, radius-0, cyan verify-seal; trust reads
loud); **SHANTA** (शान्त, "stillness") is the focus mode (warm, thin, one gold firefly + sage;
trust reads quiet). Each ships in **dark and light** — four grounds from one token set. The
compositor chrome now reads the active theme instead of hardcoded colours, with MUDRA · Carbon
(dark) as the out-of-box look. Design doc: `agnosticos/docs/development/designs/desktop_consolidated/
theme-system.html`.

### Added

- **`src/theme.cyr` — the AGNOS desktop theme tokens.** An `AeTheme` struct (colours as
  `0xRRGGBB`) with slots `bg / panel / widget / line / ink / mute / faint / accent / alert / held`
  plus `radius` and `font` (permille). Four constructors — `ae_theme_mudra_dark` / `_mudra_light`
  / `_shanta_dark` / `_shanta_light` (exact hexes from the consolidated design) — full accessors,
  the `0xRRGGBB` channel helpers `ae_color_r/g/b`, and the framebuffer packer **`ae_bhumi(c)` →
  `bhumi_xrgb`**. A by-name registry (`ae_theme_by_name`, self-contained `ae_streq`) for config-
  driven selection, and the compositor's single **active theme** (`ae_theme_active` lazy-defaults
  to MUDRA · Carbon; `ae_theme_set_active` / `ae_theme_set_active_name`).
- **`tests/theme.tcyr`** — 39 assertions locking every ground's token values, the channel helpers,
  the registry, and the active-theme selector. All pass.

### Changed

- **`src/render.cyr` chrome now reads the active theme.** The window body, titlebar, title text,
  and decoration buttons — plus a new 2px **accent seal-strip** marking the focused window — and
  the desktop root background all resolve through `ae_theme_active()` / `ae_bhumi()`, replacing the
  hardcoded `bhumi_xrgb(...)` literals. Focused windows raise to `widget` + the accent strip;
  idle windows sit on `panel` with muted title text.
- `src/main.cyr` includes `src/theme.cyr` ahead of `src/render.cyr` (the renderer consumes it).

This is distinct from the legacy `theme_bridge.cyr` (the Rust-parity accessibility
HighContrastTheme → Flutter bridge), which stays for the high-contrast a11y profile. Follow-ups:
dhancha widgets + jalwa consuming the same tokens (the trigger to extract `theme.cyr` to a shared
theme lib), and SHANTA's radius-14 rounding + firefly + luminance-provenance in the renderer.

## [0.9.1] - 2026-07-10 — full key events (press + release) for held-key clients

The compositor now honours setu 0.5.0's `SETU_SURF_FULL_KEYS` opt-in **per surface**: a client
that requests it (via the `CREATE_SURFACE` flags) receives key **press AND release**, so it can
track HELD keys — a game holds a movement key and keeps moving. `setu_srv_forward_key` no longer
drops key-UP for such surfaces (the make/break rides the `mods` arg, 1 = press / 0 = release);
press-only clients (crab, present_probe) are byte-identical to before. Proven with cyrius-doom on
the sovereign desktop — a balanced **10 press / 10 release** over setu (`agnos
scripts/aethersafha-doom-input-smoke.sh`).

### Changed

- `[deps.setu]` pinned `0.3.1` → **`0.5.0`** (the full-key-events opt-in + `mods` make/break).
- `Window` gains a `W_KEYMODE` field; `setu_srv_recv_committed` captures the requested key mode
  from the `CREATE_SURFACE` flags at the handshake, and `setu_srv_forward_key` gates
  release-forwarding on it (press-only stays the default).

## [0.9.0] - 2026-07-10 — focus over setu + hosting a real dhancha app on the sovereign kernel

The compositor now routes **focus** to clients (not just keystrokes) and hosts a heterogeneous
desktop: the slim present_probe test client next to a real **dhancha widget app**. TAB cycles
focus, the compositor tells each affected client when it gains or loses focus, and the clients
render their own focus state — an interactive window manager on the sovereign kernel.

### Added

- **Focus over setu (`setu_srv_notify_focus`)** — when focus moves, the compositor sends
  `SETU_INPUT_FOCUS(id, 0)` to the window losing focus and `(id, 1)` to the one gaining it, so a
  client can render its own focus indicator (bright vs dim) without a keypress. A per-frame
  focus-change detector in `main.cyr` (compares `comp_focused` to the last-notified index) covers
  both TAB-driven focus and connect-time focus with one path.
- **Heterogeneous clients** — the compositor's second resident is now the **dhancha widget client**
  (`/bin/dhwidget`) rather than a second present_probe, so the desktop hosts a real toolkit app
  (titled window + labelled buttons) alongside the test-pattern client. Verified on agnos via
  `aethersafha-setu-smoke.sh` plus the new `setu-input-test.py` / `setu-focus-test.py` gates.

## [0.8.2] - 2026-07-09 — MULTI-WINDOW desktop + input routed over setu on the sovereign kernel

The compositor now hosts **multiple** setu clients as distinct windows AND routes keyboard
input to the focused one — the two steps that turn "a client composites" into "a desktop."
Both are proven on agnos via QEMU screendumps (`aethersafha-setu-smoke.sh` green) and a new
input-injection harness (`setu-input-test.py`, USB-xHCI `sendkey`).

### Added

- **Multi-window desktop** — `main.cyr` spawns TWO setu clients and cascades each accepted
  one to a distinct position (`setu_srv_serve_accepted_at(cfd, comp, fb, 30+n*330, 50+n*210)`,
  an `accepted` counter carried across frames). Screendump shows two independent windows, each
  animating its own shared buffer (distinct per-client bar colours keyed off the shm id).
- **Input over setu (the S→C forwarding leg)** — `setu_srv_forward_key(comp, ev)` sends a
  `SETU_INPUT_KEY(win_id, bhumi_key_usage(ev), 0)` to the FOCUSED window's client for each
  key-DOWN. Verified: injecting a key flips only the focused client (border/bar → white) while
  the unfocused client is untouched — focus-routed, not broadcast.

### Changed

- **The setu client connection is now PERSISTENT.** `Window` gained `W_CFD` (the client's
  tagged connection fd); `setu_srv_serve_accepted_at` stores it (`win_set_cfd`) and `main.cyr`
  no longer closes `cfd` after the present handshake — the compositor keeps it open to forward
  input. After COMMIT the socket is server→client only (compositor writes input, client reads),
  so it never contends with the shared-buffer present path.
- **The compositor focuses each client as it connects** (`comp_focus(comp, comp_count-1)`), so
  forwarded keys route to the most-recently-attached window.

## [0.8.1] - 2026-07-09 — the compositor composites a SHARED-BUFFER present on the sovereign kernel

The setu present goes **shared-buffer** (setu 0.3.1): the server reads a client's pixels
from a buffer referenced by id instead of an inline socket stream — the on-device unblock
(a hundreds-of-KB inline payload deadlocks the single-CPU two-proc path through the 2 KB
`TCP_RX_RING`). `aethersafha-setu-smoke.sh` is now **green on agnos**: a setu client connects,
presents a 320×192 frame, and the compositor composites it (`setu client CONNECTED +
PRESENTED + composited on agnos`). Linux e2e (`setu_serve_probe` + `present_probe`) unchanged.

### Changed

- **`setu_srv_recv_committed` reads the pixels from the client's shared buffer** — on an
  ATTACH with `buf_id > 0` it `setu_buf_read`s the surface (agnos → the kernel shm band
  `sys_shm_read`#73; Linux → the `/dev/shm` file) instead of `setu_srv_read_exact`ing an inline
  stream. The inline path stays as the `buf_id == 0` fallback.
- **setu dep → 0.3.1** (the shared-buffer protocol + backend); **cyrius pin 6.4.25 → 6.4.34**
  (the native `sys_shm_*` wrappers). Note: aethersafha's materialized stdlib needed a
  `cyrius lib sync --full` on the pin bump — `cyrius deps` alone left it stale.

## [0.8.0] - 2026-07-08 — the compositor speaks setu over TCP (sovereign display protocol, e2e)

The setu server transport goes **cross-platform** (item 3b of the road-to-desktop):
the compositor now accepts + composites real setu clients over **TCP loopback**
(`127.0.0.1 : 7700`) on Linux **and** on agnos, so the sovereign desktop runs on
the sovereign kernel — not just the host. Proven end-to-end: `puka` (client)
connects over TCP and presents a rendered 320×192 terminal frame → this
compositor accepts it (non-blocking poll) and composites it → a valid PPM with
real content. This depends on **setu 0.3.0** (the TCP transport).

### Added

- **`programs/setu_serve_probe.cyr`** — a fork-free e2e proof of the TCP
  transport: stands up the setu listener, polls the non-blocking accept until a
  client connects, serves one present (recv surface + blit), and dumps the
  composited window to a PPM. Run against `puka`'s `puka_setu_probe` as two
  separate processes — decoupling the transport proof from `fork`+`execve`
  (which a restricted dev sandbox can't host). Complements the fork-based
  `puka_launch_probe` (compositor-spawns-client), which now documents that
  limitation.

### Changed

- **`src/setu_server.cyr` delegates to setu's cross-platform transport.** The
  server half (`setu_srv_listen` / `setu_srv_accept` / `setu_srv_read_*` /
  `setu_srv_write_frame`) now forwards to setu's `setu_listen` / `setu_accept` /
  `setu_read_*` / `setu_send`, which speak TCP on both targets and absorb agnos's
  non-blocking-recv / partial-send quirks. The earlier AF_UNIX path (Linux-only,
  fail-closed on agnos) is gone — the compositor speaks the same wire on Linux
  AND agnos.
- **`src/main.cyr`** — the setu accept block composites clients over the TCP
  listener (`sock_close`, no socket-file unlink); the log line reflects
  `TCP loopback:7700`.

### Fixed

- **Linux crash in the setu accept poll (`sys_sleep_ms` is agnos-only).** The
  would-block yield in `setu_srv_accept_one` and the launch probe called the raw
  `sys_sleep_ms` (defined only in the agnos syscall module), which compiled on
  Linux but trapped (SIGILL) at runtime. Swapped to the portable `sleep_ms`
  (chrono) — `poll()` on Linux, `#41` on agnos. This unblocked the e2e proof.

## [0.7.0] - 2026-07-08 — renderer decoupled from the shell (reusable window chrome)

### Changed

- **`src/render.cyr` is now free of `shell.cyr` symbols — the core window
  renderer is reusable on its own.** The shell status-panel renderer
  (`render_shell_panel` + `panel_bar_w` / `panel_net_color` + the `PanelK`
  metrics), which coupled to `shell.cyr`'s `SystemStatus` + `SH_NetStatus`,
  moved out of `render.cyr` into a new **`src/shell_render.cyr`** — the one place
  the desktop shell's data model meets the framebuffer (the shell → render
  bridge). `render.cyr` now holds only shell-agnostic primitives: `fill_rect`,
  `rend_blend`, the damage tracker, `deco_*` decoration hit-testing, `render_window`
  / `render_frame`, and the kashi bitmap-text `draw_char` / `draw_text` /
  `draw_text_lines`. `main.cyr` gains the `shell_render.cyr` include;
  `desktop.cyr`'s `render_desktop` is unchanged. Pure module split — behavior is
  identical.
- **`tests/render.tcyr` no longer includes `shell.cyr`** — proving `render.cyr`
  stands alone. The panel bar-graph assertions moved to the new
  **`tests/shell_render.tcyr`**, which adds `panel_net_color` mapping checks and a
  `render_shell_panel` shell → framebuffer smoke test. **18 `.tcyr` suites green.**
- **`programs/puka_desktop_probe.cyr` now reuses `render_window` + `draw_text`**
  for native window chrome. Because `render.cyr` is decoupled, the probe includes
  it directly and frames each hosted setu client (puka, dhancha) with the real
  compositor titlebar — focus tint, traffic-light buttons, **and bitmap title
  TEXT** ("puka - terminal", "dhancha - files"). The previous inlined, fill-only
  `pdp_titlebar` (managed windows without title text) is gone. Verified end-to-end
  on Linux: two managed, **titled** windows composited to a PPM.

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
