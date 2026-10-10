# 0002 — Retire the Rust protocol layer, and record what it carried

**Status**: Accepted
**Date**: 2026-10-09

## Context

ADR 0001 made setu the native display protocol and said the Rust `wayland/` surface and `xwayland.rs` are
"retired, not ported". It did not say what those files contained that the sovereign stack still lacks. It
also did not decide to delete them, and the 0.17.x arc ends by deleting `rust-old/`, the parity oracle
`CLAUDE.md` names. Once the directory is gone, these files are no longer available as written prior art for
the protocol features aethersafha will need later.

The 2026-10-09 parity audit read the layer and found that **it never worked end to end**:
- `listen()` dropped its socket (`wayland/server.rs:103-109`).
- Accepted streams never reached dispatch (`server.rs:704-706`).
- The shm fd, attach, damage and frame requests were no-ops (`server.rs:284-305,326`).
- `apply_actions` discarded `create_window`'s id and looked windows up by surface UUID instead
  (`protocol.rs:685-728`).
- The XWayland bridge carried no pixels, input, focus, clipboard or scaling, and its `security_sandbox`
  flag was never read (`xwayland.rs:28,36`).

Deleting these files therefore loses data models and 215 tests (181 in `wayland/`, 34 in `xwayland.rs`),
but no behaviour that ever ran.

## Decision

**Nothing from `wayland/` or `xwayland.rs` is ported. It is deleted with `rust-old/`, and this record keeps
the inventory:**

- **Covered natively by setu and the compositor:** transport and client identity; the surface lifecycle
  (create, attach, commit, close); toplevel states (WinState, carried in `SETU_CONFIGURE`); keyboard focus
  (`SETU_INPUT_FOCUS`); pointer motion and buttons.
- **Dropped, with no native equivalent wanted:**
  - the object-id ↔ UUID map, globals and the registry, and seat-capability objects;
  - configure serials and acks (ATTACH carries the extent);
  - decoration negotiation and client move/resize requests (decorations are server-side; that is policy);
  - `wl_region`, buffer transform and scale, viewporter, output transform and subpixel;
  - the X11 display number and socket, `WM_HINTS`, the action queue and the feature stub.
- **Owed a native design (setu), tracked as roadmap D9 and the Super-key steps:**
  - window identity: title and app_id;
  - client state requests (fullscreen, for video and games);
  - min/max size hints, without which grips can shrink a window below its app's minimum;
  - parent/transient relationships, for dialogs;
  - popups with a positioner (menus, tooltips, IME candidates); Rust stored positions and had no solver;
  - clipboard and drag-and-drop, as one global selection rather than Rust's per-surface map;
  - text input / IME (content type, surrounding text, preedit; the password content type is a privacy
    signal);
  - output geometry, refresh and scale (1/120 fixed point);
  - pointer leave and the horizontal axis;
  - modifier state on key events and a keymap — ADR 0001 promised the keymap, and it is not built;
  - damage and frame pacing;
  - a HELLO version handshake (defined in setu, never sent, and rejected by the compositor);
  - disconnect cleanup (roadmap D3).
- **mehman owes** (the compat lane that replaced XWayland): multi-window guests; guest-driven title,
  state, size and parent; input and focus; clipboard; real pixels (mehman ADR 0004).
- **Never present in Rust:** subsurfaces; layer-shell (the wlr dependency was declared and unused);
  security-context and agent-surface extensions, which setu keeps off the wire by design.

## Consequences

- **Positive**:
  - The protocol's prior art survives the deletion as a checked list, not a directory nobody reads.
  - The deletion gate for these 5,586 lines is this document, not a port.
- **Negative**:
  - The Rust data models (popups, text input, output info) are gone for good. A future setu design
    starts from this list, not from code.
- **Neutral**:
  - Each "owed" item is new protocol work in setu, sequenced on the desktop track after the parity arc.

## Alternatives considered

- **Port the pieces a desktop will want (popups, clipboard, IME) during the arc.** Rejected. Those are
  protocol design work in setu, not parity: the Rust versions never ran and stored state without behaviour.
  Porting them would put unexercised data models on the compositor side of a wire that cannot carry them.
- **Keep `rust-old/src/wayland/` after the rest is deleted, as reference.** Rejected. A partial oracle invites
  the reading that it is still authoritative. The lists above are the reference, and git history keeps the
  files at the last tag that carries `rust-old/` (`0.17.5`).
