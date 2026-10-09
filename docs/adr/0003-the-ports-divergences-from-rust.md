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
| `Uuid` ids | `i64` ids, sequential from 1 per registry |
| `DateTime<Utc>` | `i64` epoch seconds; the caller passes `now` |
| `f32` percentages, scales and scores | integer permille (1.0 = 1000) |
| `f32` gesture distances and angles | integer pixels and milliradians |
| `Option<T>` | a documented sentinel per field (−1, or 0 where 0 cannot be a value) |
| error enums with `Display` text | `i64` error codes; nothing displays the text |
| getters returning clones | getters returning the live object |
| Unicode case folding in search | ASCII case folding |

**Accepted behaviour**

- **System status gauges are seeded UNKNOWN (−1) and render `--%`, not 0.** A 0 is a claim nobody made. cpu
  and disk stay UNKNOWN on agnos, which has no source; Linux reads real values again from 0.17.2.
- **Screen capture reads a framebuffer the caller passes in**; it does not call the renderer. This is the
  shape a compositor-side screenshot needs.
- **New client windows cascade from spawn coordinates in the frame loop**, not from a fixed placement rule
  in `create_window`.
- **The minimum window is 120×60** (Rust: 200×100), sized to keep the three title-bar buttons inside the
  window that owns them.
- **Gestures:** the rotate angle is wrapped to (−π, π], so crossing atan2's branch cut does not emit a
  spurious ~2π rotation as Rust did. The swipe distance uses an integer floor square root, so a move of
  50–51 px is a swipe only in Rust.

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
