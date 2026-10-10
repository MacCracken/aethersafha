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

- **PluginMessage JSON** (0.17.3) is serde's text and serde's acceptance rules, checked against a crate
  built from Rust's own serde 1.0.228 / serde_json 1.0.149 on about 96k cases. Differences:
  - `ThemeUpdate`'s palette is written in ascending key order; Rust's `HashMap` order was unspecified.
  - A `surface_id` of 2⁶³ and up keeps its u64 bits in an i64.
  - Ill-formed UTF-8 in a string is refused (a Rust `&str` could not hold it).
  - `to_json` refuses a message no Rust value could be, such as an out-of-range enum or a 0 string.
  - Floats appear only in ignored fields, and there the port's correctly-rounded parse can accept a
    near-`f64::MAX` literal that serde_json's parse rejects.
- **The security UI's effects** (0.17.3):
  - **pid guards.** The "no pid" sentinel is INT_MIN (`SEC_PID_NONE`), not the usual −1, because −1 is
    `kill`'s broadcast value. An enforced revoke with pid 0 sends nothing; Rust's `i32::try_from(0)`
    succeeded and its `kill(0, SIGHUP)` signalled the compositor's own process group.
  - **The audit line** is serde's: keys sorted (serde_json without `preserve_order` used a BTreeMap),
    the agent id as a decimal string, and a whole-second `YYYY-MM-DDTHH:MM:SS+00:00` timestamp (Rust's
    `to_rfc3339` carried a fraction). A failed write after a successful open warns, where Rust was
    silent. On agnos, which ignores `O_APPEND`, the writer seeks to the end itself.
  - **The emergency kill's daimon DELETE** starts at once and is finished by the frame loop, at most
    four in flight. Rust spawned one per kill on its tokio runtime.
- **The agent-list poll** (0.17.3, `src/daimon_client.cyr`) speaks to daimon 2.4.3, which Rust's poll
  predates:
  - daimon sends integer ids and an integer status. Status maps through Rust's table by daimon's own
    names: 2 running → Acting, 6 failed → Error, and every other value → Idle. Rust read strings, so
    against this daimon it would have minted a random id each poll and read every agent as Idle.
  - The 5 s status tick starts the poll, and every frame steps it, with at most one GET in flight; the
    socket never blocks. Rust's `start_hud_polling` existed, but its binary never called it.
  - The answer is bounded at 256 KiB, and HTTP is narrowed to what daimon sends: no chunked encoding,
    no redirects.
  - The body is parsed with bayan, which unlike serde_json accepts leading zeros, and ill-formed UTF-8
    or raw control bytes inside a string.
  - A failed poll is not logged.
  - **On agnos the poll and the DELETE are no-ops.** Its `connect` blocks for up to 8 s, and a SYN to a
    closed port gets no RST, so the frame loop cannot dial out safely. The agnos agent gauge shows only
    what was registered locally.
- **Screen capture and recording** (0.17.3, still library code):
  - Every encoder writes alpha 0xFF. The port's framebuffer is XRGB; Rust wrote the pixel's top byte,
    which was translucent only under its panels.
  - A zero or negative size is refused before anything is allocated. That is Rust's `RegionOutOfBounds`
    and `EncodingError`; Rust's encoders accepted 0×0.
  - Buffers are freed: through `scap_result_free`, on ring eviction, and through the new
    `srec_mgr_remove_recording`.
  - Getters hand out the live record (the representation table above) rather than Rust's clones.

- **Pointer routing** (0.17.4, `src/route.cyr`; Rust's `route_input`, drag and resize state):
  - A gesture is anchored at the press, as Rust's `drag_state` and `resize_state` were, and keeps the
    grab offset. Rust's `apply_resize` snapped the moving edge onto the pointer, which with the port's
    6 px grips would jump up to 6 px.
  - A left or bottom-left resize keeps the RIGHT edge fixed at the 120×60 floor and at the screen edge.
    Rust floored only the width, so the window slid.
  - Dragged and resized windows stay on screen: the GPU ops refuse an out-of-range rect, which would
    drop the frame. This predates 0.17.4 but was never recorded.
  - The grips are 6 px strips over the content's outer pixels; the left and right strips run the full
    height. Rust's border was 1 px and started below the title bar. T / TL / TR stay retired (ADR 0004).
  - Buttons are routed as press AND release. The release of a press the compositor took (title bar,
    buttons, a grip) goes to no client, and a grip press is never forwarded. Rust's border press only
    began a resize.
  - Right and middle buttons reach a client only over its content, in content coordinates, and never
    move focus. Rust's right click carried the screen point anywhere on the surface; its middle button
    did nothing.
  - Keys under the chrome modifier are claimed by the compositor (the Super-key plan). Rust sent every
    key to the focused window.
- **Workspaces and window state** (0.17.4): four default workspaces, as in Rust. Differences:
  - The compositor holds no output rect. Callers pass the screen bounds, so `with_resolution` and
    `custom_resolution` are re-expressed as maximize-to-bounds.
  - `comp_set_window_state` refuses FULLSCREEN (owed by setu, ADR 0002) and FLOATING (retired, ADR
    0004), where Rust stored both.
  - NORMAL restores a maximized window's restore rect; Rust kept the maximized geometry. Minimizing
    the focused window hands focus to the topmost visible one; Rust left focus on it.
  - `comp_set_workspace_context` refuses a value no Rust `ContextType` could be. An invalid workspace
    id still returns success and changes nothing, as in Rust.
  - `comp_create_workspace` is port-only: Rust's set was fixed at four.
- **The CPU blend is the kernel's `#92` arithmetic** (0.17.4), so a CPU frame and a GPU frame of one scene
  are word-identical. Rust's `blend()` truncated where op 0x01 rounds to nearest, and returned the
  destination for a zero-alpha source where op 0x01 adds its colour. Over all 2²⁴ premultiplied inputs
  the two differ on about half, by at most 1. op 0x06 and op 0x02 follow the shader's f32 steps with
  ties to even; the 128.5 tie is the kernel's declared rounding mode, not yet confirmed on iron. A CPU
  frame now carries the alpha lane in the framebuffer's X byte, which scanout ignores and capture
  replaces with 0xFF.
- **Window opacity** has no Rust behaviour to port: Rust stored `SceneSurface.opacity` and never read it.
  The port applies it as the kernel does: a premultiplied surface is scaled by α/255 through op 0x06,
  so window and client alpha multiply, and chrome goes through op 0x02. A non-premultiplied surface is
  copied opaquely at any opacity, because `#87` has no alpha.
- **The window border** (0.17.4) is Rust's 1 px `COLOR_BORDER` outline in rupa's `line` token, one colour
  whether or not the window is focused, as in Rust. Rust inset its content by the border; the port draws
  the ring one pixel OUTSIDE the window rect and moves nothing. Off screen the ring is dropped rather
  than clamped onto the edge, and it is not a hit region (Rust's 1 px border was).

- **Small representation notes** (0.17.5, met while re-expressing the last Rust tests):
  - There is no `DesktopRenderer` object. Its pieces are a bhumi framebuffer, the kernel's `#84` flip and
    the frame loop's damage trackers.
  - A zero-size framebuffer is refused: `bhumi_fb_new(0, h)` returns 0. Rust returned an empty framebuffer
    whose every read was `None`, so no pixel can be read either way.
  - A window's agent flag is set on the window (`win_set_agent`), not passed to `comp_create_window` as
    Rust's `create_window` took it.

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
