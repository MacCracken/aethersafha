# Architecture Overview

## Module Map

```
aethersafha
├── compositor.rs      — Wayland backend abstraction, surface/window management, workspaces
├── renderer.rs        — Scene graph, damage tracking, decorations, framebuffer, high-contrast
├── shell.rs           — Desktop shell: app launcher, notifications, quick settings, system status
├── ai_features.rs     — Context engine, AI suggestions, agent HUD, resource metrics
├── plugin_host.rs     — Plugin lifecycle, sandbox profiles, capability system
├── wayland/           — Wayland protocol implementation
│   ├── server.rs      — Compositor server, client management
│   ├── protocol.rs    — Protocol handlers (wl_compositor, xdg_shell, wlr_layer_shell)
│   ├── popups.rs      — Popup/menu management
│   ├── types.rs       — Protocol type definitions
│   ├── stub.rs        — Stub backend for testing
│   └── tests.rs       — Wayland protocol tests
├── xwayland.rs        — XWayland manager, X11 surface mapping, property translation
├── accessibility.rs   — AccessibilityTree, focus management, screen reader announcements
├── screen_capture.rs  — Capture manager, per-agent permissions, rate limiting, encoding
├── screen_recording.rs — Recording manager, frame buffer, poll-based streaming
├── security_ui.rs     — Permission dialogs, threat alerts, security dashboard
├── shell_integration.rs — System tray, window management, notification bridge
├── theme_bridge.rs    — AGNOS↔Flutter theme translation, platform channel
├── apps.rs            — Built-in apps (web browser, file manager, terminal, model manager)
├── gestures.rs        — Touch/trackpad gesture recognition
├── hud/               — HUD overlay system
│   ├── crew_status.rs — Agent crew status display
│   ├── domain_filter.rs — Domain-specific content filtering
│   └── gpu_status.rs  — GPU monitoring overlay
├── system_tests.rs    — Cross-module system integration tests
├── lib.rs             — Library root, public API surface
└── main.rs            — Binary entrypoint
```

## Data Flow

```
User Input → Wayland Server → Compositor → Renderer → Display
                  ↕                ↕
            XWayland         AI Features ↔ daimon (port 8090)
                                  ↕
                            Plugin Host → Sandboxed Plugins
```

## Consumers

- **AGNOS desktop** — primary consumer (the OS desktop)
- **Plugin authors** — via plugin_host API
- **daimon** — screen capture/recording via agent permissions
- **agnoshi** — shell integration for AI shell
