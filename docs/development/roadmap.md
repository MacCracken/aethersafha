# aethersafha — Roadmap

> Milestone plan through v1.0. State lives in [`state.md`](state.md); the
> executable bite-level breakdown (with workflow catalog) lives in
> [`parity-plan.md`](parity-plan.md). This file is the sequencing — what ships,
> in what order, against which dependency gates. The bar is **parity with
> `rust-old/`** (the frozen 27,207-line Rust oracle).

## v1.0 criteria

- [ ] Rust → Cyrius surface parity verified (module-by-module against `rust-old/`)
- [ ] Test coverage adequate for the surface area (≥80% target)
- [ ] Benchmarks captured in `docs/benchmarks.md`
- [x] Runs on the agnos kernel via bhumi (real scanout + input) — ✅ `--selftest` → `exit 95` on
      archaemenid (0.11.1: GPU-composited a client surface at the client's coordinates); geometry read
      live from the kernel.
      ⛔ **CORRECTED 2026-08-12 — this line used to end "Pointer input is still absent on iron (agnos
      xhci matches only HID boot *keyboard*), so real input is keyboard-only." That is FALSE and was
      falsified four days after it was written.** The pointer is iron-proven end to end: `hid:
      boot-mouse interfaces bound: 2`, a titlebar `drag released ... frames held: 7`, and the cursor
      drawn as **two `#92` op 0x03 masks on the shader cores** — agnosticos
      `prior-art/desktop-drag-release-iron-0809.txt:57, :219, :295` and `iron-log.md:152-153, :160`.
      ⚠ Do not replace it with "the GPU cursor arm is unburned" either (`CHANGELOG.md:312-313`) —
      that arm burned on 2026-08-09 and four more times on 2026-08-10.
- [ ] CHANGELOG complete from v0.1.0 onward
- [ ] Security audit pass (`docs/audit/YYYY-MM-DD-audit.md`)

## Backend seam (the platform-I/O → bhumi/mehman split)

| Rust concern | Cyrius home | Notes |
|---|---|---|
| DRM/KMS + libinput + logind (platform I/O) | **bhumi** 1.0.0 | `bhumi_backend_open/fb/poll/present`, seat/cap gating. MVP. |
| Native protocol dispatch, surface tree, window mgmt | **aethersafha** | The compositor's own sovereign protocol — stays here (ADR 0001). |
| Foreign-app surface hosting (was XWayland) | **mehman** 1.0.0 | kavach-sandboxed guests (swallow backend). Wired (Bite G). |
| Shared domain primitives / errors / wire format | **agnostik** 1.3.3 | was Rust `agnostik`. |
| udev + DRM/KMS device model | **agnodrm** 1.4.5 | was Rust `agnosys` (decomposed 2026-06-19). |

## Milestones

### M0 — Port scaffold (v0.1.0) — ✅ 2026-07-03
- `cyrius port` ran; Rust → `rust-old/`; Cyrius scaffold + docs + CI.

### M1 — Foundational base (v0.1.x) — ✅ in progress
Compiling, tested compositor core on the live bhumi seam:
- `src/geom.cyr` — Rectangle primitives.  ✅
- `src/window.cyr` — Window model + WinState.  ✅
- `src/compositor.cyr` — window stack, focus, workspace, CRUD.  ✅
- `src/render.cyr` — software renderer over the bhumi XRGB framebuffer.  ✅
- `src/input.cyr` — bhumi HID → compositor input actions.  ✅
- `src/main.cyr` — entry: open bhumi backend, seed windows, frame loop.  ✅
- `tests/aethersafha.tcyr` — 21 core assertions green.  ✅

### M2 — Leaf feature parity (v0.2.0) — 🚧 first batch landed
Self-contained data-model modules (no deep compositor/bhumi coupling), ported
module-by-module against `rust-old/` (heap offset-accessor structs, module-prefixed
symbols), each compiling + smoke-tested. Driven by the parity workflow.
- `theme_bridge.cyr` (AGNOS→Flutter theme translation)  ✅ ported + smoke
- `gestures.cyr` (tap/swipe/pinch recognition)  ✅ ported + smoke
- `accessibility.cyr` (a11y tree, focus/keyboard nav, high-contrast theme)  ✅
- `ai_features.cyr` (context engine, suggestions, agent HUD, resource metrics)  ✅
- `shell.cyr` (notifications, quick settings, system status, launcher)  ✅
- `security_ui.cyr` (permission model, alerts, dashboard)  ✅
- `shell_integration.cyr` (tray, window-mgmt, notification bridge)  ✅ ported
- `plugin_host.cyr` (lifecycle, sandbox profiles, capabilities; IPC stubbed)  ✅ ported
- Behavioral parity tests for all 8 leaf modules  ✅ (~670 assertions green)
- **B3 wiring** — `desktop` aggregate owns compositor + all 8 leaf managers, created
  by `main`. `render_desktop` = themed bg + windows + shell status panel. Live links:
  compositor→accessibility, theme→renderer, shell→renderer, tray.  ✅ complete
- **Bitmap text** — `draw_char`/`draw_text` over the **kashi** VGA 8×16 console font
  (`[deps.kashi]`); window titles render in titlebars.  ✅
- Remaining (feature depth): notifications surface, input→gestures, quick-settings,
  panel text labels (cpu/mem %), scene-graph/damage-driven redraw.

### M3 — Renderer + compositor depth (v0.3.0) — 🚧 protocol SHIPPED, depth partial
- **Native display protocol — ✅ SHIPPED and LIVE.** `setu` is the sovereign wire (ADR 0001, not a
  Wayland port): server transport + dispatch (`src/setu_server.cyr`, `src/setu_dispatch.cyr`),
  shared-buffer present, full key events. **Two real clients composited as windows on agnos from a
  foreground launch** (2026-08-02, QEMU) — `/bin/puka` and `/bin/crab`, verified on the framebuffer.
- Decorations + bitmap text (kashi) ✅ · damage-limited blit ✅ **SHIPPED 0.12.3 as `AE-0a`** — this
  line used to say it was *"not safe to enable"*, which was true only of the obvious one-liner. The
  band is `union(cur, prev)`, forced by `#84 present` flipping the render target; iron-proven with no
  ghosting across 278 frames, and the moving-window case exercised in QEMU (`AE-M`). Clear cost fell
  3.937 → 2.460 ms (1.60x). ⚠ Still open: the GPU path's band under a *moving* window
  (`planning/desktop.md:204`).
- Remaining: scene graph, drag/resize state machines, workspaces, input routing depth.

### M4 — Apps + capture + plugins (v0.4.0)
- **`apps.cyr` 🚧 C1+C2 done** — app framework + data-model apps + the Command Palette allowlist/basename
  security logic + the **real process spawn** (Command Palette fork+execve capturing stdout/exit-status;
  browser/Shruti detached launch); 133-assertion test. fs/net effect bodies (C3) deferred.
- **`screen_capture.cyr` ✅** — permission model + rate-limit + secure-mode auth + history
  ring buffer + full/region/window capture + byte-exact RAW/BMP/PNG encoders; 90-assertion
  parity test.
- **`screen_recording.cyr` ✅** — recording sessions + start/capture/pause/resume/stop state
  machine + per-session frame ring buffer (on D1); 72-assertion parity test. Both standalone
  (compositor wiring is follow-on).
- `plugin_host.cyr` (Unix-socket IPC, sandbox profiles, capability grants).
- HUD widgets (`hud/{gpu,domain,crew}_status.cyr`) — HTTP polling of daimon MCP.

### M5 — mehman (foreign-app swallow backend) — 🚧 started (v0.5.0+)
- mehman 1.0.1 + kavach 3.11.0 **wired** via `src/foreign.cyr`: guest-spec +
  foreign-surface descriptor + `desktop_host_foreign` → a compositor window;
  `main` hosts a demo guest.
- Guest **execution + capture** via `foreign_run` → `mehman_sandbox_capture_guest`
  (kavach PROCESS fork+exec+reap + surface capture) — done + tested against a live
  `/bin/true` and `/bin/echo` (captured output lands in the surface buffer).
- **Presentation** — `render_desktop_foreign` / `render_foreign_content` draw the
  captured surface as the hosted window's content (line-aware `draw_text_lines`); the
  desktop tracks hosted foreign apps (`desk_foreign`); pixel-tested. ✅
- Remaining: consume mehman 1.0.0's per-ABI `guest`/`shim` modules; real XRGB pixel
  fidelity beyond the stdout-as-framebuffer MVP (mehman ADR 0004).

### M6 — Userland desktop: the arc comes home, and Linux becomes a real target (v0.13.1+) — 🚧 OPEN 2026-08-12

> ⛔ **THIS MILESTONE EXISTED BY NAME IN ANOTHER REPO BEFORE IT EXISTED HERE, AND THAT COST A SESSION.**
> agnos closed the desktop arc at **1.56.42** and handed all forward work to *"aethersafha's own
> roadmap (**M6, userland**)"* — `agnos/docs/development/state.md` and `agnos/CHANGELOG.md:32-34`, both
> naming a milestone this file did not define. A reader following the handoff arrived at a roadmap that
> stopped at M5, and the only way to find "the next desktop work" was to re-derive it from a document
> carrying five falsified claims (all struck above, 2026-08-12). **A handoff to a milestone that does
> not exist is not a handoff.**

**The two tracks are independent and neither blocks the other.** agnos is feature-complete for the
desktop and its remaining items are kernel debt; Linux has never had a screen at all.

#### M6-A — agnos: close the debt, not the features

Every `AE-` rung is shipped and iron-proven (`planning/desktop.md:203-217`). What is left is not desktop
work, it is the bill:

- **A1 — reconcile the staged binary.** `state.md` quotes 13,584,728 B as if the size identifies the
  artifact. Measured 2026-08-12: **three distinct binaries share that byte count** — `build/aethersafha_agnos`
  (`1b8b3c57`), the staged `agnos/build/rootfs/bin/aethersafha` (`690160b9`), and a fresh local rebuild
  (`a692279f`). A burn scheduled today flashes an unknown one. Rebuild, restage, re-measure, and quote a
  **hash** beside every size from now on.
- **A2 — instrument the min/max button path.** `AE-M` is the only unburned rung, and its button path is
  un-adjudicable from a capture: `input_apply` (`src/input.cyr:242-278`) prints nothing for either the
  button or the F5/F6 key, so the two are indistinguishable in the serial and the operator's eye is the
  only oracle. This arc has been burned by exactly that shape twice. Then it rides the next burn.
- **A3 — consume `#89` byte +28** (the `#92` op-support mask) in `ae_gpu_probe` and route op dispatch off
  it, instead of discovering support by calling and reading the error.
- **A4 🔴 — one iron burn with the USB mouse attached**, closing agnos tracker H3 and H5 in one boot with
  the media-key stimulus **declared in advance**. H5 is a real defect if it fails: the kernel issues
  `SET_PROTOCOL(Boot)` but never `GET_PROTOCOL` and never parses the report descriptor, so if the
  Keychron's phantom mouse interface is not mute, **a media key can move the cursor or synthesise a
  click**. Depends on A1 — we must know which binary is flashed.
- **A5 🔴 — agnos 1.56.44 is OPEN** (operator, 2026-08-12) for the two kernel defects below.
- **A6 — the `VFS_CHAN` close leak.** `vfs_close_inner` has arms for every tag but `VFS_CHAN`
  (`agnos/kernel/core/vfs.cyr:35, :1262-1282`); `chan_release_pid` runs only on process death. Fix at the
  `SYS_CLOSE` dispatch, and **derive authority first** — an inherited, non-owned channel fd in a child
  must not release its parent's endpoint. `VFS_PIPE`'s owner-aware release is the shape to copy.
- **A7 — re-derive the 1.56.41 keystroke loss from scratch.** Still open at HEAD (0/9, 4/9, 4/9 keys at a
  ~100 ms hold). ⛔ **Its recorded mechanism is contradicted by the code**: the CHANGELOG says the HID ring
  is drained *only* inside `kbscan #42`, but `hid_poll()` also runs from the 100 Hz timer ISR
  (`pic.cyr:79`) and the xHCI MSI-X handler (`pic.cyr:258`). The cause is **unexplained**, so the proposed
  fix ("a faster frame") rests on a false premise. Instrumented QEMU run counting enqueued vs delivered
  **before** any iron time is spent. This is `AE-T2`'s input path.

#### M6-B — Linux: a sovereign desktop on a second substrate

> ⭐⭐ **DECIDED 2026-08-12 (operator): Linux is a REAL display target, not merely the logic substrate.**
> This reverses the three-substrate matrix at `planning/desktop.md:53-57`, which assigned Linux
> *"protocol logic, layout maths, the render pipeline, every unit suite"* and gave the visual proofs to
> QEMU and iron. It also **supersedes bhumi's ADR 0001** (`bhumi/docs/adr/0001-scanout-via-agnos-fbinfo-blit.md`,
> Accepted, which specifies `-1` stubs for non-agnos targets) and strikes *"Out of scope (for v1.0):
> Non-AGNOS targets"* from `bhumi/docs/development/roadmap.md:99`.
>
> ⭐ **BACKEND DECISION (operator, 2026-08-12): fbdev FIRST, DRM/KMS LATER.** The reason fbdev is not a
> compromise here is that **bhumi's seam is already fbdev-shaped**: agnos `#38 fbinfo` → `FBIOGET_VSCREENINFO`
> and `#39 blit` → `mmap` + row copy, which is a 1:1 mapping onto the two functions the Linux arm stubs.
> DRM/KMS dumb buffers is the durable answer but needs buffer lifetime and page-flip semantics the current
> seam does not express — it is a **second** backend behind the same interface, not a prerequisite.
> ⛔ **Nested Wayland was considered and is NOT the path**: a sovereign compositor rendering into another
> compositor's window is a different product. ⛔ **mabda remains ruled out** (see the correction below).

**Measured baseline, 2026-08-12** — everything above the device seam already works on Linux. The AF_UNIX/
SOCK_SEQPACKET wire, the shared-buffer handoff, `CREATE_SURFACE → ATTACH_BUF → COMMIT`, window mint,
composite, chrome, damage band and exit teardown all run. **The pixels are correct and in RAM; nothing
scans them out.**

- **B1 ✅ DONE 0.13.1 — the host session is real.** Three defects, all in `src/main.cyr`, all found by
  running the binary rather than reading it:
  - `ae_now_ms()` returned **0** on the host arm, so the host build had *no timing diagnostics at all* —
    the 30 s budget never bound, the 5 s progress line never printed, and every elapsed readout was 0.
    Now `clock_now_ms()`. ⚠ It was this blindness that let the two defects below survive.
  - `running = 0` fired on the **first** client present (host arm only). It ended a desktop session on
    frame 1 **and** made `--clients` return at `accepted == 1`, so **exit 95 was structurally unreachable
    on Linux** — the same class as 0.12.9's hardcoded 92. Deleted, not gated: no mode wants it.
  - the probe was bound by the dev frame cap, so its documented **30 s** budget was really **130 ms**
    (measured). `--clients` now runs unbounded and its own three terminators govern.
  - ⭐ New `--frames N`, where **`--frames 0` runs until quit**. Default unchanged (400/3).
  - ⭐ **Verified: `--clients` with two clients now exits 95 on Linux**, both reaped at exit.
- **B2 — a host pixel oracle.** ⚠ Still wanted even though B3 landed: a PPM can be diffed and a screen
  cannot, and CI has no framebuffer. ⭐ Partly served now by `scripts/qemu-linux-desktop.sh`, whose
  screendump IS an external pixel oracle — but it needs QEMU, so an in-process `--ppm` is still the
  cheaper gate for a unit-level check. `--ppm <path>` dumping `bhumi_backend_fb(be)` as P6. ~30 lines; the dumper
  is already written twice in this repo (`programs/setu_serve_probe.cyr:32`, `setu_demo_probe.cyr:15`).
  This is what makes B3/B4 verifiable rather than hopeful, and it is worth having *even after* scanout
  lands, because a file can be diffed and a screen cannot.
- **B3 ✅ DONE — bhumi 1.2.0, 2026-08-12. THE COMPOSITOR DRAWS ON A LINUX SCREEN.** `_bhumi_kfbinfo` /
  `_bhumi_kblit` are real (bhumi ADR 0003). Measured: `screen size read from the kernel / 2560 / 1440`,
  where the same binary read `1280 / 720` from the fallback that morning. **aethersafha needed no
  compositor change** — `bhumi_backend_present` was already called every frame on the host arm.
  ⛔ Two traps avoided by measurement, both silent-failure shaped: the stdlib's `mmap_file_rw` passes
  **MAP_PRIVATE** (writes succeed, nothing reaches the screen), and `memcpy` is a **byte loop**
  (14.7 M iterations per 2560x1440 present). ⭐ Proven by an **external** oracle — a second fd, because a
  readback through bhumi's own mapping would pass under MAP_PRIVATE.
  Original scope, kept for the DRM/KMS successor: Geometry needs **no ioctl** — `/sys/class/graphics/fb0/{virtual_size,
  stride,bits_per_pixel}` fills the existing 24-byte struct exactly, and `amdgpudrmfb` reports BGRX,
  which *is* `BhumiFb`'s store order, so a raw blit is colour-correct. `cyr_mmap`/`mmap_file_rw` already
  exist in `lib/mmap.cyr`. ⚠ bhumi's `[deps].stdlib` must gain `mmap` + `fs`. ⚠ Guard must be **three-way**:
  `CYRIUS_TARGET_LINUX` *is* compiler-predefined (`cyrius/src/main.cyr:1099`), and bhumi today branches
  only on `#ifndef CYRIUS_TARGET_AGNOS`. ⚠ Only safe from a bare VT — on a box running a desktop, `/dev/fb0`
  is that desktop's framebuffer.
- **B4 ✅ CLOSED 0.13.7 / bhumi 1.4.0 — keyboard AND pointer.** One QEMU run delivered motion, a left
  click and Esc through the *emulated USB devices*; the cursor moved by exactly the injected delta
  (centre (640,400) + (400,300) → measured (1040,700)).
  ⭐ **The operator action was never a blocker for development** — this item was recorded as needing the
  dev user in the `input` group; the guest's init is PID 1, so evdev is readable there. Still required
  to run on the host.
  ⛔ **Base plane only.** evdev emits no 0xE0 prefix (`KEY_UP` is 103, not `E0 48`), so arrows,
  RCtrl/RAlt/Meta, Home/End/PgUp/PgDn and Insert/Delete are dropped as unmapped — a second table keyed
  on evdev's own numbering is the remaining work, and it is the one thing standing between this and a
  usable keyboard.
  ⚠ On Linux both streams share one fd and a read consumes, so bhumi drains ONCE and splits; and its
  device scan latched twice before it was right (see bhumi 1.4.0). Unplug is still unhandled.
- **B5 — host client launch.** Today a client must be started by hand in another shell. `plp_spawn`
  (`programs/puka_launch_probe.cyr:34-45`) already proves the fork+execve-then-accept shape on the host.
- **B6 ✅ CLOSED 0.13.6 / setu 0.8.5 — one rendezvous, named by setu.** `setu_un_path` resolves
  explicit path → `$SETU_SOCKET` → `SETU_UNIX_PATH`; all four callers pass **0** (this repo,
  crab 0.4.7, puka 0.6.12, setu's `present_probe`). Verified with no symlink: default **95**, override
  **95**, server and both clients following it.
  ⚠ **This entry was WRONG TWICE before it was right.** It claimed "three defaults across four repos";
  in fact all four spelled the SAME literal and agreed — a hazard (four files editable out of step,
  and `$SETU_SOCKET` honoured by one client in three), not a break. The evidence behind the bad
  diagnosis was an early ECONNREFUSED that was really the compositor having already **exited**.
  ⛔ Two real defects surfaced only by doing it: spawned clients were handed an **empty environment**
  (server bound the override, clients dialled the default, verdict 94), and the new "ask setu" path
  made the listener's own log line **SIGSEGV** on `strlen(0)`. Both were caught by running the binary.
- **B7 — client-side Linux gaps**, handed to the owning repos: puka's `PUKA_SHELL = "/bin/agnsh"` is a
  compile-time constant with no env override and its `pty_spawn` returns the fork pid before knowing
  execve succeeded; crab's `readdir`/`stat` and its whole input loop are agnos-only, so it presents once
  with empty panes and exits.

#### M6-C — Desktop surface: the thing a person actually looks at

Substrate-independent — these are compositor features, and every one lands on agnos and Linux at once
because they sit above the bhumi seam. The visual language is already argued in
[`planning/desktop-design-ideas.md`](planning/desktop-design-ideas.md), which set its own activation
condition (*"when aethersafha's compositor window actually opens, this page is the brief"*) and **has
fired** — it is a brief now, not an idea log.

- **C1 — WALLPAPER.** ⭐ The desktop currently draws a flat themed backdrop and nothing else; the
  backdrop is the largest single surface on screen and the most-seen pixel in the system.
  - **C1a — a wallpaper layer at all.** A backdrop that is a *source* rather than a constant: solid,
    gradient, or an image buffer, owned by the compositor beneath every window, damage-tracked like
    any other surface. This is the load-bearing bite — without a layer, a shader has nowhere to draw.
    ⚠ Interacts with `AE-0a`: the clear is currently ~48% of a GPU frame and a wallpaper REPLACES the
    clear rather than adding to it, so this is not automatically a cost. Measure, do not assume.
  - **C1b — generative / shader wallpapers.** The concept from the design brief: **computed per frame,
    not asset-streamed** — the wallpaper *is* a small program, not a stored image. On agnos this is the
    ring-3 GPU band (`#92` ops); on Linux it is the CPU path until a Linux GPU story exists.
    ⛔ **NOT via mabda** — `desktop-design-ideas.md` calls a shader wallpaper "a mabda surface", which
    is the same false claim [corrected here on 2026-08-01](#-corrected-2026-08-01--gpu-acceleration-is-not-out-of-scope-and-it-does-not-go-through-mabda)
    and left uncorrected in that doc for ten weeks because nothing linked the two. Fix it there when C1b lands.
  - **C1c — authoring + format.** The open question the design brief names and does not answer: how is a
    wallpaper expressed and shipped sovereignly — a Cyrius source compiled in, a data file, a shader
    artifact? ⚠ Decide this BEFORE C1b, or the first shader becomes the format by accident.
- **C2 — a system motion language** (design brief §2). A "the system is working" vocabulary the
  compositor OWNS and apps inherit, rather than every app shipping its own spinner. Open question is the
  boundary: what the compositor grants vs what an app may override.

**M6 exit criteria** — a Linux desktop that opens on a real screen, hosts a real client window, takes
real keyboard and pointer input, draws a wallpaper, and quits cleanly; **and** the agnos debt list
closed with A4 burned.

## Backlog

- ⛔ **CLOSED, NOT BACKLOG — corrected 2026-08-12.** This entry read *"Premultiplied compositing
  (`#92` op 0x01) has never run against a real client. No client sets `SETU_SURF_PREMULTIPLIED` — not
  puka, crab, jalwa, or dhancha…"*. **crab has set it unconditionally since 0.4.x** (`crab/src/main.cyr:38`),
  the compositor consumes it at `src/setu_dispatch.cyr:62, :401`, and **`AE-6` BURNED PASS on
  archaemenid 2026-08-07** with no fallback line (`planning/desktop.md:210`). Routing is per-window in
  `ae_gpu_present_frame`, so a blend-requesting client no longer demotes the whole frame.
  ⚠ **The one live hazard the old entry named is real and survives**: producers that emit alpha 0
  (`puka/src/render/pixfmt.cyr` writes byte 3 = 0; jalwa `gui/draw.cyr` discards a real alpha byte)
  would render as an **additive over-bright ghost** under `#92`, not as a missing window — the shader
  is `out = src + dst*(1 - src_a)`, so `a = 0` gives `out = src + dst`. Harmless while they take the
  opaque `#87` path; a trap for whoever opts them in. See `planning/desktop.md:352-356`.

## Known cleanup
- **Deferred deps** (mehman / agnostik / agnodrm): `cyrius build` auto-prepends
  every `[deps.*]` module, so these unused-but-heavy bundles broke the build —
  mehman→`[deps.kavach]` drags in `sandhi_server_*`/`thread_local_*` (reachable-
  undefined), agnostik+agnodrm collide on `ERR_*`. Deferred (mapping kept in the
  manifest). Re-enable each with a selective `modules = [...]` subset when the
  code that needs it lands (mehman at Bite G; agnostik/agnodrm/mabda as consumed).
- `cyrius lib sync --full` is required before `cyrius deps` (the declared stdlib
  set + bhumi's needs exceed the incremental pin). Documented in CLAUDE.md.

## Out of scope (for v1.0)
- Rust `system_tests.rs` port (verification code, not runtime) — re-expressed as
  `.tcyr` suites per module instead.

## ⛔ CORRECTED 2026-08-01 — GPU acceleration is NOT out of scope, and it does NOT go through mabda

This section previously read *"GPU acceleration — the software renderer is the v1.0 path… wire it
via `[deps.mabda]` when hardware acceleration is wanted."* Both halves are false, and the claim
survived in three live documents at once (here, `parity-plan.md` "Bite H", and agnosticos'
`planning/desktop-arc-handoff.md`), which is more than one session's worth of wasted direction.

- **It already shipped.** GPU compositing landed across 0.9.6–0.9.8: `src/gpu.cyr` composites client
  surfaces with `gpu_blit_shm` **#87** straight out of their GPU-visible shm slots, blends
  premultiplied surfaces with `gpu_shader_op` **#92** op 0x01, and flips with `present` **#84**.
- **There is no `[deps.mabda]` in this repo's manifest, and there does not need to be.** The path
  is the agnos kernel's ring-3 GPU band (`#82`-`#94`), reached by direct syscall. mabda's GPU
  surface rides Linux's driver stack; the sovereign compositor does not go through it.
- **What is genuinely still open** — ⛔ **RE-MEASURED 2026-08-12; this list was four items stale.**
  It used to name `#88 gpu_fill_rect`, `#92` op 0x03 GLYPH_1BPP and `#90` as unconsumed. All three
  shipped: `#88` at `src/gpu.cyr:425, :433` (`AE-9`), op 0x03 at `:482`/`:609`/`:736` (`AE-8`), and
  `#90` has a live consumer at `:915` armed from `src/selftest.cyr:344, :413`. The desktop consumes
  **7 of 13** band numbers and **2 of 14** `#92` ops, not 6 and 1.
  Genuinely unconsumed, and each for a stated reason:
  - **`#89` byte +28 — the `#92` op-support mask.** `src/gpu.cyr:107-113` reads +0..+24 and stops, so
    the desktop still discovers op support by *calling and reading the error* — the exact shape `#86`
    already proved wrong. The kernel documents the bitmask at `agnos/kernel/core/syscall.cyr:4294`
    precisely so a caller need not probe by trial. **This one is worth doing** (M6-A3).
  - **`#91 gpu_blit_bb`** — ⛔ *falsified* for this architecture, not merely unbuilt. Every window here
    is a live client surface re-composited every frame, so the pixels a `#91` copy would move are
    rewritten from the client's buffer anyway. See `planning/desktop.md:396-405`. Do not re-derive it.
  - **Batched `#92`** — ⛔ not reachable as the ladder assumed. Every record names its own `mask_id`
    slot, so N glyph runs sharing one staging slot all read the last run's mask. It needs a slot model
    or an ABI field, not a follow-up. See `planning/desktop.md:406-412`.
  - The 3D ops `0x08`-`0x10` — genuinely untouched, and no consumer wants them yet.

⛔⛔ **CORRECTED 2026-08-12 — THE PARAGRAPH THAT WAS HERE WAS FALSE IN BOTH HALVES.** It read
*"Nothing in this band has ever executed on iron. No desktop binary appears in any `stage_one` call in
agnos's `scripts/burn/stage-tools.sh`…"*. Measured: `stage-tools.sh:302` is
`stage_one aethersafha  src/main.cyr aethersafha || rc=1`, and `AE-2`, `AE-6`, `AE-8` and `AE-9` all
**BURNED PASS** on archaemenid 2026-08-07/08 — the clear, the chrome fills, the client surfaces and
the glyphs are all on the shader cores, with the `#39` blit no longer issued.

⚠ **What IS still true, and is the only part worth keeping:** QEMU cannot substitute for iron. It
exposes no AMD PCI device, so `gpu_find` never matches, `gpu_present` stays 0, `gpu_caps` reports
flags 0, and `ae_gpu_probe` answers 0. **A green desktop smoke in QEMU says nothing about the GPU
path, structurally** — it verifies the CPU fallback.
