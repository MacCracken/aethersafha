# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased] — Cyrius port

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

## [0.1.0] - 2026-04-01

### Added

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
