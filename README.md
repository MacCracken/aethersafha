# Aethersafha

**AI-augmented AGNOS desktop compositor.**

Aethersafha (Arabic: *the surface / plane*) is the desktop environment and
compositor for the AGNOS operating system — a composited desktop with AI-native
features, agent integration, and security-first design. It is a pure-Cyrius
sovereign compositor: it speaks a **native display protocol** designed from
first principles, **not** a port of Wayland (see
[`docs/adr/0001-native-display-protocol.md`](docs/adr/0001-native-display-protocol.md)).

It sits directly on the **bhumi** platform backend (output / input / seat — the
sovereign DRM/KMS + libinput + logind replacement), and hosts foreign-ABI apps,
when wanted, through the **mehman** swallow backend (kavach-sandboxed).

## Features

- **Native compositor** — sovereign window model, software renderer over the
  bhumi framebuffer, workspaces, focus + window management, decorations, damage
  tracking. Composites its own window model directly on bhumi.
- **Native display protocol** — a first-principles client↔compositor protocol
  (greenfield; `dhancha` is the client-side counterpart). Not Wayland, not
  `xdg_shell`, not `wl_shm`. See ADR 0001.
- **Foreign-app hosting (swallow)** — via mehman: foreign-ABI apps run as
  kavach-sandboxed guests whose surfaces are hosted as compositor windows (the
  compat lane that does XWayland's job the sovereign way — firewalled from the
  native path).
- **AI desktop features** — context-aware suggestions, agent HUD, resource
  metrics, agent-owned windows.
- **Plugin host** — sandboxed plugin loading with capability-based security.
- **Screen capture / recording** — per-agent permissions, rate limiting,
  byte-exact RAW/BMP/PNG encoding.
- **Accessibility** — accessibility tree, tab navigation, screen-reader
  announcements, high-contrast theme.
- **Security UI** — permission dialogs, threat alerts, agent audit dashboard.
- **Built-in apps** — file manager, agent manager, audit viewer, model manager,
  terminal (allowlisted spawn), browsers.

## Architecture

See [`docs/architecture/overview.md`](docs/architecture/overview.md) for the
full module map. In brief, the Cyrius source (`src/`) is:

- **Core** — `geom`, `window`, `compositor`, `render`, `input`, `main`: the
  window model, software renderer over the bhumi framebuffer, input routing, and
  the frame loop.
- **Shell + leaf** — `desktop`, `shell`, `theme_bridge`, `gestures`,
  `accessibility`, `ai_features`, `security_ui`, `shell_integration`,
  `plugin_host`.
- **Apps + capture** — `apps`, `screen_capture`, `screen_recording`.
- **Foreign** — `foreign`: the mehman swallow path.
- **Native display protocol** — greenfield (the redefined Bite F); the shared
  contract lib is TBD.

The Rust original is preserved at `rust-old/` (27,207 lines) as the parity
oracle; do not edit it.

## Dependencies

- **bhumi** — platform backend (output / input / seat).
- **agnostik** — shared AGNOS domain primitives.
- **agnodrm** — udev + DRM/KMS device model.
- **kashi** — bitmap console fonts (VGA 8×16).
- **mehman** + **kavach** — foreign-app swallow backend + sandbox execution.

Versions are pinned in [`cyrius.cyml`](cyrius.cyml); the toolchain pin lives in
`[package].cyrius`.

## Building

```sh
cyrius lib sync --full   # the declared stdlib set exceeds the incremental pin
cyrius deps              # resolve deps into lib/
cyrius build src/main.cyr build/aethersafha
```

## License

AGPL-3.0-only
