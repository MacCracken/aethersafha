# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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
