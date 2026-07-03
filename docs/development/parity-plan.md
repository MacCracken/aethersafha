# aethersafha — Parity Execution Plan (bite-level WBS)

> Companion to [`roadmap.md`](roadmap.md) (the milestone view). This is the
> **executable** breakdown: strategic "bites" sized for a session or a workflow,
> with dependencies, execution mode, and acceptance criteria. Bar = parity with
> `rust-old/` (27,207 lines). Updated after each bite lands.

## Snapshot (post-0.1.0)

| Layer | Rust lines | Cyrius status |
|---|---|---|
| Core: compositor, renderer, input | 2565 + 1656 + — | **partial** — window CRUD/focus + minimal fill/loop done; depth pending (Bite A) |
| Core: geom, window, main | — | ✅ done, runs on bhumi |
| Leaf: theme_bridge, gestures, accessibility, ai_features, shell, security_ui | ~6900 | **structural** parity + smoke tests; behavioral tests + wiring pending (Bite B) |
| Leaf: shell_integration, plugin_host | 516 + 848 | ⬜ not ported (Bite B1) |
| Apps | 2986 | ⬜ not ported (Bite C) |
| Capture / recording | 1299 + 938 | ⬜ not ported (Bite D) |
| HUD widgets | ~1990 | ⬜ not ported (Bite E) |
| **Wayland protocol** (types/protocol/server/popups) | ~3360 | ⬜ not ported — from scratch in Cyrius (Bite F) |
| xwayland → mehman | 823 | ⬜ blocked on mehman (Bite G) |
| system_tests | 1477 | re-expressed as per-module `.tcyr` (cross-cutting) |

**Key decision (ADR needed):** Wayland is the AGNOS-native client protocol;
aethersafha implements the wire protocol itself in Cyrius on bhumi + `lib/net`
syscalls (there is no `wayland-server` crate to lean on). This makes Bite F the
largest and highest-risk piece.

---

## Bites

Legend — **Mode**: 🔁 workflow (parallel fan-out) · ➡️ serial (tight coupling) ·
🔗 pipeline. **Size**: S (<1 session) · M (1 session/workflow) · L (multi) · XL (weeks).

### Bite 0 — Cross-cutting foundations · ➡️ · S
Do alongside everything; unblocks the process gates.
- **ADR-0001**: record "native protocol = Wayland, implemented in Cyrius".
- **Benchmark harness**: real `.bcyr` for compositor create/focus + renderer fill
  (the dev loop forbids claiming perf without numbers).
- **ERR_\* overlap cleanup**: trim agnostik/agnodrm to selective `modules` so the
  duplicate-symbol warnings go away.
- **Accept**: ADR committed; `cyrius bench` produces a CSV row; clean build w/o ERR_ warnings.

### Bite A — Core depth to full parity · ➡️ (workflow-drafted) · L
The load-bearing layer everything visual depends on. Rust: `compositor.rs` (2565),
`renderer.rs` (1656), input routing.
- **A1 compositor**: workspaces + move-window-to-workspace, drag/resize state
  machines, input routing to the focused surface, secure_mode + agent-aware mode,
  accessibility-tree bridge.
- **A2 renderer**: premultiplied-alpha blend, damage tracker (dirty rects), scene
  graph, window decorations (titlebar buttons, borders, resize edges), hit-testing,
  bitmap font + text.
- **A3 input**: HID→keysym/layout (xkb-equivalent), modifier state, key repeat.
- **Mode**: workflow can draft each file, but integration is serial (shared state).
- **Accept**: behavioral `.tcyr` mirroring rust compositor/renderer tests; binary
  renders decorated, focus-routed, damage-tracked windows.

### Bite B — Finish leaf parity · 🔁 + ➡️ · M
- **B1** 🔁 (2 agents): port `shell_integration` (516) + `plugin_host` (848).
- **B2** 🔁 (~8 agents): behavioral parity test suites (`.tcyr`) for all leaf
  modules — port the relevant `rust-old` unit tests, assert against real behavior
  (not just constructors).
- **B3** ➡️: wire leaf modules into the compositor/shell surface (notifications,
  quick settings, a11y tree from the window tree, ai suggestions, security prompts).
- **Accept**: leaf modules behavior-tested + reachable from the running compositor.

### Bite C — Built-in apps · 🔁 · L
Rust: `apps.rs` (2986) — Terminal, FileManager, AgentManager, AuditViewer, ModelManager.
- **C1** 🔁 (5 agents, one per app type). Terminal is **security-critical**
  (allowlisted `process.exec` — dedicated review pass, no path traversal).
- **C2** ➡️: app window lifecycle + compositor registration.
- **Accept**: each app opens as a compositor window; Terminal allowlist enforced + tested.

### Bite D — Screen capture + recording · 🔗 · M
- **D1** `screen_capture` (1299): permission model, rate limiting, encode
  (raw/BMP native; PNG via a ported/stdlib encoder or documented stub).
- **D2** `screen_recording` (938): depends on D1 — ring buffer + state machine.
- **Accept**: permission + rate-limit tested; capture reads the bhumi framebuffer.

### Bite E — HUD widgets · ➡️ substrate + 🔁 trio · M
- **E1** ➡️: HTTP+JSON polling substrate (`lib/http`/sandhi + `lib/bayan`) replacing
  reqwest/serde.
- **E2** 🔁 (3 agents): `gpu_status`, `domain_filter`, `crew_status` on E1.
- **Accept**: widgets poll a daimon MCP endpoint + parse JSON; band/status logic tested.

### Bite F — Native Wayland protocol surface · ➡️ · XL (highest-risk)
Rust: `wayland/{types,protocol,server,popups}` + client socket. From scratch.
- **F1** 🔁: `wayland/types` data structures (ShmFormat, OutputInfo, transforms).
- **F2** ➡️ (hard): Wayland wire codec — object registry, message marshal/unmarshal
  over the Unix socket (`lib/net` `sys_socket`/`bind`/`listen`/`accept4`), `wl_display` core.
- **F3** ➡️: protocol dispatch (surface create/attach/commit, `wl_shm`, `xdg_shell`
  roles, `wl_seat`, `wl_output`) → compositor actions; xdg popups + constraints.
- **Mode**: its own mini-roadmap; scope a minimal object subset first, grow.
- **Accept**: a native Cyrius client can bind `wl_compositor`, create a surface,
  attach an shm buffer, and see it composited.

### Bite G — mehman (XWayland successor) · 🚧 started
mehman 0.3.1 + kavach 3.6.0 **wired** (`src/foreign.cyr`): guest-spec + foreign-surface
descriptor + `desktop_host_foreign` → a compositor window; `main` hosts a demo guest;
`foreign_run` → `mehman_sandbox_capture_guest` runs the guest AND captures its output
into the surface buffer (M2 handoff — tested). **Presentation done**:
`render_desktop_foreign` / `render_foreign_content` paint the captured surface as the
hosted window's content (line-aware `draw_text_lines`); the desktop tracks foreign
apps; pixel-tested. Remaining: per-ABI shim; real XRGB pixel fidelity (mehman ADR 0004).

### Bite H — GPU acceleration (mabda) · optional · —
Wire mabda 4.0.2 (`[deps.mabda]`) when hardware accel is wanted. Off the v1.0 path.

---

## Sequencing

- **Phase 1 (make the core real + finish leaves)** — Bite 0, Bite A, Bite B. A (serial
  core depth) and B (independent leaf modules) run in parallel tracks.
- **Phase 2 (feature breadth)** — Bite C, D, E on top of the solid core. Highly
  parallel: three workflows.
- **Phase 3 (the protocol)** — Bite F. The make-or-break; its own multi-week arc.
- **Phase 4 (compat + accel)** — Bite G (when mehman ships), Bite H (when wanted).

## Workflow catalog (fan-out opportunities)

| WF | Bite | Shape | Agents |
|---|---|---|---|
| WF-1 | B1 | port 2 leaf modules | 2 |
| WF-2 | B2 | behavioral test suites | ~8 |
| WF-3 | C1 | one app per agent | 5 (+ Terminal review) |
| WF-4 | D | capture→recording pipeline | 2 stages |
| WF-5 | E2 | one widget per agent | 3 |
| WF-6 | A | core-depth translation drafts (serial integration after) | 3 |
| WF-7 | F1 | wayland/types | 1–2 |

Each bite follows the dev loop: port → cleanliness (`fmt`/`lint`) → tests → bench →
audit → docs (CHANGELOG, ADRs, source citations) → version/state sync.

## Recommended first move

Run **WF-1 + WF-2 in parallel** (finish the leaf surface: `shell_integration` +
`plugin_host` + behavioral tests for all leaves) as the low-risk, high-parallelism
win, while taking **Bite A (core depth)** as the serial focus — the compositor +
renderer are what make everything else visible and testable.
