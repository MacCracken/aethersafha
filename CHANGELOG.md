# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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

### Changed

- **Toolchain 6.3.36 → 6.3.37; bhumi 0.7.0 → 1.0.0** (API-compatible bump).
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
