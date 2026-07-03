# aethersafha — Roadmap

> Milestone plan through v1.0. State lives in [`state.md`](state.md); the
> executable bite-level breakdown (with workflow catalog) lives in
> [`parity-plan.md`](parity-plan.md). This file is the sequencing — what ships,
> in what order, against which dependency gates. The bar is **parity with
> `rust-old/`** (the frozen 27,207-line Rust oracle).

## v1.0 criteria

- [ ] Rust → Cyrius surface parity verified (module-by-module against `rust-old/`)
- [ ] Test coverage adequate for the surface area (≥80% target)
- [ ] Benchmarks captured in `docs/benchmarks.md`
- [ ] Runs on the agnos kernel via bhumi (real scanout + input)
- [ ] CHANGELOG complete from v0.1.0 onward
- [ ] Security audit pass (`docs/audit/YYYY-MM-DD-audit.md`)

## Backend seam (the wayland → bhumi/mehman split)

| Rust concern | Cyrius home | Notes |
|---|---|---|
| DRM/KMS + libinput + logind (platform I/O) | **bhumi** 0.7.0 | `bhumi_backend_open/fb/poll/present`, seat/cap gating. MVP. |
| Wayland protocol dispatch, surface tree, window mgmt | **aethersafha** | The compositor's *native* language — stays here. |
| XWayland (foreign-app surface hosting) | **mehman** 0.1.0 | kavach-sandboxed guests. Post-MVP (types-only today). |
| Shared domain primitives / errors / wire format | **agnostik** 1.3.2 | was Rust `agnostik`. |
| udev + DRM/KMS device model | **agnodrm** 1.4.4 | was Rust `agnosys` (decomposed 2026-06-19). |

## Milestones

### M0 — Port scaffold (v0.1.0) — ✅ 2026-07-03
- `cyrius port` ran; Rust → `rust-old/`; Cyrius scaffold + docs + CI.

### M1 — Foundational base (v0.1.x) — ✅ in progress
Compiling, tested compositor core on the live bhumi seam:
- `src/geom.cyr` — Rectangle primitives.  ✅
- `src/window.cyr` — Window model + WinState.  ✅
- `src/compositor.cyr` — window stack, focus, workspace, CRUD.  ✅
- `src/render.cyr` — software renderer over the bhumi XRGB framebuffer.  ✅
- `src/input.cyr` — bhumi HID → compositor input actions.  ✅
- `src/main.cyr` — entry: open bhumi backend, seed windows, frame loop.  ✅
- `tests/aethersafha.tcyr` — 21 core assertions green.  ✅

### M2 — Leaf feature parity (v0.2.0) — 🚧 first batch landed
Self-contained data-model modules (no deep compositor/bhumi coupling), ported
module-by-module against `rust-old/` (heap offset-accessor structs, module-prefixed
symbols), each compiling + smoke-tested. Driven by the parity workflow.
- `theme_bridge.cyr` (AGNOS→Flutter theme translation)  ✅ ported + smoke
- `gestures.cyr` (tap/swipe/pinch recognition)  ✅ ported + smoke
- `accessibility.cyr` (a11y tree, focus/keyboard nav, high-contrast theme)  ✅
- `ai_features.cyr` (context engine, suggestions, agent HUD, resource metrics)  ✅
- `shell.cyr` (notifications, quick settings, system status, launcher)  ✅
- `security_ui.cyr` (permission model, alerts, dashboard)  ✅
- `shell_integration.cyr` (tray, window-mgmt, notification bridge)  ✅ ported
- `plugin_host.cyr` (lifecycle, sandbox profiles, capabilities; IPC stubbed)  ✅ ported
- Behavioral parity tests for all 8 leaf modules  ✅ (~670 assertions green)
- **B3 wiring** — `desktop` aggregate owns compositor + all 8 leaf managers, created
  by `main`. `render_desktop` = themed bg + windows + shell status panel. Live links:
  compositor→accessibility, theme→renderer, shell→renderer, tray.  ✅ complete
- **Bitmap text** — `draw_char`/`draw_text` over the **kashi** VGA 8×16 console font
  (`[deps.kashi]`); window titles render in titlebars.  ✅
- Remaining (feature depth): notifications surface, input→gestures, quick-settings,
  panel text labels (cpu/mem %), scene-graph/damage-driven redraw.

### M3 — Renderer + compositor depth (v0.3.0)
- Damage tracking, scene graph, decorations, bitmap text (`renderer.rs` full).
- Input routing to focused surface, drag/resize state machines, workspaces.
- Native Wayland protocol surface (`wayland/{types,protocol,server,popups}.rs`)
  — the compositor's own job; incremental, one protocol object at a time.

### M4 — Apps + capture + plugins (v0.4.0)
- **`apps.cyr` 🚧 C1+C2 done** — app framework + data-model apps + the Terminal allowlist/basename
  security logic + the **real process spawn** (Terminal fork+execve capturing stdout/exit-status;
  browser/Shruti detached launch); 133-assertion test. fs/net effect bodies (C3) deferred.
- **`screen_capture.cyr` ✅** — permission model + rate-limit + secure-mode auth + history
  ring buffer + full/region/window capture + byte-exact RAW/BMP/PNG encoders; 90-assertion
  parity test.
- **`screen_recording.cyr` ✅** — recording sessions + start/capture/pause/resume/stop state
  machine + per-session frame ring buffer (on D1); 72-assertion parity test. Both standalone
  (compositor wiring is follow-on).
- `plugin_host.cyr` (Unix-socket IPC, sandbox profiles, capability grants).
- HUD widgets (`hud/{gpu,domain,crew}_status.cyr`) — HTTP polling of daimon MCP.

### M5 — mehman (XWayland successor) — 🚧 started (v0.5.0+)
- mehman 1.0.0 + kavach 3.6.0 **wired** via `src/foreign.cyr`: guest-spec +
  foreign-surface descriptor + `desktop_host_foreign` → a compositor window;
  `main` hosts a demo guest.
- Guest **execution + capture** via `foreign_run` → `mehman_sandbox_capture_guest`
  (kavach PROCESS fork+exec+reap + surface capture) — done + tested against a live
  `/bin/true` and `/bin/echo` (captured output lands in the surface buffer).
- **Presentation** — `render_desktop_foreign` / `render_foreign_content` draw the
  captured surface as the hosted window's content (line-aware `draw_text_lines`); the
  desktop tracks hosted foreign apps (`desk_foreign`); pixel-tested. ✅
- Remaining: consume mehman 1.0.0's per-ABI `guest`/`shim` modules; real XRGB pixel
  fidelity beyond the stdout-as-framebuffer MVP (mehman ADR 0004).

## Known cleanup
- **Deferred deps** (mehman / agnostik / agnodrm): `cyrius build` auto-prepends
  every `[deps.*]` module, so these unused-but-heavy bundles broke the build —
  mehman→`[deps.kavach]` drags in `sandhi_server_*`/`thread_local_*` (reachable-
  undefined), agnostik+agnodrm collide on `ERR_*`. Deferred (mapping kept in the
  manifest). Re-enable each with a selective `modules = [...]` subset when the
  code that needs it lands (mehman at Bite G; agnostik/agnodrm/mabda as consumed).
- `cyrius lib sync --full` is required before `cyrius deps` (the declared stdlib
  set + bhumi's needs exceed the incremental pin). Documented in CLAUDE.md.

## Out of scope (for v1.0)
- Rust `system_tests.rs` port (verification code, not runtime) — re-expressed as
  `.tcyr` suites per module instead.
- GPU acceleration — the software renderer is the v1.0 path. **mabda 4.0.2 is
  already Cyrius-ported** (`dist/mabda.cyr`, also folded into the cyrius stdlib as
  `lib/mabda.cyr`); wire it via `[deps.mabda]` (or a `gpu` build path) when
  hardware acceleration is wanted. No blocker — just deferred.
