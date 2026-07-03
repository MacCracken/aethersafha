# aethersafha — Roadmap

> Milestone plan through v1.0. State lives in [`state.md`](state.md); this file
> is the sequencing — what ships, in what order, against which dependency gates.
> The bar is **parity with `rust-old/`** (the frozen 27,207-line Rust oracle).

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
- `shell_integration.cyr` (tray, window-mgmt requests, notification bridge)  ⬜ next
- Remaining: deeper behavioral parity tests per module; wire modules into the
  compositor/shell surface.

### M3 — Renderer + compositor depth (v0.3.0)
- Damage tracking, scene graph, decorations, bitmap text (`renderer.rs` full).
- Input routing to focused surface, drag/resize state machines, workspaces.
- Native Wayland protocol surface (`wayland/{types,protocol,server,popups}.rs`)
  — the compositor's own job; incremental, one protocol object at a time.

### M4 — Apps + capture + plugins (v0.4.0)
- `apps.cyr` (Terminal allowlist exec, FileManager, AgentManager, ModelManager, AuditViewer).
- `screen_capture.cyr` / `screen_recording.cyr` (permission + rate-limit + ring buffer).
- `plugin_host.cyr` (Unix-socket IPC, sandbox profiles, capability grants).
- HUD widgets (`hud/{gpu,domain,crew}_status.cyr`) — HTTP polling of daimon MCP.

### M5 — mehman (XWayland successor) (v0.5.0+)
- Wire mehman guest lifecycle once its sandbox/surface/shim modules ship.
- Host foreign-ABI app surfaces in a kavach sandbox → native surface handoff.

## Known cleanup
- **agnostik/agnodrm `ERR_*` overlap**: both dist bundles carry the shared AGNOS
  error module → duplicate-symbol warnings ("last wins", benign today). Resolve
  when first consumed (selective `modules = [...]` or upstream ownership split).
- `cyrius lib sync --full` is required before `cyrius deps` because the declared
  stdlib set exceeds the incremental pin. Documented in CLAUDE.md quick-start.

## Out of scope (for v1.0)
- Rust `system_tests.rs` port (verification code, not runtime) — re-expressed as
  `.tcyr` suites per module instead.
- GPU acceleration — the software renderer is the v1.0 path. **mabda 4.0.2 is
  already Cyrius-ported** (`dist/mabda.cyr`, also folded into the cyrius stdlib as
  `lib/mabda.cyr`); wire it via `[deps.mabda]` (or a `gpu` build path) when
  hardware acceleration is wanted. No blocker — just deferred.
