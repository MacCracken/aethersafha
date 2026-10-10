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
- **AI desktop features** — context-aware suggestions, agent HUD state,
  resource metrics.
- **Plugin host** — sandboxed plugin loading with capability-based security.
- **Screen capture / recording** — per-agent permissions, rate limiting,
  byte-exact RAW/BMP/PNG encoding.
- **Accessibility** — accessibility tree, tab navigation, screen-reader
  announcements, high-contrast theme.
- **Security UI** — permission dialogs, threat alerts, agent audit dashboard.
- **Launcher** — Ctrl+F2 opens a launcher for windowed clients (puka, crab,
  thoth). Every window on the desktop belongs to a real client process.

## Architecture

See [`docs/architecture/overview.md`](docs/architecture/overview.md) for the
full module map. In brief, the Cyrius source (`src/`) is:

- **Core** — `geom`, `window`, `compositor`, `render`, `utf8`, `input`, `route`,
  `main`, `flags`, `sigint`: the window model, software renderer over the bhumi
  framebuffer, UTF-8 text, input routing, the command line (`aethersafha
  --help`), and the frame loop.
- **Shell + leaf** — `desktop`, `shell`, `shell_render`, `launcher`,
  `gestures`, `accessibility`, `ai_features`, `daimon_client` (the agent list
  from daimon), `security_ui`, `plugin_host`.
- **Capture** — `screen_capture`, `screen_recording` (library code, not yet
  wired into the desktop).
- **Foreign** — `foreign`: the mehman swallow path.
- **Native display protocol** — setu (ADR 0001): `setu_server` and
  `setu_dispatch` here, the message types and wire codec in the `setu`
  dependency.

The Rust original — 27,207 lines of `src/**/*.rs`, 28,072 counting its tests,
benches and example — was deleted at 0.17.6. Tag `0.17.5` is the last that
carries it, at `rust-old/`. A citation `rust-old/<path>[:<line>]`, or a bare
`<name>.rs:<line>` in a parity comment, refers to that path at tag `0.17.5`;
read it with `git show 0.17.5:rust-old/<path>`. The record of the port is ADRs
0001–0004, the `.tcyr` suites, and that tag;
[`docs/architecture/002-rust-public-api-map.md`](docs/architecture/002-rust-public-api-map.md)
maps every public Rust type to its Cyrius home or the ADR that retired it.

## Dependencies

- **bhumi** — platform backend (output / input / seat).
- **rupa** — desktop theme tokens (the MUDRA / SHANTA palettes).
- **kashi** — bitmap console fonts (VGA 8×16).
- **mehman** + **kavach** — foreign-app swallow backend + sandbox execution.
- **chitra** — PNG / JPEG decode, for the wallpaper.
- **setu** — the native display protocol's message types and wire codec.

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
