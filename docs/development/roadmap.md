# aethersafha — Roadmap

> What ships next, in what order, and against which gates. Live state is in [`state.md`](state.md), history in
> [`../../CHANGELOG.md`](../../CHANGELOG.md), and the parity gate in [`parity-plan.md`](parity-plan.md).
> ⛔ **Not a ledger.** A finished item shrinks to one line under *Done*, and its story lives in the CHANGELOG.

## v1.0 criteria

- [ ] Rust parity closed and `rust-old/` deleted (the 0.17.x arc)
- [ ] Test coverage ≥ 80%. Reference coverage is **60%** (791 of 1,297 fns, 2026-10-09)
- [ ] Benchmarks in `docs/benchmarks.md` (0.17.5)
- [x] Runs on agnos via bhumi, iron-proven: `--selftest` exits 95, keyboard and pointer work, every frame layer
      is on the GPU
- [ ] CHANGELOG complete from v0.1.0
- [ ] Security audit pass (`docs/audit/YYYY-MM-DD-audit.md`)

## 0.17.x — close the port, then delete `rust-old/` (next: 0.17.3)

**Why the arc is smaller than 28,072 lines of Rust suggests.** The Rust *binary* never created a window or
rendered a frame. It built the desktop, ran a 5 s status tick, honoured `--secure`, and exited on a signal.
Everything else was library code reached only by tests. So parity means library behaviour plus that tick.
A module that was library-only in Rust and is library-only here is at parity. Wiring one into the running
desktop is new work, and it stays out of the deletion gate (see *Desktop track*). The module-by-module
gate, with every open item, is [`parity-plan.md`](parity-plan.md).

**Decided (operator, 2026-10-09):**
- **Retire** the app layer (`apps.rs`: the window list, FileManager, Command Palette, browsers, Shruti and
  the Agent / Audit / Model managers), `theme_bridge`, `shell_integration`, quick settings with
  `KeyboardNavConfig`, and the three HUD widgets.
- **Keep** `gestures` dormant: it is tested library code, and there is no touch path yet.
- **Port** the agent-list poll (against daimon 2.4.3) and accessibility mirroring.
- The inert Rust flags (`--backend`, `--kiosk`, `--no-ai`, `--high-contrast`, `--accessibility`) retire.

| Release | Bite | What ships |
|---|---|---|
| **0.17.0** | The record | ✅ **Shipped.** ADRs 0002–0004 written; the retired surface deleted (`apps`, `theme_bridge`, `shell_integration`, quick settings, `KeyboardNavConfig`); a CI coverage floor at 61%; `scripts/qemu-linux-input.sh` and `programs/focus_client.cyr` in the tree. |
| **0.17.1** | Correctness | ✅ **Shipped.** `"terminal"` is back in AI context detection and in the shell app registry. The security dashboard's maximum comes from the active alerts (Info-only reads Info), and the UI mints ids from 2⁶²+1 so they never meet a caller's. Closing a window updates its workspace's active window, and both close and move drop *every* entry for the id, as Rust's `retain` did. Titles are cut to Rust's box and to the 64-cell run cap, and clipped by whole cells at the screen edges, so a CPU frame and a GPU frame show the same caption. The 18 wallpaper asserts run. The suggestion sort is stable and the hour read is null-checked. Coverage floor 63%. ⚠ The ellipsis is ASCII `...` and captions are measured in bytes until kashi 1.1.0 (0.17.2; ADR 0003). |
| **0.17.2** | What the Rust binary did | ✅ **Shipped.** kashi 1.1.0: text is UTF-8, one cell per scalar on the CPU and GPU paths, and a cut title ends in the real `…`. A strict, tested command line (`src/flags.cyr`): `--secure`, `--help`, `--version`, and an unknown flag is an error. Rust's initialize and 5 s tick: the welcome notice, `UserPresent`, and gauges read as Rust read them on Linux (`/proc/stat`, `MemAvailable`, `statfs`); agnos gains real disk through `statfs` `#103`. The net dot reads UNKNOWN and the agent count is fed. The accessibility tree mirrors every window live. SIGINT takes the Ctrl+Q teardown on Linux. `main.rs` and `accessibility.rs` are CLOSED in the parity plan. |
| **0.17.3** | Stubs become real | `PluginMessage` JSON in both directions (`plugin_host.cyr:320,326` return 0) via bayan, which gets declared in `[deps] stdlib`. The `plug_ipc_*` stubs are deleted; Rust had no transport. security_ui gets the audit-log writer (a JSON line with an ISO-8601 time, `security_ui.cyr:865`), plus SIGHUP on revoke and SIGKILL on kill through `sys_kill` (`:830`, `:845`). `poll_agents` runs against daimon 2.4.3's `GET /v1/agents` (integer ids and statuses), off the frame loop, and feeds the agent HUD and the panel gauge (`ai_features.cyr:735`). screen_capture writes alpha 0xFF and refuses width or height ≤ 0 (today that is a null write, `screen_capture.cyr:425-429,502,509`); it gets a decode round-trip over 64 KB and per-capture memory that is freed (about 74 MB leak per 2560×1440 PNG today). screen_recording frees what its ring evicts (`screen_recording.cyr:286`). Both stay library code. |
| **0.17.4** | Compositor and renderer depth | Pointer routing and the drag / resize state move out of `main.cyr` (`:383-678`) into a module a test can include, along with Rust's nine routing tests; the Super key builds on this module. Workspaces work as in Rust: four defaults, the active and agent window queries, and a context setter. Left and bottom-left resize grips are added (Rust reached both). The CPU path blends premultiplied surfaces and applies window opacity, so a QEMU or Linux frame matches iron's. Windows get a border (Rust's `draw_rect_outline`). Lifecycle edge tests come from a `.tcyr` version of `examples/headless_compositor.rs`. |
| **0.17.5** | Tests and benchmarks | Re-express every test left in [`parity-plan.md`](parity-plan.md) § *Rust tests still to re-express*. `docs/benchmarks.md` keeps Rust's `bench-history.csv` (two runs at `a460c3a`, machine unknown; a record, not a target) beside Cyrius benches for the same series: a frame with 1, 5, 10 and 20 surfaces, the window stack, damage, framebuffer create and clear, blit, and capture PNG. `scripts/bench-history.sh` is rewritten for `cyrius bench` (it runs `cargo bench` today). The coverage floor rises to what the suites reach. |
| **0.17.6** | Delete `rust-old/` | Once the gate below passes. The CHANGELOG names the last tag that carries the directory. |

**The gate for deleting `rust-old/`:**
1. Every [`parity-plan.md`](parity-plan.md) row is CLOSED.
2. ✅ ADRs 0002–0004 are accepted (0.17.0), and every divergence the matrix names cites one of them.
3. Nothing of value is lost:
   - the bench CSV is in `docs/benchmarks.md`;
   - the HUD specs are in ADR 0004 and the deferred protocol concepts in ADR 0002;
   - every `pub use` in `lib.rs:21-88` maps to a Cyrius home or an ADR;
   - the 27,207-line count is kept with its definition (`src/**/*.rs`; all `.rs` files total 28,072).
4. References point at that tag instead of the directory. In `CLAUDE.md` the cross-check principle and the
   do-not-modify rule become "the record is ADRs 0001–0004, the `.tcyr` suites, and tag X". The same applies
   to `README.md`, `docs/guides/getting-started.md`, `docs/architecture/overview.md`, ADR 0001,
   `desktop-arc-handoff.md`, the 18 `src/` files and 11 test headers that cite `rust-old/`, and the Rust lines
   in `.gitignore`.

Reactive repairs interleave as 0.17.x patches. Nothing in this arc needs a pin bump.

## Desktop track

The handoff, with its evidence, is [`desktop-arc-handoff.md`](desktop-arc-handoff.md); the next burn is
[`burn-card-next.md`](burn-card-next.md). These items are parked during 0.17.x except for reactive repairs.

### The Super key replaces Ctrl as the chrome modifier (planned; operator, 2026-10-09)

Every chrome key today is a Ctrl chord (`input_map`, `src/input.cyr`; the claimed set is in
[`../architecture/001-setu-client-contract.md`](../architecture/001-setu-client-contract.md) §3). Any other key
pressed while Ctrl is held is swallowed, because the wire carries no modifier state. So no client ever sees a
Ctrl chord, and a terminal cannot receive Ctrl+C. Rust had no compositor shortcuts at all: every key went to
the focused surface with its modifier mask. This is therefore desktop work, not parity. bhumi already emits
Super as HID usages 0xE3 / 0xE7 on both targets. That is verified in QEMU for Linux (evdev 125 / 126); on
agnos it comes from the boot report's modifier byte and has not yet been seen on iron.

| Step | What | Owner | Gate |
|---|---|---|---|
| SK1 | Super becomes a second chrome modifier: `input_mod_track` folds 0xE3 / 0xE7, and every chrome action answers Super as well as Ctrl. Settle the binding table first: keep the F-keys or move to Super + letters and arrows, and decide whether a bare Super tap opens the launcher. Verified with `scripts/qemu-linux-input.sh`. | here | 0.17.4's routing module |
| SK2 | Modifier state on key events (ADR 0002 lists it as owed). puka turns Ctrl + letter into a control code, and dhancha exposes the mask. | setu, puka, dhancha | can run beside SK1 |
| SK3 | The flip: the compositor stops claiming Ctrl, Ctrl chords reach clients with their modifiers, and Super is the only chrome modifier. The claimed set in the client contract changes for the third time, and `scripts/qemu-linux-input.sh` gains Super chords (QMP qcode `meta_l`) plus a check that a Ctrl chord reaches the focused client. | here and the clients | SK1 and SK2 |
| SK4 | On iron, the Keychron's GUI key through agnos's HID boot path: one keypress, declared in advance on a burn, like D8. | burn | before SK3 reaches iron |

### Open

| # | Item | Owner | State |
|---|---|---|---|
| D1 | **Burn the phase instrument**, reading 0.16.27's `cumulative us (client blit, #84 flip, input, yield, period)` line. It decides D2: blit points to damage rects, flip to vblank pacing, yield to the clients, input to the drain. Iron measured 7–15 fps on 2026-08-22, and the clear is exonerated. | burn | needs iron |
| D2 | **Present-protocol damage.** setu has no damage concept, and every live surface is re-read or re-blitted every frame whether or not it committed. A maximized 2560×1408 terminal moves about 28.8 MB per frame. This needs COMMIT as an opt-in change signal, plus dirty rects. | setu, then here | after D1 |
| D3 | **A client that exits leaves its window behind.** A dead channel only sets `chan_dead` (`main.cyr` frame loop). Its window stays focusable, and its dead buffer is re-read every frame. A client's own `SETU_CLOSE` is rejected (`setu_hs_action`, state 3). Same family, seen in QEMU at 0.16.30: on the Linux accept path a surface refused at ATTACH, or any later handshake failure, leaves the empty window minted at CREATE_SURFACE (`setu_srv_commit_from_create`). ⚠ That window also TAKES FOCUS, which makes `qemu-linux-input.sh`'s hostile scenario order-dependent: when the honest client connects first, the refused window keeps focus and the check "the honest client told it has focus" fails (seen in three runs under load, 2026-10-09). Fix the refusal path and the harness stops flaking. | here | open |
| D4 | **Per-PTY window size.** There is no TIOCSWINSZ on the `#97` band, so `ls` lays out columns for the console grid instead of puka's window. | agnos, puka | open |
| D5 | **Theme repaint on the GPU path**, reported 2026-08-18 and not reproduced. QEMU covers only the CPU path; re-test on the next burn. | burn | unconfirmed |
| D6 | **crab has no maximize view.** | crab | open |
| D7 | **A wedged compositor writes no log.** The `/klug.txt` spill covers a crash and a quit, but not a hang. | here | open |
| D8 | **`#86` slot ceiling.** The leak's cause is fixed (setu 0.8.6, crab 0.4.12), but the 16-per-boot ceiling has not been re-measured. Also on the same burn: does a Keychron media key move the cursor? Declare one keypress in advance. | burn | open |
| D9 | **Protocol depth owed by ADR 0002:** title and app_id, fullscreen and other client state requests, size hints, dialogs, popups, clipboard and drag-and-drop, IME, output info and scale, pointer leave and horizontal scroll, and a version handshake. | setu, then here | after the arc |
| D10 | **Wiring library code (new work, not parity):** a system screenshot through `screen_capture` (it doubles as the `--ppm` pixel oracle), agent capture requests (new setu kinds), a recording consumer, and gestures once touch exists. On iron a screenshot needs the CPU path for that frame or a kernel read-back. | here | after the arc |
| D11 | **The foreign-app lane** (ADR 0002, "mehman owes"): multi-window guests; guest-driven title, state and size; input and focus; clipboard; real pixels (mehman ADR 0004). | mehman, then here | open |
| D12 | **A silent client wedges the compositor.** setu's reads on an accepted socket block with no timeout, so a local process that connects and never sends stops the frame loop. Since 0.17.2 SIGINT is read once per frame, so Ctrl+C cannot end a wedged compositor (Ctrl+\\ and SIGTERM still do; ADR 0003). Bound the reads (a timeout or non-blocking serve). | here + setu | open |

**Owned by agnos, tracked here because the desktop feels them:**
- The `VFS_CHAN` close leak, still open on agnos 1.57.10's roadmap.
- Keystrokes lost at holds of about 100 ms (1.56.41). The recorded cause is contradicted by the code, and agnos's open list no longer carries it, so nobody owns it.
- The aarch64 build is broken.

**On hold by operator ruling (2026-08-17): M6-B, Linux as a display target.** Two items are left for when it
resumes:
- B2, a pixel oracle CI can diff; D10's screenshot provides it.
- B7, the client-side Linux gaps: in puka, `PUKA_SHELL` is a compile-time constant, and `pty_spawn` reports the pid before `execve` succeeds; in crab, `readdir`, `stat` and the input loop are agnos-only.

DRM/KMS becomes a second bhumi backend after fbdev.

**Toolkit boundary.** C4a's remaining question belongs to setu: whether COMMIT becomes the opt-in change
signal, which is the same decision as D2. **C4b, the `murrahir` EditorGUI port: do not start, scope or clone
it** until C4a closes (operator, 2026-08-16).

## Standing rulings — do not re-derive

- **Wayland is refused, not ported** (ADR 0001). Foreign Wayland and X11 apps go through mehman's firewalled compat lane.
- **The GPU path is the agnos ring-3 band (`#82`–`#94`) by direct syscall.** There is no `[deps.mabda]`, and
  there should not be one. QEMU exposes no AMD device, so a green QEMU run says nothing about the GPU path;
  only iron does.
- **A wallpaper is an image**, a PNG or a JPEG decoded by chitra. Generative and shader wallpapers are deleted,
  not deferred (operator, 2026-08-16).
- **App-facing behaviour lives in dhancha, not the compositor.** Any procedurally generated visual content goes
  in a library, never in the compositor or the kernel (operator, 2026-08-16).
- **`#91 gpu_blit_bb` is falsified for this architecture**, because every window is a live client surface
  re-composited each frame. **Batched `#92` is not reachable** without a slot model or an ABI field
  (`planning/desktop.md:396-412`).
- **Check the op list before calling a gap structural.** C3's "structural" cliff turned out to be one missing primitive.
- **A producer that writes alpha 0 ghosts additively under `#92`** (`out = src + dst`), so every client buffer
  must write a real alpha byte.
- **Linux is a real display target** (fbdev first, DRM/KMS later). It is on hold, not cancelled.

## Known cleanup

- **`"sys"` can be declared on 6.6.11 (found 0.16.29).** `src/sysprobe.cyr` includes `lib/sys.cyr` by path
  because declaring it once compiled nothing in. On 6.6.11 the declared form builds (scratch copy with the
  by-path include removed: host build green, same size, different hash; `--agnos` and the suites not run).
  Moving to it and correcting that file's comment is its own change.
- `README.md`'s build steps still begin with `cyrius lib sync --full`, which has been wrong since 0.16.27.
- **Stale comments the audit found.** Fix each with the bite that touches its file:
  - `render.cyr:5-6`, `:226` ("premultiplied") and `:1403` ("chars*6");
  - `shell_render.cyr:5`;
  - `sysprobe.cyr:12`, which calls the agents gauge "REAL";
  - `main.cyr:358`, on what happens when a client exits;
  - `ai_features.cyr:740`, which cites a `lib/http.cyr` that does not exist.
- **`map_u64` is walked by hand in three places** (`ai_features.cyr:671`, `security_ui.cyr:970`,
  `plugin_host.cyr:427`). Each breaks silently if its layout changes. File a cyrius issue for an iterator.

## Done — one line each; the story is in the CHANGELOG

- **M0–M1:** the `cyrius port` scaffold, and the compositor core (geom, window, compositor, render, input, main) on bhumi.
- **M2:** the eight leaf modules ported and behaviour-tested; the `desktop` aggregate owns them; kashi bitmap text.
- **M3:** the native display protocol is live (setu, ADR 0001), with real client windows on agnos and Linux, decorations, and a damage-limited blit (`AE-0a`).
- **M4 and M5, library parts:** `apps` C1+C2, `screen_capture`, `screen_recording`; mehman guests run sandboxed, with their captured output drawn as a window.
- **The desktop arc on agnos:** every `AE-` rung shipped and was iron-proven (`planning/desktop.md`), including real clients, titlebar drag, the GPU cursor, and every frame layer on the GPU.
- **M6-A1–A3:** staged-binary hashes; the min/max button path names itself; `#89`'s op-support mask is consumed.
- **M6-B1, B3–B6:** the Linux host session is real, with fbdev scanout, evdev keyboard and pointer, `--client`, and one setu rendezvous.
- **M6-C1, C3:** the wallpaper is an image; per-window opacity runs on the GPU (op 0x06, op 0x02), iron-proven.
- **M7:** the toolkit carries the apps (LIST, TEXTINPUT, the motion language, crab and puka on the widget tree, runtime theme switching, the shell panel).
- **0.16.26–0.16.29:** deps resolve from tags with every `path` dormant; the phase instrument attributes a slow frame; the cyrius 6.6.12 follow-ups shipped (a fmt gate, unused agnostik and agnodrm dropped, `M_SZ`).
- **0.16.30:** a client can no longer overflow the compositor's heap (surface geometry validated, as Rust's `ShmBufferInfo::validate` did); focus notices follow window ids; a minimized window never holds focus and comes back when focused.

## Out of scope for v1.0

- Running foreign Wayland or X11 clients natively. That is mehman's firewalled lane (ADR 0001).
- Enforcing plugin sandbox profiles with kavach. Rust's profiles were data only, so this would be new scope.
