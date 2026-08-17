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

- **A1 ✅ DONE 2026-08-16 — the staged binary is reconciled and every size now carries a hash.**
  `stage-tools.sh` → `cmp` against the local build → `burn-prep.sh`'s own staleness gate, all three
  green, and `state.md` quotes on-disk and on-iron as SEPARATE rows. ⚠ The hazard recurred twice more
  while closing it, which is why the rule is a rule: agnos `build/agnos` at **1,985,688 B** is
  `b146780831c6e442` burned and `32927af12471b44b` after a `check.sh` rebuild — same byte count, two
  binaries. Original note: `state.md` quotes 13,584,728 B as if the size identifies the
  artifact. Measured 2026-08-12: **three distinct binaries share that byte count** — `build/aethersafha_agnos`
  (`1b8b3c57`), the staged `agnos/build/rootfs/bin/aethersafha` (`690160b9`), and a fresh local rebuild
  (`a692279f`). A burn scheduled today flashes an unknown one. Rebuild, restage, re-measure, and quote a
  **hash** beside every size from now on.
- **A2 ✅ DONE 0.14.1 — the min/max button path names itself.** `input_apply_from(..., src)` records
  action + source and `main` prints one line at the click site. ⚠ Recorded rather than printed from the
  leaf module, so 44 unit assertions stay quiet and a test can assert the taxonomy with no output.
  ⛔ The KEY path had to be tagged too, or `input_last_src()` reports a stale BUTTON and a capture
  claims a click that never happened — confidently wrong is worse than absent. Original note: `AE-M` is the only unburned rung, and its button path is
  un-adjudicable from a capture: `input_apply` (`src/input.cyr:242-278`) prints nothing for either the
  button or the F5/F6 key, so the two are indistinguishable in the serial and the operator's eye is the
  only oracle. This arc has been burned by exactly that shape twice. Then it rides the next burn.
- **A3 ✅ DONE 0.14.1 — `#89` byte +28 consumed.** Both gates now required (caps bit3 = "`#92` runs at
  all" AND the op bit = "this op exists"); op 0x01 and 0x03 gated separately; the mask is printed at
  probe time; decoding is a pure `ae_gpu_op_supported(mask, op)` with 10 assertions. Original note: (the `#92` op-support mask) in `ae_gpu_probe` and route op dispatch off
  it, instead of discovering support by calling and reading the error.
- **A4 🟡 — H3 CLOSED 2026-08-16, H5 still open.** The operator drove the 1.56.45 burn **with the USB
  mouse**: `hid: first mouse report accumulated` → `ptrscan: first sample handed to ring 3` →
  `pointer motion received -- the cursor is live` → `drag started` → `pointer button click routed` →
  `drag released`, and the opacity check was done by hand with it. ⇒ **The mouse works on iron; no
  dedicated burn is owed for that.** ⚠ **H5 is NOT closed and must not be read as closed** — it asks
  whether a MEDIA KEY on the Keychron can move the cursor or synthesise a click, and that stimulus was
  never applied. It costs one keypress on any future boot; declare it in advance and watch the pointer.
  Original entry: one iron burn with the USB mouse attached, closing agnos tracker H3 and H5 in one boot with
  the media-key stimulus **declared in advance**. H5 is a real defect if it fails: the kernel issues
  `SET_PROTOCOL(Boot)` but never `GET_PROTOCOL` and never parses the report descriptor, so if the
  Keychron's phantom mouse interface is not mute, **a media key can move the cursor or synthesise a
  click**. Depends on A1 — we must know which binary is flashed.
- **A5 🔴 — agnos 1.56.45 is OPEN** (operator, 2026-08-16). 1.56.44 RELEASED and burned — it carried the
  `#92` op 0x06 α dword C3 needed. The two kernel defects below are still open against 1.56.45.
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
  ⭐ **Extended plane CLOSED too (bhumi 1.4.1 / 0.13.8)** — arrows, Home/End/PgUp/PgDn, Ins/Del,
  RCtrl/RAlt, Meta and Menu all arrive (QEMU: usages 82/80/74/76/41). ⛔ The gap existed because the
  base plane worked for free — Linux `KEY_*` base codes ARE Set-1 make codes — while the extended plane
  is 0xE0-prefixed in Set-1 and flat in evdev, so bhumi's existing table could never match here.
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
  - **C1a ✅ DONE 0.14.0, CORRECTED 0.16.0 — `src/wallpaper.cyr`.** ⛔ 0.14.0 built a COLOUR SOURCE, not a
    wallpaper: solid + gradient under a "never varies in x" doctrine that is meaningless for an image.
    0.16.0 makes it a FILE (PNG/JPEG via `[deps.chitra]`) and retires the doctrine.
    Original entry: Row-uniform sources (solid, vertical gradient) drawn
    through the same `AE-0a` band the clear used, with a serial so a wallpaper change invalidates the
    band honestly. QEMU-verified: row 799 samples exactly `c1`. ⛔ Non-solid demotes the GPU frame
    (latched + named) because `#88` fills one colour and `AE-9` is all-or-nothing. Original scope: A backdrop that is a *source* rather than a constant: solid,
    gradient, or an image buffer, owned by the compositor beneath every window, damage-tracked like
    any other surface. This is the load-bearing bite — without a layer, a shader has nowhere to draw.
    ⚠ Interacts with `AE-0a`: the clear is currently ~48% of a GPU frame and a wallpaper REPLACES the
    clear rather than adding to it, so this is not automatically a cost. Measure, do not assume.
  - **C1b — ⛔ DELETED 2026-08-16 BY OPERATOR RULING. A WALLPAPER IS AN IMAGE.**
    This rung proposed "generative / shader wallpapers — computed per frame, not asset-streamed; the
    wallpaper *is* a small program, not a stored image", carried over from
    `planning/desktop-design-ideas.md`. **It is not a goal and never was one.** Operator, verbatim:
    *"WALLPAPER SHOULD FUCKING WORK WITH A GOD DAMN IMAGE >>> HENCE THE FUCKING NAME PAPER."*
    ⇒ C1 is CLOSED by C1a: a wallpaper is a PNG or a JPEG, decoded by `chitra`, and there is no second
    kind. Do not re-open this under any name, and do not treat the design-ideas doc as a wallpaper brief.
  - **C1c ✅ ANSWERED 0.16.0 — the format is PNG and JPEG, and the question was mis-framed.** It asked
    how a wallpaper is "expressed and shipped sovereignly — a Cyrius source, a data file, a shader
    artifact?" ⛔ A wallpaper is a **file**, and the sovereign decoder already existed (`chitra`).
    ⚠ The shader-artifact option was never real on agnos anyway: `#92` exposes a **fixed op list**
    (`GPU_OP_SUPPORTED`), not programmable shaders — there is no upload-a-shader op to author for.
    ⛔ This entry used to end *"C1b's generative wallpapers therefore mean a Cyrius generator compiled
    in"* — struck with C1b. **The backdrop is the largest and least-changing surface on screen, so
    recomputing it per frame spends the most cycles on the least information** and defeats `AE-0a` by
    making every frame full-screen. A still wallpaper is an image; a moving one is a video file.
- **C3 ✅ DONE 0.16.2, IRON-PROVEN 2026-08-16 — per-window opacity, chrome and content.** On archaemenid:
  `#89 +28` = `0x1FF5F`, `#92` **op 0x06 `BLEND_ALPHA`** composites the client surface and **op 0x02
  `BLEND_COV`** (constant-coverage mask) composites the alpha chrome, both on the GPU, with a live
  translucent window over a working terminal.
  ⛔ **THE ANALYSIS BELOW IS KEPT AS HISTORY AND IS NOW FALSE IN ITS TWO LOAD-BEARING CLAIMS**, because
  the reasoning is worth reading and a silently deleted argument gets re-derived:
  - *"PER-WINDOW OPACITY IS NOT EXPRESSIBLE IN THE `#92` ABI"* — **false since agnos 1.56.44.** It was a
    correct reading of the ABI as it stood, and the answer was to change the ABI: op 0x06 puts α in
    dword 9. The item was filed as a kernel ask, agnos took it, and it burned green.
  - *"A single translucent window would force the ENTIRE frame back to the CPU — 6.40 ms → 10.58 ms"* —
    **false, and it was the more expensive mistake.** The perf cliff was treated as structural when it
    was really one missing primitive: `fill_rect_a` had no GPU route. Two burns went to discovering
    that (invisible titlebars, then mis-stacked overlap) before op 0x02 — advertised since it was `#93`
    — turned out to be exactly the operation. ⇒ **Check the op list before calling a gap structural.**
  - ⚠ Still true and still the rule: applying α to a premultiplied surface must scale colour AND alpha
    (op 0x06 does), and the knob is whole-window — chrome and content both.
  Original analysis (kept verbatim below): Recorded with its
  constraints measured, because the obvious reading — "the blend already ships, so this is a render
  tweak" — is wrong on the target that matters.
  - ⭐ **What already exists**: `#92` op 0x01 does real premultiplied src-over on the shader cores
    (`out = src + dst*(1 - a/255)`), iron-proven as `AE-6`, and the CPU path already has
    `rend_blend(dst, src, a)` (`src/render.cyr:191`, Rust-parity). So the *blending* is solved.
  - ⛔⛔ **BUT PER-WINDOW OPACITY IS NOT EXPRESSIBLE IN THE `#92` ABI.** The op record is
    `op / src_id / wh / dstxy` and **"every dword the op does not define MUST be zero — the kernel
    REJECTS a non-zero reserved field"** (`src/gpu.cyr:376-390`). There is nowhere to put an opacity
    scalar. What ships today blends **per-pixel alpha the CLIENT authored**; a compositor-applied
    uniform α over an arbitrary surface is a different operation and the kernel cannot be asked for it.
    ⇒ This is an **agnos KERNEL item** (a dword in the op record, or a new `#92` op), not userland work.
    Do not start it as a renderer change.
  - ⛔ **Premultiplied makes the naive fix wrong.** Applying α to a premultiplied surface means scaling
    **colour AND alpha** by α. Scaling alpha alone yields an over-bright composite — the same class as
    the alpha-0 additive ghost recorded in `planning/desktop.md:352-356`, and just as silent.
  - ⛔ **The perf cliff is the real cost, and it is structural.** `AE-9` deleted the `#39` chrome blit:
    on a GPU frame there is no CPU layer left to copy, so a CPU-blended window cannot be mixed into one.
    A single translucent window would therefore force the **entire frame** back to the CPU path —
    measured 6.40 ms → 10.58 ms at 2560x1440. Opacity is not a per-window cost, it is a per-frame one.
  - ⇒ **Sequencing.** (a) Do it on the **CPU path first** — Linux has no GPU path at all, so there is no
    cliff there and `rend_blend` is already the primitive; that makes the feature real and testable on
    one substrate. (b) File the `#92` opacity dword as a kernel ask on the agnos side. (c) Only then
    wire the agnos GPU route. ⚠ Until (b) lands, an opacity-bearing window on agnos must either demote
    the frame **loudly** (a latched log line, not silence) or be refused — decide which before shipping.
  - ⚠ Decide the SURFACE of the knob too: whole-window (chrome + content) vs content-only. Chrome is
    compositor-drawn and trivially fadeable; the client surface is the part with the ABI problem.

#### M7 — THE NEXT PHASE: the toolkit becomes the surface (opened 2026-08-17)

> ⭐ **WHY A NEW MILESTONE RATHER THAN MORE M6-C RUNGS.** M6-C was "the thing a person looks at" —
> wallpaper, opacity, chrome. Those are COMPOSITOR properties and they are done and iron-proven. What is
> left is not more compositor surface: it is that **apps still cannot be built cheaply**. crab hand-draws
> its dual-pane UI, puka hand-draws a terminal grid, and the launcher I just added hand-draws a list —
> three programs, three private renderers, one toolkit none of them fully uses. The next phase moves
> that weight into the library.
>
> ⛔ **THE OPERATOR RULING THAT SHAPES ALL OF IT:** app-facing behaviour lives in **dhancha**, not in the
> compositor. aethersafha owns COMPOSITING; the toolkit owns what an app draws and how it behaves. Every
> row below is filed on that basis, and the design brief's §2 claim that a motion language is
> *"a compositor-level concept"* is superseded by it.

| Rung | What | Why it is next | Gate |
|---|---|---|---|
| **M7-A — dhancha widget depth** ✅ **DONE (dhancha 0.9.7)** | Shipped: `LIST`, a scrolling + selectable container, with the scroll offset applied at LAYOUT time so hit-testing needs no knowledge of scrolling. ⛔ It also surfaced a standing defect the rung had not anticipated: the PAINTER never clipped, so a child drawn outside its parent was visible-but-unclickable (the hit-test has always rejected it). `dh_draw_widget` now clips, on every container. 55 checks, mutation-tested. ⚠ Original entry: The tree has WINDOW / BOX / LABEL / BUTTON / TEXTINPUT and nothing else. A file manager needs a LIST/scroll container; a terminal needs a text surface; the launcher needs exactly the list crab and I both hand-rolled. ⚠ `dh_hit_test` was taught to CLIP to the parent at 0.9.5 precisely so a scroll container is expressible — that fix is this rung's prerequisite, already paid. | Three programs are re-implementing the same list | run-tests + a pixel proof per widget |
| **M7-B — TEXTINPUT actually takes input** ✅ **DONE (dhancha 0.9.7)** | Shipped: a UTF-8 buffer on `DH_W_TEXT` (the same field a LABEL draws, so no new painting code), a caret that steps whole characters, and the edit step wired into `dh_dispatch` — AFTER propagation, so an app handler that claimed the key wins. 80 checks, mutation-tested. ⚠ Two of those checks exist only because the first suite passed a mutation that unwired the dispatcher entirely. ⚠ Original entry: The kind exists in the enum. Nothing routes keys into it, so no dhancha app can have a text field — which is why the launcher has no filter box and crab has no rename. | The first widget an app asks for after a button | `event_test` sub-tests |
| **M7-C — the motion language, IN dhancha** | Design brief §1/§2: one "the system is working" vocabulary apps inherit instead of each shipping a spinner. ⛔ In the LIB. ⚠ Open question the brief never answered and this rung must: what the toolkit GRANTS vs what an app may override. Decide before code. | The brief's own two surviving ideas | pure easing/field functions are host-testable; the draw needs a pixel proof |
| **M7-D — crab and puka onto the widget tree** ✅ **DONE (crab 0.4.10, puka 0.6.15, dhancha 0.9.8+0.9.9)** | Both consumers DELETED their hand-rolled equivalents, which was the gate. crab's panes are a `dh_list` — its 7-row cap is gone (it was a REACHABILITY limit: 33 of 40 files could not be selected, not just not drawn) and its selection-highlight rule moved into the toolkit. puka's window is a widget tree with the grid as a `CANVAS`; its renderer is untouched and its present path gained no copy. ⛔ The port found what A had missed: LIST stored a selection it never PAINTED (0.9.8), and there was no "text surface" for a terminal at all (0.9.9 CANVAS). ⚠ It also found a real defect — puka's present blit left the alpha byte at ZERO, which under `#92` op 0x01 ghosts additively on iron and is invisible to every host-side RGB check. ⚠ Original entry: The reunion. dhancha is *"the spiritual extraction of puka's windowing code"*, and puka still hand-draws. crab uses dhancha for pixels but hand-rolls its panes. ⚠ Gated on A and B — porting onto a toolkit that lacks a list and a text field just moves the hand-rolling inside. | Proves the toolkit carries real weight; a consumer that STOPS hand-rolling is the only evidence | `presented: 2` + per-app pixel proofs |
| **M7-E — runtime theme switching** | rupa 0.1.2 ships **four** variants (MUDRA/SHANTA × dark/light) and `rupa_theme_set_active`, and the desktop picks one at boot and never changes it. The themes exist; the switch does not. ⚠ Cheap, and it exercises the damage model hard — a theme change invalidates every pixel without moving a window, which `rend_band_compute` already has a serial for. | The design work is already done and unused | a pixel proof that the whole frame repaints |
| **M7-F — the shell panel earns its place** ✅ **DONE (0.16.4)** | Closed the M2 remainder AND found two things it was not looking for: the panel was the ONE surface M7-E's theme switching could not repaint (it hardcoded its colours, so a light theme left a dark slab across the top), and the whole `Notification` record — title, body, app, priority, requires_action — had NEVER been drawn; only the count was, as a red square. `disk` and `agents` also had zero readers. ⚠ Two of the new tests exist because earlier versions passed for the WRONG reason: a label check scanned past the framebuffer's right edge, where out-of-bounds reads return black. Slot geometry is now pure functions the painter must use. ⚠ Original entry: `render_shell_panel` draws a status strip; M2 still lists *"notifications surface, quick-settings, panel text labels (cpu/mem %)"* as unbuilt. With a launcher on F2, the panel is the natural home for it. | Closes the oldest open M2 remainder | pixel proofs |

⚠ **NOT IN THIS PHASE, DELIBERATELY:** M6-B (Linux as a display target) is **on hold by operator ruling,
2026-08-17**; the generative/shader wallpaper is **deleted, not deferred**; and the `murrahir` EditorGUI
port stays hard-gated behind M7-A/B — porting a second unfinished surface alongside the first is how
both stall.

⛔ **SEQUENCING IS A → B → D, with C, E and F independent.** A and B are the toolkit's missing primitives
and D is the proof they work; running D early produces a port that re-implements the gaps inside the
consumer, which is the state the stack is already in.

> **STATUS 2026-08-17: E ✅, C ✅ (rupa 0.1.3 + dhancha 0.9.6), A ✅ B ✅ (dhancha 0.9.7),
> D ✅ (crab 0.4.10 + puka 0.6.15 on dhancha 0.9.9), F ✅ (0.16.4). **THE PHASE IS COMPLETE.**
>
> ⭐⭐ **WHAT D ACTUALLY PROVED, AND IT IS NOT WHAT THE ENTRY EXPECTED.** The gate was *"a consumer
> that STOPS hand-rolling is the only evidence"*, and both did — but the port's real value was that it
> falsified two claims A had already banked:
> - **LIST owned a selection it never drew.** crab ported, deleted its truncation and its scroll
>   arithmetic, and kept `if (selected) { bg = accent }` completely untouched. A container that owns
>   the selection must own how it LOOKS. -> dhancha 0.9.8.
> - **There was no "text surface", which M7-A had explicitly named.** A terminal grid does not
>   decompose into widgets — 1920 cells as widgets costs more than the scrollback and discards the
>   dirty-row scanner. -> dhancha 0.9.9 `CANVAS`.
>
> ⚠ **AND IT FOUND A LIVE DEFECT NOTHING ELSE COULD HAVE.** puka's present blit packed
> `(r<<16)|(g<<8)|b`, leaving the alpha byte at ZERO on every pixel it ever presented. Under `#92`
> op 0x01 (`out = src + dst*(1 - src_a)`) that collapses to `out = src + dst` — an additive
> over-bright ghost, not a black window. Every host-side RGB dump of that buffer was correct, and
> `sd_surface_pixel_at` drops alpha by contract, so no existing test could see it.
>
> ⛔ **THE LESSON FOR THE REMAINING RUNGS:** a primitive is not finished when its tests pass, it is
> finished when a consumer deletes code. Both gaps above had green suites on both sides.

- **C2 — a system motion language** (design brief §2). A "the system is working" vocabulary rather than
  every app shipping its own spinner.
  ⛔⛔ **OPERATOR RULING 2026-08-16 — IT LIVES IN THE LIBRARY, NOT THE COMPOSITOR.** This entry used to
  say the vocabulary is one "the compositor OWNS and apps inherit", and the design brief §2 calls it
  *"a **compositor-level** concept, not an app concept"*. **Both are wrong.** App-facing widget
  behaviour belongs to **[`dhancha`](https://github.com/MacCracken/dhancha)** — the pure-Cyrius
  client-side widget toolkit, the Qt/GTK-equivalent layer. ⚠ CORRECTED: an earlier draft of this line
  said dhancha was "already a dep of this repo and of crab". It is a dep of **crab only** — aethersafha
  has no `[deps.dhancha]` block, and the compositor should not: dhancha is the CLIENT side.
  Operator: *"THERE IS SUPPOSE TO BE A FUCKING LIB FOR APP STUFF."*
  ⇒ The "open question is the boundary" framing is answered: aethersafha owns COMPOSITING, dhancha owns
  what an app draws and how it behaves. A motion vocabulary is the latter.
  ⚠ **THE SAME RULE GOVERNS ANY PROCEDURAL/GENERATED VISUAL FUNCTIONALITY**, wallpaper included if it
  ever returns: *"if we have any wallpaper functionality that would be procedurally generated I would
  prefer that it be a part of a library for the functionality and not baked into the compositor or
  kernel."* ⇒ Generated-content code goes in a library, always. The compositor composites; the kernel
  provides the op band. Neither is a place to put content generation.

- **C4 — THE DESKTOP'S REAL FOCUS (operator, 2026-08-16): the GUI toolkit, then editors on the desktop.**
  Ahead of any further compositor-surface polish. Two rungs, in order:
  - **C4a — `dhancha`, the GUI toolkit lib. ⛔⛔ IT IS DISCONNECTED FROM THE SOVEREIGN DESKTOP, MEASURED
    2026-08-16.** dhancha 0.9.3 has the widget tree, flexbox layout, event dispatch, capture/bubble,
    drag-drop and a rupa-themed draw stack — all shipped and RUN-tested. What it does NOT have is a way
    to reach the compositor **on agnos**: `src/dh_client.cyr` and `src/setu_client.cyr` contain **zero**
    `CYRIUS_TARGET_AGNOS` arms and no `AGNOS_CHAN` / `chan_op` / `#97` reference at all.
    ⇒ **Every windowed app on agnos hand-rolls its own client**, which is the exact thing the toolkit
    exists to prevent (its README: apps build on dhancha *"instead of hand-rolling raw GPU + a raw
    compositor connection"*). Verified on the reference consumer: `crab/src/main.cyr` carries 3
    `CYRIUS_TARGET_AGNOS` arms and calls `setu_client_connect` itself, using **no** `dh_client` and no
    `dh_surface_present` — dhancha is its DRAWING library only.
    ⚠ **HOW IT REGRESSED, and this is the operator's "again":** dhancha 0.7.0 shipped *"a real dhancha
    app on the sovereign desktop"* over setu's **TCP** transport (0.6.3 adapted to it). That transport
    was then RETIRED as the wrong primitive for local display IPC, replaced by the agnos channel band
    `#97` where the compositor mints a channel and endows an end at spawn. setu came across (0.8.5);
    **dhancha did not.** A working client was left behind by a substrate change under it.
    ⇒ The bite: give `DhClient` an agnos arm over the channel band, then make crab consume it — a
    consumer that stops hand-rolling is the only proof the toolkit actually carries the weight.
    ⚠ README scope is STALE and must not be planned from: it lists *"v0.6+ — next: the compositor-fd
    input source … the present path"* while the repo is at 0.9.3 with both long shipped (for TCP).
  - **C4b — LATER, AND HARD-GATED ON C4a. The EditorGUI (`murrahir`) port** — still Rust on GitHub, to be
    ported the way this repo was, for editor programs at the desktop level (`cyim` is the sovereign
    Cyrius-native VIM-inspired editor). ⛔ **DO NOT START, DO NOT SCOPE, DO NOT CLONE IT.** Operator,
    2026-08-16: *"we haven't even gotten the remaining gui basics off the ground again and you think I'm
    ready to bring it down for porting?"* A port drags a second unfinished surface alongside the first.
    ⇒ It is a NAME on the map so the direction is known, nothing more. C4a finishes first.

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
    precisely so a caller need not probe by trial. ✅ **DONE 0.14.1** — consumed; the desktop no longer
    discovers op support by issuing one and reading the error.
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
