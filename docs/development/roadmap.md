# aethersafha — Roadmap

> What ships next, in what order, and against which gates. **Only open work lives here:** when an item ships it
> leaves this file, and its story is in the [`CHANGELOG`](../../CHANGELOG.md). Live state is in [`state.md`](state.md),
> and the Rust original's API map is [`002-rust-public-api-map.md`](../architecture/002-rust-public-api-map.md).
> Item numbers (D2, SK1) stay stable, because source comments and ADRs cite them. `CL-`, `R-`, `STATE-` and `SVC-`
> numbers are findings in [`../audit/2026-11-06-audit.md`](../audit/2026-11-06-audit.md).

## v1.0 criteria

- [x] Rust parity closed and `rust-old/` deleted (the Rust lives on in tag `0.17.5`)
- [x] Benchmarks in `docs/benchmarks.md`, with `scripts/bench-history.sh` keeping the history
- [x] Runs on agnos via bhumi, iron-proven: `--selftest` exits 95, keyboard and pointer work, every frame layer
      is on the GPU
- [x] CHANGELOG complete from v0.1.0: every tag has an entry
- [ ] Test coverage ≥ 80%. Reference coverage is **69%** (921 of 1,317 fns); CI's floor is `--min 69`. See 0.22.x
- [ ] Security audit pass. The first ran at 0.18.2 ([`2026-11-06-audit.md`](../audit/2026-11-06-audit.md)); it is
      closed when 0.18.x's iron checks pass and no finding above LOW stays open

## The release lines

| Line | Theme | Owner | Gate |
|---|---|---|---|
| **0.18.x** | Hardening: prove 0.18.2 on iron, read the phase line, clear the cleanup list | here, burn | a burn |
| **0.19.x** | Input: the Super key, and who may take focus | here, setu, puka, dhancha | 0.18.x |
| **0.20.x** | Present and pacing: damage, band present, a Linux frame clock | setu, bhumi, here | D1's answer |
| **0.21.x** | Protocol depth and client identity | setu, here, clients | 0.19.x |
| **0.22.x** | Library code wired, and coverage to 80% | here | the arc above |
| **0.23.x** | The foreign-app lane | mehman, here | 0.21.x |
| **1.0.0** | Every v1.0 criterion ticked | | |

Reactive repairs interleave as patches on the current line. A pin bump happens only when the operator asks for it or
the work needs it.

## 0.18.x — hardening

### Iron checks (the next burn)

The burn card is [`burn-card-next.md`](burn-card-next.md). QEMU exposes no AMD device, so a green QEMU run says nothing
about the GPU path; only iron does. Every item here has run on a host or not at all.

| # | Item | State |
|---|---|---|
| D1 | **Burn the phase instrument**, reading the `cumulative us (client blit, #84 flip, input, yield, period)` line. It decides D2: blit points to damage rects, flip to vblank pacing, yield to the clients, input to the drain. Iron measured 7–15 fps on 2026-08-22, and the clear is exonerated. | needs iron |
| D5 | **Theme repaint on the GPU path**, reported 2026-08-18 and not reproduced. QEMU covers only the CPU path. | unconfirmed |
| D8 | **`#86` slot ceiling.** The leak's cause is fixed (setu 0.8.6, crab 0.4.12), but the 16-per-boot ceiling has not been re-measured. Also on the same burn: does a Keychron media key move the cursor? Declare one keypress in advance. | open |
| B1 | **0.18.2's agnos arm has never run.** Quit puka from its own shell and read the `frame path` log line: the GPU path must survive a client's exit. Resize a client on SMP, because setu frees the old slot right after ATTACH. | needs iron |
| B2 | **0.18.2's asm rows on agnos.** `rep stosd` and `rep movsd` are unprivileged, but they have run only on Linux. A QEMU run of agnos covers this one. | needs agnos QEMU |

### Here

| # | Item | State |
|---|---|---|
| D3 | **A client that exits leaves its window behind.** 0.18.2 bounds the cost; the window itself still stays, and a client's own `SETU_CLOSE` is still rejected (`setu_hs_action`, state 3). | open (the rest) |
| D7 | **A wedged compositor writes no log.** The `/klug.txt` spill covers a crash and a quit, but not a hang. | open |
| D13 | **A window changed during a gesture snaps back.** A drag or resize rebuilds the rect from the press-time anchor (`src/route.cyr`), so a keyboard chord that maximizes or moves the window while the button is held is undone by the next motion, and a maximized window keeps WS_MAXIMIZED with the anchored rect. End the gesture when the window changes under it. | open |
| B3 | **agnos client rows are append-only and spawned children are never reaped** (CL-9, STATE-5). The ninth launch of a session is refused, and each exited child pins 1 of 16 process slots. Needs the kernel's `waitpid` read first, and a burn, because a blocking wait would freeze the desktop. | after a burn |

### Cleanup

- `README.md`'s build steps still begin with `cyrius lib sync --full`, which has been wrong since 0.16.27.
- `programs/puka_desktop_probe.cyr` does not build (`ae_bhumi` undefined, reachable). CI builds no program, so nothing
  catches this; CI should build them.
- **`"sys"` can be declared.** `src/sysprobe.cyr` includes `lib/sys.cyr` by path because declaring it once compiled
  nothing in. The declared form built on 6.6.11 and has not been re-measured since. Moving to it and correcting that
  file's comment is its own change.
- **`map_u64` is walked by hand in three places** (`ai_features.cyr:671`, `security_ui.cyr:970`, `plugin_host.cyr:427`).
  Each breaks silently if its layout changes. An iterator would need a cyrius issue.
- **`--aarch64` does not build** (`SYS_UNLINK`, `src/setu_dispatch.cyr:55`), so the asm rows' fallback arms are compiled
  by nothing. Nothing ships for aarch64 yet.
- CI could add `cyrius deps --locked`: a stale lock passes today, because `cyrius deps` rewrites it.

## 0.19.x — input

### The Super key replaces Ctrl as the chrome modifier (planned; operator, 2026-10-09)

Every chrome key today is a Ctrl chord (`input_map`, `src/input.cyr`; the claimed set is in
[`001-setu-client-contract.md`](../architecture/001-setu-client-contract.md) §3). Any other key pressed while Ctrl is
held is swallowed, because the wire carries no modifier state. So no client ever sees a Ctrl chord, and a terminal
cannot receive Ctrl+C. Rust had no compositor shortcuts at all: every key went to the focused surface with its modifier
mask. This is therefore desktop work, not parity. bhumi already emits Super as HID usages 0xE3 / 0xE7 on both targets.
That is verified in QEMU for Linux (evdev 125 / 126); on agnos it comes from the boot report's modifier byte and has not
yet been seen on iron.

| Step | What | Owner | Gate |
|---|---|---|---|
| SK1 | Super becomes a second chrome modifier: `input_mod_track` folds 0xE3 / 0xE7, and every chrome action answers Super as well as Ctrl. Settle the binding table first: keep the F-keys or move to Super + letters and arrows, and decide whether a bare Super tap opens the launcher. Verified with `scripts/qemu-linux-input.sh`. | here | none |
| SK2 | Modifier state on key events (ADR 0002 lists it as owed). puka turns Ctrl + letter into a control code, and dhancha exposes the mask. | setu, puka, dhancha | can run beside SK1 |
| SK3 | The flip: the compositor stops claiming Ctrl, Ctrl chords reach clients with their modifiers, and Super is the only chrome modifier. The claimed set in the client contract changes for the third time, and `scripts/qemu-linux-input.sh` gains Super chords (QMP qcode `meta_l`) plus a check that a Ctrl chord reaches the focused client. | here and the clients | SK1 and SK2 |
| SK4 | On iron, the Keychron's GUI key through agnos's HID boot path: one keypress, declared in advance on a burn, like D8. | burn | before SK3 reaches iron |

### Who may take focus

- **I1 (CL-4): any client's first present takes keyboard focus,** with no operator action and no rate limit, so a local
  process can reconnect while the operator types and read keystrokes meant for another client. This matches Rust's
  `create_window` focus and is not an ADR 0003 divergence, so changing it is a decision: focus only a window the operator
  launched, and map the rest unfocused. It needs an ADR.

## 0.20.x — present and pacing

| # | Item | Owner | Gate |
|---|---|---|---|
| D2 | **Present-protocol damage.** setu has no damage concept, and every live surface is re-read or re-blitted every frame whether or not it committed. A maximized 2560×1408 terminal moves about 28.8 MB per frame. This needs COMMIT as an opt-in change signal, plus dirty rects. It also settles the toolkit boundary's C4a. | setu, then here | after D1 |
| P1 | **Band or skip the Linux present.** Every CPU frame falls to a full-frame `bhumi_backend_present`. Measured by proxy at 2,140 µs against 13 µs for 19 rows, so after 0.18.2 present costs more than render. Needs a bhumi band export, and the panel, notifications, focus and titles must join the band. | bhumi, then here | D2's damage model |
| P2 | **A Linux frame clock.** The only pacing is the agnos yield, so the Linux loop spins one core whatever a frame costs, and a faster frame only spins faster. | here | none |
| P3 | **Skip an unchanged window.** Measured 561 → 24 µs on an idle frame, but focus, title and z-order changes are not in the damage model, so it would draw stale. Do it after P2, or it only spins faster. | here | P2 and D2 |
| P4 | **Mixed GPU and CPU frames (LC-4).** The plan is all-or-nothing, so one client on a slot the GPU cannot read keeps every window on the CPU while it is on screen. That beats the permanent demotion it replaced, but the cost falls on the other clients. | here | after D1 |

## 0.21.x — protocol depth and client identity

| # | Item | Owner | Gate |
|---|---|---|---|
| D9 | **Protocol depth owed by ADR 0002:** title and app_id, fullscreen and other client state requests, size hints, dialogs, popups, clipboard and drag-and-drop, IME, output info and scale, pointer leave and horizontal scroll, and a version handshake. | setu, then here | SK2 |
| D4 | **Per-PTY window size.** There is no TIOCSWINSZ on the `#97` band, so `ls` lays out columns for the console grid instead of puka's window. | agnos, puka | open |
| D6 | **crab has no maximize view.** | crab | open |
| I2 | **A buffer id is not bound to the client naming it (CL-10).** One client can show another's pixels in its own window, on screen only. On Linux compare the file's owner with `SO_PEERCRED`; agnos needs a kernel grant. | here, agnos | open |
| I3 | **The default rendezvous is `/tmp/setu-display.sock` (CL-11),** with no mode or peer check, so another uid can squat it. Move the default under `$XDG_RUNTIME_DIR`, mode 0600, and refuse a foreign node loudly. | setu, then here and every client | open |

## 0.22.x — library code wired, and coverage

| # | Item | Owner | Gate |
|---|---|---|---|
| D10 | **Wiring library code (new work, not parity):** a system screenshot through `screen_capture` (it doubles as the `--ppm` pixel oracle), agent capture requests (new setu kinds), a recording consumer, and gestures once touch exists. On iron a screenshot needs the CPU path for that frame or a kernel read-back. Each wiring owes its caps: suggestions, active apps, agent permissions, gesture lists, requires-action notifications and stopped recording sessions all grow without one today. | here | none |
| D14 | **Per-window opacity (PX-2).** `win_set_opacity` has no caller, so the coverage slot uploads its 3.69 MB once per run. Wiring it makes that per frame: track a high-water mark and upload only that, first. | here | none |
| T1 | **Coverage to 80%.** Raise `--min` as it climbs, and never lower it to make a red run green. | here | none |

## 0.23.x — the foreign-app lane

| # | Item | Owner | Gate |
|---|---|---|---|
| D11 | **ADR 0002's "mehman owes":** multi-window guests; guest-driven title, state and size; input and focus; clipboard; real pixels (mehman ADR 0004). | mehman, then here | D9 |

## On hold, and owned elsewhere

**Owned by agnos, tracked here because the desktop feels them:**
- The `VFS_CHAN` close leak, still open on agnos 1.57.10's roadmap.
- Keystrokes lost at holds of about 100 ms (1.56.41). The recorded cause is contradicted by the code, and agnos's open
  list no longer carries it, so nobody owns it.
- `#72` and `#73` have no owner gate, so a client can read or write the compositor's glyph and wallpaper slots directly.
- `scripts/burn/stage-tools.sh`'s exit list should gain 77, the selftest's "far check never completed" (0.18.2).
- The aarch64 build is broken.

**On hold by operator ruling (2026-08-17): M6-B, Linux as a display target.** Two items are left for when it resumes:
- B2, a pixel oracle CI can diff; D10's screenshot provides it.
- B7, the client-side Linux gaps: in puka, `PUKA_SHELL` is a compile-time constant, and `pty_spawn` reports the pid before
  `execve` succeeds; in crab, `readdir`, `stat` and the input loop are agnos-only.

DRM/KMS becomes a second bhumi backend after fbdev.

**Toolkit boundary.** C4a's remaining question belongs to setu: whether COMMIT becomes the opt-in change signal, which is
the same decision as D2. **C4b, the `murrahir` EditorGUI port: do not start, scope or clone it** until C4a closes
(operator, 2026-08-16).

## Standing rulings — do not re-derive

- **Wayland is refused, not ported** (ADR 0001). Foreign Wayland and X11 apps go through mehman's firewalled compat lane.
- **The GPU path is the agnos ring-3 band (`#82`–`#94`) by direct syscall.** There is no `[deps.mabda]`, and there should
  not be one. QEMU exposes no AMD device, so a green QEMU run says nothing about the GPU path; only iron does.
- **A wallpaper is an image**, a PNG or a JPEG decoded by chitra. Generative and shader wallpapers are deleted, not
  deferred (operator, 2026-08-16).
- **App-facing behaviour lives in dhancha, not the compositor.** Any procedurally generated visual content goes in a
  library, never in the compositor or the kernel (operator, 2026-08-16).
- **`#91 gpu_blit_bb` is falsified for this architecture**, because every window is a live client surface re-composited
  each frame. **Batched `#92` is not reachable** without a slot model or an ABI field (`planning/desktop.md:396-412`).
- **Check the op list before calling a gap structural.** C3's "structural" cliff turned out to be one missing primitive.
- **A producer that writes alpha 0 ghosts additively under `#92`** (`out = src + dst`), so every client buffer must write a
  real alpha byte.
- **Linux is a real display target** (fbdev first, DRM/KMS later). It is on hold, not cancelled.
- **Retired with the Rust original (ADR 0004):** the app layer (`apps.rs`), `theme_bridge`, `shell_integration`, quick
  settings with `KeyboardNavConfig`, the three HUD widgets, and the inert flags `--backend`, `--kiosk`, `--no-ai`,
  `--high-contrast` and `--accessibility`. **`gestures` stays dormant:** it is tested library code with no touch path yet.
- **A module that was library-only in Rust and is library-only here is at parity.** Wiring one into the desktop is new work.

## Out of scope for v1.0

- Running foreign Wayland or X11 clients natively. That is mehman's firewalled lane (ADR 0001).
- Enforcing plugin sandbox profiles with kavach. Rust's profiles were data only, so this would be new scope.
