# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).


## [0.16.11] - 2026-08-19 — `sys_open` is one name with two contracts

### Fixed — `--wallpaper <file>` could never open the file on agnos

`wp_load_file` called `sys_open(path, 0, 0)`. That is the LINUX contract —
`open(path, flags, mode)`. agnos is `open(name, NAMELEN, flags)` (`SYS_OPEN` = 7), so the kernel
received **namelen 0**, a zero-length filename, and refused every path. The failure was reported
as `rc -1`, which is `sys_open() < 0` — indistinguishable from a missing file.

The arities match, so nothing between the call and the kernel could catch it: it compiles clean on
both targets and only the kernel's answer differs. Now gated per target with
`#ifdef CYRIUS_TARGET_AGNOS`, passing `strlen(path)` on agnos.

Iron 2026-08-19: `ls` listed the PNG at the rootfs root and argv carried the exact path
(`/verify-2560x1440.png`) while the load returned -1. The mangled path in the console transcript
was the keystroke echo, not argv — the compositor echoed the path back correctly.

⚠ `src/apps.cyr` also uses the Linux form (`sys_open("/dev/null", 1, 0)`); it is in no build graph
on either target and `sys_fork`/`sys_execve`/`sys_dup2` do not exist on agnos, so it is unreachable
rather than latent.

### Added — `agnos/scripts/harness/ae-wallpaper-load-test.py`

`wp_load_file` had no test at all, which is why a target-specific contract error shipped. The check
has to run ON agnos — a Linux run exercises the other arm and passes either way. Boots the image,
runs `aethersafha --wallpaper /<file>`, and reads the console: pre-fix **FAIL** (`-1`), post-fix
**PASS** (loaded, 2560x1440). A run that never reached the compositor exits INCONCLUSIVE (2)
rather than scoring a test that was not performed.

The decoded size is read from the two lines FOLLOWING the load message, not by substring: the
compositor also prints the screen size as `2560` / `1440`, so `"2560" in out` passes on a run where
nothing decoded.


## [0.16.10] - 2026-08-18 — the no-band clear reaches the GPU

### Fixed — ⛔⛔ THE FULL-SCREEN CLEAR WAS LOST ON THE GPU PATH

`wp_fill_all`'s SOLID arm called **`bhumi_fb_clear` directly**, which writes the USERLAND framebuffer.
Its banded sibling `wp_fill_band` calls **`rend_clear_band`**, which ENQUEUES a `#88` rect when chrome
is on the GPU. With chrome on the GPU the `#39` blit is skipped — so anything written only to userland
never reaches the screen.

⇒ **The no-band path is exactly the one a THEME SWITCH takes** (`rend_band_compute` refuses to band
across a background change) **and the one the FIRST FRAME takes.** Both silently lost their clear.

⭐ **One root cause for two 2026-08-18 burn reports:**
- *"theme switching works for the titlebar but not background or desktop top bar"* — titlebars are
  chrome rects, enqueued separately, so they updated; the background's clear was discarded.
- *`--selftest` residue surviving into the desktop launched after it* — the first frame is unbanded,
  so its clear went the same way and selftest's pixels stayed in the GPU back buffer.

⚠ **INVISIBLE IN QEMU, which has no AMD device.** With no GPU the `#39` blit runs and the userland
framebuffer IS the screen, so a per-region screendump measured both regions repainting correctly. The
CPU path was never broken; only the path that iron actually takes.

The SOLID arm now delegates to `wp_fill_band`. One clear path, not two.

### Testing

`render.tcyr` 272 (+6). ⭐ The pixel checks alone **cannot** catch this — on the CPU path both calls
write identical pixels, and a mutation restoring the bypass passed them. The discriminating assertion
sets `ae_gpu_frame_ok` and counts `ae_rect_count()`: the clear must ENQUEUE, not merely paint. With
that, restoring the bypass fails.

## [0.16.9] - 2026-08-18 — the launcher joins the damage model

### Fixed — ⛔ THE FLASHING BACKGROUND BEHIND THE LAUNCHER

2026-08-18 burn: *"f2 brought up launcher but started flashing background."*

`render_launcher` paints its panel every frame, and `src/launcher.cyr` registered **no damage at
all** — no `comp_retire_add`, no serial. **No window moves when the launcher opens**, so
`rend_band_compute` banded the frame to whatever the windows produced and the `#39` chrome blit
copied only that. The panel was drawn into the framebuffer and then partially not copied to the
screen, frame after frame.

`render_desktop` now retires the panel rect while the launcher is open, and for one frame after it
closes — the close is the vacate, and a region that stops being drawn is in neither `cur` nor `prev`.

⚠ **Fifth site for this exact fix.** F4-close, F6-minimize, un-maximize and drag-resize each needed
it; it belongs to "a region changed without a window moving", not to any one gesture.

### Investigated — theme repaint: NOT reproducible off the GPU path

Also reported: *"theme switching appears to work for application titlebar but not background or
desktop top bar."* Measured in QEMU with per-region screendump comparison
(`agnos/scripts/harness/ae-theme-repaint-test.py`): **top bar 55/55 and background 39/39 pixels
changed — both repaint.**

⚠ QEMU has no AMD device, so that measurement only covers the **CPU** path; the burn ran the GPU
path (`#92 op 0x01`). Every layer on the GPU side reads correct — `rend_band_compute` marks a theme
change stale (no band = full frame), the GPU frame defaults to full, and `rend_clear_band` enqueues
the background as a chrome rect carrying the live token.

⇒ **Not fixed, because it is not yet reproduced.** ⭐ The most likely explanation is that it was the
SAME defect: the operator pressed F2 first, and a flashing frame reads exactly like "the background
is not following the theme". Re-test after this release before treating it as a separate bug.

### Testing

`tests/launcher.tcyr` +9 (52): the retire union covers the panel rect and the panel lands on screen.

⛔ **The WIRING is not unit-covered, and the changelog says so rather than implying it is.** A test
that discriminates needs a narrow band to exist, and `rend_band_compute` returns early unless the
damage trackers are live — allocation only `main.cyr` performs. Priming frames still leave
`rend_band_valid() == 0`, making any "is the panel covered" assertion vacuously true: a mutation
removing the damage call PASSED one. `tests/desktop.tcyr` states that gap in place of a test that
cannot fail.

## [0.16.8] - 2026-08-18 — the compositor stops reading past a client's buffer

### Fixed — ⛔⛔ MAXIMIZE FAULTED THE DESKTOP ON IRON

2026-08-18 burn: `fault: pid=6 vec=e cr2=0x12600000 err=0x4`, `run: exit 142`. The compositor died.

The GPU client blit takes its extent from the **window**:

```
var w = win_w(win);  var h = win_h(win);
sys_gpu_blit_shm(bid, (h << 16) | w, dstxy)
```

`win_maximize` sets those to the full screen. **The client's buffer does not follow** — it is the
client's, and on agnos a `#86` slot it owns. puka's 640x384 surface was asked for 2560x1416 out of
the same slot, so the blit read ~14x past the end of it.

⛔ **The one gate before that blit checked the wrong thing.** `ae_gpu_window_admissible` compared the
window to the SCREEN — `dx + w > fw`, `dy + h > fh` — which a maximized window passes *trivially*: it
fits exactly. Nothing anywhere compared the extent to the buffer. The comment above the blit read
*"NO BOUNDS RE-CHECK HERE — ae_gpu_window_admissible already cleared every window"*: both sides
believed the other had checked.

⚠ **Same root cause as the drag report.** `win_resize_by` mutates `win_w`/`win_h` identically, which
is why dragging crab "destroyed its visible FB" — the same divergence, landing as corruption instead
of a fault.

### Fixed — the size now travels WITH the buffer id

`win_set_bufid(p, id)` is gone; `win_set_client_buf(p, id, w, h)` replaces it. Three call sites set a
buffer id and only ONE was near the ATTACH that knew the dimensions — recording the size separately
would have been forgotten at exactly the sites that forgot it. `win_buf_covers` answers "is this
window backed", and both the GPU gate and the CPU blit consult it.

### Added — the compositor ASKS the client to resize

`SETU_CONFIGURE` (S->C: id, w, h, state) has been in setu's protocol since it was written, with a
constructor and **zero senders and zero handlers**. `win_notify_resize` now sends it from maximize,
unmaximize and drag-resize; dhancha 0.9.12 delivers it as `WINDOW_CONFIGURE`.

⚠ **The ask and the guard are two halves of one fix.** Shipping the ask alone leaves the fault open
until the client answers; shipping the guard alone freezes the surface at its old extent forever.

### Testing — reproduced in QEMU before and after

`agnos/scripts/harness/ae-resize-fault-test.py` boots the desktop, spawns a client through the
launcher, and drives the sequence. `SEQ=f5` — **maximize alone, no resize and no minimize** — crashed
the pre-fix build on demand and survives on the fixed one.

⚠ The harness scored its first post-fix run as "survived" while `client spawned: False` — a pass for
a test it never performed. It now retries the spawn and exits INCONCLUSIVE rather than passing.

`gpu_fallback` +7 checks (67 total): a maximized window is refused, and admitted again once the client
re-attaches at the new size. Mutation-tested.

## [0.16.7] - 2026-08-17

### Changed

- mehman 1.0.1 -> **1.0.2**, kavach 3.11.13 -> **3.11.14**. 25/25 suites green.
- ⛔ Closes a declared-vs-vendored split: 0.16.6 shipped `lib/kavach.cyr` at **3.11.14** while the
  manifest still said 3.11.13, because kavach resolves by `path` and its tree was mid-flight. kavach
  3.11.14 is now tagged on a clean tree, so the declaration matches the vendored copy.
- ⚠ All nine deps now equal their released versions — audited tag-by-tag against each sibling's
  VERSION, 0 stale.

## [0.16.6] - 2026-08-17

### Changed

- Deps to the 6.5.27 sweep's releases: bhumi 1.4.1 -> **1.4.2**, agnostik 1.3.4 -> **1.3.5**,
  agnodrm 1.5.0 -> **1.5.1**, chitra 0.3.0 -> **0.3.1**. 25/25 suites green.
- ⚠ kavach (3.11.13) and mehman (1.0.1) are unchanged — both are held at their old cyrius pins
  because they do not build under 6.5.27; see the roadmap's C4a note.

## [0.16.5] - 2026-08-17 — toolchain pin to 6.5.27

### Changed — `cyrius = "6.5.21"` -> **6.5.27**

Stack-wide sweep so every repo in the desktop stack declares one toolchain. Pins had drifted across
three lines (6.5.5 / 6.5.20 / 6.5.21) while the installed wrapper was 6.5.27, so every build ran with
a drift warning and the declared graph did not describe what was actually compiled.

⚠ **THE ARTIFACT CHANGED, so this is not a cosmetic edit.** The build went `4171560 -> 4171560` bytes and the binary differs. The pin is not a comment: it selects the stdlib snapshot under `~/.cyrius/versions/<pin>/lib`, so moving it swaps the library code this repo compiles against.

⛔ **THE PIN IS NOT JUST DOCUMENTATION**, which is the half-truth that made this sweep's first
prediction wrong. `cycc` is the installed binary either way — but the **stdlib** resolves from
`~/.cyrius/versions/<pin>/lib`, so the pin decides which library sources compile in. Measured before
any other change: the pin bump ALONE moved these bytes.

⚠ The vendored `lib/` was then re-synced to the 6.5.27 bundled set, clearing the
`./lib/ shadows version-pinned` warning. Tests re-run green after both changes.

## [0.16.4] - 2026-08-17 — the panel earns its place: text labels, and notifications you can read

### Fixed — ⛔ THE PANEL WAS THE ONE SURFACE THEME SWITCHING COULD NOT REACH

M7-E shipped runtime theme switching and every other painter reads `rupa_theme_active()` — the desktop
void, the window chrome, the launcher. `shell_render.cyr` hardcoded `bhumi_xrgb(28, 30, 38)` and
friends, so cycling to a LIGHT theme repainted the whole desktop and left **a dark slab pinned across
the top of it**. Not a subtle mismatch: the panel is the one element always on screen.

The panel now draws every colour from the active theme.

⚠ **The net-status dot is deliberately NOT themed.** Green/amber/red is a *state encoding*, not
decoration; recolouring "disconnected" to suit a palette would make the one indicator whose whole job
is to be legible at a glance depend on which theme is active.

### Added — panel text labels: the oldest open M2 remainder

M2 has listed *"panel text labels (cpu/mem %)"* as unbuilt since the milestone was written, and the
reason it stayed unbuilt is visible in the diff: **the compositor had no integer formatter.**
`draw_text` existed the whole time. Bars alone answer "is it high" and never "how high" — at a glance
71% and 78% are the same bar.

⛔ **The label sits LEFT of its bar, and that is not a style choice.** The gauge group is laid out
right-to-left from the screen edge, so the first gauge placed is already flush against it; a label
drawn to the right of that bar lands past `fw`, where `bhumi_fb_set` discards it **silently**. Not a
visual glitch — a number that simply is not there.

### Added — ⭐⭐ a notification surface. The model has existed since the port and was NEVER DRAWN

`Notification` carries id, app_name, title, body, priority, timestamp, requires_action and is_agent.
The only field any renderer read was the **count**, drawn as a red square. Every notification the
system produced arrived, was stored with its text, and was displayed as *"there is at least one"* — an
operator could not learn what it said, which app sent it, or whether it was critical. A battery
warning and an agent finishing a task rendered identically.

`render_notifications` draws up to four cards below the panel, newest first, each with its app name,
title, body, a priority stripe, and a marker when `requires_action` is set.

⚠ **The overflow is stated, not swallowed** — a "+5" row, because silently drawing four of nine tells
the operator there are four.

### Added — `disk` and `agents`, which also had zero readers

`shell_status_disk` and `shell_status_agents` were modelled, maintained, and read by nothing outside
`shell.cyr`. A full disk was invisible; on a desktop whose shell treats agents as first-class, "how
many are running" was tracked and never shown.

⚠ This is the same shape as `WS_MAXIMIZED`, `DECO_RESIZE_*` and dhancha's client layer. **The fix is
never a new model.**

### Fixed — a font-init order that only bit once something reached it

`draw_text` needs `kashi_font_init`, called once from `main.cyr` at boot — so every production path
was fine and the requirement was invisible. Teaching the panel to draw labels made the glyph reader
reachable from `tests/shell_render.tcyr` for the first time, and cyrius links an undefined function
that nothing reaches: the moment something did, the suite died on **SIGILL (exit 132)** rather than
failing at build time. `draw_text` now ensures the font itself, guarded so the table is written once.

### Testing — `tests/shell_panel.tcyr` (33 checks)

Formatter clamping, bar arithmetic, the panel repainting under a theme change, labels reaching the
framebuffer, notification text and priority stripes on pixels, and the overflow count.

⛔ **Two of these checks exist because earlier versions passed for the wrong reason**, which is worth
recording:
- A label check scanned x = 638..672 on a 640-wide framebuffer. Out-of-bounds reads return black,
  which is never the panel background, so an **off-screen label passed a "there is ink here" test**.
- After that was fixed, a mutation restoring the off-screen label **still passed** — the neighbouring
  gauge's label bled into the sampled column.

⇒ Slot positions are now pure functions (`panel_slot_x` / `panel_label_x` / `panel_bar_x`) that the
painter is required to use, and the suite asserts on them directly: every label and bar ends inside
the framebuffer, and consecutive slots do not overlap. A pixel scan cannot make that claim.

Mutation-tested: hardcoded panel colour, label placed after the bar, undrawn notification title,
swallowed overflow, wrapping percentages, a single shared priority stripe, and overlapping slots are
each caught.

## [0.16.3] - 2026-08-17 — the application launcher, and the active window comes to the front

### Added — `src/launcher.cyr`: start apps FROM the desktop instead of pre-loading them

⛔⛔ **NOTHING COULD START A CLIENT AFTER BOOT, AND THE REASON WAS STRUCTURAL.** Every windowed app was
spawned by a hardcoded `while (ci < 2)` loop inside `main()` — `/bin/puka` and `/bin/crab`, always both,
always at startup — with the mint → endow → `spawn_path_env` sequence written INLINE and its state in
FUNCTION-LOCAL arrays. The only code that knew how to host a client was a loop that had already
finished. Operator: *"an application launcher of some sort so we don't have to pre load programs on
start."*

⭐ **`ae_client_spawn` is that loop's body, lifted verbatim** — not a reimplementation — and the client
table moved to module scope. One writer, two callers (the `--clients` pre-spawn and the launcher).
⚠ Writing the sequence twice is how copies drift: the launcher would spawn clients that never connect
while boot kept working, and each would read as a transport bug.
⛔ `endow` stays immediately before the spawn it belongs to — the arm is per-CPU and one-shot, so
minting both channels first puts the SECOND endowment into the FIRST child.
⚠ The table also moved from function-local `[32]` (32 BYTES = 4 slots) to module-scope `[8]` (8 real
slots): the unit rule flips, and this exact table already paid for that once when a second client's
store ran off the end of the first.

**The default changed; the harnesses did not.** A bare `aethersafha` REGISTERS puka and crab and
launches nothing (`launcher ready -- F2 lists the apps, nothing pre-loaded`). ⚠ `--clients` still
pre-spawns both, unchanged — `puka-terminal-test.py` re-ran green (`presented: 2`, `exit 95`). Changing
the default is the feature; changing what the gates measure would silently recalibrate the burn evidence.

**Keys**: `F2` opens · `Up`/`Down` select, wrapping · `Enter` launches AND closes · `Esc` closes.
⛔ **Closed, the launcher is inert** — including Escape, which is the compositor's QUIT; swallowing it
would make the desktop unquittable. ⛔ **Open, it swallows everything else** — a modal chooser that
leaks keys types into the window behind it while the operator believes they are picking an app.
⚠ HID usages added (`F2 0x3B`, `Enter 0x28`, `Down 0x51`, `Up 0x52`), cross-checked against crab's own
key handling rather than a table: a guessed usage silently binds the wrong key.

⚠ **Not `src/apps.cyr`'s Command Palette.** That runs SHELL UTILITIES via fork+execve to capture stdout.
This launches WINDOWED CLIENTS, which on agnos means minting a `#97` channel and announcing
`AGNOS_CHAN`. agnos has no fork, and a captured stdout is not a window.

### Fixed — the focused window was not the top layer

⛔ Seen on iron 2026-08-17: *"active window should be the highest layer"* — puka focused with crab drawn
over it. `comp_focus` stored an index and never touched the window vector, and **both** draw paths walk
that vector bottom-to-top, so z-order was pure creation order and clicking did nothing.
⭐ Raising is a vector move (remove + push), because paint order IS the vector — no z field, and adding
one would give two sources of truth. TAB still visits every window: with focus always on top, "next"
raises the bottom-most and rotates the stack.
⚠ Three assertions encoded the old "focus is just an index" semantics and went red; they now assert the
raise, plus a no-op check for re-focusing the top window.

### Verification — three layers, because each is blind to the next

| gate | proves | mutation |
|---|---|---|
| `tests/launcher.tcyr` (44) | registry, keys, geometry — no compositor needed | — |
| `tests/desktop.tcyr` pixel proofs | the panel and the raise reach the FRAMEBUFFER via `render_desktop` | unhooking `render_launcher` fails it while all 44 pure assertions stay green |
| `agnos/.../launcher-panel-test.py` | a REAL F2 over the real HID path on the real kernel opens it | — |

⛔ **The QEMU harness failed on its first run and that was worth having.** The panel rect was unchanged
and `first key-down seen` never appeared — no key reached the compositor at all. The non-vacuity gate
(the compositor must LOG `launcher opened`) refused to call it a pass. Cause was the harness: agnos
drains the HID ring once per FRAME, so a single injected key can land in a gap and vanish — the same
one-record-per-frame shape behind the old *"puka didn't register key commands"* report. Repeated
injection makes arrival certain. Result: accent seal spanning the full 260 px panel at (894,661).

### Changed — `[deps.setu]` 0.8.5 -> 0.8.6

`present_probe` honours `SETU_CLOSE`, closing the `#86` slot leak that cost one of 16 system-wide slots
per desktop launch (measured on iron: 16 → 15 → 14 → 13).

## [0.16.2] - 2026-08-16 — per-window opacity is real on iron, and the chrome regression it exposed

### Verified on iron — the whole opacity path, 2026-08-16 (agnos 1.56.45)

⭐⭐ **M6-C3 CLOSES ON HARDWARE.** A translucent `crab` you can see through, over a working `puka`
terminal, with **chrome on the GPU** — `#92` op 0x06 for the client surface, op 0x02 for the alpha
chrome. Both burns' defects are gone: titlebar backgrounds present, and the operator reports the
stacking correct.

⚠ **What that is and is not.** "See-through and correct" is an eye, not a framebuffer oracle. The
z-order PARTITION is host-gated and mutation-tested; the EMISSION now has a hardware run behind it that
shows no defect, which is stronger than 1.56.45 (which showed one) and weaker than a measured overlap.

⛔ **`/bin/puka` was setu's `present_probe`, not the terminal, for two burns** — a staging default, read
as a capability limit because two files still claimed puka could not build `--agnos`. It builds fine
and had run on iron weeks earlier. `PUKA_TERMINAL=1` stages the real one; the false claims are deleted
at both sites with what they cost recorded.

### Fixed — `fill_rect_a` gets a GPU route (`#92` op 0x02), which was the missing piece behind BOTH chrome burns

⛔⛔ **Two consecutive iron burns, one absent primitive.** `fill_rect_a` was the only drawing call with no
GPU path. Burn 1 (1.56.44): a translucent frame kept chrome "on the GPU" while every alpha fill went to
the userland framebuffer and the skipped `#39` blit discarded it — **no titlebar backgrounds**. Burn 2
(1.56.45): the repair took chrome *off* the GPU for those frames, which restored the titlebars and
immediately reintroduced the layer-vs-z-order defect — one band blit lays every window's chrome down
before any client surface, so a **lower** window's surface covers an **upper** window's titlebar.
Operator: *"zlevel of topbar and content don't seem to be the same"*. Both are the same missing route.

⭐ **op 0x02 `BLEND_COV` already is this operation.** `blend_cov.s` computes `f = cov/255 ·
sa' = colour_a·f · out = sc' + dst·(1 − sa'/255)`. A coverage mask whose every byte is the window's
alpha, plus an opaque colour, gives `colour·α/255 + dst·(1−α/255)` — `rend_blend` exactly. It has been
advertised since it was `#93`; nothing in the kernel changed.

⭐ **One slot serves every rect at a given alpha**, because a uniform mask makes the pitch irrelevant —
a titlebar, a body and an accent strip all read the same buffer, and the upload happens when the alpha
or the high-water size moves, not per rect and not per frame.

⚠ **`| 0xFF000000` on the colour is load-bearing, not defensive.** The shader forms `sa'` from
`colour_a`; a theme colour arrives as `0x00RRGGBB`, and A = 0 would make `sa' = 0`, i.e. `out = sc' +
dst` — an additive glow. `#88 gpu_fill_rect` ignores that byte, which is why nothing upstream sets it.

⚠ **CPU and GPU chrome may differ by ±1 per channel** — `blend_cov.s` documents two rounding sites
against `rend_blend`'s integer-exact one. Invisible, and not a licence to compare the paths pixel-wise.

**`ae_gpu_alpha_chrome` survives as a FALLBACK only**, for a kernel with no op 0x02: chrome to the CPU,
`#39` carries it, overlap mis-stacks — still strictly better than burn 1's invisible titlebars.

⛔ **And the routing rule is now gateable.** `ae_gpu_frame_plan` short-circuits to 0 off agnos, so the
branch that sets the fallback is unreachable on a host — **measured: inverting its condition passed all
54 assertions**. Extracted as pure `ae_gpu_chrome_cpu_needed(translucent, alpha_ok, cov_ok)`, the same
move `ae_gpu_window_admissible` made for the same reason; the mutation now fails two assertions.
Mutation-tested both ways: dropping the queued alpha to 255 fails the render gate.


⭐⭐ **M6-C3 CLOSES.** `#92` op 0x06 `BLEND_ALPHA` composited a translucent client surface on
archaemenid — the first hardware evidence that per-window opacity works at all, after 0.15.0's
verification turned out to have measured convergence rather than blending.

⛔ **The same burn found a regression this cut introduced**, and it is the honest headline: admitting a
translucent window onto the GPU frame lost every window's titlebar background, because `fill_rect_a`
has no `#88` route and nothing had added it to the list of layers that drag chrome back to the CPU.

⭐ **THE BINARY LOST ~10 MB, AND IT IS A DEP DOING ITS JOB.** The staged rootfs entry is 14,069,784 B;
this tree builds **4,075,560 B**, all of it `.bss` (11,545,776 → 1,551,568) with `.text` unchanged.
Cause: **sigil 3.12.9 (2026-08-14) unbanked the RSA sign path — "9.53 MiB of `.bss` goes with it"**,
9,993,748 B declared against 9,994,208 B measured here. It reaches this tree transitively through kavach
and landed hours after the tool was last staged. ⚠ **Restage before any flash** regardless — the staged
entry is 0.16.1 at the old footprint.

### Changed — cyrius pin 6.5.20 → **6.5.21**, matching agnos, and kavach 3.11.12 → **3.11.13**

The kernel and the desktop now build on one language version: agnos moved to 6.5.21 at its 1.56.44 cut.
`lib sync --full` (107 stdlib files), `deps` relocked, pin-drift warning gone, 24 suites green on both
targets.

⭐ **THE PIN ITSELF IS PROVENANCE, MEASURED — agnos's finding DOES transfer.** Building the agnos target
with cycc **6.5.20** and cycc **6.5.21** gives the same binary, `e6789468be0aa31b` both. The pin is a
declared floor, not a codegen input.

⛔⛔ **AND THE FIRST VERSION OF THIS ENTRY SAID THE OPPOSITE — CORRECTED.** It read *"a pin move is not
free here … the rebuild changed, `294e6559` → `e6789468`"*, attributing a real hash change to the one
variable I had deliberately moved. The hash change is real; the cause was not the compiler. Three things
changed in that window — the pin, `lib sync --full`, and a `deps` relock — and the compiler was the one
that made no difference. **The delta is dep CONTENT**: `lib/sandhi.cyr` 1.9.9 → 1.9.10 adds a real
`body_ptr == 0` guard. ⚠ A hash that moves when you change three things is evidence about the set, not
about your favourite member of it; the isolating build costs two minutes and was worth it.

⚠ **kavach was declared 3.11.12 against a sibling already at 3.11.13** — the `path`-wins hazard firing a
third time. ⛔ **Measured: the tag edit is inert for the binary.** Building at 3.11.12 and at 3.11.13
produces the same hash, because `path` had been supplying 3.11.13 to every build regardless of what the
manifest said. The declaration was wrong; the artifact never was. That is the whole reason a green build
proves nothing about the declared graph.

⚠ **`lib/sigil.cyr` stays at 3.12.9 against 6.5.21's bundled 3.12.7, and that is correct.** kavach
3.11.13 declares sigil 3.12.9, so the newer transitive dep wins over the toolchain snapshot. Syncing to
silence the warning would downgrade a declared requirement and the next `deps` would revert it.

### Verified on iron — per-window opacity composites on the GPU, agnos 1.56.44 burn

⭐⭐ **`#92` op 0x06 `BLEND_ALPHA` ran on archaemenid.** `#89 gpu_caps` byte +28 reported `130911`
(`0x1FF5F`, bit 6 set), `ae_gpu_alpha_ok()` answered 1, and both `--opacity 128` runs logged
*"compositing a TRANSLUCENT client surface with #92 op 0x06 (uniform alpha)"* and completed —
244 and 537 frames, `frame loop ok`, no `GPO_E_*`, no demote line. The crab window is visibly
translucent in the operator's photographs and opaque at `--opacity 255`.

⚠ **Scope, stated.** The burn shows the op DISPATCHES and produces a translucent window. It does not
discriminate the rounding tie (`blend_alpha.s:59` — 5,905 reachable ties, RTNE vs half-away differ on
3,010 outputs); nothing on screen distinguishes ±1 on a channel. That question stays open.

### Fixed — a translucent window lost its titlebar, body fill and accent strip on the GPU frame

⛔⛔ **The regression the same burn found, and it was mine.** Narrowing the frame plan's translucency
demote let an `--opacity 128` desktop stay on the GPU. But `fill_rect_a` (`render.cyr:64`) has **no GPU
route** — it is a per-pixel read-modify-write into the userland framebuffer and never calls
`ae_rect_enqueue`. The rest of the frame still behaved as if chrome were on the GPU, including the band
decision, which zeroes the `#39` blit the moment **any** rect is queued — and the three deco buttons
(plain `fill_rect`) always are. ⇒ Body fill, titlebar and focus strip were painted into a framebuffer
nothing copied to screen. On iron: every window lost its titlebar background at 128 and kept it at 255.

⚠ **`ae_chrome_on_gpu()`'s own banner already stated the rule this broke** — *"any layer that falls back
to per-pixel drawing must drag chrome back with it, because the blit is the only thing that carries
per-pixel output to the panel"*. Text and the cursor were both on that list. `fill_rect_a` never was,
because before the narrowing a translucent window demoted the whole frame and the case could not arise.

⭐ **The fix is the frame plan's third answer.** `ae_gpu_alpha_chrome` joins `ae_gpu_frame_ok` and
`ae_gpu_plan_n`: set when a translucent window is ADMITTED, and read by `ae_chrome_on_gpu()`. The mode it
selects is coherent rather than a fallback — chrome, glyphs and the backdrop go on the CPU and ride the
`#39` band blit, while the **client surfaces stay on the GPU** with op 0x06. ⚠ `#87`-staged wallpaper is
suppressed in that mode: it and CPU chrome are mutually exclusive (`wp_fill_band` paints nothing when the
GPU owns the backdrop, and the band blit would then copy an empty framebuffer over it — a black desktop).
An image or gradient wallpaper plus a translucent window demotes the whole frame exactly as before.

⚠ **A NON-premultiplied client surface is still composited opaque at any window opacity** — `#87` has no
alpha and the CPU path's `blit_rect` never had one either, so both paths agree. Unchanged by this repair;
naming it because the chrome around such a window now blends and its content does not.

### Fixed — three test suites whose verdict never reached the caller

⛔⛔ Found while writing the regression gate above, each measured rather than inferred:

- **`tests/render.tcyr` returned from the middle of `main`.** Four groups below the `return
  assert_summary();` — the wallpaper scaler, per-window opacity, `fill_rect_a` blending, and the
  default-opacity assertions — had **never executed**. `fill_rect_a` is the function at the centre of the
  regression above; the assertions pinning its blending were among the unreachable ones.
- **`tests/gpu_fallback.tcyr` had no `assert_summary()` at all** — the only suite in `tests/` without one.
  An injected failure printed its `FAIL:` line and the run still exited **0**. This is the oracle for
  *"the client's word reaches the screen whatever the GPU decided"*, and it has been unable to fail for
  as long as it has existed — alongside the dead `return 0;` that had already cost it 20 assertions.
- **`tests/selftest.tcyr` ended `assert_summary(); return 0;`** — printed `N failed`, exited 0.

⚠ Reviving the render groups surfaced two genuine defects in them: the `fill_rect_a` group inherited
`ae_gpu_frame_ok = 1` from the group above and so measured the enqueue path rather than the blend, and
`black at 50%` expected `0x102030` when `rend_blend` weights the destination by `255 - a` — 128 keeps
127/255, giving `0x0F1F2F`. The renderer was right; the literal was wrong and had never run.

**265 · 54 · 36** assertions in those three suites, all green, and a failure now returns non-zero.

### Fixed — the log named op 0x01 on a frame composited by op 0x06

⛔ The `ae_gpu_blend_seen` latch sat outside the per-window routing, so the 1.56.44 burn log printed
*"compositing a client surface with #92 op 0x01 (shader blend)"* on a frame whose only blended surface
went through op 0x06 — both lines, one op. These lines are this tree's record of which kernel path
composited; one that names the wrong op is worse than none.

### Removed — 29 build artifacts from version control, ~280 MB

`.gitignore` covered only `build/qemu-linux/`. Tracked under `build/` were **15 static binaries at
~15 MB each** (~220 MB) plus **62 MB** of QEMU screendumps and guest images — the latter added this
session, by my own harness runs with `AE_QEMU_OUT=build/qemu-a128`, which slipped past a rule that named
one path instead of the prefix. Now `build/` is ignored wholesale.

⚠ **Checked before removing, not after.** Every reference to these paths either BUILDS the artifact
(`cyrius build --agnos src/main.cyr build/aethersafha-agnos` in README.md, CLAUDE.md and the guides) or
is a `# Run:` header in `programs/*.cyr`. The single script that READS one —
`docs/development/agnos-render/aethersafha-smoke.sh:37` — already errors with the exact build command
when it is absent, which is better behaviour than finding a stale one. Files are untracked, not deleted;
both targets rebuild clean and 133 + 23 stay green.

⛔ **A committed build artifact is actively dangerous, not merely large.** In the sibling agnos repo
`tests/gpu/build/edgeasm` was committed, ran, and printed *"B4 PASS — the tool reproduces a shipped
iron-proven shader byte-for-byte"* while `edgeasm.cyr` beside it **could not compile at all**. A
committed binary is evidence about whatever source existed when someone last ran a compiler — not about
the source next to it.

### Fixed — the GPU frame composited in LAYER order, not Z order

⛔ `ae_gpu_present_frame` drained the **entire** chrome queue, then blitted **all** client surfaces,
then repainted **all** chrome text. With two overlapping windows the lower window's surface covered the
upper window's titlebar background, while `:1100` repainted the upper title glyphs on top — its own
comment saying *"so a title cannot be hidden under a window"*. Signature: floating title text over a
neighbour's surface with no titlebar behind it.

⚠ **Not an opacity bug** — it is why the GPU path could not be trusted with overlapping windows at all,
and therefore why M6-C3 per-window opacity could not be allowed onto it.

**The queues were already in z-order; only the boundaries were missing.** `render_desktop` paints
bottom-to-top, so a mark after each window recovers which rects and text runs belong to whom. The frame
now emits `[chrome_i][surface_i][text_i]` per window, with a prelude (the clear/wallpaper) and a
postlude (shell panel) either side. No new queue, no new pass.

⛔ **The band blit must stay BEFORE the client surfaces, and my first attempt moved it after.** When
chrome is on the CPU that blit *carries* the chrome, so running it after the surfaces paints titlebars
over the windows — the mirror image of the bug being fixed. The decision no longer waits for a running
`chrome_rc`: "is chrome on the GPU this frame" is knowable up front, and a rect refusal mid-frame now
demotes to a full CPU present rather than latching and blitting from a partly-stale framebuffer.

⚠ **The emission path is `#ifdef CYRIUS_TARGET_AGNOS` and CANNOT be verified here.** A QEMU Linux
screendump is **byte-identical** before and after (0 differing pixels) precisely because the code never
runs there. What is gated is the **partition**, which is pure and portable: `ae_mark_*` slices both
queues per window, with a content check that every rect in window 0's slice lies inside window 0.
Mutation-tested — marking before the render instead of after (2 red), dropping a window's mark (4 red),
dropping the prelude mark (2 red).

⛔⛔ **And that test was VACUOUS on its first two mutations.** `ae_chrome_on_gpu()` is a conjunction of
**four** flags; setting one left it 0, no rects were queued, every range was empty, and both mutations
stayed green. Instrumented rather than assumed — the counters read `prelude=0 m0=0 m1=0 total=0`. It now
carries an explicit non-vacuity guard asserting the queue is non-empty and each slice non-empty, and the
prelude is deliberately populated so "forget the prelude mark" is not an equivalent mutation.

⚠ Still **unverified on hardware**: this path runs only on agnos, and the run-to-run noise floor in the
Linux harness is one character cell (a clock digit, 128 px).

## [0.16.1] - 2026-08-13 — the wallpaper rides `#87`, where it fits

⭐ **The image wallpaper goes to the GPU by the same path a client surface already takes**: an
`#86 shm_create_gpu` slot, then `#87 gpu_blit_shm` into the back buffer. That is `AE-2`, iron-proven
for an opaque surface — no new kernel op, no shader, no driver stack, and **not mabda** (its GPU
surface rides Linux's driver stack; this band is the agnos kernel's own ring-3 `#82`-`#94`).

### ⛔⛔ It only fits at some geometries — and this caps UPLOADS, not the desktop

⚠ **Read this before drawing the wrong conclusion.** The 2 MB cap is on `#86` slots, i.e. buffers
USERLAND UPLOADS. It is **not** a cap on the compositing target: `gpu_blit_arm` sizes the back buffer
from boot_info as `pitch * height` rounded to 2 MB pages (14.7 MB at 2560x1440), so the clear, every
chrome rect (`#88`), the glyphs and the blend (`#92`) and the flip (`#84`) all run at full native
resolution. `AE-9` burned at 2560x1440 and that result stands.
⇒ What the cap bites is (a) a full-screen wallpaper, which exceeds a slot above ~800x600 by definition,
and (b) any single client surface over 2 MB — a window larger than ~724x724, which falls back to the
CPU per-window exactly as `AE-6`'s routing intends. Client windows fit today because a window is
smaller than the screen; **the wallpaper is simply the first full-screen thing anyone has uploaded.**

A `#86` slot is capped at **2 MB** (`SHM_MAX_SIZE = 2097152`, 16 slots system-wide), so a full-screen
XRGB wallpaper is `w*h*4` against that:

| geometry | bytes | |
|---|---|---|
| **800x600** — archaemenid's actual scanout | 1,920,000 | **fits** |
| 1280x800 | 4,096,000 | does not |
| 2560x1440 | 14,745,600 | does not |

⚠ Gated on the caps-reported `shm_slot_max` at **runtime**, never on an assumed 2 MB — a later kernel
raising the cap lights this up with no edit here. A wallpaper that does not fit demotes with its size
and the cap both named.
⛔ **Tiling across slots was considered and rejected**: 8 slots for a 1440p wallpaper is half the
system's supply held permanently, and re-uploading strips per frame moves more bytes through the CPU
than the CPU blit it would replace.

### ⚠ Ordering is the whole correctness of this

`AE-9` makes the CLEAR the first `#88` rect in the queue, so a wallpaper blitted after the chrome flush
would be painted over by it. The `#87` blit therefore runs **first**, and `wp_fill_band` suppresses its
clear enqueue whenever the GPU is about to blit — otherwise the CPU would also paint a full-screen
backdrop into a userland framebuffer that `AE-9` then discards, every frame.
⭐ Re-upload happens only when the wallpaper serial or the geometry changes; the scale is a full-screen
per-pixel loop and the `#86` write copies the whole buffer, so doing it per frame would cost more than
the path it replaces.

### ⛔ NOT VERIFIED, and structurally cannot be here

QEMU exposes no AMD PCI device, so `gpu_find` never matches, `ae_gpu_probe` answers 0, and **every
branch of this is dead in the harness**. It compiles `--agnos`, the Linux/CPU path is unchanged
(23/23 + the QEMU desktop gate still green), and the only part a machine without an AMD GPU can execute
is the arithmetic — factored into a pure `ae_gpu_wp_fits(fw, fh, slot_max)` with 9 assertions covering
both sides of the exact boundary. ⚠ `slot_max <= 0` is asserted NOT to read as "fits": that value comes
from a caps record that was never populated, and treating it as unlimited would turn a clean CPU
fallback into a demotion mid-frame. **A burn is the only thing that runs the rest.**

## [0.16.0] - 2026-08-13 — a wallpaper is a FILE

⛔⛔ **0.14.0 SHIPPED THE WRONG FEATURE, and it had a convincing rationale.** The wallpaper layer landed
as a *colour source* — solid and vertical-gradient — under a doctrine that "the colour may depend on
`y`, never on `x`", justified by keeping the `AE-0a` band a flat row run. That constraint is correct
for a gradient and **meaningless for a wallpaper, because an image varies in both axes.** It came from
reading the design brief's "generative / shader wallpapers" idea log as the spec instead of asking what
a wallpaper is. The doctrine is retired; solid and gradient survive as cheap special cases.

⭐⭐ **`--wallpaper <file>` loads a PNG or JPEG.** Verified in QEMU with a 1280x800 **16-bit** PNG, and
on the host with both PNG and JPEG.

### ⛔ And the decoder was already written — in a sibling repo

A PNG decoder was hand-rolled here first, covering a strict subset (8-bit only, RGB/RGBA only, no
Adam7). **`chitra` 0.3.0 already does all of it and more**: PNG at every bit depth including Adam7, and
baseline JPEG with YCbCr 4:4:4/4:2:2/4:2:0, into canonical RGBA8. ⚠ The test file that proved this is
**16-bit**, which the hand-rolled version would have refused. Deleted rather than kept as a "lighter
path"; `[deps.chitra]` added, with `sankoch` + `thread` joining `[deps].stdlib` for its inflate.

### Added

- `wp_load_file(path)` — read, decode via chitra, repack RGBA8 → XRGB8888 once (not per frame).
  ⚠ Alpha is **discarded deliberately**: the wallpaper is the bottom layer, so there is nothing beneath
  it for an alpha to blend against and honouring it would only darken the image against black. A
  translucent wallpaper is a different feature from a translucent window.
- `wp_pixel(x, y, fw, fh)` + `wp_src_index` — nearest-neighbour scale to the screen. 9 assertions, and
  the ones that matter are the clamps: an off-by-one samples past the buffer on the final row/column,
  which reads as a torn edge rather than an error.
- ⚠ A failed load **names itself** and falls back to the theme — silence would be indistinguishable
  from "no wallpaper was asked for".
- ⚠ Reads to EOF rather than trusting one `sys_read`: a short read on a large file is normal, and
  treating it as the whole file truncates the last chunk and surfaces as a decode error blaming the
  wrong thing.

### ⭐ The GPU route for an image wallpaper is already proven, and it is NOT mabda

A decoded wallpaper is just a full-screen surface, so it takes the **same path a client surface takes**:
an `#86 shm_create_gpu` slot then `#87 gpu_blit_shm` into the back buffer before the windows — which is
`AE-2`, iron-proven for an opaque surface. No new kernel op, no shader, no driver stack.
⛔ mabda is not involved and must not be re-proposed: its GPU surface rides **Linux's** driver stack
while this band is the agnos kernel's own ring-3 `#82`-`#94` (roadmap, corrected 2026-08-01). The only
thing mabda could have offered is decode acceleration, and decode is chitra's, on CPU.
⚠ Not wired yet — the upload wants a persistent slot (16 system-wide, each rounded to a 2 MB page) and
a re-upload only when the image or geometry changes. Next bite.

## [0.15.0] - 2026-08-13 — M6-C3: per-window opacity

⭐⭐ **Windows can be translucent.** `win_set_opacity(win, 0..255)` applies to chrome AND content — a
"translucent window" that kept an opaque titlebar would read as a rendering fault, not a setting.
`--opacity N` sets the default for windows created afterwards.

⭐ **Verified against a NULL, not by eye.** Three QEMU runs at 255 / 150 / 60 over the gradient
wallpaper, sampling identical coordinates:

| sample | 255 | 150 | 60 |
|---|---|---|---|
| inside the window | `1B1E24` | `1A1D24` | `171A24` |
| **outside** it | `151727` | `151727` | `151727` |

Window pixels move monotonically toward the wallpaper; the outside pixel is **byte-identical** across
all three, so the effect is scoped to windows and is not a global tint. ⚠ Two inside samples also
diverge at 60 (`171A24` vs `1B1A24`) — that is the gradient showing through at different rows, which is
the property that distinguishes real blending from a flat tint.

### ⛔ It DEMOTES the GPU frame, and that is a kernel limit, not a shortcut

`#88 gpu_fill_rect` writes a flat colour with no read-modify-write, and `#92` op 0x01 blends the
per-pixel alpha the **client** authored — its op record is `op / src_id / wh / dstxy` and the kernel
**rejects a non-zero reserved dword**, so there is nowhere to put a compositor-applied uniform α.
Window opacity is therefore not expressible on the band at all today. `AE-9` deleted the `#39` blit, so
a GPU frame has no CPU layer to mix into and the whole frame must fall back. **Latched and named**, for
the same reason the wallpaper demotion is: silence would look exactly like opacity that failed to
apply. ⇒ The kernel ask (an α dword or a new op) is roadmap M6-C3.

### ⚠ Two traps in a field that lives in zeroed memory

- **Default 255, not 0.** An opacity slot in zero-init memory means *invisible*, so a window created
  before anyone set one would silently vanish — the same class as the `W_PREMUL` slot once missed in
  `win_new`.
- **Clamped on the way in.** `rend_blend` computes `255 - a`; a stored 300 makes that negative and
  wraps the arithmetic into garbage rather than erroring.

⭐ `fill_rect_a(..., 255)` delegates to `fill_rect` unchanged — including its `#88` GPU enqueue — so an
opaque window costs exactly what it did before and the common path is untouched.

## [0.14.1] - 2026-08-13 — two agnos-track items: stop probing by trial, and make AE-M readable

### A3 — `#89` byte +28 is CONSUMED: the `#92` op-support mask

⛔⛔ This compositor read caps `+0..+24` and stopped, then discovered which `#92` ops existed **by
issuing one and reading the error** — the exact shape `#86` already proved wrong. The kernel publishes
a bitmask at `+28` (bit N = op code N, `GPU_OP_SUPPORTED = 0x1FF1F`, `agnos syscall.cyr:4294`)
precisely so a caller need not probe by trial.

⚠ **caps bit3 answers a DIFFERENT question** — "will `#92` run at all", i.e. is the dispatch envelope
proven on this boot. A kernel can have that and still not implement the op you want, so **both** gates
are now required: bit3 AND the op bit. Op 0x01 (blend, `AE-6`) and op 0x03 (glyph, `AE-8`) are gated
separately, and a kernel offering one but not the other was previously discoverable only mid-frame,
per window, forever.
⭐ A missing op 0x03 now falls back to the CPU glyph path silently-by-design (that fallback is what
`AE-8` kept so a glyph refusal never costs the frame), and the mask is **printed** at probe time —
"which ops did the kernel offer" was unanswerable after the fact.
⚠ Decoding is a pure `ae_gpu_op_supported(mask, op)`, split out for the same reason
`_bhumi_fbinfo_rc` is: a caps-decoding mistake should never need hardware to find. 10 assertions,
including blend-only and glyph-only masks.

### A2 — a window action now says whether a CLICK or a KEY produced it

⛔ `AE-M` is the only unburned rung, and its BUTTON path was **un-adjudicable from a capture**: min/max
route through `input_apply` identically whether they came from F5/F6 or a titlebar click, and neither
printed a line. The 2026-08-10 iron burn saw usages 62/63 with the operator's eye as the only oracle —
the "a burn that cannot distinguish its own failure modes" shape this arc keeps paying for.

⇒ `input_apply_from(..., src)` records action + source; `main` prints one line at the click site.
⚠ **Recorded, not printed from the leaf module**: printing there would make 44 unit assertions noisy
and put the log line where a burn cannot control it. Recording lets a test assert the taxonomy with no
output at all.
⛔ **The key path had to be tagged too**, and that is the load-bearing test: without it
`input_last_src()` would report a STALE BUTTON from an earlier action and a capture would claim a click
that never happened — worse than no instrument, because it is confidently wrong.

## [0.14.0] - 2026-08-12 — M6-C1a: the backdrop is a LAYER

⭐⭐ **`src/wallpaper.cyr`** — the desktop backdrop is a *source* instead of a single `bg` i64. Verified
in QEMU with a vertical gradient beneath both windows; sampled rows are monotonic in all three channels
and row 799 reads **2B1B3A**, exactly `c1`, which is the endpoint the unit test asserts.

⭐ **This is the load-bearing bite of the wallpaper work.** Everything the design brief wants —
gradients, shader wallpapers, "computed per frame, not asset-streamed" — needed somewhere to draw, and
until now there was no layer and no place a second colour could exist.

### The cost model IS the design, not an afterthought

⛔ `AE-0a` measured the clear at **3.83 ms of a 6.40 ms GPU frame (60%)** and cut it to a damage band.
A wallpaper **replaces** that clear rather than adding to it — but only if it keeps the band's shape,
and `rend_clear_band` is fast because a full-width band is ONE contiguous run (1.093 ns/px; going
through `fill_rect` cost 23% more in per-row overhead alone).
⇒ **Every source is ROW-UNIFORM: colour may depend on `y`, never on `x`.** A gradient then costs the
same stores as a solid and differs by one add per row. An x-varying source is a different performance
class and belongs with C1b. ⚠ SOLID takes the original code path unchanged, so the default cannot regress.

### ⛔ Three ways this could have silently undone AE-0a, all closed

- **Tracking must not bump the serial when the colour is unchanged.** The layer follows the theme until
  something sets a wallpaper explicitly (otherwise the desktop boots black — the backdrop used to be
  read from rupa every frame). Bumping unconditionally would invalidate the band **every frame**,
  repaint the whole screen forever, and undo the entire saving with no visible symptom. Compare, then set.
- **A wallpaper change is invisible to the `bg` compare.** Two gradients can share a top colour, so the
  source carries a serial; without it, switching wallpapers would repaint only the rows a window
  happened to be moving in.
- **The two invalidation gates must be sampled together.** Written as sequential
  `if (x != last) { last = x; return 0; }` gates, each CONSUMES a frame on first sight — the `bg` gate
  returns before the wallpaper gate stores its baseline, so priming took two frames and the first band
  arrived late. ⭐ `render.tcyr`'s existing "idle: a band IS computed" caught this immediately.

### ⛔ A non-solid wallpaper DEMOTES the GPU frame, loudly

`AE-9` routes the clear through `#88 gpu_fill_rect`, which fills ONE colour, and is all-or-nothing (the
`#39` blit is gone, so there is no CPU layer to mix in). Compositing windows on the GPU over a backdrop
the GPU cannot draw would show chrome on a stale void. The demotion is **latched and named** — silence
would be indistinguishable from a wallpaper that failed to paint, a shape this arc has paid for twice.
⚠ A gradient IS expressible as ~64 stacked `#88` bands; that is C1b and removes the demotion.

### Added — `--wallpaper`, and 22 assertions

A demo knob, not the configuration story — **C1c owes the authoring format**, and choosing one here by
accident is exactly what that item warns against. Tests cover the pure source: endpoints (an off-by-one
in the `h - 1` divisor makes the bottom row *nearly* `c1` — invisible by eye, and a band repainting the
last row would then disagree with a full repaint), clamping, serial behaviour and GPU expressibility.

## [0.13.8] - 2026-08-12 — the Linux keyboard is whole

⭐ **Arrows, Home/End/PgUp/PgDn, Insert/Delete, RCtrl/RAlt, Meta and Menu reach the compositor** —
bhumi **1.4.1**. All of them were silently dropped since the Linux keyboard landed. Verified in QEMU:
`up left home delete esc` logged usages **82 80 74 76 41**, exactly HID Up/Left/Home/Delete/Esc.

⛔ **The gap existed because the base plane worked TOO well.** Linux `KEY_*` base codes ARE Set-1 make
codes, so the base plane needed no translation at all — and that free win is what made the extended
plane easy to overlook. Set-1 encodes those keys as a 0xE0 **prefix** plus a byte and bhumi's table was
keyed on the byte, while evdev emits one flat number and never a prefix, so the table could not match
on this arm no matter what was pressed. ⇒ a second table keyed on evdev's own numbering.

⚠ M6-B4's "base plane only" caveat is now retired; the Linux keyboard is complete for ordinary use.

### Changed — `[deps.bhumi]` 1.4.0 → **1.4.1**

## [0.13.7] - 2026-08-12 — M6-B4 closed: the Linux desktop takes mouse AND keyboard

⭐⭐ **`pointer motion received -- the cursor is live`, `pointer button click routed`, `quit on a key
usage 41`** — all three in one QEMU run, through the *emulated USB devices*: QMP → evdev → bhumi
**1.4.0** → the compositor.
⭐ **Quantitative, not impressionistic**: the cursor started centred at (640,400), the injected deltas
summed to (+400,+300), and the screendump put it at **(1040,700)**. It moved by exactly the delta.

### Added — pointer + click to `scripts/qemu-sendkey.py`

`mouse:DX,DY` and `click` drive QEMU's `input-send-event`, so the path under test is emulated device →
Linux evdev → bhumi, never a shortcut into the consumer.

### ⛔ What the QEMU guest caught that the dev box could not — twice

bhumi's evdev device scan **latched**, and each latch silently lost an entire input device:
1. **Scan-once** — the first poll runs milliseconds after boot, before USB enumeration, so with a USB
   keyboard the scan found nothing and never looked again. Invisible on the i8042 keyboard (present
   from reset), so the bug surfaced only when the guest was given a *more realistic* device.
2. **Retry-while-empty** — the obvious fix still failed the ordinary case: the mouse enumerated first,
   the count rose above zero, the scan latched, and the keyboard was never opened. Measured precisely
   that: motion arriving, Esc never.

⚠ Neither is a compositor defect and both were fixed in bhumi — but they are the second and third
findings this harness has produced that the host box was structurally incapable of showing, after the
`mmap`-vs-`pwrite` scanout bug. A second machine keeps paying for itself.

### Changed — `[deps.bhumi]` 1.3.0 → **1.4.0**

## [0.13.6] - 2026-08-12 — M6-B6 closed: one rendezvous, named by setu

⭐ **The display socket is named in ONE place.** setu **0.8.5** makes `setu_un_path` resolve an explicit
path → `$SETU_SOCKET` → `SETU_UNIX_PATH`, and all four callers now pass **0**: this compositor,
`crab` 0.4.7, `puka` 0.6.12 and setu's own `present_probe`. The name is protocol-shaped
(`/tmp/setu-display.sock`) because setu owns the wire.
⭐ Verified both ways with no symlink: default → **exit 95**, `SETU_SOCKET=/tmp/ae-alt.sock` → **95**,
with the server and both clients following the override.

⛔ **What this replaces was a hazard, not a break** — four repos spelled the same literal and agreed.
But `$SETU_SOCKET` worked for exactly **one client in three** (puka implemented it itself; crab and
present_probe did not), which is the shape of a config that looks supported and mostly is not.

### Fixed — spawned clients were given an EMPTY environment

⛔⛔ `ae_spawn_client` passed `envp = 0`. With `SETU_SOCKET` set, the compositor bound the override and
every client it launched still dialled the default — `--clients` answered **94** while the server log
looked perfectly correct. ⇒ the child is now **told** where the compositor listens, via a hand-built
`SETU_SOCKET=<resolved>` blob, exactly as the agnos arm announces `AGNOS_CHAN=<fd>` in its `#43` env
blob. ⭐ That is strictly better than propagating the parent's environment: the child follows the
**server**, rather than recomputing a default and happening to match. One variable only, so a
compositor cannot leak its environment into a client.

### Fixed — the listener's own log line SIGSEGV'd the compositor

⛔ Passing `setu_path = 0` ("ask setu") left `sys_write(1, setu_path, strlen(setu_path))` dereferencing
NULL — **exit 139** on the line whose entire job is to make the socket knowable. It now prints
`setu_un_path(setu_path)`. ⚠ Caught in seconds because the E2E test runs the real binary; a log line
exercised only by eye would have shipped.

### Changed — `[deps.setu]` gains a `path` override, and 0.8.4 → **0.8.5**

setu was the one dep declaring a tag with no `path`, so a local setu change could not be built against.
⚠ The path-wins hazard applies as it does to every other dep: re-verify tag vs sibling VERSION at each cut.

## [0.13.5] - 2026-08-12 — a real keypress reaches the Linux desktop

⭐⭐ **Esc typed at the guest quits the compositor.** bhumi **1.3.0** adds the Linux evdev keyboard, and
the proof goes through the *emulated input device* rather than poking the compositor: QMP
`send-key esc` → i8042 → `atkbd` → `/dev/input/event*` → bhumi → **HID usage 41** →
`IA_QUIT`, ending an unbounded `--frames 0` session at frame 2729.
⭐ **Negative-controlled**: the identical run with no key sent ran 20 s and never quit. A quit path that
cannot be shown NOT to fire proves nothing.

⚠ This closes the last thing that made `--frames 0` awkward on Linux — there is now a real quit key,
not only SIGINT. ⛔ **Base plane only**: evdev emits no 0xE0 prefix, so arrows, RCtrl/RAlt/Meta,
Home/End/PgUp/PgDn and Insert/Delete are dropped as unmapped (bhumi 1.3.0's own note). Pointer is
still stubbed on Linux.

### Added — `scripts/qemu-sendkey.py`

A stimulus/capture step kept separate from `qemu-screendump.py` on purpose: input verification needs a
strictly ordered wait → press → react → read, and folding it into the dumper would make "did the
picture change" and "did the key arrive" share one failure mode.
⚠ Holds each key 300 ms. evdev is edge-driven so it lacks the poll-sampling defect agnos measured
(0-of-9 keys at QEMU's ~100 ms default, because a USB HID keyboard reports STATE ON POLL), but a
generous hold costs nothing and keeps the harness honest against both input paths.

⭐ **The operator action was not a blocker after all** — M6-B4 was recorded as needing the dev user in
the `input` group. True for a host run; irrelevant in the guest, whose init is PID 1.

### Changed — `[deps.bhumi]` 1.2.1 → **1.3.0**

## [0.13.4] - 2026-08-12 — the Linux desktop hosts a real client, launched by itself

⭐⭐⭐ **A real setu client window on the Linux desktop, in QEMU** — `setu-surface` (present_probe's
green grid and red animation band) composited beside the `echo (foreign)` mehman window, correctly
z-ordered, with the compositor having **launched the client itself**. Non-black rose 0.225 → **0.289**;
the harness now gates on `setu client presented surface` as well.

### Added — `--client PATH` (repeatable), the Linux half of M6-B5

The counterpart of the agnos mint/endow/spawn block: fork+execve after the listener is up (the reverse
order is a race the child loses against a socket that does not exist yet). ⭐ `--clients` is now a
**one-command** probe on Linux — measured, `--clients --client present_probe --client present_probe`
returns **95** with nothing started by hand. That is also what lets a one-shot QEMU guest show a window
at all: there is no second shell in there to start a client from.
⛔ `src/apps.cyr` was deliberately NOT pulled into the build graph for this — it is linked on neither
target, so reaching `app_launch_terminal` would drag a 976-line module and its stdlib fallout in for
six lines of process code.
⚠ The child inherits the listening fd (setu sets no CLOEXEC). Harmless for short-lived children of a
longer-lived compositor, but it is a reason to want CLOEXEC in setu.

### Fixed — the guest had no `/dev/shm`, and the symptom pointed at the wrong layer

⛔ setu's Linux shared buffer is a tmpfs file `/dev/shm/setu-buf-<id>`. With no such mount the client
**connects, completes the handshake, and only then dies** on `buf_create failed` — so the compositor
logged `setu client connected` and never a presented surface, which reads as a protocol or compositor
fault rather than a missing mount. ⚠ It must be mounted *after* devtmpfs, or the directory lands in the
initramfs's `/dev` and is hidden by the mount over it. The harness now names this exact cause when it
sees `buf_create failed`, instead of reporting an undifferentiated "never presented".

### Changed — the listener names its socket, and `$SETU_SOCKET` is honoured

⚠ `sys_write`, not `println`: `setu_path` is an i64 holding a cstring and `println` on an untyped i64
prints the **pointer** (measured — it emitted `17851195`).

### ⛔ CORRECTED — M6-B6 was described wrongly, twice, and the correction is the point

The roadmap claimed *"three defaults across four repos"*, with setu's clients on a different socket.
**False.** All four spell the same literal `/tmp/aethersafha-setu.sock` independently — this repo,
`crab/src/main.cyr:178`, `puka/.../window_setu.cyr:58`, and setu's own `present_probe.cyr:100`.
Verified with **no symlink**: `--clients` + two probes returns 95.
⛔ The evidence behind the wrong diagnosis was an early `-111` (ECONNREFUSED) that was really the
compositor having **already exited** — the host run was 142 ms at the time. A symlink appeared to fix
it only coincidentally, because the probe was dialling the real path all along, and the bad diagnosis
then survived two further retellings. ⇒ **Check the socket still exists before blaming a rendezvous.**
⚠ B6 remains open as what it actually is: one literal duplicated in four files (a latent hazard, not a
break), fixable only in setu plus all four callers together — which needs a setu tag.
⛔ Attempting it one-sidedly here **broke every client with ENOENT**, and that was measured, not feared.

## [0.13.3] - 2026-08-12 — the Linux desktop, proven in QEMU, by an oracle it cannot influence

⭐⭐⭐ **The sovereign desktop runs on Linux inside a QEMU guest and was photographed doing it** — MUDRA
backdrop, shell status panel, a hosted `echo (foreign)` window with titlebar, traffic lights and cyan
focus strip, and the centred cursor. `scripts/qemu-linux-desktop.sh`.

⛔ **And the point of testing in a guest was not convenience.** It found a defect that the dev box
structurally could not: bhumi 1.2.0's fbdev arm used `mmap`, which works on **amdgpu** and displays
**nothing** on `simpledrm`. Fixed in **bhumi 1.2.1** (`pwrite`), repinned here. One machine was the
blind spot — 1.2.0's oracle was already an external fd and still missed it, because the second reader
went through the same shadow buffer.

### Added — `scripts/qemu-linux-desktop.sh` + `programs/qemu_init.cyr`

The guest userland is **two static binaries and nothing else**: a ~180-line Cyrius PID 1 that mounts
/proc, /sys and devtmpfs, forks the compositor, reaps it and powers off — no busybox, no distro image.
⭐ The oracle is QEMU's `screendump`, which reads the emulated VGA device's own memory: nothing
aethersafha or bhumi can influence, so it cannot report a picture that is not there.

### ⛔ Four ways this harness lied before it told the truth, all worth keeping

- **The gate passed on the FIRMWARE SPLASH.** It required ">= 8 distinct colours"; OVMF's TianoCore
  logo has **65**, and the run returned a confident "✅ DRAWN DESKTOP" for a photograph of it. A
  colour count is blind to layout by construction — the same blindness `puka-terminal-test`'s pixel
  count had to a staircase. ⇒ the gate is now the **non-black fraction**, calibrated on both arms:
  splash **0.011**, desktop **0.225**, threshold 0.10. Negative-controlled — `--selftest` draws
  nothing, measures 0.0015, and the harness **exits 1**.
- **Dumping on a timer photographed nothing, then photographed the wrong thing.** A fixed 9 s sleep
  missed entirely (the guest had already powered off at 1.3 s); syncing on `desktop up` fired *before*
  the frame loop. ⇒ `--hold` makes PID 1 pause after the compositor exits and print a marker, so the
  dump happens on an observable event and captures the last frame.
- **`args_init()` before mounting /proc silently ate every argument.** It reads `/proc/self/cmdline`;
  with no /proc, `argc()` is 0 forever, and the log read `exec /aethersafha with argc 1` while the
  compositor ran its default 400-frame cap instead of the requested `--frames`.
- **`CONFIG_DEVTMPFS_MOUNT=y` does not apply to an initramfs.** `/sys/class/graphics/fb0/*` read back
  perfectly while `/dev/fb0` did not exist — the framebuffer was simultaneously "obviously present"
  and "absent", because sysfs and the device node come from different mounts.

⚠ **And a platform constraint, not a bug:** a shadow-buffer fbdev is flushed only while it is actively
driven. `console=ttyS0` alone, or `quiet`, means fbcon never does its first draw and **even `pwrite`
never reaches scanout**. Both measured with one variable changed. The guest cmdline needs
`console=tty0` and no `quiet`; kernel text is harmless because the compositor repaints every pixel.

### Changed — `[deps.bhumi]` 1.2.0 → **1.2.1**

## [0.13.2] - 2026-08-12 — the compositor draws on a Linux screen

⭐⭐⭐ **`aethersafha: screen size read from the kernel / 2560 / 1440`** — the same binary read
`1280 / 720` out of its built-in fallback this morning. **bhumi 1.2.0** landed the Linux fbdev scanout
arm (bhumi ADR 0003), so every layer of this compositor now reaches a real screen on a second substrate.

⛔ **No compositor change was needed to get pixels, and that is the point.** `bhumi_backend_present` was
already called every frame on the host arm; nothing above the device seam was ever broken. The whole
Linux desktop was blocked on two functions in another repo that returned -1. What follows is the
follow-up work that becoming a real display target *created*.

### Changed — `[deps.bhumi]` 1.1.5 → **1.2.0**

Verified a real tag on a clean tree before bumping, per the rule the kavach entry twelve lines below
already documents. 23/23 suites, both targets build.

### Fixed — three claims the Linux arm falsified the moment it landed

- ⛔ **`ae_query_geometry`'s banner said "`bhumi_output_query` is agnos-only and answers -1, so the Linux
  dev build is unaffected."** False as of bhumi 1.2.0 — it now reads real geometry from sysfs. ⚠ And the
  correction matters beyond the wording: the Linux arm is now subject to the *same* hazard the rest of
  that comment documents for agnos, where laying out to the wrong width shows as a desktop occupying one
  corner of the screen rather than as an error.
- ⛔ **`bhumi_output_format_ok` had ZERO call sites in this repo**, while bhumi's own module banner says
  callers must verify byte order before presenting. Free while agnos was the only target (archaemenid
  reports BGRX, which *is* `BhumiFb`'s store order, so the raw blit was correct as much by luck as by
  design). An fbdev console reporting **RGBX** would render every frame with red and blue swapped,
  silently, on a path where nothing returns an error. ⇒ checked once at geometry time and **named**.
  ⚠ Report, don't refuse: a colour-swapped desktop is still usable, and the conversion belongs in bhumi
  (its ADR 0001 deferred an RGBX swizzle for exactly this case).
- ⛔⛔ **The present return code was discarded.** Defensible while the host arm was a compile-time -1 —
  every host run "failed" and that was the expected answer. Now it is the **only** thing separating a
  compositor painting a screen from one running a full frame loop into a void: both reach `frame loop
  ok`, both exit 0, both look identical in the log. That is precisely the 0.12.0 leaked-listener shape —
  healthy-looking output from a structurally incapable run.

### Added — a latched scanout-refused report, negative-controlled in both directions

⚠ Latched, not per-frame: a refusing backend refuses *every* frame, and 95,439 identical lines is how a
real signal gets scrolled past. `BHUMI_SEAT_DENIED` (-2) is distinguished from a device failure (-1)
because they point at different repos — the seat gate versus the framebuffer under it.

⭐ **Proven reachable, which is the part this arc keeps paying for.** `strings` finding a literal proves
it was compiled, never that it can run — the F1-F4 mover flew to hardware as dead code on exactly that
reasoning. So the instrument was driven both ways: with `/dev/fb0` hidden in a private mount namespace it
prints `scanout refused — no framebuffer; this run is drawing to nothing / -1` **and** falls back to
1280x720; with the framebuffer present it is silent and reads 2560x1440.

## [0.13.1] - 2026-08-12 — the host arm gets a clock, and stops quitting on the first frame

⭐⭐ **Linux is now a declared display target** (operator, 2026-08-12), superseding bhumi's ADR 0001 and the
three-substrate matrix in `planning/desktop.md:53-57` that assigned Linux the logic-only role. Backend
decision: **fbdev first, DRM/KMS later** — bhumi's seam is already fbdev-shaped (`#38 fbinfo` ↔
`FBIOGET_VSCREENINFO`, `#39 blit` ↔ `mmap` + row copy). Sequencing lives in `roadmap.md` **M6**, a
milestone two agnos documents had been handing work to **by name** since 1.56.42 while it did not exist here.

### Changed — toolchain pin 6.5.13 → **6.5.20**, and the kavach tag stops lying

⛔ **The manifest declared a dependency graph that nothing here had built since 3.11.7.** `[deps.kavach]`
said `tag = "3.11.7"` while HEAD vendored **3.11.8** and any local build silently re-materialised **3.11.10**
from `../kavach` — because these deps declare a `tag` *and* a `path`, and **the path wins**. That is the
hazard `state.md` has warned about in the abstract since 0.13.0; here it was live, and wrong in *committed*
state for two cuts. Bumped to **3.11.10** after verifying it is a real tag on a clean kavach tree, so the
declared graph is now both fetchable and identical to the vendored copy.

⚠ **Everything else was already honest.** All eight deps were checked against their sibling `VERSION` *and*
against an existing git tag: bhumi 1.1.5 · rupa 0.1.2 · agnostik 1.3.4 · agnodrm 1.5.0 · kashi 1.0.4 ·
mehman 1.0.1 · setu 0.8.4 all matched. Only kavach drifted.

The toolchain pin was seven patches behind the installed `cycc`, which the build reported on every
invocation as *"cyrius.cyml pins 6.5.13 but cycc is 6.5.20 — toolchain drift"* alongside a shadow-lib
warning (`sigil 3.12.7` vendored against `3.12.5` pinned). ⇒ pin **6.5.20**, `cyrius lib sync --full`
(107-file snapshot, 17 changed), `cyrius deps` relocked. **Both warnings are gone.** 23/23 suites green,
both targets build, and the Linux `--clients` run still returns 95.
⚠ `--agnos` moves 13,584,728 → **13,584,832 B** (+104), which is the toolchain bump and nothing else — but
it does mean the staged `agnos/build/rootfs/bin/aethersafha` is now toolchain-stale as well as
hash-divergent. Restaging is M6-A1 and is owed before the next burn.
⛔ **Not touched, deliberately**: sibling repos keep their own pins. `bhumi` pins 6.5.13, and regenerating
its `dist/` with the local 6.5.20 would be the wrong toolchain for that repo.

### Fixed — `ae_now_ms()` answered 0 on the host, and that hid the two defects below

⛔ The host arm was `return 0`, justified in place as *"inert off agnos rather than wrong — the host loop is
already bounded by setu_cap"*. That was true of the elapsed budget and false of every other reader: `el`
was always `0 - 0`, so the 30 s budget never bound, the 5 s progress line never printed, and the
end-of-run "probe ran for milliseconds" readout always printed **0**. A host run that was slow, starved or
wedged was byte-identical in the log to a fast one. ⇒ `clock_now_ms()` (`lib/chrono.cyr:46`), which has
always been available with `chrono` always in `[deps].stdlib`. ⚠ **Inert is not free when the thing made
inert is the instrument** — and with the clock restored, the whole bounded host run measures **142 ms**.

### Fixed — the Linux compositor quit on the first presented frame, and exit 95 could never fire

⛔⛔ `running = 0` sat under `#ifndef CYRIUS_TARGET_AGNOS` in the present handler, commented *"host: one
present proves the wire; exit the bounded loop"*. It was wrong in **both** modes:
- **desktop mode** — the session ended on the first frame a client presented, so the host build was a wire
  proof that could not become a desktop, by construction;
- **`--clients` mode** — it fired at `accepted == 1`, so the loop left before the `accepted >= 2` test could
  ever see 2. **Exit 95 was structurally unreachable on Linux** and the ladder could only answer 94.

⚠ Gating it to either value of `ae_probe` preserves one of the two bugs, so it is **deleted, not gated**.
Same class as 0.12.9's hardcoded 92: a documented verdict the code cannot emit.

### Fixed — `--clients` had a 30-second budget and spent 130 milliseconds of it

⛔ The probe's three terminators (`accepted >= 2`, a 200000-frame backstop, a 30 s wall-clock budget) were
all defeated by the host's `setu_cap = 400` dev bound. Measured: **130 ms**, over-stating the budget by
~230x, so every host `--clients` run gave a spawned client a ninth of a second to load, be scheduled and
connect. A client that "connected" did so by already being up and racing the listener. ⇒ the cap no longer
applies when `ae_probe == 1`; the probe now runs its full budget (measured 30000 ms / 95439 frames).
⚠ The comment claiming 400 frames was *"long enough to catch a client"* could not be contradicted by
anything in the build while `ae_now_ms` answered 0. Two defects, one blind instrument.

### Added — `--frames N`, and `--frames 0` means run until quit

The host loop stays bounded by default (400/3, unchanged — nothing that runs this binary expects it to
block). `--frames 0` gives a real session. ⚠ Linux has no quit key yet — bhumi's `_bhumi_kbscan` answers 0
off agnos, so Escape cannot arrive and SIGINT ends the run until M6-B4 lands.

⭐ **Verified on Linux 2026-08-12**: `--clients` with two setu clients now prints `setu client connected` /
`presented surface` twice, reaps both (`clients told to close: 2`) and **returns 95**. 23/23 suites;
`--agnos` builds unchanged.

### Changed — five falsified claims struck from `roadmap.md`

All five were work-generating, and each is contradicted by a code path or an iron capture: *pointer input
still absent on iron* (iron-proven 2026-08-09, drag **and** the two-`#92`-mask cursor); *premultiplied never
run against a real client* (crab sets it unconditionally; `AE-6` burned); *nothing in the GPU band has ever
executed on iron* (`stage-tools.sh:302` stages the desktop; `AE-2/6/8/9` burned); *`#88` and `#92` op 0x03
unconsumed* (both shipped; the band is 7 of 13 and 2 of 14, not 6 and 1); *the damage-limited blit is not
safe to enable* (shipped 0.12.3 as `AE-0a`). ⚠ `#91` and batched `#92` are recorded as **falsified**, not
pending, so they stop being re-derived as tasks.

## [0.13.0] - 2026-08-10 — the desktop survives being relaunched

⭐⭐⭐ **The headline is a class of bug, not a feature**: a compositor that exits must release what it
spawned. Until this cut it released nothing on agnos, and four launch/quit cycles were enough to exhaust the
system's 16 process slots and leave the desktop hosting nothing. Found by an iron burn that produced **no
evidence at all**, root-caused in QEMU, and closed on hardware the next day with four launches in one boot.

### Changed — `cyrius.cyml` interpolates its version at last, and the bump script stops undoing it

⚠ `version` was a hardcoded literal sitting directly under a comment that said *"⚠ INTERPOLATED, NOT
DUPLICATED"* and explained, at length, why duplicating it was the agnoshi drift bug. `release.yml` resolved
`${file:VERSION}` too, and its comment asserted the manifest used it. **Every sibling in the stack — puka,
crab, setu, bhumi, agnoshi, kriya — already did.** aethersafha was the lone holdout, and the reason it never
converted is that `scripts/version-bump.sh` `sed`-ed a fresh literal over the line at every cut. ⇒ The script
now NORMALISES to `${file:VERSION}` instead of assigning, and its self-check resolves the value exactly the
way release.yml does — so a green bump means the tag will be accepted, not that the script agrees with
itself. **Two comments describing a fix that a third file silently reverted** is the shape to watch for.

### ⚠ The tagged binary is NOT byte-identical to the one that burned — and this is why

The iron proof below was taken on **13,568,328 B**; this cut builds **13,584,728 B**. The delta is entirely
`lib/agnodrm.cyr` + `lib/agnostik.cyr`, which `cyrius build` **re-materialised** during the release build
(`cyrius.lock` moved with them). ⭐ Neither dep's *source* has changed since 2026-08-07 and both stayed at
their declared tags (agnodrm 1.5.0, agnostik 1.3.4) — so the burned binary was linked against **stale
vendored copies** and this one matches the declared graph. That is the right direction, not a regression.
⚠ **What is verified**: 23/23 suites, `--agnos` builds clean, and the growth landed in the *unreachable*
count (5144 → 5168 fns), which is what vendoring more dead surface looks like on an arm where agnodrm's
DRM/udev paths are not reachable at all. ⚠ **What is NOT verified**: that nothing newly vendored became
*reachable* — **no burn was taken on this binary.** Read the iron results below as proof of the FIX, not of
this artifact. [[reference_stale_vendored_lib_masks_stdlib_fix]]

### Fixed — the compositor never released its clients on agnos, and four relaunches killed the desktop

⛔⛔ There was **no session teardown on agnos at all**. This file's exit path releases exactly one thing —
`sys_close(setu_sfd)` — and it sits inside `#ifndef CYRIUS_TARGET_AGNOS`, so on the ONE target that has no
listener, and the only one a person quits and relaunches by hand, nothing was released. Everything the
compositor itself holds is reclaimed when its process dies; everything its **spawned children** hold is
not, because they do not die. A `--clients` probe or an Esc quit therefore left puka, puka's own agnsh and
crab alive forever, each holding a `#97` channel end, an `#86` shm slot and a row in a process table with
exactly **16** of them (`agnos/kernel/core/proc.cyr:275`).

⭐ **The serial names the asymmetry in one grep:** every F4 produced
`puka/crab: compositor closed the window -- exiting`; every `frame loop ok` produced **none**. F4 reaps a
client, Esc reaps nothing — three rows leaked per relaunch, four cycles to the cap, after which
`proc_alloc_slot` refuses every spawn and the desktop comes up hosting nothing.

⚠ **This is what the 2026-08-09 iron burn hit** (*"just FB lines"*), and the comment above the listener
close had already written the lesson down — *"predates the transport and outlives it"*. It did outlive it;
the fix had been applied only to the arm that stopped mattering.

⇒ `comp_close_all_clients()` in `compositor.cyr`, called at exit. The same `SETU_CLOSE` (kind 7) that
`comp_close_window` sends for F4 and that burned PASS on iron 2026-08-08 — as that function's own note
says, the client's exit is what releases the endpoint and the slot, so the send is not a courtesy, it is
the mechanism. crab and puka both already honour it. ⚠ It deliberately does **not** retire rects or unlink:
a closing compositor never repaints, and walking the vector while removing from it is how that would break.

⭐ **Before/after with one variable**, `AE_CLIENTS_MODE=relaunch` in agnos's clients harness: the sequence
broke at **relaunch #4** before, and ran **8/8 clean** after, with the exit teardown firing 10 times and
reaping both clients every time. 23/23 suites.

⭐⭐⭐ **BURNED PASS ON IRON 2026-08-10** (agnos 1.56.42, archaemenid, `cpus online: 4`). **Four launches in
one boot — `--clients` foreground, `--clients &`, plain, plain — and all four hosted two clients**, where
QEMU reproduced the old failure at exactly #4. Operator: *"works correctly, --clients, multi launch, drag
drop window, close window, min/max window"*.
⭐⭐ **The instrument read BOTH ways on hardware, which is what makes it believable**: `at exit — clients
told to close:` printed **2 · 2 · 0 · 2**, and the **0** is the run where F4 had already closed both windows.
It counts LIVE clients rather than printing a constant, and every non-zero count is followed by the clients'
own `compositor closed the window -- exiting`. ⭐ `--clients` also returned **95** rather than the hardcoded
92 that every previous iron run was told.
⚠ **The min/max BUTTONS are not distinguishable from the F5/F6 KEYS in the serial** — usages 62/63 were
seen, `input_apply` routes click and key identically, and neither action prints a line. The operator's eye is
the oracle for that pair and for F6-leaves-no-ghost; the log does not corroborate the button path.

## [0.12.9] - 2026-08-09 — the Known-open list, cleared

### Changed — `[deps.bhumi]` 1.1.4 → **1.1.5**

Picks up bhumi's two function-local `var X[N]` byte-sizing fixes. ⚠ Neither touches the consumable lib —
they are a program and a test — so this is not a functional change for the compositor; it is the declared
graph tracking the released dep. The vendored `lib/bhumi.cyr` re-materialised to `Version: 1.1.5` and
`cyrius.lock`'s content hash moved with it.

Every item the 0.12.8 audit left open is fixed. All were found by reading, none by a burn.

### Fixed — `--clients` returned a hardcoded verdict on agnos (the second burn blocker)

⛔⛔ `setu_sfd` is assigned in exactly one place, inside `#ifndef CYRIUS_TARGET_AGNOS`, so on agnos it kept
its `-1` initialiser for the whole run — and the verdict ladder's first test returned **92 = "the display
socket never opened"** no matter what happened. The entire 95/94/93/91 ladder was dead on the only target
that uses the probe, and every iron run that read this exit code was told the wrong thing. On agnos there
IS no listener by design (the channel band replaced it), so a missing socket is not a failure and is no
longer tested there.
⛔ **And exit 91 could never fire either.** The retired `if (0 == 1)` TCP-launch block held the ONLY two
writers of `ae_spawn_failed`, while the live `sys_spawn_path_env` path set nothing. The live path sets it
now — mint, endow and spawn failures all — and the dead block is deleted rather than left owning a live
flag's assignments.

### Fixed — a minimized window was still drawn, and clicks fell through it

⛔ `comp_window_at` has always skipped minimized windows for hit-testing; nothing skipped them for
RENDERING. F6 left the window fully on screen — chrome, buttons, live client surface — while every click
passed straight through into whatever was behind. Render and hit-test now agree, minimizing **retires the
vacated rect** (the same `#84`-flip requirement a close has, or the window stays as a ghost), and the GPU
frame plan no longer judges the admissibility of a surface it is not compositing.

### Fixed — clients were told the pointer was above their own surface

⛔ `comp_window_at` hit-tests from `win_y` so the titlebar is inside the window (it must be, or a drag is
impossible), but client coordinates are CONTENT-relative — so any pointer on the titlebar sent
`ry` in [-30, -1] down the wire, and setu carries full i64 so it arrived intact and wrong. A position
outside the content rect is now dropped: chrome interaction belongs to the compositor.
⚠ **Motion is also deduped** — every frame the pointer sat over a client cost a `#97` ring slot even when
the surface-relative position had not changed, and that ring drops the OLDEST record when it fills, so a
stationary pointer could push a client's real events out. A per-surface opt-in flag is the proper fix and
needs a protocol bit.

### Fixed — a refused cursor frame was recorded as DRAWN

⛔ `render_cursor` marks the position drawn BEFORE the emit can refuse, so a refused frame left a position
recorded as painted that never was — the move-gate then saw no movement and damaged nothing, leaving a
STATIONARY pointer invisible until the user happened to move it. `rend_cursor_unmark()` is called on every
refusal path.

### Fixed — the titlebar's maximize and minimize buttons did nothing

⛔ `deco_hit` has returned nine regions since it was written and the handler consumed two. Both buttons
were painted in their own theme colours on every window — they looked live and were inert, which reads as
a broken desktop. Wired through `input_apply`, so click and F5/F6 take the identical path.

### Fixed — two byte-sizing traps in bhumi, and an SMP race in the kernel

- `bhumi/programs/backend-demo.cyr` declared `var evs[32]` — **32 BYTES** — and authorised
  `bhumi_backend_poll` to write 32 events = **256 bytes** into it, over `frame`, `fb`, `be`, `now` and the
  return address. Invisible on the host (the non-agnos arm returns 0 events); CI cross-compiles it for
  agnos, where the first poll drains the whole agnsh-prompt backlog.
- `bhumi/tests/bhumi.tcyr` had `var prec[2]` for a 16-byte record and `var ptev[4]` for a 32-byte event
  budget, smashing ~40 bytes of its own frame — and the asserts passed anyway, which is the worst version:
  a green test standing on a corrupted stack.
- **`hid_mouse_take` now takes `hid_poll_lock`.** Its banner claimed "CALLER MUST HOLD IF=0" was
  sufficient; `cli` is per-CPU, and the syscall runs wherever agnsh migrated to while the MSI-X lands on
  another CPU, so a whole report's delta could be lost between the read and the reset. A try-lock, so a
  busy drain costs nothing — the accumulator persists to the next poll.

### Removed — `input_btn_edge`, the superseded edge API

⛔ It computed one edge from a single level, which is the wrong shape for this kernel, and survived only
because its unit test fed it an argument pair the real caller could not produce. Two edge APIs where one is
subtly wrong is an invitation to reach for the wrong one. Its mask coverage was ported to
`input_btn_transitions` rather than dropped.

### Fixed — the harness's drag gate was flaky

⚠ Two runs of the same binary disagreed: one aimed and dragged, one found no titlebar, because the
screendump can catch the panel mid-composite. A gate that passes intermittently teaches you to re-run until
green. The aim now re-measures up to four times, and the gesture retries once if the press missed.

## [0.12.8] - 2026-08-09 — the pointer chain audited; an empty desktop; windows that say what they are

An adversarial read-only audit of the whole `AE-7` chain across agnos/bhumi/aethersafha produced 27
findings; 12 survived independent refutation. The burn was held for these.

### Fixed — a titlebar drag could be entered but never exited (burn blocker)

⛔⛔ `hid_mouse_take` (agnos `kernel/arch/x86_64/usb/hid.cyr:232`) ends with
`hid_mouse_btn_seen = hid_mouse_btn` — the OR-accumulator is re-seeded to the current LEVEL, so a held
button reports `seen = 1` on every drain. An edge test on `cur | seen` therefore reads 1 while the button
is down AND on the poll where it comes up: **the release transition never appears.** On hardware the first
titlebar click glues that window to the pointer for the rest of the session and no client ever receives a
button-up. ⇒ `input_btn_transitions` — PRESS on `cur | seen` (so a click folded into one frame is not
invisible), RELEASE on `cur` alone, and a folded click emits **both** so no client is left holding a
phantom button. 12 asserts; the mutant restoring the old logic fails 3.

### Fixed — the cursor's CPU fallback did not actually put the cursor on screen

⛔⛔ Two separate reasons, both of which made the previous cut's retirement latch inert:
- `ae_chrome_on_gpu()` consulted only `ae_text_gpu_ok`, so retiring the CURSOR left chrome on the GPU, the
  `#39` blit skipped, and `render_cursor_cpu` painting into a buffer nothing copies to screen.
- Restoring the blit is still not enough: the blit is the **BOTTOM** layer and client surfaces composite on
  top of it, so a CPU cursor sits under every hosted window. ⇒ `ae_gpu_frame_plan` now refuses the GPU
  frame outright when the cursor path is retired, so clients composite per-pixel and `render_desktop`
  draws the cursor LAST. Same trade the text path takes, one layer further.

### Fixed — five more in the cursor emit and the pointer handler

- **The staging slot is ensured, not assumed.** `ae_gpu_glyph_ensure_slot()` is factored out of
  `ae_gpu_emit_text`, which created the slot BELOW its no-text early exit — so a frame with nothing to
  caption left the slot at 0 and the cursor retired permanently on a desktop whose only fault was having
  no window titles.
- **Early returns no longer discard an already-QUEUED cursor** without retiring, which lost the pointer for
  the rest of the boot when the `#86` slot could not be had.
- **A failed `sys_shm_write` is no longer silent** — it retires and logs.
- **No `alloc(32)` per frame** in the emit; agnos's allocator is a bump over 2 MB chunks and never frees.
- **The drag holds a window ID, never a raw `Window` pointer.** `comp_close_window` unlinks the object, so a
  stored pointer kept moving a window that was no longer in the scene.
- **A close-button click no longer forwards a phantom press OR release** to whatever was underneath —
  `ae_ptr_forward` re-derives `comp_window_at` at the same cursor position, so both forwards needed the guard.
- **The pointer starts at screen centre.** `input_ptr_set` had zero production call sites, so the cursor
  booted at (0,0) — directly under the shell status panel, where "never appeared" and "parked under the
  panel" look identical on a burn photo.

### Known open (verified, not yet fixed)

`--clients` always exits **92** on agnos (`setu_sfd` is only assigned under `#ifndef CYRIUS_TARGET_AGNOS`,
so the whole 95/94/93/91 verdict ladder is dead on the only target that uses it) · minimized windows are
still rendered but excluded from hit-testing, so clicks fall through a visible window · every pointer event
over a titlebar is forwarded with a NEGATIVE surface-relative y · no shipped client decodes PTR_MOVE/PTR_BTN,
so each forward spends a `#97` ring slot for no visible effect · a refused cursor frame is still recorded as
DRAWN, so the missing pixels are never re-damaged · `bhumi/programs/backend-demo.cyr` hands a 32-byte buffer
where 256 are needed · `bhumi/tests/bhumi.tcyr` smashes its own frame with `var prec[2]` for a 16-byte record.

### Changed — the desktop starts EMPTY, and windows carry the application's name

⭐ **The "Files" demo window is gone** (operator call). It was a compositor-seeded placeholder from before
there were real clients, and keeping it past that point was not neutral: it had **no client fd**, so it
silently swallowed every key focused onto it — that produced the "puka didn't register key commands"
report, cost a burn to diagnose, and then needed a latched `a key reached NO client` line whose only job was
to describe a window that should not have existed. It also sat at index 0 in the TAB cycle, so one tab from
the last-presented client wrapped onto it, and two burn cards had to carry a "keep TABbing" warning.
⇒ Every window on screen now belongs to a real process, and `desktop up — windows:` reads **0** until one
connects.

⭐ **Titlebars say `puka` and `crab`, not `setu-surface`.** setu carries no title in `CREATE_SURFACE`, but
the compositor **spawns** these clients and therefore already knows what each channel will carry — the name
rides down from the spawn site through `setu_srv_handshake_drain`, so no protocol change was needed.
⚠ Clients that arrive by the accept paths still get `setu-surface`, which is honest: nothing there knows
better yet.

### Added — the drag phase aims by MEASUREMENT, not by arithmetic

⛔ The harness's drag aim was computed from the compositor's cascade formula (`pcx = 30 + pstep * (w/6)`).
It worked once and then silently missed — the window is not where that formula says — and a missed press
reports "says nothing about the release", i.e. the phase quietly stops testing without failing.
⇒ `find_titlebar()` locates the focused window's cyan accent underline in the captured framebuffer and aims
at the titlebar above it. Reading the compositor's own output cannot drift out of step with its layout.

## [0.12.7] - 2026-08-09 — the pointer is an arrow: a derived-outline cursor that clips

### Added — a REAL cursor: `src/cursor.cyr`, an arrow instead of a `+`

⭐⭐ **The pointer was a text glyph.** `AE-7`'s first cut drew the character `+` at the pointer position —
deliberately, to reuse the iron-proven `#92` op 0x03 GLYPH_1BPP path rather than debug a second overlay kind
alongside the pointer itself. Operator's verdict on seeing it on the panel was exactly right: that is a
*pixel*, not a cursor. It has no direction, no visible hotspot, and it vanishes against any surface near the
theme accent.

⭐ **And it needs no new kernel path either**, because op 0x03 paints an *arbitrary* 1bpp mask in a colour —
it is not font-specific. The arrow is **two masks through the same op**: outline in black, then fill in
white. Two-tone is not decoration; it is the only reason a cursor stays legible over a white terminal *and*
a dark desktop, which is precisely where the accent-coloured `+` failed.

- **The outline is DERIVED, never hand-drawn.** The fill shape is the only art; the outline is computed as
  its 8-neighbourhood boundary, so it is closed by construction. A hand-authored outline is one typo from a
  hole that leaks the desktop through the arrow's edge — invisible until it is on a panel.
  ⚠ 8-adjacency, not 4: with 4 the diagonal leading edge leaks at every staircase corner (**measured: 13
  leaks**, and the suite catches it).
- **It CLIPS instead of refusing, so the hotspot reaches the last pixel.** `#92` op 0x03 refuses an
  out-of-range rect, and the 1.56.42 fix for that was to shrink the pointer's clamp by the glyph extent —
  correct then, but it meant the pointer stopped short of the right and bottom edges. For a 20 px arrow that
  is the whole bottom-right corner of the screen going unreachable. `cursor_visible_rect` computes the
  on-surface sub-rectangle and only that is sent; the clamp is now the full geometry.
- **Emitted LAST, after the text drain**, which is what makes the arrow top-most over chrome, captions and
  client surfaces alike. ⛔ That ordering is also required because both share the one `#86` glyph staging
  slot — emitting the cursor first would have its mask overwritten by the first caption packed after it.
- **A CPU arm as well**, taken whenever the GPU is not compositing the frame. It is gated on
  `ae_text_gpu_ok` — because `ae_chrome_on_gpu()` consults that latch to decide whether the `#39` blit runs,
  and that blit is the only thing that makes CPU-drawn pixels reach the screen — **and** on its own
  `ae_cursor_gpu_ok`.
- ⛔⛔ **`ae_cursor_gpu_ok`: a refused mask must RETIRE the path, not lose the pointer.** `render_cursor`
  queues instead of drawing, so an emit that refuses leaves the pointer absent for that frame — and since the
  next frame queues and refuses identically, absent for the rest of the boot, with one log line. That is what
  a refused glyph did to every caption on the desktop, one layer over. The first refusal now retires the GPU
  cursor and every later frame goes to `render_cursor_cpu`, which draws per-pixel and cannot be refused: one
  frame of missing pointer instead of a boot without one. ⚠ Kept SEPARATE from `ae_text_gpu_ok` — retiring
  captions because the *pointer* was refused would be the same mistake pointing the other way.
- **The emit consumes its queue.** `ae_cursor_q_clear()` runs in `ae_gpu_emit_cursor`, so the one-entry queue
  is genuinely per-frame. Left unconsumed the flag latches at 1 for the run, which is harmless only while
  every present happens to be preceded by a render that re-queues — a present without one would composite a
  STALE cursor position.
- **The borrowed staging slot is checked, not assumed.** The cursor packs into the text path's `#86` slot,
  sized for the widest glyph run; `cursor_max_bytes()` (40 B) is compared against `ae_text_bytes(AE_TEXT_CHARS)`
  (1152 B) at emit time and retires to the CPU if it ever stops fitting, instead of overrunning the slot.
- Damage now uses the cursor's **own** extent. It was `FONT_W + 2` x `FONT_H + 2` (10x18), which is *almost*
  the arrow's 10x20 — a stale rect would have smeared only the bottom two rows, only while moving. Exactly
  the defect that survives a screenshot and reappears as "the pointer leaves a trail" three burns later.

### Added — the shape is unit-tested, and the harness now checks the PANEL

⭐ `tests/cursor.tcyr` (**61 asserts**): the silhouette's slope, the seal invariant (no fill pixel may touch
a transparent one), layer disjointness, clipping at every edge including a 1x1 at the last pixel, and
MSB-first bit layout. It also **prints the arrow as ASCII** — the shape is art, and art is reviewed by
looking at it. `tests/render.tcyr` 194 → **217**: the damage extent (a font-sized rect fails at 218 vs 220),
and the retirement fallback — on a GPU frame with `ae_cursor_gpu_ok` cleared, all **131** arrow pixels must
reach the framebuffer; removing the latch check fails 4 asserts including *0 pixels drawn*.
⚠ That fallback assertion pre-paints a background first: the outline colour is `bhumi_xrgb(0,0,0)` = **0**,
byte-identical to an untouched framebuffer, so a `!= 0` count sees only the 73 fill pixels and a
`== xrgb(0,0,0)` check on the tip passes whether anything drew or not.
22 → **23 suites**.

⭐⭐ **And an external oracle at last.** Every previous cursor check was a serial line, i.e. the compositor's
*opinion* — `pointer motion received -- the cursor is live` printed on the 1.56.42 burn while the cursor was
invisible. `aethersafha-clients-test.py` now matches the **full 10x20 shape** in the captured framebuffer,
position-independently. Verified against a real capture: the rendered pixels are **bit-identical** to the
shape module's own dump, and the matcher returns None both when the arrow is erased and when a *single*
interior pixel is broken — a shape check, not a pixel count.

⚠ **Scope: the GPU arm is unburned.** QEMU has no amdgpu, so the harness exercises `render_cursor_cpu`; the
two-`#92`-mask path is iron-only and is not claimed until it burns.

## [0.12.6] - 2026-08-08 — the terminal gets its keys: draining a queue that drops the oldest

⭐⭐⭐ **BURNED PASS on archaemenid 2026-08-08** (AGNOS 1.56.42, userland-only flash). Two clients present
(`setu client presented surface` twice vs once), `forwarded a key to the focused client` →
`puka: key received` x5 (usages 11/8/15/19/40 = `help` + Enter) → `puka: line sent to the shell`, and the
terminal's window shows a live `agnoshi 1.8.9` that answered `help`. The cursor is visible and
`#92 op 0x03 refused a glyph run` never printed. 270 frames, clean Esc.
⚠ **The RESYNC is not iron-proven** — `resynced past a stale record` never fired, meaning the ring did not
overflow that run, so only the **drain** was exercised. The 47-assert suite covers resync from every drop
offset; hardware has not seen it. ⛔ Do not record it as iron-proven.

### Fixed — a placed client that presents every frame can finish its handshake

⛔⛔ **The compositor sipped one record per frame from a queue that drops the OLDEST.** agnos's channel is
a 64-slot ring and `chan_queue` (`kernel/core/syscall.cyr:4586`) advances the *read* cursor on overflow.
A client that presents every frame emits ATTACH+COMMIT per present, and its frame is an order of magnitude
faster than a full-screen composite — so it wrapped the ring between two compositor reads and the read
cursor landed at an **arbitrary** point in the endless `…A,C,A,C…` stream. `setu_srv_handshake_step`
demanded the exact next kind, so it then failed **forever** on a parity it did not choose.

⚠ **Measured on iron** (agnos 1.56.42, archaemenid): puka got window **id 2** from handshake state 0 but
never reached COMMIT, and `win_set_cfd` only runs on COMMIT — so `setu_srv_forward_key` dropped every key
at its `cfd == 0` guard and the terminal "didn't register key commands". crab survived **only because it
presents once**; its three records sat in the ring in order. That difference is the whole reason this
looked like a puka bug.

- **`setu_srv_handshake_drain`** — drain the channel every frame instead of taking one record, bounded by
  one ring's depth (64) so a client that writes faster than the loop reads cannot hold the frame loop
  hostage. `main.cyr` calls the drain; the step keeps its contract (`> 0` id, `0` in progress, `< 0` error).
- **`setu_hs_action`** — the state/kind decision is now a **pure function** that **resyncs** rather than
  erroring on the two shapes record *loss* produces: a COMMIT with no ATTACH is a stale orphan (skip it);
  a repeat ATTACH means our COMMIT was eaten (re-apply). Kinds loss cannot explain are still `-8`/`-11`/`-13`.
- **Idempotent re-attach** — the attach arm reuses the surface buffer when the geometry is unchanged. Now
  that it is reachable more than once, allocating per attach would have leaked a **full surface** per
  dropped commit.
- ⚠ The `-11` / `-13` diagnostics were reworded: they no longer mean "the second/third frame", because the
  machine resyncs past loss. They fire only on a kind that is wrong in a way loss cannot explain.

### Fixed — the placed-client retry counter was cumulative while its comment claimed "consecutive"

⛔ Nothing zeroed `chan_fail` after its initial store, so 200 transient handshake errors spread across a
long run would retire a perfectly live client — and the comment directly above it said "count consecutive
failures", so a reader would never suspect it. A frame that consumed records without erroring is the
definition of "the client is alive and behaving", and that is now the reset (`setu_hs_drained > 0`, exposed
by the drain because a productive drain and an idle one both return 0).

### Fixed — the "a key reached NO client" latch was spent by a key-RELEASE

⛔ A press-only surface drops every key-UP **by design**, so the one-shot latch fired on the first release
of the boot — measured at serial line 148, during crab's startup — and then said nothing about the key
actually under test, while the QEMU gate simultaneously reported "neither marker fired" for two letters
that demonstrably reached the compositor. It is now gated on `bhumi_key_pressed(ev) == 1`, reports on
**this** key (`fk == 0`) rather than "no key has ever been forwarded", and names the **usage** alongside
the focus index. ⚠ Same shape as the F8-hold that exhausted the first-N trace budget: a budget an
uninteresting event can exhaust is not an instrument. [[feedback_instrument_discipline]]

⭐ With that corrected, QEMU shows `aethersafha: forwarded a key to the focused client` followed by
`puka: key received` **twice** — the S→C leg was never broken, it had never been exercised, because the
harness injected only keys the compositor consumes as actions.

### Fixed — two more instrument holes, both found by an independent audit rather than by testing

⛔ **`a key reached NO client` was boot-scoped**, so the first key pressed on the compositor's own
clientless window consumed it and a later real failure printed nothing. Now **one report per distinct focus
index** — a repeat costs nothing and every window that swallows a key gets a line.
⛔ **Handshake codes `-9` (window table full) and `-12` (attach geometry/buffer id refused) had no print at
all**, so a client could be retired in complete silence — indistinguishable from "the retry budget has not
expired yet". Both are named, and a `pnamed` catch-all now prints for any code without its own line, so a
silent retirement is impossible by construction.

### Added — the handshake finally has a test (`tests/setu_handshake.tcyr`, 47 asserts)

⭐ It had **none**, which is why this shipped to iron wrong. The oracle is external, not a mirror of the
code: **from every offset a dropping ring can leave, the machine must complete within 3 records, and must
never report a protocol error for something loss explains.** Restoring the old strictness fails **11** and
**9** asserts respectively, so the suite is not vacuous. 21 → **22 suites**, all green.
⚠ No battery assert total is quoted, deliberately: `cyrius test` prints a SUITE tally and the *last
suite's* own count, so the trailing number is `apps.tcyr` alone (133), not the repo. Only per-suite figures
mean anything here.

## [0.12.5] - 2026-08-08 — `AE-7`: THE POINTER WORKS — cursor, click-to-focus, titlebar drag

⭐ **The whole `AE-7` arc in one cut** (P0 + P4 here; the kernel's P1-P3 are agnos 1.56.42 and bhumi
1.1.4). A USB mouse now moves a cursor, focuses a window by clicking it, and drags one by its
titlebar — verified end to end in QEMU, and the first ring-3 use of `ptrscan #98`.
⛔ **Not yet on iron.**

### Added — the compositor consumes pointer events (`AE-7` P4)

⭐ **Mostly wiring, which was the point.** `comp_window_at` (z-ordered, skips minimized), the 9-region
`deco_hit`, and `input_move`'s GPU-safe clamp were all written long ago and **never called** — they were
waiting for a pointer. The setu protocol already carried `SETU_INPUT_PTR_MOVE`/`_BTN`. No new mechanism.

- **Cursor** — position + clamp in `input.cyr` (testable; the high edge is exclusive, so `bound_w - 1`),
  drawn by `render_cursor` **last**, after every window and the shell panel.
- **Click to focus** — `comp_window_at` gets its first consumer, ever.
- **Titlebar drag** — reuses `input_move`, so a drag inherits the clamp that keeps a window from crossing
  an edge and dropping the frame off the GPU. A drag *is* a move by a delta.
- **Close button** — `deco_hit` → `DECO_CLOSE` → `comp_close_window`, which already retires its damage.
- Pointer events are forwarded to the window under the cursor **surface-relative**, over the setu
  messages that already existed.

⛔ **THE CURSOR CANNOT BE DRAWN AS CHROME.** On a GPU frame the chrome rect queue is emitted BEFORE the
client surfaces and `#39` is skipped, so a `fill_rect` cursor lands *under* every app. It goes through
`draw_text`'s queue, which is drained AFTER the client blits — that ordering exists for exactly this.

⚠ **The cursor damages only when it MOVES.** The first cut damaged both rects unconditionally and made an
idle desktop write 18 rows every frame — which destroys the whole value of the `AE-0a` band. Four asserts
caught it. ⚠ `rend_cursor_damage` and `render_cursor` are a **pair**: damage without draw leaves the
previous position stale and the cursor re-damages forever.

### Changed — the event batch is MIXED, so the loop classifies before reading

⛔ Pointer events share the array with keys, and every key accessor is `ev & 0xFF`. `bhumi_key_usage` is
the **low byte of dx** and `bhumi_key_pressed` is **bit 8 of dx** — so `dx = 297` (0x129) is a *pressed
Escape* and a horizontal flick would **quit the desktop**. The frame loop now routes on `bhumi_ev_kind`
once, and `input_map` carries a second guard because it is the part a unit test can reach.

⚠ **The first version of that test was vacuous** and a mutant proved it: it used `dy = 41`, but dy lives
in bits 24-47 and cannot reach bit 8, so the *pressed* check caught it and the kind guard was never
exercised. Deriving the discriminating value (`dx = 297`) is what turned it into a real test — removing
the guard now fails three asserts, one reading `got 1` = `IA_QUIT`.

**Tests** input 91 asserts, render 179. `cyrius` pin 6.5.9 → **6.5.13** (`sys_ptrscan`, transitively via
bhumi). ⭐ **QEMU end to end**: motion and clicks both reach the compositor — and this is the first
ring-3 caller of `ptrscan #98`, so it is what proves that syscall at all.

### Fixed — `AE-7` **P0**: one window-geometry convention, so the first click lands where it looks

⛔⛔ **Five consumers had drifted into THREE conventions for a window's vertical extent, and one function
held two of them.** `win_h` is the CONTENT height with the titlebar ABOVE it, so a window occupies
`win_y .. win_y + TITLEBAR_H + win_h`. That is what the damage model, the GPU client composite and
`render_window`'s client blit all do — the three **iron-proven** paths. But:

- `render_window`'s **theme body fill** used `h - TITLEBAR_H`, contradicting the client blit **50 lines
  below it in the same function**: the theme body stopped 30 px short of the surface drawn over it.
  Invisible while a client covered it, and 30 px short on the compositor's own clientless window.
- `deco_hit` and `comp_window_at` ended the window at `y + win_h`, i.e. 30 px too high.

⚠ **Inert only because nothing clicks yet.** The first pointer click would have landed 30 px out on every
live setu window, the bottom-most 30 px of every client surface would have registered as *outside* the
window (finding whatever was behind it), and the bottom resize strip would have fallen **inside** the
client's content rather than on its edge — a drag there starting a resize in the middle of a window.

⭐ **The convention now has exactly one definition**, in `src/window.cyr`: `win_total_h`, `win_body_y`,
`win_bottom_y`, `win_prev_total_h`, with the whole history written beside them. Every consumer was routed
through them — `deco_hit`, `comp_window_at`, the damage model's `cur` *and* `prev` terms, the GPU composite
(both call sites), the client blit, the focus accent strip and foreign.cyr. **Do not open-code
`+ TITLEBAR_H` at a call site again** — that is precisely how the drift happened.

⚠ **`TITLEBAR_H` moved from render.cyr's `RenderK` to window.cyr's `WinGeom`**, because that was the
mechanical cause: it sat in the renderer's *button-geometry* enum, and `compositor.cyr` is included
EARLIER, so `comp_window_at` could not reach it and open-coded the wrong extent instead. A
window-geometry fact belongs with the window model. Button geometry stays in the renderer.

**Tests** `tests/render.tcyr` 160 → **179 asserts**, with the convention pinned explicitly
(`win_body_y == 130`, `win_total_h == 330`, `win_bottom_y == 430` for a window at y=100 h=300) and every
boundary asserted in both directions. ⚠ Two pre-existing asserts were **changed, not added**: `(300,397)`
had been asserted as `DECO_RESIZE_B` for as long as that test existed, and under the real convention it is
33 px inside the client surface — it is now the regression guard that fails if chrome-inside returns.
⚠ Mutation-verified in both halves: reverting `win_bottom_y` fails 7 asserts, and reverting the body fill
fails 1 — **that second test exists because the first mutant of it PASSED**, so the body's height was
untested until four pixel asserts were added for it.

⭐ No pointer code. This is the latent debt `AE-7` would otherwise have inherited, paid down on its own
with its own tests. Design: agnos [`planning/pointer.md`](https://github.com/MacCracken/agnos/blob/main/docs/development/planning/pointer.md).

## [0.12.4] - 2026-08-08 — the desktop's last CPU work moves to the GPU, and window management starts working

⭐⭐⭐ **Iron-validated across four flashes of AGNOS 1.56.41** (a byte-identical kernel every time, so this
userland was the only variable). `AE-6` / `AE-8` / `AE-9` put every layer of the frame on the GPU and the
`#39` blit is no longer issued; `#89` is read in full and its BATCHED contract is confirmed on silicon;
**`AE-0a`'s damage band finally met a window that moves** — the arc's oldest untested claim — and then the
close path it exposed was fixed. Sections below, newest work first.

### Fixed — `#92`'s BATCHED failure mode demanded a recovery this compositor never performed

⛔⛔ **`#89` caps bit4 (BATCHED) is SET whenever the shader is available, so it is set on archaemenid — and
this compositor never read it.** The bit is semantic, not a hint: one `#92` call is one GPU submission, so a
failure cannot name the op that died. The kernel answers `GPO_E_BATCH` and **an indeterminate prefix has
already landed in the back buffer**, with an explicit contract for ring 3 — **re-clear with `#85` and do NOT
call `#84`**. The kernel's own caps comment says a caller that ignores the bit cannot know which recovery it
owes. This one owed the batch recovery and was doing the per-op one: on a glyph failure it returned and let
the frame **present a half-composited buffer**.

Both `#92` call sites now distinguish the two. `ae_gpu_err_is_batch()` reads the reason out of the
`-((idx << 8) | reason)` encoding — ⚠ `idx` is meaningless for a batch error and must not be read as one —
and a batch failure triggers `#85` whole-buffer re-clear plus a refusal to flip. A per-op `GPO_E_DISPATCH`
keeps the old, sufficient answer: demote the frame to the CPU path.

⚠ **The full-screen clear is correct in exactly this one place.** The landed prefix could have touched
anywhere, so `AE-0a`'s damage band is not a sound bound for the repair.

### Added — the whole `#89` record is read, not just `flags`

Bytes +4..+31 were being ignored: `+4` bb_pitch, `+8` bb_w, `+12` bb_h, `+16` shm_max, `+20` shm_free,
`+24` shm_slot_max. The slot budget is now **printed at probe time**, so a later "no graphics-visible slot for
glyphs" has its cause in the same log instead of being a mystery — there are only 16 `#86` slots system-wide
and `shm_create` rounds every request up to a 2 MB page.

⛔ **`bb_w` IS NOT A LAYOUT WIDTH** and is deliberately not used as one. It is `pitch / 4`, and on archaemenid
the pitch in pixels (832) exceeds the visible width (800) — laying out to it would push chrome into the
invisible gutter, which is the mistake this repo already made once with `#38 fbinfo`. It is the right bound
for "will `#87`/`#88` ACCEPT this rect", because those check against the back buffer and reject rather than
clip. Two questions, two numbers.

### Added — F7-F10 move the focused window, which finally exercises `AE-0a`'s one untested case

⭐⭐ **The damage model's whole reason for `union(cur, prev)` is a window that MOVES leaving its old pixels
behind — and this arc recorded three times that nothing could move one without `AE-7` pointer input.** So the
band shipped, and burned five times, on reasoning that had never been exercised. F7-F10 nudge the focused
window 40 px. That is not a workaround for the pointer; it is the **missing stimulus** for a model that was
already in production.

⭐⭐⭐ **BURNED PASS 2026-08-08 — operator: "f7-f10 move the window, no ghost".** That is `AE-0a`'s
`union(cur, prev)` band doing, on the **GPU path**, the one job it was written for, and it had never been
asked to do it: the band shipped in 0.12.3 and rode **five flashes** on reasoning nobody could exercise,
because a moving window needs a pointer (`AE-7`, a kernel xhci item) or a keyboard mover and neither existed.
⭐ Proven in QEMU first, on `qemu-xhci` + `usb-kbd` — the same USB HID producer archaemenid uses: eleven
injected keys produced eleven usage traces in exactly the right order (`43, 64,64,64, 65,65, 66,66, 67,67,67`)
and the window moved **40 px left and 40 px down**, the exact net of the sequence, leaving its old position
clean. ⚠ QEMU's leg was the **CPU blit path** (no amdgpu there); iron supplied the GPU half.

⛔ **F-keys because a modifier combo is not detectable here.** `bhumi_key_usage(ev)` is `ev & 0xFF` and
`bhumi_key_pressed(ev)` is one bit — **no modifier state reaches the compositor at all**, the same gap that
stops the terminal seeing Shift. ⛔ And not arrows: crab navigates with Left/Right/Up/Down and h/j/k/l, so
consuming those would break the file manager.

⛔ **F7-F10 AND NOT F1-F4, because F4 was ALREADY `IA_CLOSE_FOCUSED`.** The first cut of this mover bound
F1-F4 and shipped to hardware that way, which made "move the focused window down" and "destroy the focused
window" the same keystroke — with the close applied first, printing nothing. F7-F10 is the contiguous
unclaimed block below the three window-management keys `HidUsage` already owned. `_bhumi_set1_to_hid`
(lib/bhumi.cyr:699) decodes Set-1 `0x3B-0x44` → HID `0x3A-0x43` as one range, so the choice costs nothing at
the seam. ⚠ `tests/input.tcyr` now asserts F4 is *still* close and F5 *still* maximize, so a future edit
cannot re-open the collision quietly.

⛔ **The move CLAMPS into the screen**, and that is not cosmetic: `#88`/`#87` reject an out-of-range rect
rather than clipping, and `ae_gpu_window_admissible` refuses a window past an edge — so a mover that could
push a window off-screen would turn one keypress into a silent loss of hardware compositing. ⚠ The clamp
**re-clamps low** afterwards: a window larger than the bound makes the low and high clamps disagree, the high
one wins, and the coordinate goes NEGATIVE — precisely the rect the GPU refuses. A unit test covers it.

### Fixed — ⛔ the first mover was DEAD CODE, and it flew to hardware that way

⛔⛔ **The entire F1-F4 block was nested INSIDE the TAB branch.** `if (bhumi_key_usage(ev) == 0x2B) {` opened
and did not close until after the mover, so the mover's own guard `if (bhumi_key_usage(ev) >= 0x3A)` asked
whether 43 is at least 58 — two mutually exclusive predicates on the same pure expression, the inner one
lexically inside the outer. **Unreachable for every possible input.** It compiled, it linked, `strings` found
its console text in the flashed binary, and the only symptom on a real boot was one absent log line.

⇒ **The mapping now lives in `input_map`/`input_apply`, where a unit test can reach it.** That is the actual
fix; a re-indent alone would have left the next one just as invisible. An inline block inside a 250-line frame
loop is unreachable *by construction* as far as testing goes, and `tests/input.tcyr` went **15 → 44 asserts**
the moment the logic moved somewhere addressable. ⚠ Two mutants confirm the new asserts bite: deleting the
re-clamp fails 2, zeroing the step fails 11.

### Fixed — one TAB press advanced focus TWICE, and claimed keys leaked to clients

⛔ **`input_handle` already APPLIED the action** (main.cyr:437 → `input_apply` → `comp_focus`), and the frame
loop's TAB branch then called `comp_focus` again — so every TAB was a **double hop**. With three windows that
reads as cycling backwards, and it was invisible on iron because both hops are legal. The 2026-08-08 burn's
four `focus cycled by TAB` lines were four double hops.

⛔ **And every key the compositor claimed was also forwarded.** The loop threw `input_handle`'s return value
away except for the quit test, so F4/F5/F6 were applied *and* delivered to the focused client — one keypress
closing a window and typing into it. `input_handle` now returns the action and **any non-`IA_NONE` action
consumes the key**, which closes the double-hop and the leak in one place rather than per-key.

### Changed — `input_apply` / `input_handle` take the usable screen bounds

`input_apply(comp, action, bound_w, bound_h)` and `input_handle(comp, ev, bound_w, bound_h)`; the caller
passes the height already reduced by reserved chrome. ⚠ **Bounds are parameters, not globals, and that is
load-bearing:** the geometry lives in `main.cyr`, which is included *after* `input.cyr`, and Cyrius resolves
forward references to functions but **not to variables** — so a global would have been invisible here. It also
means the unit test can supply its own bounds without pulling in `render.cyr` for `TITLEBAR_H`. `AE_MOVE_STEP`
moved into `input.cyr` for the same ordering reason.

### Added — the compositor names every DISTINCT key usage it sees

⭐ **The instrument the 2026-08-08 burn did not have.** That boot printed every other line and not the
mover's, and from the log alone "the keys were never pressed" and "they arrived and were dropped" were
**indistinguishable** — a burn that cannot say which usages arrived cannot be re-run into an answer, only
re-run into the same ambiguity. Now every usage the compositor sees is named the first time it appears.

⛔ **DISTINCT, and not "the first N events", because the first-N version failed on its very first outing.**
Bounded at 12 key-downs, it went to hardware, the operator held F8, and **eleven of the twelve slots went to
eleven copies of usage 65** — so the budget was gone before they pressed F4, the one key whose behaviour then
came into question. An instrument a held key can exhaust goes quiet exactly when a session gets interesting.
A repeat now costs nothing. 256 bits in 4 u64 slots; `ae_inp_traced < 32` is a second belt so a pathological
run cannot flood a console three procs share unserialised.

⚠ **It lives in `input.cyr`, not the frame loop, so a test can call it** — the same structural lesson as the
dead mover, applied without waiting for another burn. `tests/input.tcyr` is at **67 asserts**, and the
load-bearing ones were derived from a mutant: dropping the `* 8` slot stride makes the four words overlap at
byte offsets 0-3, which usages 191/192 do **not** detect (they stay independent under that aliasing). The
pairs that actually collide are **(8, 64), (16, 128), (24, 192)**, and with those asserted the stride mutant
fails on exactly three lines. ⚠ The `& 63` on the shift is documentary only — x86 `shl` masks the count in
hardware, so no test can catch its removal, and the range guards do the real work. Said here rather than
implied by a green suite.

### Fixed — ⛔⛔ closing a window left it on screen, FLASHING, and orphaned its client alive

Operator, 2026-08-08 iron: *"closing application with f4 appears to have issues with flashing / not
disappearing or closing properly."* ⚠ **Two defects wearing one sentence**, both pre-existing since the
Rust→Cyrius port of `input.cyr` and both unexercised until the first session that pressed F4.

⭐ **THE FLASHING — a close is not a move, and `union(cur, prev)` only saves the move.** `#84 present`
FLIPS the render target, so erasing a region takes **two** consecutive frames of band coverage, one per
buffer. Anything in `cur` gets that for free: it is `cur` this frame and `prev` the next. A MOVED window
is in `cur` because its rect changed — which is exactly why the burn proved it leaves no ghost. A CLOSED
window is in **neither**, because every damage producer walks the LIVE window list and it has just left
it; it got one frame of coverage from the previous frame's `prev` and no more. So the buffer drawn on the
close frame lost the window and the other buffer kept it, and `#84` alternated them: **a window blinking
at half the frame rate.**

⛔ **And the STATIC clientless window was worse — a second symptom from the same cause.** A
compositor-seeded window has `win_bufid == 0` and, after its first frames, `win_rect_changed == 0`, so it
was in neither `cur` nor `prev` **even on the close frame**: closing it damaged *nothing* and it simply
stayed on screen in both buffers. That is "not disappearing" as distinct from "flashing".

⇒ `comp_close_window` now records the dying window's rect (`comp_retire_add`) **before** unlinking it —
after `vec_remove` the geometry is unreachable and it is the only record of which pixels need repainting
— and `rend_frame_damage` folds that rect into `cur` exactly once, **after** its own clear. Folding into
`cur` rather than into the band directly is the point: it buys the same two frames a move gets, through
the same mechanism, with no new special case in the band. ⚠ The rect is stored raw and the consumer adds
`TITLEBAR_H`, because that constant lives in render.cyr which is included *after* compositor.cyr.

⚠ **Mutation-tested, and the asserts are on the BAND rather than on pixels** — one framebuffer cannot
show a flip, but band coverage on two consecutive frames is what the flip requires. Removing either half
of the fix fails 5 and 11 asserts respectively, including the literal *"one frame of coverage is what
flashed"* line. `tests/render.tcyr` is at **160 asserts**, with the static-clientless case and a
two-closes-in-one-frame union as their own groups.

### Fixed — a closed client was left ORPHANED ALIVE, holding a `#97` endpoint and one of 16 shm slots

⛔ A setu client is a **separate process**, and removing its window from the compositor's vector is
invisible to it. So F4 left `crab`/`puka` running for the rest of the boot, still holding their `#97`
channel end and their `#86` GPU-visible shm slot — and there are only **16 of those slots system-wide**,
so a handful of closes would exhaust hardware compositing for everything.

⭐ **`SETU_CLOSE` (kind 7, C<->S) has been in the protocol from the start** (`lib/setu.cyr:140`) with a
constructor ready to use (`setu_close`, `:382`). Nothing ever sent it and no client ever handled it. This
change is **wiring an existing message, not extending the protocol**: `comp_close_window` sends it to
`win_cfd`, and crab and puka exit on receipt (both unreleased — the change is in their working trees). ⚠ The client's **exit** is what
actually releases the endpoint and the slot — the kernel reclaims both on process death — so honouring
the message is the release mechanism, not a courtesy.

⚠ `win_cfd == 0` is a compositor-seeded window with no client, and notifying nobody is CORRECT there. The
close path now says which case it took, because it previously printed nothing and that cost a QEMU run:
F4 arrived, no client acknowledged, and the log could not separate "correctly told nobody" from "the send
is broken" from "the client ignored it".

⭐ **Verified in QEMU end to end**: compositor sent `SETU_CLOSE`, **crab acknowledged and exited**, and
the screendump shows the window gone with clean background where it was — no ghost, no doubling.

### ⛔ `#91 gpu_blit_bb` has NO correct consumer here, and the ladder assumed otherwise

`#91` copies a rect **within** the back buffer — the natural use being "move a window without recompositing
it". It does not apply to this compositor, for a reason that is structural rather than fixable by care:

**Every window here is a live client surface that is re-composited every frame by design.** `render_window`
re-reads a `bufid` window's buffer each frame and the damage model marks such windows damaged unconditionally
— *"Treating those as undamaged would freeze animating windows"*. So the content a `#91` copy would move is
already being rewritten from the client's buffer regardless, and copying stale composited pixels saves
nothing. For a chrome-only window `#88` is already the cheap answer.

⚠ A second obstacle sits behind that one: `#84 present` FLIPS, so a copy's source is 1 or 2 frames stale
depending on a buffer parity **ring 3 cannot observe** — `gpu_bb_back` is kernel-private and is not in the
`#89` record. Parity could be derived by counting successful presents, so this is the weaker of the two
reasons; the content one stands on its own.

⇒ `#91` becomes useful when the compositor preserves the back buffer and touches only damage, which is a
different frame architecture from the one `AE-9` just finished. Recorded in the falsified list rather than
implemented, because implementing it would add a path that saves nothing and can silently copy stale pixels.

### Also in 0.12.4 — `AE-9`: the chrome fills move to the GPU and the `#39` blit is GONE

### Added — `AE-9`: `#88 gpu_fill_rect` for the clear and every chrome rect

⭐⭐ **The `#39` chrome blit is no longer issued on a GPU frame.** With the clear, the chrome rects, the
glyphs (`AE-8`) and the client surfaces (`AE-6`) all going to the GPU, **nothing remains in the userland
framebuffer for `#39` to copy** — so it is skipped. That is the order this file's own `#85`-removal note
wrote down in advance: clear the back buffer, fill the window and panel rects, glyph the title text, and
nothing overwrites the clear because there is no CPU layer left to copy over it.

⭐ **The clear was the big one.** `AE-0a` measured it at 3.83 ms of a 6.40 ms frame and cut it to 2.46 ms by
shrinking it to a damage band. It is now not a CPU store loop at all — the band is simply the first rect in
the queue, filled by the same CP-DMA engine as the chrome.

`fill_rect` and `rend_clear_band` enqueue instead of writing pixels; `ae_gpu_emit_chrome()` drains first, as
the frame's bottom layer. ⚠ Uses the existing `sys_gpu_fill_rect` wrapper — no raw syscall, no cyrius change.

⛔ **THE MIGRATION IS ALL-OR-NOTHING PER FRAME.** The `#39` blit is the bottom layer, so a frame that draws
half its chrome into the userland framebuffer and half through `#88` has the two overwriting each other in
whichever order the frame happens to run. `ae_chrome_on_gpu()` answers once per frame and every fill in that
frame follows it.

⛔ **CLIPPING MOVED INTO THE QUEUE, and it is not cosmetic.** `#88` bounds-checks against the back buffer and
**REFUSES** an out-of-range rect, while `fill_rect` has always clipped silently. Passing an unclipped rect
would turn a window dragged one pixel past the edge into a refusal — and a refusal retires the chrome path
for the whole session.

⛔ **A refusal must still blit THIS frame.** The rects were queued instead of drawn, so the userland
framebuffer holds no chrome; a frame that neither fills nor blits shows **nothing at all**. `ae_gpu_emit_chrome`
returns `-1`, the caller leaves the band intact so `#39` runs once, and the latch sends every later frame back
to the CPU. Overflow behaves the same way and is counted.

⛔ **A REAL BUG, CAUGHT BY RE-READING BEFORE IT SHIPPED.** The band-zeroing that skips the blit was first
placed **above** the damage-band block — which assigns `band_h` unconditionally, so the zero was silently
overwritten and the blit would have copied an **empty** framebuffer straight over the chrome the GPU had just
filled. Order is the whole correctness of that block. The comment there now says so.

⚠ **Batched `#92` (64 records per submission) is NOT part of this** and cannot be, as the ladder assumed:
batching requires each record to name its own slot, and the glyph path deliberately reuses **one** staging
slot per run (`#86` slots are scarce — 16 system-wide, each rounded to a 2 MB page). Records in one
submission are processed after it is handed over, so a shared slot would give every record the last run's
mask. Batching needs N slots or a per-record offset field the ABI does not have.

**134/134** in `tests/render.tcyr` (was 103): queueing instead of drawing, the framebuffer left untouched,
clipping on all four edges, the fully-off-screen no-op, the clear band as a rect, overflow refusing and
counting, the CPU fallback still drawing, and the latch retiring the path.

⚠ **QEMU proves the FALLBACK, not the GPU path** (no AMD device ⇒ `ae_gpu_frame_ok` is 0). Desktop counters
byte-identical to pre-`AE-9`: dim-green 900514, red bar 972, non-black 3975361, terminal gate PASS twice.

### Also in 0.12.4 — `AE-8`: chrome text on the shader cores, and `AE-6`: premultiplied surfaces

### Added — `AE-8`: glyphs off the CPU, via `#92` op 0x03 GLYPH_1BPP

⭐⭐ **Titlebar and panel text was the last chrome layer drawn a pixel at a time.** `draw_char` calls
`bhumi_fb_set` per lit pixel — a function call, four bounds compares and seven framebuffer-header `load64`s
to store four bytes, the same overhead that made `fill_rect` cost 12.72 ns/px before it walked rows. And
`AE-T` made glyphs the desktop's hot content by putting a terminal on it.

⭐ **The format already matched, which is why this is small.** `#92` op 0x03 takes a 1bpp mask and paints
`color` where a bit is set, leaving the destination untouched — text, exactly. kashi's VGA 8x16 font is one
byte per row with `bit (7 - col)` leftmost, and the kernel's shader reads **MSB-first**. So
`ae_text_pack_1bpp` copies bits at an x-offset rather than converting a format.

**Render enqueues, present drains.** Chrome reaches the back buffer through the `#39` DEFER blit inside
`ae_gpu_present_frame`, which runs *after* render — so a glyph op issued during render would be erased by
that blit. `draw_text` queues the run when `ae_gpu_frame_ok == 1` and `ae_gpu_emit_text()` emits it after
the chrome and the client surfaces but before the flip. ⚠ The queue lives in render.cyr for the same reason
`ae_frame_dmg` does: a cyrius *variable* forward-reference does not resolve and `tests/render.tcyr`
compiles that module without gpu.cyr. It is reset in `rend_frame_damage`, next to the damage roll, so no
caller can forget.

⛔ **A refusal falls back to the CPU, and is counted.** Queue full, pool full, or a run longer than the
packer's cap → `draw_text` draws it per-pixel as before, so text is never lost; `ae_text_dropped_get()`
records it because a silent drop is a lie. Same for the slot: if `#86` cannot serve the 1bpp staging slot,
the compositor says so once and leaves titles on the CPU permanently rather than retrying every frame.

⛔ **One slot, allocated once, reused every frame.** `#86` slots are scarce — 16 system-wide, and
`shm_create` rounds every request up to a 2 MB page — so per-run or per-frame allocation would exhaust the
table in seconds. ⚠ And a glyph failure does **not** demote the frame: the chrome, window bodies and client
surfaces are already correct in the back buffer by then, so losing a whole desktop over a caption would be
the wrong trade.

#### Verified on the host, bit-exactly, with no GPU

`tests/render.tcyr` packs a string, expands the mask **the way the kernel's shader does**, and compares it
pixel-for-pixel against `draw_text`'s own output on an identical framebuffer: **0 differing pixels.**

⛔ **The negative control is the mistake the kernel names in its own source: reading the mask LSB-first**,
which mirrors every glyph horizontally. A mirrored glyph is still a glyph-shaped blob to a pixel count, so
only an exact comparison against the reference renderer finds it — and the control asserts that reading
LSB-first *fails*. Plus the queue's routing, capacity and refusal paths. **99/99** (was 80).

⛔ **The first run of that test was VACUOUS and its own guard caught it.** Without `kashi_font_init()` every
`kashi_glyph_row` answers 0, so the reference and the mask both came out blank and compared EQUAL. The
`lit > 100` assert — "the reference actually drew glyphs" — is what turned a green into a red.

⚠ **QEMU cannot exercise the GPU path** (no AMD device ⇒ `ae_gpu_frame_ok` is 0 ⇒ `draw_text` takes the CPU
branch), so what the desktop run verifies is that the **fallback is intact**: framebuffer counters identical
to the pre-`AE-8` desktop, terminal gate still PASS. That is the half that could have broken, since
`draw_text` gained a branch. The shader path runs on iron.

### Added — `AE-6`: the compositor composites premultiplied surfaces on the SHADER

⭐⭐ **Premultiplied client surfaces are now composited with `gpu_shader_op #92` op 0x01** — a real
per-pixel `out = src + dst * (1 - src_a)` on the compute units, the one thing `#87`'s ALU-less byte mover
structurally cannot do. The path has been complete in this repo and in the kernel for weeks with no client
requesting it; crab now does, so it runs.

### Fixed — a premultiplied window no longer demotes the WHOLE FRAME to the CPU

⛔ **`ae_gpu_window_admissible` REFUSED a premultiplied window when GPU caps bit3 was clear, and a single
refused window fails the entire frame to software.** So one client asking for a blend cost **every** window
its hardware compositing — on any box whose shader envelope is absent. That is a far worse outcome than
compositing that one surface opaquely.

Which op composites a surface is a **per-window routing question**, and it is answered where the routing
happens (`ae_gpu_present_frame`): `#92` when the envelope exists, `#87` otherwise. ⚠ The old refusal reasoned
that `#87` "would paint a half-transparent window at full strength" — true only for real translucency; for
the alpha-255 surfaces clients produce today the fallback is **byte-exact**, not approximate.

The frame log now names which op ran (`compositing a client surface with #92 op 0x01`, or
`premultiplied surface but no shader envelope -- using #87`), first occurrence only, both cases — so
absence of a line is not the signal for either.

### Changed — `--selftest` exercises the production path, and its oracle now survives an alpha byte

The selftest window is premultiplied like every real client surface, so the arm tests what the desktop
actually does. Its source is painted **alpha 255**, where premultiplied and straight alpha are the same bytes
and the shader collapses to `out = src + dst*0 = src` — so **every band expectation is reusable unchanged**
and the check is a differential test of the shader against the copy rather than against a hand-computed
number.

⛔ **The sentinels were the trap, and this is why it was not a one-line change.** `AE_ST_S0..S3` carry byte
3 = **0** — correct for `#87`, which ignores it. Under `#92`, `a == 0` gives `out = src + dst`: an
**additive over-bright ghost**, not a vanished window. Flipping `win_set_premul` alone would have failed
every band, and the obvious reading — *"`#92` is broken on this box"* — would have been false.

`ae_st_px` now masks byte 3 (low 24 bits), which is load-bearing rather than lax: the shader writes
`out_a = 255`, so the readback carries `0xFF` where an opaque copy carries `0x00`. The four sentinels stay
distinct in 24 bits, so every spatial discrimination survives — and `tests/selftest.tcyr` asserts both
halves: two sources differing only in alpha compare **equal** through the accessor, while two different
**bands** still compare **unequal**.

**36/36** in `tests/selftest.tcyr`, 21 suites green, both targets build.

## [0.12.3] - 2026-08-07 — `AE-0a`: the damage band drives the frame, and TAB cycles focus (it never did)

### Added — `AE-0a` damage tracking: the clear and the `#39` chrome blit are band-limited

⭐⭐ **The damage model has been computed every frame since 0.11.0 and drove NOTHING.** It now drives the
two things that cost the most: the full-screen clear, measured at **3.83 ms of a 6.40 ms GPU frame (60%)**,
and the deferred `#39` chrome blit.

⛔ **The band is `union(cur, prev)`, and that is forced by the kernel, not chosen.** `#84 present` flips the
render target on every present (`agnos/kernel/core/gpu.cyr`), and `#39` writes whichever buffer that same
predicate selects — so a band-limited blit updates buffer A this frame and B the next, leaving A's rows
outside this frame's band holding what was there **two frames ago**. Writing the union of this frame's and
last frame's damage covers exactly the set the target buffer can be missing. ⚠ Getting it wrong shows as a
trailing ghost at a two-frame lag in the rows a window vacated, which reads as *"the framebuffer is
stale"* rather than *"the damage model is wrong"* — the misreading that stalled this bite twice.

⭐ **The first two frames come out full-screen for free**: frame 1 is primed full, so frame 2's union still
contains it — which is precisely what is needed to initialise *both* buffers.

⛔ **Full-width rows only.** `#39` takes its source tightly packed at `w*4` bytes per row with **no stride
parameter**, so an arbitrary damage rect's rows are not contiguous and cannot be handed to it at all. A
full-width band is contiguous, so the band carries rows `[y, y+h)` and the x extent is discarded. True rect
damage needs a stride on `#39` — a kernel ABI change, not this bite. The packing is **asserted, not
assumed**: a padded pitch falls back to the full frame, because a skewed image is far worse than a slow one.

**New `rend_clear_band` walks the band as ONE flat run.** Going through `fill_rect` cost **1.347 ns/px**
against `bhumi_fb_clear`'s **1.093 ns/px** for byte-identical stores — 23% of pure per-row overhead
(recomputing a base, a span and a bound 780 times) for rows that are already contiguous. Same finding as
the 0.12.1 per-pixel fix, one level up.

Two invariants are held by construction rather than by convention: **the roll happens inside
`rend_frame_damage`**, so no caller can forget to make this frame's damage next frame's `prev`; and **the
band is read through `rend_band_valid/y/h()` functions, never as a variable** — a cyrius *function*
forward-reference resolves and a *variable* one does not, so reading `ae_band_valid` from desktop.cyr
compiled fine under `cyrius build src/main.cyr` and broke `tests/desktop.tcyr`, which compiles that module
in a unit render.cyr never enters. That is the trap already recorded for `ae_frame_dmg`, sprung by a new
consumer.

⚠ **A background change refuses the band entirely** and forces a full clear: a theme switch damages every
pixel without moving a single window, and a band-limited clear would leave the old colour everywhere
outside it.

#### Measured — `tests/aethersafha.bcyr`, host, two-window desktop

| at 2560x1440 | before | after | |
|---|---|---|---|
| clear | 3.937 ms (whole screen) | **2.460 ms** (band) | **1.60x** |

⚠ **The win is bounded by geometry and this must not be overstated.** Two 1280x720 windows cover **780 of
1440 rows**, so 46% of the clear is the most any band can remove at that layout; the band pays off more as
windows get smaller or more static, and pays nothing when one window spans the screen. In frame terms the
clear drops from **60% to ~48%** of the GPU frame.

**Correctness: 80/80 in `tests/render.tcyr`** (was 56) — union folds in both orders, screen clamping at
both edges, the theme-switch refusal and its one-frame scope, the missing-tracker full-screen fallback, the
idle case (a valid band of height **0**, which is a result and not a failure), the roll, and a **pixel-level
ghost test** that moves a window and asserts the vacated region reads background.

⛔ **Both mutation-tested, and the first version of the ghost test was VACUOUS.** It moved the window on
frame 2 and passed against a mutant that dropped the old rect from the fold entirely — because frame 1
primes damage full, so frame 2's band is full no matter what the fold does. It now settles for two frames
first, and asserts the idle band is empty on the way. Mutant A (old rect dropped) fails the ghost assert
and the OLD-top assert; mutant B (prev dropped from the union) fails four union asserts.

**On agnos at `AE_CLIENTS_MODE=desktop`, the framebuffer is pixel-count IDENTICAL to the pre-change run** —
dim-green 900514, red bar 972, non-black 3975362, glyphs 4991 → 5176 → 6032. The same picture, fewer stores.

⚠ **What is still unexercised: the banded `#39` blit itself.** QEMU has no AMD PCI device, so
`ae_gpu_frame_plan` refuses every frame and the CPU present path runs full-frame — the banded *clear* is
proven there, the banded *blit* is not, on any substrate. ⛔ And the `--selftest` cannot prove it either:
its two frames are both inside the priming window, so both blit full-frame by design. **It needs ≥3 frames
with a settled window, i.e. the desktop run on iron.**

### Changed — TAB cycles focus (it never did)

⛔ **THERE WAS NO TAB HANDLER, WHILE THE FRAME LOOP CLAIMED THERE WAS.** A comment on the
focus-change detector read *"a TAB in the input loop above"* — and no such code existed. The
consequence is not cosmetic: `comp_focus` was only ever called when a client was ADDED, so focus
settled on whichever client came last and **could never move again**. Every forwarded key went to that
one window for the life of the session, and a second client could not be typed into at all.

⭐ **Found by a client that was working.** puka's terminal received **zero** key events on a run where
the compositor was forwarding correctly — to crab, the other window. "Nothing arrives" and "everything
arrives somewhere else" look identical from inside the client.

TAB (HID usage `0x2B`) on key-down now advances focus and is **consumed, not forwarded** — a focus key
that also reached the client would type a literal tab into the window you just switched away from.

⚠ No-op with fewer than two windows, so a single-client desktop behaves exactly as before.

### Added — a PERMANENT input ladder, because this path has gone dark twice

⛔ **The input path had no instrumentation at all, and that cost two runs of pure confusion.** With TAB
fixed, a run still delivered zero keystrokes to the client and there was no way to tell whether the poll
returned nothing, the seat was denied, TAB never arrived, focus refused to move, or the forward failed —
every one of those looks identical from outside, and the first four leave no trace whatsoever.

Five first-occurrence latches now print once each, so **the last rung printed names the layer that
failed**: `input poll answered SEAT_DENIED` · `input poll delivered its first event` · `first key-down
seen` · `a TAB reached the compositor` · `TAB ignored -- fewer than two windows` · `forwarded a key to the
focused client`. Distinct strings, never a bare integer — three procs share this console unserialised and a
number lands mid-line inside another proc's output.

⭐ This immediately paid for itself. The full ladder printing in order, followed by puka's own markers,
is what turned *"keystrokes vanish somewhere"* into a measured statement: the keys that arrive are handled
correctly at every stage, and the ones that go missing are **never sampled** — a USB HID keyboard reports
state on poll and agnos drains the xHCI ring only inside `kbscan #42`'s bounded `sti` window, which the
compositor calls once per frame. ⚠ **A key pressed and released inside one frame does not exist.** Measured
on the QEMU CPU path: 0/9, 4/9, 4/9 keys delivered at a ~100 ms hold; 9/9 at 500 ms. The real fix is a
faster frame (`AE-0a`) or IRQ-buffered HID reports; nothing here works around it.

`setu_srv_forward_key`'s return value is now read rather than discarded, which is what makes the last rung
truthful (it returns 0 when the focused window has no client fd).

## [0.12.2] - 2026-08-07 — the compositor stops listening (ipc bite 7)

### Changed — clients are PLACED, not accepted

⭐⭐ **`setu_srv_listen` and the accept block are gone on agnos.** For each client the compositor now
**mints** a channel (`CH_MINT`), **endows** one end to the child (`CH_ENDOW`, which returns the fd
number the child will hold), stages `AGNOS_CHAN=<fd>` into the `#43` env blob, and spawns the client
**already holding a connected end**. There is no port, no dial, no accept, and no window in which a
client can connect to something that isn't the compositor.

Handshake driving moved to `setu_srv_handshake_step` — a 4-state machine (await CREATE → await ATTACH
→ await COMMIT → done) stepped once per client per frame, because *the batch IS the poll*: a
would-block is a result, not an error, so one slow client cannot stall the frame loop or the others.

Requires **agnos ≥ 1.56.40** and **setu > 0.8.1**. ⚠ `crab` and this repo carry a TEMP
`path = "../setu"` override in `cyrius.cyml` until setu is cut; both must revert to the tag.

### Fixed — the diagnostic that ate the evidence

⛔ The poll-failure branch called `sys_chan_recv` to report *which* kernel error class it hit. On a
record transport **a read is not a peek**: that probe consumed the handshake record it was trying to
explain, so the instrumented build failed differently from the build under diagnosis. Removed, with a
comment at the site so it is not re-added.

### Fixed — function-local `var chan_fd[4]` is FOUR BYTES, not four slots

⛔ The per-client arrays were sized as if module-scope rules applied (N×u64). Function-local `var X[N]`
allocates **N bytes**, so client 1's store at offset 8 walked off the end and smashed adjacent locals.
This is exactly why one client worked and two did not — and it read as an IPC bug for hours. Arrays
widened to `[32]`. [[feedback_cyrius_var_array_u64_units]]

Proven: two independent clients (`present_probe` + `crab`) both present under QEMU `-smp 4` —
`presented: 2`, external framebuffer oracle 3500 client-coloured px.

### Changed — TCP is gone from this repo too (setu 0.8.4)

The listener block, the frame-loop accept and the shutdown close carried the last TCP references here.
`sock_close` is replaced by `sys_close`, and `setu_srv_listen`/`setu_srv_accept`/`setu_srv_accept_one`
are Linux-only paths now — setu's agnos arm refuses both, so on agnos they return a negative code and
this compositor never calls them. On the host the listener is **AF_UNIX/SOCK_SEQPACKET**, the same
record semantics as the `#97` band.

⛔ **A comment here read "ON agnos THE COMPOSITOR NO LONGER LISTENS" while sitting directly above code
that still opened loopback:7700.** It had said so since bite 7 landed. A confident comment asserting
the opposite of the code beneath it is worse than no comment: it is what a reader checks *instead of*
the code. Both the comment and the code are now correct.

⚠ Requires **setu >= 0.8.4** and **agnos >= 1.56.40**.


## [0.12.1] - 2026-08-05 — the per-pixel cost, not the pixel count

### Changed — the compositor's per-pixel loops walk rows; ~4x faster frames at native resolution

Closes the carry-forward from the agnos 1.56.36 iron burn — operator: *"working... aethersafha is a
little slow but we can deal with that later."*

⭐ **That report and the native-resolution modeset were the SAME burn.** 1.56.36 took archaemenid's
panel from 800x600 to 2560x1440 — **7.68x the pixels** — so the question was never "is the compositor
slow", it was "which per-pixel loop just grew". Measurable on the host: the clear and the chrome fills
are portable code that runs identically on Linux and agnos.

⛔ **The pixel COUNT was not the interesting number — the per-pixel COST was.** Measured at 2560x1440
(`tests/aethersafha.bcyr`, added for this):

| | pixels | before | per pixel |
|---|---|---|---|
| `bhumi_fb_clear`, whole screen | 3,686,400 | 3.83 ms | **1.03 ns** |
| chrome, 2 windows (`fill_rect`) | 1,843,200 | 23.45 ms | **12.72 ns** |

**Half the pixels, six times the time — 12.3x more expensive per pixel for byte-identical stores.**
The whole gap was per-pixel overhead, not memory: every pixel went through a function call, four
bounds comparisons, and `_bhumi_fb_addr`, which between them re-read **seven** `load64`s out of the
framebuffer header (width, height, pixels, pitch, bpp) and did two multiplies — to store four bytes.
None of those seven values can change during a fill.

`fill_rect` now clamps the rectangle once and walks rows from a precomputed base and stride. New
`blit_rect` does the same for the client-surface copy, which `render_window` previously did with the
same per-pixel call.

| at 2560x1440 | before | after | |
|---|---|---|---|
| chrome, 2 windows | 23.45 ms | **2.57 ms** | **9.1x** |
| CPU surface copy, 2 windows | 23.97 ms | **4.18 ms** | **5.7x** |
| clear (untouched) | 3.83 ms | 3.83 ms | — |
| **GPU frame** (clear + chrome; kernel does the surface via `#87`) | **27.24 ms** | **6.40 ms** | **4.3x** |
| **CPU frame** (GPU refused, and every QEMU run) | **51.21 ms** | **10.58 ms** | **4.8x** |

Uses only bhumi's public accessors (`bhumi_fb_pixels` / `_pitch` / `_bpp`), so no bhumi change and no
tag bump. `bhumi_fb_set` remains correct and is still the right call for scattered single pixels.

⚠ **The clear is now 60% of a GPU frame** and is the next item. It wants the damage model, not a
faster loop — and that needs `union(cur, prev)` first, because `#84 present` flips the render target
(desktop.md §2 `AE-0a`).

⚠ **The CPU surface copy does not run on iron when the GPU takes the frame** — the kernel composites
out of the client's GPU-visible shm slot and the pixels never enter userland. It is fixed anyway
because it is the live path on every QEMU run and on any frame `ae_gpu_frame_plan` refuses. A
fallback nobody measures is how a "GPU-accelerated" desktop ships its slow path.

### Added — direct clipping coverage for `fill_rect` and `blit_rect`

Both functions had only indirect coverage (via `shell_render` and `gpu_fallback`), and clamp-once is
exactly where a rewrite can silently disagree with a per-pixel original. `tests/render.tcyr` now
asserts every edge: in-bounds, clipped left/top/bottom-right, fully offscreen both directions, and
zero/negative dimensions.

⛔ **The source-offset case is the one that matters.** The per-pixel loop got source clipping for
free — a dropped write still advanced the source index. Row-walking does not: the source pointer must
advance by exactly the amount the destination was clipped by, or a partly-offscreen window draws the
**wrong part of its own surface**, which reads as a corrupt client rather than a clip bug. The test
encodes each source pixel's own coordinates so a wrong offset is visible, not merely wrong.
**Negative control:** removing the source-offset advance fails exactly those two assertions
(54 passed, 2 failed), so the test can detect the bug it was written for.

**Verified:** 21/21 suites, 56/56 in `render.tcyr`, `--agnos` build clean, and on agnos at
`AE_CLIENTS_SMP=4` both launch paths reach connected 2 / presented 2 / exit 95. ⭐ In `desktop` mode
the framebuffer is **pixel-count identical** to the pre-change run — dim-green 952,731, red bar 3,500,
non-black 4,194,304 — i.e. the same picture, drawn 4x faster.

## [0.12.0] - 2026-08-02

The desktop's first setu clients reached iron and neither one arrived. Two of the three reasons
turned out to be defects in this file; the third is now a question the compositor answers itself.

### Fixed — ⛔ THE LISTENING SOCKET LEAKED ON agnos, WHICH SILENTLY DISABLED EVERY RUN AFTER THE FIRST

`sock_close(setu_sfd)` sat inside `#ifndef CYRIUS_TARGET_AGNOS`. So the one target whose compositor
loop is designed to run forever — and therefore the one that actually gets quit and relaunched by
hand — was the only target that never released port 7700.

The burn log shows it exactly: run 1 prints `setu listener up` and launches both clients; **run 2
prints no listener line at all**, hosts nothing, and is otherwise indistinguishable from a healthy
desktop (`desktop up — windows: 1`, geometry correct, Esc works).

⚠ There was never a reason for the guard. `sock_close` is portable — `net.cyr` routes it to BSD
sockets on Linux and the agnos kernel TCP band — and the comment beside it ("TCP, no socket file to
unlink") is if anything *more* true on agnos. It reads as a leftover from the AF_UNIX era that was
not revisited when the transport became TCP on both targets.

### Fixed — a failed listen, and a failed client launch, printed NOTHING

`setu_srv_listen` was only reported on success. A compositor that cannot open its display socket is
**structurally incapable of hosting an app**, and it announced that by staying quiet — which is also
exactly what a working listener with slow clients looks like. Same for `spawn_path`: only `pk >= 0`
printed, so a client that never left the ground read identically to one still connecting.

⭐ The general shape, and it has now cost two separate readings this week: **the absence of a line is
not a signal.** Anything worth printing on success is worth printing on failure, or the log cannot
be read backwards.

### Added — `--clients`: a bounded, self-reporting run of the real desktop

⛔ **The burn could not be read, and no amount of staring at it would have helped.** The log said
`launched setu client #1` / `#2 (crab)` and then nothing, and the run ended at frame 128 when no
window had appeared. That is consistent with two *opposite* states — the clients died on load, or
they were still connecting — and the desktop's normal output cannot separate them. Asking the
operator to hold a desktop open for an unspecified time and judge by eye is not an experiment.

`run /bin/aethersafha --clients` runs the **same loop**, stops on its own once both clients have
presented or at a fixed 5000-frame cap, and exits with a code:

| exit | meaning |
|---|---|
| 95 | both apps connected and presented |
| 94 | one did |
| 93 | neither, though both started — the client or the setu path |
| 92 | the display socket never opened |
| 91 | an app could not be started at all — the kernel's `spawn_path` or the image |

⚠ **93 and 91 point at different repos**, which is the distinction the last burn could not make.
⚠ **93 is not "too early"** — the run goes to a fixed **30 s wall-clock** budget and ignores the quit
key, so it means they genuinely never arrived. (The first draft of this bounded on FRAMES and honoured
Esc, which made that sentence false; see the two sections below, folded in before release.)
⭐ It reuses the production loop rather than a copy of it, so the answer is about the desktop that
ships, not about a probe that resembles it.

### Added — an exit summary on every run

`frame loop ok` is equally true of a desktop that hosted two apps and one that hosted none. The loop
now reports the frame count and the number of clients that connected, so an ordinary run is legible
without the probe flag.

### Fixed — ⛔ the first draft of `--clients` COULD BE TRUNCATED BY A KEYPRESS

The first cut of `--clients` reused the desktop's frame loop, which honours `HID_ESC` → `IA_QUIT`,
so a quit key ended the run wherever it landed and the verdict was emitted anyway — while this entry
claimed the opposite (*"93 is not 'too early' — the run goes to a fixed cap"*). Caught on iron before
the version was ever tagged.

The first iron `--clients` run proves it: `quit on a key … 41 … 191`, then `at exit — frames 192,
apps connected 0`, then `run: exit 93` — a cap of 5000 frames, stopped at 192. **The code meant
"nothing had connected yet" and the documentation said "nothing ever connected".** A diagnostic a
keypress can truncate is not a diagnostic, and one whose docs overstate it is worse than none.

A probe run now **ignores the quit key**, counts how many it ignored, and reports that count at exit
alongside the elapsed time. ⚠ Counted rather than silently dropped: an ignored keypress is a fact
about the run, and on this box a stray scancode is plausible — the console echoed `-0 -clients` for
a line whose argv was demonstrably correct (exit 93 is only reachable when the exact `--clients`
match sets probe mode), which is the known flaky xHCI HID path showing up in the echo.

### Changed — ⛔ the probe budget is WALL TIME, not frames

Frames were the wrong unit and the reason the cap was set where it was. What a spawned client needs
in order to load from ext2, start, and complete a loopback TCP connect is **seconds and scheduler
slices**; a frame count measures neither, and frame duration differs by more than an order of
magnitude between QEMU and iron. Any single frame cap is far too short on one and far too long on
the other.

The budget is now **30 s of `sys_uptime_ms`**, with the frame cap kept at 200000 purely as a backstop
should the clock be broken — deliberately far above anything the timer will reach, because a frame
cap that can bind before the timer *is* the bug above restated. Progress prints every 5 s while
nothing has connected, so the log shows the run stayed alive rather than wedged.

⭐ **Exit 93 now means what it always claimed**: 30 seconds elapsed, both clients started, neither
ever connected. The exit line reports the elapsed milliseconds so that is checkable rather than
asserted.

### Note — one hypothesis checked and discarded before it reached anyone

archaemenid has a live NIC, unlike the QEMU boxes this path was proven on, so "loopback TCP is being
routed to the wire" was worth ruling out. It is wrong: agnos `net_tx` (`kernel/core/net.cyr`) tests
`net_is_loopback(dst)` and queues to the loopback ring **before** it ever consults `nic_ready`, so a
live NIC does not divert `127.x` traffic. Recorded because a plausible-and-wrong theory left lying
around gets picked up later as fact.

## [0.11.1] - 2026-08-02

**The desktop composited on real silicon.** `run /bin/aethersafha --selftest` on archaemenid
(AMD gfx90c / DCN2.1) returned **`run: exit 95`** — the client's exact sentinel words read back out
of the GPU's own back buffer at the client's screen coordinates, margin clean, far frame clean.
First time any of this repo's GPU path has executed on hardware. The bare desktop also rendered:
MUDRA chrome, kashi titlebar text, cyan focus strip, traffic lights, on the panel.

The same burn found one defect that only hardware could show — and one *suspected* defect that the
instrumentation added here promptly cleared.

### Fixed — ⛔ THE COMPOSITOR ASKED THE KERNEL FOR THE SCREEN SIZE AND THREW THE ANSWER AWAY

The boot log printed `gpu: console geometry matched to surface 800x600` and then, four lines later,
this compositor printing `1280` / `720` — its fallback.

`ae_query_geometry` tested `bhumi_output_query(&info) != BHUMI_FBINFO_SIZE`. agnos `#38 fbinfo`
fills the struct and returns **0** (0-ok convention), and bhumi passed that 0 through while
documenting "returns 24". So `0 != 24` sent **every agnos boot** to the hardcoded fallback. Fixed at
the source in **bhumi 1.1.3**; this repo now also accepts any non-negative return and validates the
struct's *contents* (the present bit plus sane dimensions), so a return-code convention changing one
repo away cannot silently resize the desktop again.

⛔ **This defeated 0.10.0's headline fix, in full, for its entire life.** That release shipped "THE
COMPOSITOR WAS SIZED 1.6× WRONG FOR THE PANEL" specifically to stop assuming 1280x720 and *ask* —
and the asking never once worked on iron. The photo from this burn shows the seeded "Files" window
at (240,180) 520×360, which is the 1280x720 arithmetic; at the true 800x600 it is (150,150) 325×300.

⚠ **The exit-95 result stands.** The oracle reads the kernel back buffer at the window body's
coordinates, and `#87` blits to those same coordinates — both well inside 800x600 either way. What
the wrong geometry corrupted was the chrome layer (`#39` clipped it) and, more importantly,
`ae_gpu_window_admissible`, which was judging windows against a screen 1.6× larger than the real
one — so it would admit a window the kernel then *rejects rather than clips*, dropping the whole
frame to the CPU.

### Added — the geometry line now says where the number came from

⛔ It used to print two bare integers. `1280` / `720` is a perfectly plausible desktop size; nothing
about it said *"I asked and discarded the reply."* A number alone cannot distinguish a measurement
from a default, and that distinction was the entire bug. It now states which of the two it is.

### Added — the frame loop names the key that stops it, and the answer was "nothing was wrong"

The previous burn's log ended `aethersafha: frame loop ok` with nothing said about why, which read
as a compositor that quit on its own. `IA_QUIT` has exactly one source (`HID_ESC`), so either an Esc
genuinely arrived or the Set-1 decode manufactured one — opposite fixes. Rather than guess, the loop
now prints the HID usage and the frame number on quit.

✅ **Resolved on the next boot: `usage 41` (0x29 = Esc), `frame 113`.** The operator pressed Esc to
leave the desktop, after 113 rendered frames. The exit path was correct the whole time; the log
simply could not say so. **No code defect existed here** — the defect was in what the program chose
to report about itself.

⭐ Kept, because it is cheap and it is the difference between a fact and a theory. An instrument that
costs one `println` on a path taken once per run, and that can retire a suspected bug without
spending a boot on the operator's only machine, has already paid for itself.
⚠ Worth knowing when reading these: `kbscan#42` is a **raw** drain, so the compositor sees every
scancode buffered since boot, including whatever was typed at the agnsh prompt to launch it.

### Changed — `[deps.bhumi]` 1.1.2 → 1.1.3, `[deps.mehman]` 1.0.0 → 1.0.1

⚠ mehman's published 1.0.0 tag still called `backend_name`, which kavach renamed to
`os_backend_name` at 3.8.2; its source had followed but was never cut, and `path = "../mehman"`
masked that in every local build until CI resolved the real tag. Fixed in **mehman 1.0.1**.

### Verified

Host + `--agnos` green; 21 suites, 0 failures; `--selftest` still exits 96 on a GPU-less host.

## [0.11.0] - 2026-08-02

The compositor can be flashed, run on iron, and **report what happened**. Every piece of that
sentence was missing yesterday.

### Fixed — ⛔ THE `--agnos` BUILD, BROKEN SINCE 2026-07-25, IS GREEN

`cyrius build --agnos` now emits a 15,591,456-byte static x86-64 ELF64 — the shape
`agnos/scripts/burn/stage-tools.sh` requires.

The fault was never in this repo. kavach's Linux-only backends named `sys_getgid`, `sys_lstat`,
`sys_fork`, `sys_dup2`, `sys_execve` and `sys_setsid` — none of which exist on agnos — and since
`cyrius build` prepends every `[deps.*]` module, the whole consumer failed even though the
compositor sandboxes nothing.

⛔ **kavach 3.10.0 claimed to have fixed this and had not, for a reason worth carrying forward: an
`#ifdef` early-return does not remove the rest of a function from the build.** That release added
`#ifdef CYRIUS_TARGET_AGNOS return 0; #endif` guards to six confinement entry points and reported
the compositor unblocked. The guard is a *runtime* branch — every statement after it still compiles
and every symbol it names must still resolve — so `spawn_namespaces_available` returned early on
agnos and *still* referenced `sys_getgid` twenty lines later. Fixed properly in **kavach 3.11.0**
with six compile-time shims (`kv_getgid` / `kv_lstat` / `kv_fork` / `kv_dup2` / `kv_execve` /
`kv_setsid`), whose two arms are selected by the preprocessor.

⚠ `[deps.kavach]` still declares `path = "../kavach"` alongside its tag, and **the path wins** — the
vendored copy tracks the local checkout regardless of what the tag says. That is how this repo's
build changed with no change to this repo. The tag now reads 3.11.0; the hazard is unchanged.

### Added — `--selftest`: the compositor's iron verdict

⛔ **Nothing in this desktop's GPU path has ever run on hardware, and QEMU cannot make it.** It
exposes no AMD PCI device, so `gpu_find` never matches, `gpu_caps` reports flags 0, `ae_gpu_probe`
answers 0, and every GPU branch in `gpu.cyr` is dead for the entire run. A green desktop smoke
proves the CPU fallback — which was never the part in doubt. Hardware compositing has been shipping
since 0.9.6 with **zero** evidence behind it.

`run /bin/aethersafha --selftest` seeds a window's shared buffer with a four-band sentinel pattern,
drives the **real** `ae_gpu_frame_plan` → `render_desktop` → `ae_gpu_present_frame` sequence, captures
the finished frame through `#90 gpu_readback_shm`, pulls it back with `#73`, and exits with a coded
verdict — `95` = the GPU composited the client's surface at the client's coordinates. Same contract
as `gpufill` / `gpublit` / `gpucopy` / `gputri`, because an exit code is the only channel a program
that owns the screen has.

⭐ **The oracle is external.** The sentinels (`0x00C0FFEE`, `0x00BADA55`, `0x00FACADE`, `0x005EEDED`)
are chosen by the test and derivable from no compositor state — not a rupa token, not a chrome fill,
not the framebuffer's boot contents. A renderer sharing every one of the compositor's premises still
cannot produce one by accident. This arc has already published a wrong claim that three documents
agreed on; agreement is not evidence.

⭐ **Self-contained by design** — one process, no setu client, no `spawn_path`, no loopback TCP, no
two-proc scheduling. Every one of those is a way for a run to fail for reasons unrelated to the GPU.
A non-95 exit points at the compositing path and nothing else.

⚠ **Three negative controls, because the positive check alone is not sufficient.** A *whole-buffer
smear* satisfies every band probe while the picture is completely wrong, so the margin ring must
come back clean; a second frame captured 300 px away must contain no sentinel at all; and the
landing buffer is poisoned and scanned **before** the first frame, because `#90` fails silently —
a stale read returns plausible pixels, never an error.

### Added — a pre-flip capture hook inside the shipping frame function

⛔ **It fires before `#84`, and that is not a preference.** `present` toggles `gpu_bb_back`, so a
readback issued after the flip samples the buffer the frame did *not* draw into and returns the
previous frame in full, with no error. On a static desktop that looks almost right — the worst
failure mode an instrument can have.

⚠ The hook lives in `ae_gpu_present_frame` itself, not in a probe that mimics it. A verification
path that is not the production path grades the mimic; this repo has paid for that twice already
(`render_frame` carrying a damage model the live loop never called, and the GPU decision living
apart from its consumer).

### Fixed — the compositor had no entry point, no exit code, and silently ignored every argument

`main.cyr` ended at `main`'s closing brace and relied on cycc's auto-emitted call. It ran, which is
why nobody noticed, and it cost two things that only matter on iron: **there was no exit code** (the
return value went nowhere), and **argv was unsafe to read**. Now enters through a bare top-level
`_ae_entry()` + `SYS_EXIT`, the form every staged agnos tool uses.

⛔ **And `args_init()` was never called, so the first `--selftest` build launched the DESKTOP.**
`argc()`/`argv()` are not ambient — without that call `_args_base` is 0 and `argc()` answers 0
forever. The flag was not rejected; it was invisible. Exit 0, a desktop on screen, and a burn that
would have read as "the desktop came up" while the oracle never ran.

⚠ `SYS_EXIT`, never a literal 60: agnos renumbers the enum and exit is **0** there, so `syscall(60,
r)` is a no-op that only appears to work because cycc's implicit exit follows it.

### Added — `tests/selftest.tcyr`, 29 assertions (21 suites green, was 20)

The verification logic is deliberately `#ifdef`-free so it is testable on a machine with no GPU —
otherwise the part most likely to be wrong (band arithmetic, negative-control scans) would be
verifiable only by burning a boot on the operator's single AMD box.

⚠ **Every case is a fault the checker must REJECT**, not a restatement of what it accepts: a
one-row shift, a full-band displacement, a short row, a whole-buffer smear, a lone stray pixel, a
pre-contaminated buffer.

⛔ **The first draft of `ae_st_check_bands` sampled band MIDDLES and would have missed a shift of up
to 7 rows** — the sample simply lands elsewhere in the same 16-row band and compares equal. It now
probes the first and last row of every band plus the left and right columns. The titlebar offset is
applied independently in three places that have already disagreed once this arc, so a one-row error
is the realistic one.

⚠ The suite was verified to be capable of failing: neutering `ae_st_scan_margin` to `return 0` turns
it red at exactly the two cases only it covers (`27 passed, 2 failed`). A suite that has never been
seen red is not known to be a gate.

### Changed — the desktop is staged for burn

`agnos/scripts/burn/stage-tools.sh` grows its first desktop row, and `burn-prep.sh` grows the guard
it applies to every other oracle: the flag the operator is told to type must exist in the binary
that gets flashed. ⚠ Absent the flag, `--selftest` falls through to the desktop — success-shaped,
with no verdict.

### Known — still unproven

⚠ **This is a staged binary and a green host suite, not a burn.** Exit 95 has never been observed.
Everything above makes the question *askable* on iron for the first time; it does not answer it.

⚠ `src/apps.cyr` still names `sys_fork`/`sys_dup2`/`sys_execve` for its Command Palette spawn. They
are unreachable on agnos today and DCE drops them, but that is reachability analysis holding the
line, not a guard — and `app_launch_terminal` should route to `sys_spawn_path` #43 there instead.

## [0.10.0] - 2026-08-02

The compositor runs its own code, at the size the screen actually is, and no longer blanks the
desktop when the GPU declines a frame.

### Fixed — ⛔ THE FRAME PATH SHIPPED IN 0.9.8 WAS NEVER CALLED

`render_frame` (`src/render.cyr`) carried the damage model and the `#85` GPU clear. The live loop
called `render_desktop` (`src/desktop.cyr`). Nothing called `render_frame` — every other occurrence
of the name in `src/` was a comment.

So none of 0.9.8's rung-0a work ever executed: `#85 gpu_fill` was never issued, `rend_dmg_new` was
reached only from `tests/render.tcyr`, `ae_frame_dmg` stayed 0 so every damage computation was a
no-op behind its own null guard, and the `#39` chrome blit was **unconditionally full-frame** while
this file described it as damage-limited. Both runtime bisect gates (`/.ae-no-gpu-clear`,
`/.ae-no-damage`) sat behind the dead function and were never stat'd — the one debugging affordance
0.9.8 claimed to ship did not exist at runtime.

There is now **one** frame path. ⚠ The merge direction was forced, not stylistic: `render_desktop`
paints the shell status panel and `render_frame` does not, so pointing the loop at `render_frame`
would have silently dropped the panel. The damage half moved to `rend_frame_damage`, which
`render_desktop` calls.

⚠ **The two paths also disagreed about the theme.** `render_desktop` cleared to `desk_bg_color`
(the a11y HighContrastTheme, via theme_bridge) while the windows drawn on top of it were already
themed from **rupa** — so the desktop void and its windows came from different palettes. rupa wins:
it is the shared token core the chrome, dhancha, crab and jalwa all read. `desk_bg_color` is
retained (the accessibility profile is a real, separate concern) but no longer paints the desktop.

### Fixed — ⛔ THE COMPOSITOR WAS SIZED 1.6× WRONG FOR THE PANEL

`AE_W = 1280; AE_H = 720` was hardcoded and handed straight to `bhumi_backend_open`. The compositor
never called `bhumi_output_query` (`#38 fbinfo`) — `grep` found no use of it anywhere in `src/`.

archaemenid's scanout surface is **800×600, pitch 832 px**: the firmware leaves an 800×600 surface
DCN-scaled up to the 2560×1440 link, and agnos reads the real viewport from the live HUBP register
at boot and prints it in every burn (`console geometry matched to surface 800x600 pitch 3328 bytes`).

⚠ **The failure was silent, which is why it survived.** `#39 blit` **clips** to `fb_width`/`fb_height`
rather than erroring, so a 1280×720 desktop presented onto an 800×600 surface simply shows its
top-left corner. Nothing logs and nothing returns non-zero. The shell panel — which correctly places
its indicators at `fb_width - pad - w` — put every one of them past x=1234, entirely outside the
visible 800 columns. **The panel code was right; it was handed a lie.**

New `ae_query_geometry()` reads fbinfo at startup and falls back to 1280×720 only when the query
answers −1 (every host build), so the Linux dev build is unaffected. The seeded "Files" window and
the setu client cascade are now proportional to the queried surface, and the cascade **wraps** —
the old fixed step put the third client at (690,470), off the bottom of an 800-tall surface.

⚠ **Layout uses fbinfo's width, never `gpu_caps#89`'s.** caps reports `bw = gpu_bb_pitch / 4`, and on
this box the pitch in pixels (832) **exceeds** the visible width (800); laying out to caps would push
chrome into the invisible gutter — the same class of mistake, one register removed.

### Fixed — the CPU fallback was not a fallback: a declined frame showed chrome and no windows

`render_window` skipped its per-pixel client composite whenever `ae_gpu_active()` was 1 — a flag set
once at startup meaning only *"a GPU exists and is armed"*. But `ae_gpu_present_frame` refuses
individual **frames** for reasons the renderer never saw (a window past an edge, a premultiplied
surface with no shader path), and on refusal the caller presented the framebuffer as it stood. The
decision and its consumer lived in different places and read different data.

⛔ **Reachable by ordinary use, not by an edge case.** The client cascade put the third setu client
where `dy + h` ran past the bottom edge, and the kernel **rejects rather than clips** — so one more
client than expected took the entire desktop blank, permanently, with no error anywhere.

Fixed by **reordering, not by adding capability**. New `ae_gpu_frame_plan(comp, fb)` runs every
window through the admissibility rule **before** the frame is rendered, issues no syscall, and
publishes one answer (`ae_gpu_frame_ok`) that both `render_window` and `ae_gpu_present_frame` read.
⚠ All-or-nothing, and that is **forced by z-order**: CPU-composited pixels reach the back buffer
inside the `#39` chrome blit, which lands first, so a GPU-blitted window would paint over a
CPU-composited window that is meant to be above it.

- New `ae_gpu_window_admissible(win, fw, fh)` holds the rule, deliberately **free of `#ifdef`** so it
  is testable on a machine with no GPU instead of by burning a boot on the operator's only box.
- A **staleness guard**, not a second derivation: the setu accept block runs between the plan and the
  present and can push a window the plan never saw and the renderer never drew. A changed window
  count means present on the CPU this frame and re-plan next.
- New `ae_gpu_demote()` — a hard refusal from `#87`/`#92`/`#84` now clears the mode, so a wedged GPU
  degrades to a working software desktop. `ae_gpu_mode` was previously write-once, so a GPU that
  started refusing left the compositor blank **forever** with no way back.

### Fixed — `win_new` never initialised `W_PREMUL`

Every other slot was zeroed and this one was not, and `lib/alloc.cyr` is a bump pointer with no
memset — so a recycled Window reads whatever the previous tenant left. A garbage premul flag routes
an ordinary opaque surface down the `#92` blend path: a wrong picture, with no error anywhere.

### Removed — the `#85 gpu_fill` clear arm, which was wrong by construction

⛔ **Do not re-add it without deleting the `#39` chrome blit in the same change.** `#85` clears
`gpu_bb_a`/`gpu_bb_b`; step 1 of `ae_gpu_present_frame` then blits the whole chrome layer into **the
same rows of the same buffer** (the `#39` target and the `#85` target are selected by the same
`gpu_bb_back` predicate). Every pixel `#85` wrote was overwritten, in the same frame, before anything
was presented.

⚠ **And it cleared the wrong buffer for the CPU half.** `bhumi_fb_clear` targets the *userland*
framebuffer — different memory — so on the GPU arm the userland buffer was never cleared at all and
chrome would accumulate frame over frame. Wiring rung 0a up as written would have shipped that bug,
and the symptom (smearing chrome) reads as a renderer fault.

`#85` becomes correct **and** load-bearing once the chrome layer moves wholly onto the GPU and the
`#39` blit is deleted — clear, then `#88 gpu_fill_rect` the window and panel rects, then `#92` op 0x03
the title text. That is a separate, atomic bite: the `#39` blit is the *bottom* layer of the frame, so
a half-migration has GPU-drawn and CPU-drawn chrome overwriting each other.

### Added — the damage tracker is allocated and computed, and deliberately NOT wired to the blit

`ae_frame_dmg = rend_dmg_new()` at startup, and `rend_frame_damage` runs every frame.

⛔ **Enabling the damage-limited blit is a one-line change that looks obviously correct and is not.**
`#84 present` **flips the render target** on every present (agnos `kernel/core/gpu.cyr`:
`if (gpu_bb_back == 0) { gpu_bb_back = 1; } else { gpu_bb_back = 0; }`), and `#39` renders into
whichever buffer that same predicate selects. A band-limited blit therefore updates buffer A this
frame and B the next, so rows outside this frame's band in A still hold what was there **two frames
ago**. This file unions only the current frame's rects and accumulates nothing across the flip. The
symptom is a trailing ghost at a two-frame lag in exactly the rows a window vacated — which reads as
*"the framebuffer is stale"*, not as *"the damage model is wrong"*. Enabling it requires carrying the
previous frame's damage and blitting `union(cur, prev)`.

### Added — `tests/gpu_fallback.tcyr`, an external-invariant oracle (20 suites green, was 19)

*"The exact 32-bit word a client wrote into its surface is readable at that client's screen
coordinates after a frame, whatever the GPU decided."* Sentinels `0x00C0FFEE` / `0x00BADA55` are
chosen by the test and derivable from no compositor state, so a renderer sharing every one of the
compositor's premises cannot produce them by accident.

⚠ **It was run against the unfixed tree first and it failed** — `got 1777188` (`0x1B21A4`, the theme's
window-body fill) where the client's word belonged. A test that passes before the fix is testing
nothing. Covers the accepted shape, the refused shape, the `bufid == 0` shape, the admissibility rule
directly (both sides of each bound, negative origin, premultiplied with and without a shader path),
`W_PREMUL` initialisation, and demotion.

### Fixed — the "an alpha-0 surface VANISHES under `#92`" claim, asserted in three places, is wrong

`src/window.cyr` said a surface whose byte 3 is 0 is *"FULLY TRANSPARENT under #92 and its window
vanishes"*; setu's `client.cyr` and dhancha's CHANGELOG say the same. The kernel shader is
`out = src + dst*(1 - src_a)`. At `src_a = 0` that is `out = src + dst` — an **additive, over-bright
ghost**, not a disappearance. It is nearly invisible over the near-black desktop void and blows out
to clipped white over lighter chrome, which makes it **harder** to spot than a missing window, not
easier. Anyone hunting a vanished window will not find it. The opt-in rule stands; only the predicted
symptom was wrong. Three documents agreeing was a shared-premise oracle.

### Known — ⛔ the `--agnos` build is BLOCKED by a kavach defect, so this change is host-verified only

`cyrius build --agnos` fails in vendored `lib/kavach.cyr`, at both the pinned 6.4.78 and 6.5.5:
kavach 3.9.3's Firecracker and OCI backends call the **Linux** `sys_unlink(path)` / `sys_rmdir(path)`
(agnos takes `(path, pathlen)`), plus `sys_mount` and a bare `SYS_CHDIR`, with no
`CYRIUS_TARGET_AGNOS` guard anywhere in either file. Since `cyrius build` prepends every `[deps.*]`
module, this breaks the whole consumer, not just the backend.

⚠ **The pin did not hold and nothing said so.** This manifest declares `[deps.kavach] tag = "3.7.0"`
*and* `path = "../kavach"`; the path override wins, so `lib/kavach.cyr` is byte-identical to the local
checkout at **3.9.3**. The build worked on 2026-07-25 and stopped working with no change to this repo.

Filed in both trees as `docs/development/issues/2026-08-01-linux-only-backends-break-every-agnos-consumer.md`.
**Host build and all 20 test suites are green; the agnos build of this change is unverified.**

### Changed — cyrius pin 6.4.78 -> 6.5.5; bhumi 1.1.2, rupa 0.1.2, kashi 1.0.4, setu 0.7.1, kavach 3.10.0

Part of the whole-desktop-stack toolchain catch-up cut on this date — fourteen repos moved together
so the next burn runs binaries built by ONE compiler instead of six different ones.

⚠ **The pin was documentation, not enforcement.** `cyrius build` compiles with the INSTALLED `cycc`,
prints a `toolchain drift` warning, and carries on — so 0.9.8 was already being built by 6.5.5.

⛔ **`[deps.kavach]` declared `tag = "3.7.0"` while also carrying `path = "../kavach"`, and the path
WINS.** The vendored copy silently tracked a sibling checkout, so this repo's `--agnos` build broke
on 2026-07-25 — the day kavach 3.9.1 landed `confine.cyr` — with no change to this repo and nothing
to announce it. The tag now names what is actually consumed. **A pin that is not enforced is worse
than no pin: it is what a reader checks INSTEAD of reality.**

### Why the minor bump rather than a patch

Two removals: `render_frame` (`src/render.cyr`) and `ae_gpu_clear` / `ae_gpu_clear_band`
(`src/gpu.cyr`), plus a behaviour change in what `render_window` gates on. Nothing outside this repo
consumed any of them — the compositor is `publish = false` — but a removal is a removal.

## [0.9.8] - 2026-07-25

3D-arc Phase 0 (`chrome-gpu`): the compositor stops doing per-frame full-screen work on the CPU.

### Added — rung 0a `chrome-clear`: the per-frame clear moves to the GPU

`render_frame` called `bhumi_fb_clear`, a CPU `store32` loop over the **entire framebuffer**, once
per frame — 480,000 stores at the console's 800x600, 921,600 at 1280x720. `#85 gpu_fill` had exposed
a CP-DMA fill for four minor versions and **the compositor never called it.**

New `ae_gpu_clear()` / `ae_gpu_clear_band()` route through `#85` when the probe reports present +
armed + carveout, and fall back to the identical CPU loop otherwise.

⭐ **Why this is rung 0 and not a later optimisation:** rung 9 puts a rasteriser on the GPU. Landing
it into a compositor that still burns its frame budget on a CPU clear plus a full-framebuffer copy
makes the win **invisible and unmeasurable** — you cannot see a GPU rasteriser through a CPU memcpy.

⚠ **The CPU path is retained, not deleted.** It is both the fallback (no GPU, QEMU, an older
kernel) and the **reference** the `#90` byte-compare oracle diffs against.

### Added — a damage model that actually reduces work

Damage is computed from what **changed**, not from where the windows are:

```
damage = union over windows whose rect differs from last frame of { OLD rect, NEW rect }
```

A static desktop damages nothing; a dragged window damages the band it vacated plus the band it now
occupies. Both the clear (`#85` band form) and the chrome copy (`#39`) are limited to that band.
`rend_dmg_*` had existed at `render.cyr:68-105` the whole time and the frame loop never called it.

**Three correctness traps, each of which fails in a way that misdirects:**

⛔ **Both rects are required.** Without the OLD one, a moving window leaves its previous pixels
behind — the vacated region is never cleared and never re-copied. That reads as *"the framebuffer is
stale"* rather than *"the damage model is wrong"*. New `W_PX/W_PY/W_PW/W_PH`, stamped **after** the
window is drawn: stamping at the top of the frame would destroy the "was" before damage is computed
from it.

⚠ **Live client surfaces are damaged without moving.** `render_window` re-reads a `bufid` window's
buffer every frame, so content changes while geometry does not. Treating those as undamaged would
freeze animating windows — and would look like a *client* bug.

⚠ **The first frame forces a full clear.** On frame 1 every window is "changed" (prev rect = -1), so
damage covers only the window rects and the background outside them would never be initialised,
leaving whatever the framebuffer held at boot. The symptom is garbage around the windows on the
first frame only — which gets dismissed as a boot artifact instead of diagnosed.

⛔ **Full-width bands only, in both the clear and the present, and it is an ABI limit not a
shortcut.** `#39` takes its source **tightly packed, w*4 bytes per row**, with no stride
(`agnos syscall.cyr:3794`); CP-DMA fills **contiguous** memory. A damage rect's rows are not
contiguous in the framebuffer; a full-width band's are. True rect support needs a stride on `#39`
and a per-row fill loop for `#85` — `h` packets instead of 1 — and should be measured before being
assumed worthwhile.

### Changed — cyrius pin 6.4.71 → 6.4.78

⛔ **The gap contained a P0 this repo was exposed to.** 6.4.75 fixed *"`fn_table` growth past 8192
silently corrupted six fn-indexed side tables"* — and aethersafha's compiled unit carries **10,492
functions**, well past that threshold. Every build at 6.4.71 was made with those tables corrupted,
silently.

Also in the gap: 6.4.72 added the agnos GPU-band `#90`/`#91` wrappers (this repo still reaches `#91`
by raw `syscall` — follow-up), 6.4.77 fixed 67 reserved intrinsic names misreporting as
`got unknown`, and 6.4.73/78 fixed `cyrius audit` compiling project sources with no stdlib includes.

### Fixed — a VARIABLE forward-reference broke three test files

`render.cyr: undefined variable 'ae_frame_dmg'` in `tests/render.tcyr`, `tests/shell_render.tcyr`
and `tests/foreign.tcyr`. The damage globals were first declared in `gpu.cyr`, which is included
**after** `render.cyr` (`main.cyr:16` vs `:17`). A cyrius **function** forward-reference resolves —
which is why `render_frame` can call `ae_gpu_clear()` defined later, and why `ae_gpu_active()` has
always worked — but a **variable** forward-reference does not. Now declared in `render.cyr` before
use; `gpu.cyr` reads them as a backward reference.

⛔ **The lesson is not "move the variable."** `cyrius build src/main.cyr` was **green** while those
three tests failed to compile: a green umbrella build does **not** prove the modules compile
independently, and the test harness compiles them in units that never reach `gpu.cyr`.

### ⛔ Deferred — rung 0c `chrome-move` (`#91 gpu_blit_bb`)

0c would move a window by copying pixels instead of re-rasterising chrome. It was blocked while
`render_frame` unconditionally cleared and repainted everything every frame — there was no "only a
window moved" signal for `#91` to hook into. **The damage model above supplies that signal**, so 0c
is now unblocked and is the next bite rather than a blocked one.

Tests: **19/19 files, 133/133 assertions.** Host and `--agnos` targets both build clean.

## [0.9.7] - 2026-07-23

### Added — `#92 gpu_shader_op` op 0x01: per-pixel PREMULTIPLIED blending on the shader cores

`src/gpu.cyr` gains `ae_gpu_blend_rect`. A window whose client set `SETU_SURF_PREMULTIPLIED` is now
composited with **#92** (premultiplied src-over on the GPU) instead of **#87** (opaque copy). This is the
operation CP-DMA structurally cannot do: `#87` is a byte mover with no ALU and can only copy, while `#92`
runs a real `out = src + dst * (1 - a/255)` per pixel.

⚠ **Opt-in per surface, gated twice.** A surface is blended only if (a) its client set the flag at the
CREATE_SURFACE handshake, and (b) `gpu_caps` **#89 bit3** says shader compositing is available — the
kernel refuses `#92` with `E_UNPROVEN` unless the dispatch envelope was proven on this boot. Bit3 is a
*separate* capability from bits 0-2, so opaque windows still take the hardware `#87` path when it is clear.

⚠ **A premultiplied surface is never copied opaquely as a fallback.** `#87` ignores byte 3, so a
half-transparent window would paint at full strength and look wrong rather than blended. If the shader path
is unavailable the whole frame drops to the CPU present instead.

⚠ **No cyrius wrapper exists for #92** — the band's wrappers stop at `sys_gpu_caps` (#89) and cyrius is
hands-off, so this uses a raw `syscall(92, ...)` behind the `CYRIUS_TARGET_AGNOS` gate, exactly as agnos's
own `/bin/gpublend` does. On Linux #92 is `chown(path, uid, gid)` and arg1 would be read as a real path —
a metadata write if it ever resolved. The `#ifdef` is the only barrier.

New `W_PREMUL` window slot (`W_SZ` 96 → 104) captures the flag at the handshake.

## [0.9.6] - 2026-07-23

### Added — HARDWARE COMPOSITING on agnos: the first consumer of the ring-3 GPU display band

`src/gpu.cyr`. agnos ships an iron-proven GPU band — `#84 present`, `#85 gpu_fill`, `#86 shm_create_gpu`,
`#87 gpu_blit_shm`, `#88 gpu_fill_rect`, `#89 gpu_caps` — and until this release it had **zero callers
anywhere in the ecosystem**. Roughly fifteen iron burns of proven capability that nothing used.

**What it replaces, per window, per frame:** a `setu_buf_read` #73 full-surface copy kernel→userland, then a
`bhumi_fb_set` loop doing one `load32` + bounds-checked store **per pixel** into the software framebuffer —
after which the whole framebuffer went to the kernel anyway via `blit` #39. `#87` does all of it in-kernel;
the pixels never leave GPU-visible memory and never touch the CPU.

⚠ **The ordering is the design, which is why this is not a drop-in substitution.** `#87` composites into the
**kernel's** back buffer while chrome is drawn into bhumi's **userland** software framebuffer. A correct
frame needs both in the same buffer in the right order:

1. chrome → back buffer via `blit` #39 with **`DEFER_PRESENT` (a4 bit 40)**, so it does *not* flip
2. each client surface → back buffer via `#87`, landing on top of the chrome
3. one `#84 present` to flip the finished frame

Without the defer bit, step 1 auto-presents and every window flashes a chrome-only frame before its content
arrives.

⚠ **The blocker was never alpha.** The published diagnosis blamed the pixel convention (sadish forcing
`0xFF`, bhumi packing `X=0`). Those facts are real but gate `#92`'s *premultiplied* blend only. What actually
stopped `#87` was memory: setu allocated client surfaces with `shm_create` #71 = system RAM, which the agnos
GPU cannot reach at all. setu **0.6.0** fixed that. `#87` is an opaque blit and needs no alpha convention.

**Fallback is not optional and is the tested path.** `ae_gpu_probe()` runs **once** at startup and requires
all three of `PRESENT | ARMED | CARVEOUT` — a GPU with no armed back buffer has nothing to composite into,
and one with no carveout cannot serve the `#86` slots the surfaces live in. Any refusal — no GPU, an
off-screen window (`#87` rejects rather than clips), a failed blit — returns 0 and the untouched CPU path
runs. `ae_gpu_active()` never probes; a renderer must not be the thing that issues a syscall.

⚠ `#84` returns **1 = presented** / 0 = nothing, the one call in this band that does not use the 0-ok
convention. Reading it as `!= 0` cost an iron burn on `/bin/gpublend`; this checks `!= 1`.

**Validated:** host + `--agnos` builds green · 133 + 19 tests pass · `cyrius fmt --check` clean ·
`cyrius lint` 0 warnings · **`agnos/scripts/aethersafha-smoke.sh` PASS** — boots on agnos under QEMU where
there is no AMD GPU, so the probe correctly answers no and the CPU path renders the desktop unchanged
(12 distinct colours in the screendump). **The GPU arm itself is iron-only** — QEMU emulates no AMD GPU, so
nothing here can prove a GPU-composited pixel without a burn.

### Notes

Patch rather than minor deliberately: no public API changed, and on any machine without an AMD GPU the
behaviour is byte-identical. It only diverges on iron.

## [0.9.5] - 2026-07-23

### Changed — setu 0.6.0: client buffers are GPU-visible on agnos

Picks up `setu` **0.6.0**, whose `setu_buf_create` now asks for `shm_create_gpu` **#86** before falling back
to `shm_create` **#71**.

⚠ **Why this matters beyond a version number.** `#71` allocates **system RAM**, which the agnos GPU cannot
reach at all — bus-master is off by design and the engines see only the framebuffer aperture. The kernel
rejects a `#71` slot at both GPU entry points (`gpu_blit_shm` #87: `src_mc == 0 ⇒ the GPU cannot read it`;
`gpu_shader_op` #92: `GPO_E_BADSLOT`). Every shared surface in the desktop was allocated that way, so the
whole iron-proven ring-3 GPU band had **no reachable consumer**. Buffers from this release are eligible for
a hardware blit.

No API change and no call-site change here — the buffer id behaves identically, and `#86` falls back to
`#71` automatically on a machine with no GPU carveout (every QEMU boot).

### Changed — cyrius pin → 6.4.71
### Fixed — the host build was BROKEN before this cut

`cyrius build src/main.cyr` failed with `refusing to emit binary with 1 reachable undefined function(s):
thread_local_alloc`. The symbol exists in cyrius 6.4.71's stdlib but **not** in the materialised `lib/`
snapshot this repo carried from its 6.4.61 pin — the stale-vendored-lib trap. The pin bump plus
`cyrius lib sync --full` fixes it. The compositor now builds again; 133 + 19 tests pass.

### Changed — `[deps.bhumi]` tag 1.0.0 → 1.1.0

Stale: bhumi has been at 1.1.0 (agnos Set-1 keyboard decode) and the local `path` override was masking it,
so CI resolved a different bhumi than local builds did.


## [0.9.4] - 2026-07-12 — theme tokens extracted to the shared rupa lib

The desktop theme system gained its second consumer (dhancha), so the tokens move out of
the compositor into a shared leaf lib — **rupa** (रूप, "form / appearance") — that the
compositor chrome, the widget toolkit, and apps all read. A toolkit/app cannot depend on
the compositor, so the single source of truth has to live below both. Same look, same
MUDRA · Carbon default; only the token *home* moved.

### Changed

- **`src/theme.cyr` slimmed to the compositor-specific packer.** The `AeTheme` struct, the
  four MUDRA / SHANTA grounds, the accessors, the by-name registry, and the single active
  theme all moved to **rupa 0.1.0**. This file now holds only `ae_bhumi(c)` — packing a
  logical rupa `0xRRGGBB` token into the bhumi framebuffer colour via `rupa_color_r/g/b`.
- **`src/render.cyr`** now reads the tokens through `rupa_theme_*` (`rupa_theme_active` /
  `_widget` / `_panel` / `_accent` / `_ink` / `_mute` / `_faint` / `_alert` / `_held` /
  `_bg`); `ae_bhumi` still does the pack. No visual change — MUDRA · Carbon stays the
  out-of-box look.
- **`tests/theme.tcyr`** is now an *integration* test (rupa tokens reachable from the
  compositor + `ae_bhumi` packs correctly, 6 assertions); the 39 token-value assertions
  moved to rupa's own `tests/theme.tcyr`.

### Added

- **`[deps.rupa]`** (`0.1.0`) — the shared theme-token core. Both targets build identically
  (host 15,066,680 B / agnos 14,999,608 B) and the full suite stays green (20 files, 1071
  assertions, 0 failed).

## [0.9.3] - 2026-07-12 — repin to cyrius 6.4.61 (setu-listener accept loop is now leak-free)

The compositor's frame loop polls a **non-blocking `setu_srv_accept`** every frame
(`main.cyr`), and `net.cyr`'s `sock_accept` used to allocate on every would-block poll —
a bump-heap leak that would grow over a long desktop session. **cyrius 6.4.61** fixes it
(`accept(NULL, NULL)` for the never-read peer address + a shared `_net_eagain()` `Err`
singleton for the would-block path, regression-gated by cyrius). This patch repins to
receive the fix; the setu listener's accept loop no longer drips.

### Changed

- **cyrius pin 6.4.34 → 6.4.61** — picks up the `net.cyr` `sock_accept` per-poll
  alloc-leak fix (consumer-filed, fixed upstream). Both targets rebuild identically
  (host 15,066,680 B / agnos 14,999,608 B) and the full test suite stays green
  (20 files, 1104 assertions, 0 failed).
- **Formatting baseline refreshed to 6.4.61.** The newer formatter reformats a handful
  of files that were clean under 6.4.34 (`ai_features`, `gestures`, `security_ui`,
  `shell`, `shell_integration`, `main` + three test files) — mechanical whitespace only,
  no logic change, so CI's fmt gate stays green on the new pin.

## [0.9.2] - 2026-07-12 — the sovereign desktop theme system (MUDRA / SHANTA)

Six divergent aesthetic explorations were consolidated to **two desktop themes**, unified by the
question every one of them was really answering — *how do you make trust visible?* **MUDRA**
(मुद्रा, "the seal") is the sovereign default (signed, radius-0, cyan verify-seal; trust reads
loud); **SHANTA** (शान्त, "stillness") is the focus mode (warm, thin, one gold firefly + sage;
trust reads quiet). Each ships in **dark and light** — four grounds from one token set. The
compositor chrome now reads the active theme instead of hardcoded colours, with MUDRA · Carbon
(dark) as the out-of-box look. Design doc: `agnosticos/docs/development/designs/desktop_consolidated/
theme-system.html`.

### Added

- **`src/theme.cyr` — the AGNOS desktop theme tokens.** An `AeTheme` struct (colours as
  `0xRRGGBB`) with slots `bg / panel / widget / line / ink / mute / faint / accent / alert / held`
  plus `radius` and `font` (permille). Four constructors — `ae_theme_mudra_dark` / `_mudra_light`
  / `_shanta_dark` / `_shanta_light` (exact hexes from the consolidated design) — full accessors,
  the `0xRRGGBB` channel helpers `ae_color_r/g/b`, and the framebuffer packer **`ae_bhumi(c)` →
  `bhumi_xrgb`**. A by-name registry (`ae_theme_by_name`, self-contained `ae_streq`) for config-
  driven selection, and the compositor's single **active theme** (`ae_theme_active` lazy-defaults
  to MUDRA · Carbon; `ae_theme_set_active` / `ae_theme_set_active_name`).
- **`tests/theme.tcyr`** — 39 assertions locking every ground's token values, the channel helpers,
  the registry, and the active-theme selector. All pass.

### Changed

- **`src/render.cyr` chrome now reads the active theme.** The window body, titlebar, title text,
  and decoration buttons — plus a new 2px **accent seal-strip** marking the focused window — and
  the desktop root background all resolve through `ae_theme_active()` / `ae_bhumi()`, replacing the
  hardcoded `bhumi_xrgb(...)` literals. Focused windows raise to `widget` + the accent strip;
  idle windows sit on `panel` with muted title text.
- `src/main.cyr` includes `src/theme.cyr` ahead of `src/render.cyr` (the renderer consumes it).

This is distinct from the legacy `theme_bridge.cyr` (the Rust-parity accessibility
HighContrastTheme → Flutter bridge), which stays for the high-contrast a11y profile. Follow-ups:
dhancha widgets + jalwa consuming the same tokens (the trigger to extract `theme.cyr` to a shared
theme lib), and SHANTA's radius-14 rounding + firefly + luminance-provenance in the renderer.

## [0.9.1] - 2026-07-10 — full key events (press + release) for held-key clients

The compositor now honours setu 0.5.0's `SETU_SURF_FULL_KEYS` opt-in **per surface**: a client
that requests it (via the `CREATE_SURFACE` flags) receives key **press AND release**, so it can
track HELD keys — a game holds a movement key and keeps moving. `setu_srv_forward_key` no longer
drops key-UP for such surfaces (the make/break rides the `mods` arg, 1 = press / 0 = release);
press-only clients (crab, present_probe) are byte-identical to before. Proven with cyrius-doom on
the sovereign desktop — a balanced **10 press / 10 release** over setu (`agnos
scripts/aethersafha-doom-input-smoke.sh`).

> ⛔ **RETRACTED 2026-08-03 — "proven on the sovereign desktop" was a FALSE GREEN.**
> `aethersafha-doom-input-smoke.sh` was one of the `aethersafha-*` smokes that built its kernel with
> `AETHERSAFHA_SETU_SELFTEST=1`, and that hook assigned `net_ip = 0x7F000001` so the setu client's
> SYN source matched its destination and the handshake closed. No ordinary agnos boot could get a
> setu client connected, so the press/release counts prove the *forwarding logic*, not that it ran
> over a real agnos connection. The hook and that smoke script were deleted 2026-08-03; the desktop
> transport is the agnos socket (`anu`), not TCP. See agnos
> `docs/development/planning/ipc.md` §10.

### Changed

- `[deps.setu]` pinned `0.3.1` → **`0.5.0`** (the full-key-events opt-in + `mods` make/break).
- `Window` gains a `W_KEYMODE` field; `setu_srv_recv_committed` captures the requested key mode
  from the `CREATE_SURFACE` flags at the handshake, and `setu_srv_forward_key` gates
  release-forwarding on it (press-only stays the default).

## [0.9.0] - 2026-07-10 — focus over setu + hosting a real dhancha app on the sovereign kernel

The compositor now routes **focus** to clients (not just keystrokes) and hosts a heterogeneous
desktop: the slim present_probe test client next to a real **dhancha widget app**. TAB cycles
focus, the compositor tells each affected client when it gains or loses focus, and the clients
render their own focus state — an interactive window manager on the sovereign kernel.

### Added

- **Focus over setu (`setu_srv_notify_focus`)** — when focus moves, the compositor sends
  `SETU_INPUT_FOCUS(id, 0)` to the window losing focus and `(id, 1)` to the one gaining it, so a
  client can render its own focus indicator (bright vs dim) without a keypress. A per-frame
  focus-change detector in `main.cyr` (compares `comp_focused` to the last-notified index) covers
  both TAB-driven focus and connect-time focus with one path.
- **Heterogeneous clients** — the compositor's second resident is now the **dhancha widget client**
  (`/bin/dhwidget`) rather than a second present_probe, so the desktop hosts a real toolkit app
  (titled window + labelled buttons) alongside the test-pattern client. Verified on agnos via
  `aethersafha-setu-smoke.sh` plus the new `setu-input-test.py` / `setu-focus-test.py` gates.
  > ⛔ **RETRACTED 2026-08-03 — "verified on agnos" here was a FALSE GREEN.** The gate passed only
  > because the `AETHERSAFHA_SETU_SELFTEST` kernel hook assigned `net_ip = 0x7F000001` before
  > launching the compositor, making src == dst so agnos's 4-tuple match succeeded. On an ordinary
  > boot the compositor↔client handshake could not complete, so nothing about the two-client desktop
  > was proven on agnos at this version. The hook, its `build.sh` define, and
  > `aethersafha-setu-smoke.sh` were all deleted 2026-08-03. TCP-on-loopback was the WRONG PRIMITIVE for the desktop transport (it is retired on architectural grounds, not because it could not work) —
  > the replacement is the agnos socket (`anu`). See agnos
  > `docs/development/planning/ipc.md` §10.

## [0.8.2] - 2026-07-09 — MULTI-WINDOW desktop + input routed over setu on the sovereign kernel

The compositor now hosts **multiple** setu clients as distinct windows AND routes keyboard
input to the focused one — the two steps that turn "a client composites" into "a desktop."
Both are proven on agnos via QEMU screendumps (`aethersafha-setu-smoke.sh` green) and a new
input-injection harness (`setu-input-test.py`, USB-xHCI `sendkey`).

> ⛔ **RETRACTED 2026-08-03 — "proven on agnos" here was a FALSE GREEN, and it is the load-bearing
> claim of this entire version.** `aethersafha-setu-smoke.sh` was green only because the
> `AETHERSAFHA_SETU_SELFTEST` kernel hook assigned `net_ip = 0x7F000001` before launching the
> compositor. agnos puts `net_ip` in a SYN's SOURCE, so a client dialling `127.0.0.1` got its
> SYN-ACK addressed to `net_ip`, `tcp_find_conn` never matched, and the connection died — *except*
> under that hook, where src == dst == 127.0.0.1 made the match succeed. Before `net_src_for` (agnos 1.56.34) no ordinary boot could
> complete a setu handshake. ⭐ AFTER that fix it DID connect un-rigged — `aethersafha-clients-test.py`
> reached "connected: 2, presented: 2" on 2026-08-02 (QEMU, `-smp 1`; never on iron; `-smp 4` fault-kills). The multi-window desktop and setu input routing are proven on **Linux
> only** at this version. Hook, `build.sh` define and smoke script all deleted 2026-08-03; TCP is a
> **retired** desktop transport, replaced by the agnos socket (`anu`). See agnos
> `docs/development/planning/ipc.md` §10.

### Added

- **Multi-window desktop** — `main.cyr` spawns TWO setu clients and cascades each accepted
  one to a distinct position (`setu_srv_serve_accepted_at(cfd, comp, fb, 30+n*330, 50+n*210)`,
  an `accepted` counter carried across frames). Screendump shows two independent windows, each
  animating its own shared buffer (distinct per-client bar colours keyed off the shm id).
- **Input over setu (the S→C forwarding leg)** — `setu_srv_forward_key(comp, ev)` sends a
  `SETU_INPUT_KEY(win_id, bhumi_key_usage(ev), 0)` to the FOCUSED window's client for each
  key-DOWN. Verified: injecting a key flips only the focused client (border/bar → white) while
  the unfocused client is untouched — focus-routed, not broadcast.

### Changed

- **The setu client connection is now PERSISTENT.** `Window` gained `W_CFD` (the client's
  tagged connection fd); `setu_srv_serve_accepted_at` stores it (`win_set_cfd`) and `main.cyr`
  no longer closes `cfd` after the present handshake — the compositor keeps it open to forward
  input. After COMMIT the socket is server→client only (compositor writes input, client reads),
  so it never contends with the shared-buffer present path.
- **The compositor focuses each client as it connects** (`comp_focus(comp, comp_count-1)`), so
  forwarded keys route to the most-recently-attached window.

## [0.8.1] - 2026-07-09 — the compositor composites a SHARED-BUFFER present on the sovereign kernel

The setu present goes **shared-buffer** (setu 0.3.1): the server reads a client's pixels
from a buffer referenced by id instead of an inline socket stream — the on-device unblock
(a hundreds-of-KB inline payload deadlocks the single-CPU two-proc path through the 2 KB
`TCP_RX_RING`). `aethersafha-setu-smoke.sh` is now **green on agnos**: a setu client connects,
presents a 320×192 frame, and the compositor composites it (`setu client CONNECTED +
PRESENTED + composited on agnos`). Linux e2e (`setu_serve_probe` + `present_probe`) unchanged.

> ⛔ **RETRACTED 2026-08-03 — this "green on agnos" was a FALSE GREEN, and it is the first one.**
> The smoke passed only because the `AETHERSAFHA_SETU_SELFTEST` kernel hook assigned
> `net_ip = 0x7F000001` before launching the compositor, making the client's SYN source equal its
> destination so agnos's `tcp_find_conn` matched. Before `net_src_for` (agnos 1.56.34), without the hook the handshake could not close,
> so **"a setu client connects … on agnos" was not true on an ordinary boot AT THIS VERSION** (it did
> become true, un-rigged, post-`net_src_for` — QEMU `-smp 1`, 2026-08-02) — every later entry
> that cites this smoke inherits the same defect. The shared-buffer present change itself
> (`setu_buf_read` over inline pixels) stands; the *agnos connection* it was demonstrated over does
> not. Hook, `build.sh` define and smoke script deleted 2026-08-03; TCP-on-loopback as the display
> transport is retired in favour of the agnos socket (`anu`). See agnos
> `docs/development/planning/ipc.md` §10.

### Changed

- **`setu_srv_recv_committed` reads the pixels from the client's shared buffer** — on an
  ATTACH with `buf_id > 0` it `setu_buf_read`s the surface (agnos → the kernel shm band
  `sys_shm_read`#73; Linux → the `/dev/shm` file) instead of `setu_srv_read_exact`ing an inline
  stream. The inline path stays as the `buf_id == 0` fallback.
- **setu dep → 0.3.1** (the shared-buffer protocol + backend); **cyrius pin 6.4.25 → 6.4.34**
  (the native `sys_shm_*` wrappers). Note: aethersafha's materialized stdlib needed a
  `cyrius lib sync --full` on the pin bump — `cyrius deps` alone left it stale.

## [0.8.0] - 2026-07-08 — the compositor speaks setu over TCP (sovereign display protocol, e2e)

The setu server transport goes **cross-platform** (item 3b of the road-to-desktop):
the compositor now accepts + composites real setu clients over **TCP loopback**
(`127.0.0.1 : 7700`) on Linux **and** on agnos, so the sovereign desktop runs on
the sovereign kernel — not just the host. Proven end-to-end: `puka` (client)
connects over TCP and presents a rendered 320×192 terminal frame → this
compositor accepts it (non-blocking poll) and composites it → a valid PPM with
real content. This depends on **setu 0.3.0** (the TCP transport).

> ⛔ **RETRACTED 2026-08-03 — the "and on agnos" half of this claim was not honestly proven AT THIS VERSION.**
> The e2e above ran on **Linux**. On agnos, a client dialling `127.0.0.1` sent a SYN sourced from
> `net_ip`, so its SYN-ACK came back addressed to `net_ip`, `tcp_find_conn` found no match, and the
> connection died — the handshake could only close under the `AETHERSAFHA_SETU_SELFTEST` kernel
> hook, which assigned `net_ip = 0x7F000001` and made src == dst. That hook is the rigging behind
> every "green on agnos" from 0.8.1 through 0.9.x, and it was deleted 2026-08-03 along with its
> `build.sh` define and the smoke scripts built on it. **TCP-on-loopback was the wrong primitive for
> a local display protocol and is retired**; the sovereign desktop transport is the agnos socket
> (`anu`). See agnos `docs/development/planning/ipc.md` §10.

### Added

- **`programs/setu_serve_probe.cyr`** — a fork-free e2e proof of the TCP
  transport: stands up the setu listener, polls the non-blocking accept until a
  client connects, serves one present (recv surface + blit), and dumps the
  composited window to a PPM. Run against `puka`'s `puka_setu_probe` as two
  separate processes — decoupling the transport proof from `fork`+`execve`
  (which a restricted dev sandbox can't host). Complements the fork-based
  `puka_launch_probe` (compositor-spawns-client), which now documents that
  limitation.

### Changed

- **`src/setu_server.cyr` delegates to setu's cross-platform transport.** The
  server half (`setu_srv_listen` / `setu_srv_accept` / `setu_srv_read_*` /
  `setu_srv_write_frame`) now forwards to setu's `setu_listen` / `setu_accept` /
  `setu_read_*` / `setu_send`, which speak TCP on both targets and absorb agnos's
  non-blocking-recv / partial-send quirks. The earlier AF_UNIX path (Linux-only,
  fail-closed on agnos) is gone — the compositor speaks the same wire on Linux
  AND agnos.
- **`src/main.cyr`** — the setu accept block composites clients over the TCP
  listener (`sock_close`, no socket-file unlink); the log line reflects
  `TCP loopback:7700`.

### Fixed

- **Linux crash in the setu accept poll (`sys_sleep_ms` is agnos-only).** The
  would-block yield in `setu_srv_accept_one` and the launch probe called the raw
  `sys_sleep_ms` (defined only in the agnos syscall module), which compiled on
  Linux but trapped (SIGILL) at runtime. Swapped to the portable `sleep_ms`
  (chrono) — `poll()` on Linux, `#41` on agnos. This unblocked the e2e proof.

## [0.7.0] - 2026-07-08 — renderer decoupled from the shell (reusable window chrome)

### Changed

- **`src/render.cyr` is now free of `shell.cyr` symbols — the core window
  renderer is reusable on its own.** The shell status-panel renderer
  (`render_shell_panel` + `panel_bar_w` / `panel_net_color` + the `PanelK`
  metrics), which coupled to `shell.cyr`'s `SystemStatus` + `SH_NetStatus`,
  moved out of `render.cyr` into a new **`src/shell_render.cyr`** — the one place
  the desktop shell's data model meets the framebuffer (the shell → render
  bridge). `render.cyr` now holds only shell-agnostic primitives: `fill_rect`,
  `rend_blend`, the damage tracker, `deco_*` decoration hit-testing, `render_window`
  / `render_frame`, and the kashi bitmap-text `draw_char` / `draw_text` /
  `draw_text_lines`. `main.cyr` gains the `shell_render.cyr` include;
  `desktop.cyr`'s `render_desktop` is unchanged. Pure module split — behavior is
  identical.
- **`tests/render.tcyr` no longer includes `shell.cyr`** — proving `render.cyr`
  stands alone. The panel bar-graph assertions moved to the new
  **`tests/shell_render.tcyr`**, which adds `panel_net_color` mapping checks and a
  `render_shell_panel` shell → framebuffer smoke test. **18 `.tcyr` suites green.**
- **`programs/puka_desktop_probe.cyr` now reuses `render_window` + `draw_text`**
  for native window chrome. Because `render.cyr` is decoupled, the probe includes
  it directly and frames each hosted setu client (puka, dhancha) with the real
  compositor titlebar — focus tint, traffic-light buttons, **and bitmap title
  TEXT** ("puka - terminal", "dhancha - files"). The previous inlined, fill-only
  `pdp_titlebar` (managed windows without title text) is gone. Verified end-to-end
  on Linux: two managed, **titled** windows composited to a PPM.

## [0.6.0] - 2026-07-08 — native display protocol (setu server + frame-loop integration)

### Added

- **The setu server — aethersafha serves the native display protocol, wired into
  the real frame loop.** The full compositor side of the sovereign
  dhancha ↔ aethersafha wire (`setu`), built incrementally and proven end-to-end
  on Linux:
  - **`src/setu_server.cyr`** — the setu socket transport (AF_UNIX; fail-closes
    on agnos): `setu_srv_listen` / `accept` / `setu_srv_read_msg`
    (length-from-header framing) / `setu_srv_read_exact` / `setu_srv_write_frame`.
  - **`src/setu_dispatch.cyr`** — message → compositor state: `CREATE_SURFACE` →
    a real `comp_create_window` `Window` + `SURFACE_CREATED` reply; `ATTACH` +
    inline BGRA payload + `COMMIT` received byte-exact and blitted into the bhumi
    framebuffer, the framebuffer region verified against the source
    (`setu_srv_serve_present` / `serve_session` / `serve_accepted`).
  - **`src/main.cyr`** — the real compositor frame loop now stands up a
    **non-blocking** setu listener and, each frame, accepts + composites client
    presents on top of its desktop. `AF_UNIX` fail-closes on agnos, so the setu
    block is skipped there and the loop keeps its bounded agnos form untouched.
  - Adds a dependency on the new **`setu` 0.1.0** contract lib, and blits client
    surfaces into the bhumi framebuffer.
  - Proven on Linux: `build/aethersafha` accepting a real dhancha `DhClient`,
    minting its `Window`, and compositing its rendered widget tree — the display
    slice end-to-end (the final kernel-scanout hop, `bhumi_backend_present` →
    `fbinfo#38`/`blit#39`, is agnos-only).

### Changed

- **Direction: Wayland refused — the native protocol is sovereign, not a port.**
  aethersafha's client↔compositor protocol is redefined as a native,
  first-principles display protocol; the Rust `wayland/` surface (~3360 lines) is
  **retired, not ported**, and "Bite F" is redefined from "reimplement Wayland in
  Cyrius" to "design + build the native protocol." See
  [`docs/adr/0001-native-display-protocol.md`](docs/adr/0001-native-display-protocol.md)
  and the ecosystem pivot in `agnosticos/docs/design-patterns.md`. Docs
  (`README`, `docs/architecture/overview.md`, `roadmap.md`, `parity-plan.md`,
  `state.md`) reconciled to the native direction. No code change.

## [0.5.0] - 2026-07-03 — built-in apps (Bite C1 + C2)

### Added

- **Built-in apps (`src/apps.cyr`, Bite C1 + C2)** — port of the app framework, data-model
  apps, and the process-spawn bodies from Rust `apps.rs` (2986 lines).
  - **C1** — `AppError` / `AppType` / `AppWindow`, the **`DesktopApplications`** aggregate
    (open / close / list windows, live sub-app getters), the data-model apps
    **FileManager** / **AgentManager** / **AuditViewer** / **ModelManager**, the 8
    **WebBrowser** factory configs, **Shruti**, and the **Terminal** security surface — the
    30-program allowlist + `Path::file_name()`-faithful **basename-strip** (path-traversal
    neutralisation) + `split_whitespace` tokenizer.
  - **C2** — the real **process spawn**: `Terminal.execute_command` resolves the allowlisted
    bare name via a PATH search, fork+execve's it (direct + unsandboxed like Rust — the
    allowlist is the security control), captures stdout, and maps the real `WEXITSTATUS` to
    `Ok(stdout)` / `WindowError`; `WebBrowser` / `Shruti` `launch` guard on `is_installed`
    then detached-spawn with a merged (inherited + override) environment.
  The filesystem / network effect bodies (`list_agents`, `get_logs`, the model gateway) remain
  stubbed to their clean-env fallback, deferred to **C3**. `tests/apps.tcyr`
  (**133 assertions** — incl. real `echo` / `true` / `false` execution + launch guards; all
  green). Standalone; compositor wiring is follow-on.

### Changed

- **Toolchain `6.3.42` → `6.3.43`** — matches the installed `cycc`; refreshes the vendored
  `lib/` snapshot.

## [0.4.2] - 2026-07-03 — screen capture + recording (Bite D)

### Added

- **Screen capture (`src/screen_capture.cyr`, Bite D1)** — port of Rust
  `screen_capture.rs`: a `ScreenCaptureManager` with a per-agent **permission model**
  (grant/revoke/list/get, allowed-target kinds, expiry), **sliding-window rate
  limiting** (per-permission `max_captures_per_minute`), **secure-mode** +
  system-vs-agent authorization, a **capture-history ring buffer** (cap 100), and
  full-screen / **region** (clamped, saturating) / **window** capture off a
  caller-supplied bhumi framebuffer. Includes byte-exact **RAW / BMP / PNG encoders**
  (hand-rolled Adler-32 + CRC-32 + zlib STORED deflate). `tests/screen_capture.tcyr`
  mirrors the Rust unit tests (**90 assertions**, all green). Not yet wired into the
  compositor surface (follow-on, like the M2 leaf modules).
- **Screen recording (`src/screen_recording.cyr`, Bite D2)** — port of Rust
  `screen_recording.rs`, built on D1: a `ScreenRecordingManager` with recording
  **sessions** (config: target / format / frame-interval / `max_frames` / `max_duration`),
  a start → capture-frame → pause/resume → stop **state machine**, a per-session frame
  **ring buffer** (cap 100; `frame_count`/`total_bytes` count all frames ever), and
  one-recording-per-agent enforcement. `capture_frame` delegates to D1's
  `scap_mgr_capture` and wraps the result as a `RecordedFrame`; `max_frames` /
  `max_duration` are hard pre-capture limits and caps use `-1 == None` (so `Some(0)` is
  distinct). `tests/screen_recording.tcyr` mirrors the 22 Rust unit tests
  (**72 assertions**, all green). Standalone; **Bite D (capture + recording) complete**.

### Changed

- **mehman `0.3.1` → `1.0.0`** — API-compatible for the consumed
  `types`/`surface`/`sandbox` modules (the 1.0.0 delta only *adds* mehman's per-ABI
  `guest`/`shim` modules). Foreign capture + presentation unchanged and still green.
- **Toolchain `6.3.40` → `6.3.42`** — matches the installed `cycc`; refreshes the
  vendored `lib/` snapshot (sankoch 2.4.9).

## [0.4.1] - 2026-07-03 — foreign guest surface presentation

### Added

- **Foreign windows now show their guest's output** — the mehman "swallow" loop is
  visible end to end. A hosted guest's captured stdout is presented as the window's
  content: new `render_foreign_content` / `render_desktop_foreign` (`src/foreign.cyr`)
  paint the captured surface buffer into the window body via a new line-aware
  `draw_text_lines` (`src/render.cyr`, honors `\n`). The `desktop` aggregate now
  tracks its hosted foreign apps (`DSK_FOREIGN` vec + `desk_foreign` /
  `desk_foreign_count`), and the `main` frame loop presents them each frame
  (`render_desktop_foreign` after `render_desktop`). `main` hosts + runs a live
  `/bin/echo` guest so its window shows real output. Stdout-as-framebuffer MVP
  (mehman ADR 0004); real XRGB pixel fidelity is the next step. `tests/foreign.tcyr`
  gains a presentation group with a pixel-level assertion that the captured text
  paints the window body (**23 assertions**, all green).

### Changed

- **Foreign guests are captured, not just run.** `foreign_run` uses
  `mehman_sandbox_capture_guest(guest, surface, &exit)` — it runs the guest in a
  kavach PROCESS sandbox (real fork+exec+reap) **and captures its stdout into the
  hosted window's surface buffer** (the M2 handoff).
- **mehman `0.2.1` → `0.3.1`** (0.4.0 shipped 0.2.1; `0.3.0` added the capture API,
  `0.3.1` is the current pin); **toolchain `6.3.39` → `6.3.40`.**
- **`scripts/version-bump.sh`** rewritten for the Cyrius layout — bumps `VERSION` +
  `cyrius.cyml [package].version` with a post-write self-check (the stale Rust-era script
  targeted a nonexistent root `Cargo.toml`, ran `cargo check`, and never touched
  `cyrius.cyml`, so it would crash mid-run and leave the manifest un-bumped).

## [0.4.0] - 2026-07-03 — mehman foreign-app hosting

Wires in **mehman** (the XWayland-successor "swallow" backend) as a real dependency —
foreign-app hosting *and* execution, end to end.

### Added

- **mehman foreign-app hosting + execution.** `src/foreign.cyr` — the "swallow" seam
  onto **mehman** 0.2.1 + **kavach** 3.6.0: builds a sandboxed-guest spec (swallow
  caps) + an XRGB8888 foreign-surface descriptor; `desktop_host_foreign` registers a
  compositor window backed by it; and **`foreign_run` executes the guest in a kavach
  PROCESS sandbox** (real fork+exec+reap, returns `MehmanError.OK`). `main` hosts a
  demo foreign `xterm` and runs a benign guest. Required declaring the full TLS/crypto
  stdlib cascade for kavach→sandhi (net/sandhi/thread_local/tls*/sha1/keccak/sigil/
  sakshi/…) + `[deps.kavach]` explicitly. `tests/foreign.tcyr` (11 assertions, incl.
  live guest execution). Toolchain pin → 6.3.39.

## [0.3.0] - 2026-07-03 — kashi fonts + desktop wiring

Adds the `kashi` font dependency (real bitmap text) and completes the B3 desktop
wiring (shell status panel, theme-driven background).

### Added

- **Bitmap text** — `draw_char`/`draw_text`/`text_width` in the renderer, backed by
  the **kashi** font subsystem (`[deps.kashi]` 1.0.2, freestanding `font_data.cyr`
  core — IBM VGA 8×16 glyphs). **Window titles now render** in their titlebars.
  Pixel-level test verifies glyph blitting. (Replaced an initial hand-rolled 5×7 font.)
- **B3 wiring completed** — a **shell status panel** rendered from the desktop shell
  (cpu/mem/battery bar-graphs, net-status dot, notification badge; `render_shell_panel`
  + pure `panel_bar_w`), and **theme → renderer** (`render_desktop` clears to the
  theme's high-contrast background via `desk_bg_color`, then paints windows + panel).
  All 8 leaf subsystems are now wired into the running frame.

## [0.2.0] - 2026-07-03 — parity milestone

Compositor + renderer depth, the full M2 leaf-module set, B3 wiring (desktop
aggregate), Bite A window interaction (decorations + input routing), and the
sovereign-dependency de-collision (agnostik/agnodrm/aegis) landed on top of the
0.1.0 port.

### Added

- Compositor depth: workspaces + context types, move-window-to-workspace,
  switch-workspace, secure + agent-aware modes, window-at-point hit-testing.
- Renderer depth: alpha blend (`rend_blend`) + damage tracking (bounding-box
  `DamageTracker`).
- Ported `shell_integration` + `plugin_host` — completes all 8 M2 leaf modules.
- Behavioral parity test suites for all 8 leaf modules (~670 assertions, all green).
- **B3 wiring**: a `desktop` aggregate owns the compositor + all 8 leaf managers and
  is instantiated by `main`, so the subsystems are reachable + running. First live
  cross-subsystem connection — compositor → accessibility
  (`desktop_sync_accessibility` mirrors the window stack into the a11y tree).
  `tests/desktop.tcyr` (14 assertions).
- **Bite A**: window **decorations** — close/maximize/minimize titlebar buttons +
  `deco_hit` decoration hit-test (body/titlebar/buttons/resize edges). **Input
  routing** — window-management keyboard shortcuts (Tab focus-cycle, F4 close, F5
  maximize-toggle, F6 minimize) via a pure `input_map` + `input_apply`, wired into
  the frame loop. `tests/render.tcyr` (22) + `tests/input.tcyr` (13).

### Changed

- **Toolchain 6.3.36 → 6.3.38; bhumi 0.7.0 → 1.0.0** (API-compatible bump).
- **Dependency de-collision + re-enable.** agnostik + agnodrm namespaced their error
  families (`STIK_ERR_*` / `DRM_ERR_*`) to end the `ERR_*` symbol collision — cut as
  **agnostik 1.3.3** + **agnodrm 1.4.5** — and are now active deps (reviewed stdlib:
  `+trait`, `+ct`). Active deps: **bhumi, agnostik, agnodrm**. The one downstream
  consumer, `aegis`, was migrated to the new names + cut as 1.1.3.
- **mehman deferred to Bite G.** Cyrius stdlib is opt-in (declare what each dep needs);
  reviewing mehman showed its `[deps.kavach]` → sandhi → the full `tls_native` TLS
  stack, too large a surface for a types-only, unused dep. Re-enable when the
  compositor actually hosts guests.

## [0.1.0] - 2026-07-02 — Cyrius port

First release of the Cyrius incarnation of aethersafha. The prior Rust crate is
frozen at `rust-old/` as the parity oracle (see the pre-port baseline below).

### Changed

- **Ported Rust → Cyrius via `cyrius port`.** The 27,207-line Rust tree moved to
  `rust-old/` (frozen parity oracle); the project is now a Cyrius crate pinned to
  toolchain 6.3.35 in `cyrius.cyml`.
- **Replaced the Wayland platform stack with sovereign AGNOS backends**: bhumi
  0.7.0 (platform I/O — DRM/KMS + libinput + logind → `output`/`input`/`seat`),
  mehman 0.1.0 (foreign-surface hosting — the XWayland successor, post-MVP).
  Native Wayland protocol dispatch stays in aethersafha proper.
- **Dependency mapping**: `agnostik` → agnostik 1.3.2 (Cyrius); `agnosys` →
  agnodrm 1.4.4 (agnosys decomposed 2026-06-19, device model → agnodrm);
  GPU (`mabda`) deferred — software renderer is the v1.0 path.

### Added

- Foundational compositor base on the bhumi seam: `geom`, `window`, `compositor`,
  `render` (software renderer over the bhumi XRGB framebuffer), `input`
  (bhumi HID → actions), and a `main` frame loop. Compiles + runs.
- M2 leaf-module parity batch (structural parity vs `rust-old/`, prefixed symbols,
  compiling + smoke-tested): `theme_bridge`, `gestures`, `accessibility`,
  `ai_features`, `shell`, `security_ui`.
- `tests/aethersafha.tcyr` (21 core assertions) + `tests/leaf_modules.tcyr`
  (11 leaf-coexistence assertions) — 32 green.
- Parity roadmap (`docs/development/roadmap.md`) mapping every Rust module to its
  Cyrius target, backend binding, and milestone (M1–M5).
- Toolchain pin advanced to 6.3.36.

### Notes

- Structural parity for the M2 leaf batch (compiles + smoke-tested); deeper
  behavioral parity tests against `rust-old/` are the next increment.
- Known: agnostik + agnodrm both bundle the shared `ERR_*` module → benign
  duplicate-symbol warnings ("last wins"). See roadmap "Known cleanup".

## Pre-port Rust baseline - 2026-04-01

_Not a Cyrius release — the extracted Rust crate that the 0.1.0 Cyrius port
targets for parity. Source frozen at `rust-old/`._

- Initial extraction from `agnosticos/userland/desktop-environment/`
- Wayland compositor with backend abstraction (`WaylandBackend`)
- AI desktop features — context-aware suggestions, agent HUD
- Desktop shell — app launcher, notifications, quick settings
- Renderer — scene graph, damage tracking, decorations, high-contrast
- Accessibility — AccessibilityTree, tab navigation, announcements
- Plugin host — sandboxed plugin loading, capability system
- XWayland manager — surface mapping, property translation
- Shell integration — tray, window management, notification bridge
- Theme bridge — AGNOS→Flutter ThemeData, platform channel
- Desktop applications — web browser, file manager, terminal, model manager
- Screen capture — per-agent permissions, rate limiting, PNG/BMP/raw encoding
- Screen recording — frame-by-frame, poll-based streaming, ring buffer
- Security UI — permission dialogs, threat alerts, agent dashboard
- Gesture recognition system
- HUD overlays — crew status, domain filter, GPU status
- Criterion benchmarks for compositor and screen capture
