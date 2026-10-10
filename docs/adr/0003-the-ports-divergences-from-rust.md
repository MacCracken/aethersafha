# 0003 — The port's divergences from Rust: substrate, representation, and accepted behaviour

**Status**: Accepted
**Date**: 2026-10-09

## Context

`CLAUDE.md` sets the port's correctness bar as "matches what Rust did" and allows divergence "only with an
ADR". The 2026-10-09 parity audit found many deliberate divergences recorded only in file headers, comments
or the CHANGELOG, which disappear as evidence once `rust-old/` is deleted. It also found several silent ones;
those are fixed rather than recorded (roadmap 0.17.1–0.17.4: the `"terminal"` context match, the security
dashboard's Info level, workspace defaults, the CPU blend). This ADR records the divergences that stay.

## Decision

**The divergences below are the port's intended behaviour.** Parity for these modules means "matches Rust
under these mappings". A difference from Rust that is not listed here or in ADR 0002 / 0004 is a defect to
fix.

**Substrate**

| Rust | The port | Why |
|---|---|---|
| DRM/KMS, libinput and logind behind `CompositorBackend` / `WaylandBackend` | the `bhumi` platform backend: fbdev scanout and evdev on Linux, the agnos framebuffer and HID on agnos | bhumi is the sovereign platform layer; Rust's backends never drove a screen |
| GPU acceleration via an optional `mabda` feature (it gated nothing) | the agnos ring-3 GPU band (`#82`–`#94`), by direct syscall; there is no `[deps.mabda]` | the GPU path that exists on the target |
| an ARGB `Pixel` with per-pixel alpha in the framebuffer | an XRGB8888 framebuffer (`bhumi`), with no alpha channel | the scanout format on both targets |
| front/back buffer swap | the kernel's `#84` flip | the kernel owns double-buffering |
| a palette in the renderer | `rupa` theme tokens | one token source for the compositor, dhancha and every client |
| a hand-drawn 5×7 font | `kashi`'s VGA 8×16 glyph data | the sovereign font subsystem |
| a damage region list with flush, re-damaged in full every frame by `render_frame` | a damage band: `union(cur, prev)` rows, driven by window-rect changes, live surfaces and retired rects | Rust's tracker never limited anything; the band is what `#84`'s flip requires |
| a `Layer` type ordering the scene | the window vector's order, with raise-on-focus | one source of truth for paint order and hit-testing |
| tokio, `Arc<RwLock>`, a 5 s async tick | one thread and a frame loop | the compositor owns its state; nothing else touches it |

**Representation**

| Rust | The port |
|---|---|
| `Uuid` ids | `i64` ids, sequential from 1 per registry (security UI: see below) |
| `DateTime<Utc>` | `i64` epoch seconds; the caller passes `now` |
| `f32` percentages, scales and scores | integer permille (1.0 = 1000) |
| `f32` gesture distances and angles | integer pixels and milliradians |
| `Option<T>` | a documented sentinel per field (−1, or 0 where 0 cannot be a value) |
| error enums with `Display` text | `i64` error codes; nothing displays the text |
| getters returning clones | getters returning the live object |
| Unicode case folding in search | ASCII case folding |

**Accepted behaviour**

- **System status gauges are seeded UNKNOWN (−1) and render `--%`, not 0.** A 0 is a claim nobody made, and
  Rust's `unwrap_or(0.0)` made it whenever a read failed. A computed negative (a counter that ran backwards,
  `MemAvailable > MemTotal`) also reads UNKNOWN, since negatives are the sentinel range. Sources (0.17.2):
  - **Linux** reads what Rust read: cpu from `/proc/stat` through `parse_cpu_line`, memory from
    `/proc/meminfo` `MemAvailable`, disk from `statfs("/")`.
  - **agnos** reads memory from `sysinfo` `#35` and disk from `statfs` `#103`. Its cpu stays UNKNOWN: the
    per-core tick counters in `#35` include halted ticks and the kernel keeps no idle count, so Rust's
    non-idle ÷ total cannot be computed.
  - **The cpu window is the tick interval** (5 s), not Rust's two reads 100 ms apart, because the frame
    loop cannot sleep. The first reading, and the first after a failed read, is UNKNOWN.
- **The status tick runs on a monotonic 5 s clock and never bursts** (tokio's interval did). Where that clock
  is unavailable it falls back to every 120 frames.
- **The panel claims nothing the port does not measure.** Rust hardcoded `NetworkStatus::Connected` and
  `agent_count: 0` in its tick. The port seeds and ticks the network as UNKNOWN (an empty dot, never green),
  and the agent count comes from the AI layer's live agent registry.
- **The command line is strict** (0.17.2, `src/flags.cyr`). An unknown flag, a retired Rust flag (ADR 0004),
  a missing value or a non-numeric count prints an error naming it and exits 2, like clap. The port takes
  the *first* of a repeated single-value flag and accepts a repeated boolean; clap refuses both. There is no
  `--flag=value`, no combined short flags and no `--` terminator. `--version` prints `aethersafha <VERSION>`.
- **Accessibility mirroring** (0.17.2). As in Rust, a node per window keyed by the window id, with
  "Window opened / closed" announcements. Differences:
  - Tree focus follows compositor focus on *every* change, including window creation and the close
    hand-off. Rust synced it only in `focus()`, so its tree focus lagged a create, and a close left it
    empty or on a window that no longer had focus. A compositor that focuses nothing (every window
    minimized) clears tree focus.
  - Node ids are window ids, and both count from 1 (Rust's `Uuid`s could not collide). The desktop tree
    therefore holds window nodes only; minting a tree id into it would collide.
  - `desktop_sync_accessibility` is a port-only reconcile; with the hooks in place it changes nothing.
- **SIGINT** (0.17.2, Linux). Rust's `ctrl_c` ended the run loop; the port blocks SIGINT and reads it from
  a signalfd once per frame, then takes the Ctrl+Q teardown. Only SIGINT is handled, as in Rust.
  - A `--clients` probe that is interrupted exits 130, not 93/94, which would claim the whole budget was
    spent. A 91/92 verdict or a completed 95 still stands.
  - ⚠ A compositor wedged inside a blocking client read never reaches its next frame, so it does not see
    Ctrl+C; Ctrl+\ or `kill` (SIGTERM) still end it. Before 0.17.2 Ctrl+C killed it outright, without the
    teardown.
  - agnos compiles the module to no-ops: SIGINT has no default action there and the keyboard delivers
    Ctrl+C as byte 3.
- **Screen capture reads a framebuffer the caller passes in**; it does not call the renderer. This is the
  shape a compositor-side screenshot needs.
- **New client windows cascade from spawn coordinates in the frame loop**, not from a fixed placement rule
  in `create_window`.
- **The minimum window is 120×60** (Rust: 200×100), sized to keep the three title-bar buttons inside the
  window that owns them.
- **Gestures:** the rotate angle is wrapped to (−π, π], so crossing atan2's branch cut does not emit a
  spurious ~2π rotation as Rust did. The swipe distance uses an integer floor square root, so a move of
  50–51 px is a swipe only in Rust.
- **Closing the focused window focuses the topmost window that is not minimized** (0.16.30). Rust focused
  the last id in the active workspace's list, which can be minimized, so closing the last visible window
  focused one nobody could see. The workspace's *active* window still follows Rust exactly: the last id left
  in its list, or none.
- **The security UI mints its own ids from 2⁶²+1** (`SEC_ID_MINTED_BASE`, 0.17.1). One alert list holds ids
  the caller supplies and ids the UI mints. Rust's random `Uuid`s never met, but two counters both starting
  at 1 do, and `dismiss_alert` then resolves the wrong alert. Callers keep their own ids below the base. This
  is a contract on callers and is not checked.
- **Window captions** (0.17.1, `render_title`):
  - The title box is derived from the decoration geometry: it starts `TITLE_PAD` inside the left edge and
    ends `TITLE_PAD` short of the minimize button. That comes to Rust's `width − 80` today, and it moves
    with the buttons if they change.
  - No caption is longer than 64 cells (`AE_TEXT_CHARS`), however wide the window, and this holds on every
    frame kind. A longer run never reached the screen on a GPU frame, because the CPU fallback is not
    presented there. Rust drew such a title whole.
  - At the left and right screen edges the caption loses whole 9 px cells; Rust clipped per pixel. Clipping
    by cell keeps the GPU run from being refused, which would drop the whole title.
  - The cut is Rust's own: `max_chars − 1` scalars, then `…` in one cell (0.17.2, kashi 1.1.0). Rust's 5×7
    font had no `…` glyph and drew a blank cell there; the port draws the glyph. The 64-cell cap cuts the
    same way. (0.17.1 measured captions in bytes and cut with ASCII `...` until kashi had the glyph.)
- **Text is UTF-8, one cell per scalar** (0.17.2, `src/utf8.cyr`, kashi 1.1.0's Unicode→CP437 map). As in
  Rust, every char takes one cell and a char with no glyph is a blank cell that still advances. Differences:
  - The glyph set is kashi's, not Rust's 5×7 subset: printable ASCII, the CP437 high half (`é`, `ß`, box
    drawing…), `⌂`, `…` and `�`. So `é` draws a glyph where Rust drew a blank, and every other scalar is
    blank in both.
  - Ill-formed UTF-8, which a Rust `&str` cannot hold, decodes per Unicode §3.9: each maximal ill-formed
    subpart is one `�`. That includes a foreign guest's raw non-UTF-8 output bytes, which 0.17.1 drew as
    their CP437 glyphs.
  - U+007F draws blank (kashi ADR 0011's rule, and Rust's); a raw 0x7F used to draw `⌂`.

## Consequences

- **Positive**:
  - Every divergence a reader could mistake for a bug now has a stated reason in one place.
  - A test that encodes one of these mappings is a parity test, not a deviation.
- **Negative**:
  - Matching Rust is now conditional on this list. Anyone porting a test from the parity matrix has to
    apply the mapping first; a Rust test asserting `f32` 0.5 becomes a permille-500 assertion.
- **Neutral**:
  - The list should only shrink. A new divergence needs an entry here or its own ADR.

## Alternatives considered

- **Keep each divergence in the comment beside the code that implements it.** Rejected as the only record.
  Comments describe one file and rot with it, and the audit found several that misstated parity (e.g.
  `render.cyr` calling a straight-colour blend "premultiplied"). The comments can stay; this is the record.
- **Mimic Rust's representation (UUIDs, `f32`, clones) to make the port line up test for test.** Rejected.
  It would import costs (allocation per getter, floating point in a fixed-point codebase) for a parity that
  the mappings above already give.
