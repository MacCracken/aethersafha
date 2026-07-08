# 0001 — Native display protocol (Wayland refused)

**Status**: Accepted
**Date**: 2026-07-07

## Context

aethersafha is the AGNOS desktop compositor. Every GUI app must reach it over
some client↔compositor display protocol: attach, create a surface, submit a
pixel buffer, receive input. The question is what that protocol is.

The port from Rust (`rust-old/`, 27,207 lines) carried a ~3360-line
`wayland/{types,protocol,server,popups}` surface plus an `xwayland.rs` bridge —
so the plan of record recorded in the early `roadmap.md` / `parity-plan.md`
drafts was "the native client protocol *is* Wayland, reimplemented in Cyrius on
bhumi + `lib/net` syscalls" (the original Bite F, and the
`agnos_agent_surface_v1` / `zwp_security_context_v1` custom-protocol sketch in
agnosticos ADR-005).

That plan was reconsidered at the ecosystem level on 2026-07-06 and refused. The
reasoning is recorded canonically in `agnosticos/docs/design-patterns.md`
§"Sovereign substitution at the protocol layer — Wayland refused" and framed for
this repo in `dhancha/docs/development/sovereign-desktop.md`. In brief:

- **Porting a protocol adopts its design decisions wholesale.** Wayland's
  client-allocates-opaque-buffers model, the `wl_shm` / `xdg_shell` / registry
  handshake, and the extension sprawl each encode a Linux-desktop compromise
  AGNOS never made. "For compatibility with Wayland" fails the living-reason
  test — the sovereign desktop is not meant to run foreign Wayland clients (that
  is the separate, firewalled compat lane, mehman's job — never the native
  protocol's).
- **The opaque-buffer model structurally forbids what an AI-native OS wants.**
  The compositor cannot see inside a client's surface, so it cannot introspect a
  widget tree, nor let an agent drive or reason about another surface.
  aethersafha already carries an "agent window" concept and mehman foreign
  guests; a native protocol can make agent-owned, introspectable surfaces
  first-class.
- **Timing is the crux.** There is no client↔compositor ABI yet — aethersafha
  composites its own internally-created windows through its `Window` model on
  bhumi with **zero client IPC**. The cheapest moment to choose sovereignty is
  *before* that ABI is minted; porting Wayland first would make reclaiming
  sovereignty later a migration across every app's surface contract.

## Decision

**aethersafha's client↔compositor display protocol is a native, sovereign
protocol designed from first principles — not a port of Wayland.**

- The Rust `wayland/` surface (~3360 lines) and `xwayland.rs` are **retired, not
  ported.** "Bite F" is redefined from "reimplement the Wayland wire protocol in
  Cyrius" to "design and build the native display protocol."
- Wayland is treated as **prior art, not a template** (the §9 "reference, don't
  mimic" discipline): surface/role separation, client-submits-buffer,
  compositor-owns-composition-and-input, damage tracking, and event-ordering are
  studied and kept as design inputs; the wire, the lifecycle protocol, and the
  extension mess are dropped.
- The protocol contract (types + wire codec) will live in a **shared sovereign
  library** that both aethersafha (server) and dhancha (client) depend on, so
  coupling is at the ABI, not the codebase (monolithic by design). Naming +
  scaffolding that repo is a founder decision — not done here.
- **In scope for the protocol:** transport / connection; surface lifecycle
  (create / configure / commit / close); buffer submission (**CPU
  shared-memory first** — memfd/mmap; no GPU / mabda in the first cut); input
  events (compositor → focused client, mapping onto dhancha's existing
  `DhEvent`s) + a sovereign keycode→keysym keymap.
- **Out of scope here:** the AGNOS desktop *paradigm* (the AI-native window /
  interaction model) — a separate, founder-led design; and running foreign
  Wayland / X11 clients — the firewalled compat lane (mehman), never the native
  protocol's job.

## Consequences

- **Positive** — sovereignty over the seam between every app and the compositor;
  agent-owned, introspectable surfaces become expressible; none of a decade of
  Linux-desktop protocol compromises are inherited; the ABI is chosen clean
  *before* it is minted.
- **Negative** — we now own a protocol design (transport, lifecycle, wire codec,
  keymap) that Wayland would have handed us; there is no `wayland-server`
  equivalent or existing client ecosystem to lean on; the highest-risk XL piece
  becomes net-new design, not translation.
- **Neutral** — foreign Wayland / X11 clients, if ever wanted, arrive only
  through the mehman swallow backend (kavach-sandboxed), decoupled from the
  native path. dhancha owns the client-side binding; each app owns its
  connection.

## Alternatives considered

- **Port the Wayland wire protocol into Cyrius (the original Bite F).**
  Rejected: adopts Wayland's design decisions wholesale, forfeits
  introspectable agent-owned surfaces, and mints a foreign ABI at the one moment
  it is cheapest to avoid. Prior art to learn from, not a template.
- **X11.** Rejected long before this (any client can access any other's windows
  — structurally insecure); recorded in agnosticos ADR-005.
- **Defer the choice / co-evolve with whatever the client happens to need.**
  Rejected: the ABI, once apps depend on it, is the most expensive thing to
  change — design-patterns.md's "timing is the crux" argument applies.

## See also

- `agnosticos/docs/design-patterns.md` §"Sovereign substitution at the protocol
  layer — Wayland refused" — the canonical ecosystem pivot record.
- `dhancha/docs/development/sovereign-desktop.md` — the cross-repo seam, bites,
  and open (founder-led) decisions.
- Supersedes the "native protocol = Wayland" intent in this repo's earlier
  `roadmap.md` / `parity-plan.md` drafts, and the Wayland-compositor decision in
  `agnosticos/docs/adr/adr-005-desktop-environment.md` (marked superseded for the
  display-protocol layer).
