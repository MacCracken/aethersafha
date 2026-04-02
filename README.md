# Aethersafha

**AI-augmented Wayland compositor for AGNOS.**

Aethersafha (Arabic: the surface/plane) is the desktop environment and Wayland compositor for the AGNOS operating system. It provides a composited desktop with AI-native features, agent integration, and security-first design.

## Features

- **Wayland compositor** — `wl_compositor`, `xdg_shell`, `wlr_layer_shell` protocol support
- **AI desktop features** — context-aware suggestions, agent HUD, resource metrics
- **Plugin host** — sandboxed plugin loading with capability-based security
- **XWayland** — X11 compatibility via surface mapping and property translation
- **Screen capture/recording** — per-agent permissions, rate limiting, multi-format encoding
- **Accessibility** — AccessibilityTree, tab navigation, screen reader announcements
- **Theme bridge** — AGNOS↔Flutter theme synchronization
- **Security UI** — permission dialogs, threat alerts, agent audit dashboard
- **HUD overlays** — crew status, domain filters, GPU monitoring

## Architecture

```
aethersafha
├── compositor    — Wayland backend, surface management, workspaces
├── renderer      — Scene graph, damage tracking, decorations
├── shell         — App launcher, notifications, quick settings
├── ai_features   — Context engine, suggestions, agent HUD
├── plugin_host   — Plugin lifecycle, sandbox profiles, capabilities
├── wayland/      — Protocol implementation (server, popups, types)
├── xwayland      — XWayland manager, surface mapping
├── accessibility  — AccessibilityTree, focus, announcements
├── screen_capture — Capture manager, permissions, encoding
├── screen_recording — Recording manager, frame buffer, streaming
├── security_ui   — Permission requests, threat dashboard
├── shell_integration — Tray, window mgmt, notification bridge
├── theme_bridge  — AGNOS→Flutter theme translation
├── apps          — Built-in apps (browser, file manager, terminal)
├── gestures      — Touch/trackpad gesture recognition
└── hud/          — HUD overlays (crew, domain, GPU)
```

## Dependencies

- **agnostik** — AGNOS shared types
- **agnosys** — Kernel interface (Landlock, seccomp)
- **wayland-server** — Wayland protocol (feature-gated)

## Building

```bash
cargo build --release --all-features
```

## License

AGPL-3.0-only
