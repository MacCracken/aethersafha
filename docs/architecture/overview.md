# Architecture Overview

> aethersafha is a pure-Cyrius sovereign compositor. It speaks a **native
> display protocol** (not Wayland — see
> [`../adr/0001-native-display-protocol.md`](../adr/0001-native-display-protocol.md))
> and sits on the **bhumi** platform backend. The Rust original is frozen at
> `rust-old/` as the parity oracle; this map is the Cyrius port (`src/`).

## Module map (`src/`)

```
aethersafha
├── geom.cyr             — geometry primitives (rectangles, hit-tests)
├── window.cyr           — window model + window state
├── compositor.cyr       — window stack, focus, workspaces, CRUD, secure/agent modes
├── render.cyr           — software renderer over the bhumi framebuffer: alpha blend,
│                           damage tracking, decorations, bitmap text (kashi)
├── input.cyr            — bhumi HID → compositor input actions, window-mgmt shortcuts
├── main.cyr             — entry: open the bhumi backend, seed windows, frame loop
├── desktop.cyr          — aggregate: owns the compositor + all leaf managers; the
│                           unified frame (themed bg + windows + shell status panel)
├── shell.cyr            — desktop shell: launcher, notifications, quick settings, status
├── theme_bridge.cyr     — theme translation / synchronization
├── ai_features.cyr      — context engine, AI suggestions, agent HUD, resource metrics
├── accessibility.cyr    — accessibility tree, focus, announcements, high-contrast
├── gestures.cyr         — touch / trackpad gesture recognition
├── security_ui.cyr      — permission dialogs, threat alerts, security dashboard
├── shell_integration.cyr — system tray, window management, notification bridge
├── plugin_host.cyr      — plugin lifecycle, sandbox profiles, capability system
├── apps.cyr             — built-in apps (file manager, agent manager, audit viewer,
│                           model manager, command palette, browsers)
├── screen_capture.cyr   — capture manager: per-agent permissions, rate limiting, encoders
├── screen_recording.cyr — recording manager: sessions, frame ring buffer
└── foreign.cyr          — mehman swallow path: host a foreign-ABI app as a
                            kavach-sandboxed guest, capture + present its surface
```

The **native display protocol** surface (client↔compositor: transport, surface
lifecycle, buffer submit, input dispatch) is greenfield — the redefined Bite F.
Its contract will live in a shared sovereign library consumed by both
aethersafha (server) and dhancha (client). See ADR 0001 and
`dhancha/docs/development/sovereign-desktop.md`.

## Data flow

```
bhumi (HID) → input → compositor → render → bhumi (framebuffer) → display
                          ↕              ↕
                    desktop aggregate   ai_features ↔ daimon
                          ↕
                    plugin_host → sandboxed plugins
                          ↕
                    foreign → mehman (kavach-sandboxed foreign apps)

client apps:  dhancha client  ↔  native display protocol (greenfield)  ↔  compositor
```

## Consumers

- **AGNOS desktop** — the OS desktop (primary; `publish = false` top-level app).
- **dhancha** — the client-side toolkit on the other side of the native protocol.
- **daimon** — screen capture / recording via agent permissions.
- **Plugin authors** — via the plugin_host capability API.
