---
name: AGNOS Desktop Arc
description: The single source of truth for the sovereign desktop — the compositor ladder, what is proven on which substrate, open blockers, and the falsified list.
type: planning
---

# The AGNOS Desktop Arc

> **This is THE desktop document.** It is to the desktop what
> [`agnos/docs/development/planning/gpu.md`](https://github.com/MacCracken/agnos/blob/main/docs/development/planning/gpu.md)
> is to the GPU: one file, exhaustive, no sibling arc docs. Before this existed the arc's live state
> lived only as prose in `CHANGELOG.md` and comments in `src/gpu.cyr` — and `src/gpu.cyr:105` appealed
> to *"The plan"*, which was not on disk. Opened 2026-08-02.
>
> ⛔ **"Rung 9" is ambiguous and must always be qualified.** It means *future compositor work* here and
> *shipped-at-1.56.17* in agnos. Compositor rungs in this file carry the `AE-` prefix; kernel band
> numbers keep their `#NN` syscall form.

**Last refresh** 2026-08-07 — ⭐⭐⭐ **BURNED AND PASSED on archaemenid (AGNOS 1.56.41, `smp: cpus online: 4`).** The `#97` channel band runs
the desktop on real silicon: two clients minted/endowed/spawned already connected, no listener, 278 frames, clean Esc. **`AE-T2` is
iron-proven** — a live agnsh answered in a composited window, `h-e-l-p-Enter` = 5 keys with none lost. `AE-0a` showed no ghosting.
⇒ [`iron-nuc-zen-log.md#tracker-15641-desktop-band`](https://github.com/MacCracken/agnosticos/blob/main/docs/development/iron-nuc-zen-log.md#tracker-15641-desktop-band).
⚠ That burn also found the **ONLCR** gap; the fix was **re-flashed and confirmed on iron the same day** — the terminal now wraps correctly.
Earlier: the boot-console half closed on iron (1.56.36/37/38); 1.56.39 landed three kernel items.

---

## 1. State — measured, dated, and attributed to a substrate

Three substrates, and they answer different questions. Confusing them has cost this arc real burns.

| Substrate | What it can prove | What it is structurally blind to |
|---|---|---|
| **Host (Linux)** | protocol logic, layout maths, the render pipeline, every unit suite | anything target-armed (`#ifdef CYRIUS_TARGET_AGNOS`), the kernel ABI, scheduling |
| **QEMU (agnos)** | `spawn_path`, setu, scheduling, two-proc concurrency, the CPU composite path, SMP races | **the GPU, entirely** — no AMD PCI device ⇒ `gpu_find` never matches ⇒ `gpu_present` 0 ⇒ `ae_gpu_probe` answers 0, every GPU branch dead for the whole run |
| **Iron (archaemenid)** | the GPU band, real scanout, real xHCI input, real timing | nothing is cheap here; every question answerable in QEMU must be answered there first |

### Proven on IRON (archaemenid, AMD Cezanne gfx90c / DCN2.1) — 2026-08-02

- ⭐ **`run /bin/aethersafha --selftest` → `run: exit 95`.** The first execution of this repo's GPU
  path on real silicon, ever. `#86` GPU-visible slot → `#87 gpu_blit_shm` → `#90` **pre-flip**
  readback → `#73` to userland: the client's sentinel bands intact at the client's screen
  coordinates, margin ring clean, far frame clean.
- **The bare desktop rendered on the panel** (photo): MUDRA chrome, kashi titlebar text, cyan focus
  strip, traffic lights.
- **Geometry is read from the kernel, not guessed** — `aethersafha: screen size read from the kernel
  / 800 / 600`. bhumi **1.1.3** works on hardware.
- **The quit path was always correct.** `usage 41` (0x29 = Esc) at `frame 113` — the operator left
  after 113 rendered frames. Retired a suspected defect for the cost of one `println`.

⚠ Scope of the GPU proof: `#87`/`#90`/`#84` are iron-proven **for a single opaque client surface**.
`#92` (premultiplied) is still unexercised on iron.

### ⭐⭐ Proven on IRON at 4 CPUs — 2026-08-03 — the desktop hosts real client windows

**`aethersafha` from the agnsh prompt on archaemenid, with `smp: cpus online: 4`.** Both clients
connected AND presented; **the panel shows both windows** — `setu-surface` (green-bordered grid with its
red animation band) and `crab`'s dual-pane file manager. 278 frames, clean Esc quit (`usage 41` at frame
277), `frame loop ok`. Readout: serial via `klug` **plus** a panel photograph.

⭐ **crab did real work.** ~65 `stat` calls across `/bin` and `/` with correct sizes (`owl 15213040`,
`DOOM1.WAD 4196020`, `hello.txt 56`), rendered into both panes with a live selection highlight, and
**keys reached the client** (`crab: key received` ×6). A file manager running as a window on agnos.

This was unblocked by the `EFER.NXE` fix below — every earlier desktop result on this box was
single-CPU *in effect*, because any proc scheduled onto an AP died on its first NX-page touch.
⚠ Scope: this is the **CPU** blit path (scanout 800×600 on a 2560×1440 panel — hence the upper-left
quadrant). The GPU composite path is iron-proven only from the 1.56.34 `--selftest` burn. The setu/TCP
transport it rode is still **retired** as a wrong premise; it working here is consistent with that
ruling, not a reprieve from it.
→ [`agnosticos/.../iron-nuc-zen-log.md#tracker-15635-desktop-smp`](https://github.com/MacCracken/agnosticos/blob/main/docs/development/iron-nuc-zen-log.md#tracker-15635-desktop-smp)

### ⭐⭐⭐ Proven on IRON — 2026-08-07 — the CHANNEL BAND and a SHELL YOU CAN TYPE INTO

**AGNOS 1.56.41 bare on archaemenid, `smp: cpus online: 4`.** Card + full falsification table:
[`iron-nuc-zen-log.md#tracker-15641-desktop-band`](https://github.com/MacCracken/agnosticos/blob/main/docs/development/iron-nuc-zen-log.md#tracker-15641-desktop-band).

- ⭐⭐ **The `#97` band carries the desktop on hardware.** `client spawned on a placed channel` ×2 (fds 5
  and 7), both `setu client presented surface`, **no listener anywhere**. 278 frames, 2 apps, clean Esc at
  frame 277, `frame loop ok`. ⭐ The retired TCP path's 2026-08-03 numbers were *also* 278 frames and 2
  clients — the band reproduced them exactly, on the same box, with the transport deleted.
- ⭐⭐ **`AE-T2` — the shell answered.** The full input ladder printed in order, then `puka: key received`
  **×5** (`h-e-l-p-Enter`) and `puka: line sent to the shell`; the panel shows agnsh's help with a live
  `[ASSIST] >` prompt.
- ⭐ **The key-sampling defect does NOT reproduce on iron.** QEMU lost 5 of 9 keys at a ~100 ms hold (the
  xHCI HID ring is drained once per compositor frame); on iron **19 of 19** arrived at human typing speed.
  A 6.40 ms GPU frame polls fast enough. ⚠ Still real for any slow frame.
- ✅ **`AE-0a` — no ghosting** across 278 frames, on the only substrate that could show one. ⚠ An absence,
  observed once, by eye, with **static** windows; a moving window is the ghosting case and `AE-7` blocks it.
- ⛔ **And it found a bug two QEMU gates could not see** — the missing ONLCR, §4. Fixed and **re-flashed the
  same day: the terminal wraps correctly on the panel.** ⇒ `AE-T2` is iron-proven in both directions.

### Proven in QEMU — 2026-08-02, **all at `-smp 1`**

- **`aethersafha --clients` → exit 95**: both clients connect and present — setu's slim
  `present_probe` (staged as `/bin/puka`) and the real dhancha `crab`, composited as windows.
- Foreground launch from the agnsh prompt reaches 2/2 under agnoshi's `spawn_path #43` + waitpid
  routing.
- The **armed** control (`AE_CLIENTS_MODE=armed`): on the unfixed kernel the sequence kills the
  machine; on the fixed kernel it reaches 2 connected + presented.

⚠ **`AE_CLIENTS_SMP` still defaults to `"1"`** (`agnos/scripts/harness/aethersafha-clients-test.py:114`),
so any QEMU proof taken *before* 2026-08-03 was silently single-CPU. **Pass `AE_CLIENTS_SMP=4`** — it
passes now, and 4 is what archaemenid runs. (Resolved as a blocker; retained as a harness gotcha.)

### ⭐ FIXED 2026-08-03 — `-smp 4` now passes

`run /bin/aethersafha` was fault-killed (`run: exit 142` = 128 + vector 14) at `-smp 4` while passing at
`-smp 1`. **Root cause: the APs never enabled `EFER.NXE`.** The AP trampoline set `EFER |= 0x100` (LME
only); the BSP sets `0x900` (LME | NXE). Without NXE, bit 63 of a paging-structure entry is RESERVED —
and `proc_map_page_nx` sets it on every W^X data page and every user stack, so the first NX-page touch
on an AP took a reserved-bit `#PF`. Code pages (`proc_map_page`, no NX) were always legal, which is why
the faulting address was always a stack or data page. One-line fix, `agnos/kernel/arch/x86_64/smp.cyr:514`.

**Now measured at `-smp 4`: connected 2, presented 2, `exit 95`** — foreground and background — with
`-smp 1` unchanged. ⚠ This means every prior "QEMU-proven at `-smp 1`" caveat in this document can be
re-tested at 4 CPUs, which is what archaemenid actually runs.
→ [`agnos/docs/development/issues/2026-08-02-large-image-ptload-pde-absent-smp.md`](https://github.com/MacCracken/agnos/blob/main/docs/development/issues/2026-08-02-large-image-ptload-pde-absent-smp.md)

### ⭐ Frame cost — measured 2026-08-05, and the "a little slow" carry-forward is CLOSED

The operator's *"aethersafha is a little slow"* came from the **1.56.36** burn — the same burn that
took the panel from 800x600 to native **2560x1440**, i.e. **7.68x the pixels**. One burn apart, and
that is the whole diagnosis: the question was never "is the compositor slow", it was "which per-pixel
loop grew". Answerable on the HOST, no burn needed — the clear and the chrome fills are portable.

⛔ **The pixel COUNT was not the interesting number. The per-pixel COST was.** At 2560x1440, before:

| | pixels | time | per pixel |
|---|---|---|---|
| `bhumi_fb_clear`, whole screen | 3,686,400 | 3.83 ms | **1.03 ns** |
| chrome, 2 windows (`fill_rect`) | 1,843,200 | 23.45 ms | **12.72 ns** |

Half the pixels, six times the time — **12.3x more expensive per pixel for byte-identical stores**.
All overhead, no memory: each pixel took a function call, four bounds compares and `_bhumi_fb_addr`,
which together re-read **seven** `load64`s from the framebuffer header to store four bytes. None of
those seven can change during a fill. `fill_rect` now clamps once and walks rows; new `blit_rect`
does the same for the client-surface copy.

| at 2560x1440 | before | after | |
|---|---|---|---|
| chrome, 2 windows | 23.45 ms | **2.57 ms** | **9.1x** |
| CPU surface copy, 2 windows | 23.97 ms | **4.18 ms** | **5.7x** |
| **GPU frame** (clear + chrome) | **27.24 ms** | **6.40 ms** | **4.3x** |
| **CPU frame** (GPU refused / every QEMU run) | **51.21 ms** | **10.58 ms** | **4.8x** |

Bench: `tests/aethersafha.bcyr`. Correctness: 21/21 suites, 56/56 in `render.tcyr` (direct clipping
coverage added, with a negative control proving it detects a missing source-offset advance), and on
agnos at `-smp 4` both launch paths reach 2/2 exit 95 while the `desktop`-mode framebuffer is
**pixel-count identical** to the pre-change run — the same picture, 4x faster.

⚠ **What this does NOT fix:** the clear, now **60% of a GPU frame**. That is `AE-0a`'s job.

### Build state — measured 2026-08-07

Versions: **agnos 1.56.41** (open, empty) · **aethersafha 0.12.3** · **setu 0.8.4** (kernel floor
agnos ≥ 1.56.40, **enforced by `CH_CAPS`, not documented** — a client built from it refuses on an older
kernel rather than appearing to work) · **crab 0.4.5** · **bhumi 1.1.3** · **puka 0.6.10** · **mishran 0.5.4**.

| Binary | `--agnos` | Size |
|---|---|---|
| `aethersafha` (`src/main.cyr`) | ✅ | 16,133,912 B (15,592,080 B at 0.12.0; the full sigil link dominates — [[reference_sigil_link_explains_binary_bloat]], expected, not a regression) |
| `crab` (`crab/src/main.cyr`) | ✅ | 348,064 B |
| `present_probe` (`setu/programs/present_probe.cyr`, staged as `/bin/puka` in the **shared** rootfs) | ✅ | 110,312 B |
| **real `puka`** — ⭐ **NO LONGER BLOCKED** | ✅ | 1,532,320 B. `[deps.mabda]` was dropped at **puka 0.6.8** (D3's interim), and that is what unblocked the target |

⚠ **Read sizes off the UNDERSCORE artifacts.** `build/<name>_agnos` is what `cyrius build --agnos` writes and
what burn-prep flashes. The hyphen-named `build/<name>-agnos` files are stale July artifacts that some smokes
still consume — `aethersafha-agnos` 15,435,960 B (07-23), `crab-agnos` 310,104 B (07-10),
`present_probe-agnos` 80,784 B (07-09). Quoting one of those quotes a build from before the band existed.

---

## 2. The compositor ladder (`AE-` rungs)

The ladder that previously existed only in CHANGELOG prose. A rung is *shipped* only when it is proven
on the substrate that can actually see it.

| Rung | What it is | Status |
|---|---|---|
| `AE-0` | Frame loop, chrome, shell panel, window decoration, focus | shipped; iron-rendered |
| `AE-0a` | Damage tracking (damage → banded clear + damage-limited `#39` blit) | ⭐ **LANDED 2026-08-07 (QEMU), UNRELEASED.** The band is `union(cur, prev)` — forced by `#84 present` flipping the render target, so anything narrower leaves the *other* buffer two frames stale in the rows a window vacated. Full-width rows only: `#39` has **no stride parameter**, so an arbitrary rect's rows are not contiguous and cannot be passed at all (a real rect needs a `#39` ABI change). Measured on the host: clear **3.937 → 2.460 ms, 1.60x**, and a new flat `rend_clear_band` because going through `fill_rect` cost 23% in per-row overhead (1.347 vs 1.093 ns/px). ⚠ The win is bounded by geometry — two 1280x720 windows cover 780 of 1440 rows, so 46% is the ceiling at that layout; the clear falls from 60% to ~48% of the frame. 80/80 in `render.tcyr` incl. a pixel-level ghost test, **both directions mutation-tested** (the first ghost test was vacuous — it moved the window inside the priming window, where the band is full regardless). On agnos the framebuffer is **pixel-count identical** to the pre-change run. ⭐ **IRON, 2026-08-07: no ghosting across 278 frames** with the band driving both the clear and the `#39` blit — the first and only substrate that could show one. ⚠ **Weakest of the three results and not to be overstated**: an absence, observed once, by eye, with STATIC windows (so the band stayed narrow and stable). A *moving* window is the ghosting case, and `AE-7` is still blocked so nothing could move one |
| `AE-1` | ~~setu listener + accept~~ → **mint / endow / spawn-placed** on the `#97` channel band | ⭐ **REPLACED 2026-08-07 (ipc bite 7).** `setu_srv_listen` and the accept block are **deleted**, not bypassed: the compositor mints a channel per client, endows one end (`CH_ENDOW` returns the child's fd), announces it as `AGNOS_CHAN` in the `#43` env blob, and spawns the client already connected. Nothing dials. QEMU `-smp 4`: `placed: 2`, `presented: 2`, framebuffer-confirmed. The prior TCP listener was iron-proven 2026-08-03 (2/2 on the panel) — that rung is history, not the current mechanism |
| `AE-2` | GPU composite of a client surface (`#86`→`#87`→`#84`) | **iron-proven** for one opaque surface |
| `AE-3` | Frame plan before render (`ae_gpu_frame_plan` publishes `ae_gpu_frame_ok`; both GPU and CPU sides read it; `ae_gpu_demote()` degrades a refusing GPU) | shipped 0.11.0 — all-or-nothing is **forced by z-order**, not chosen |
| `AE-4` | Geometry from the kernel (`bhumi_output_query` / `#38 fbinfo`) | shipped, **iron-confirmed** 800×600 |
| `AE-5` | Two concurrent real clients | ⭐ **IRON-PROVEN 2026-08-03 at 4 CPUs** (over TCP; **re-proven over the channel band in QEMU, then BURNED on the band 2026-08-07** — 278 frames, 2 clients placed, `cpus online: 4`; the iron re-burn is DONE and the band owes nothing) — `present_probe` + `crab` both composited as windows on archaemenid's panel, 278 frames, clean Esc quit. Also QEMU at `-smp 1/4/8/16` |
| `AE-6` | Premultiplied blend (`#92` op 0x01) with a real consumer | ⭐⭐ **SHIPPED 2026-08-07.** crab declares `SETU_SURF_PREMULTIPLIED` unconditionally — no arm, no flag — so the desktop composites it on the shader cores. crab satisfies the precondition by construction (sadish stores **alpha 255**; gated per pixel on the production render in `crab/tests/crab.tcyr`, `a == 255 && c <= a`, 11/11 with negative controls). ⭐ **Fixed on the way: a premultiplied window no longer demotes the WHOLE FRAME.** `ae_gpu_window_admissible` used to REFUSE one when caps bit3 was clear, so a single blend-requesting client cost every window its hardware compositing; routing is now per-window in `ae_gpu_present_frame` (`#92`, else `#87`), which is byte-exact at alpha 255. The frame log names which op ran, both cases. ⛔ **No burn owed** — `#92` was proven on this silicon by the GPU arc; wiring the desktop to a proven kernel op is a userland change |
| `AE-T` | ⭐ **A TERMINAL — puka hosts a live agnsh in a composited window** | ⭐ **DONE 2026-08-07 (QEMU).** puka 0.6.9 opens a setu window, mints a PTY on the `#97` band, spawns `/bin/agnsh` and renders it. Oracle: **4991 px of exact RGB (192,192,192)** (puka's `fb_def_fg`) on the panel — negative control **0** without puka. `agnos/scripts/harness/puka-terminal-test.py`. ⭐ **IRON-PROVEN 2026-08-07** — the panel showed agnsh's help in the window; the ONLCR fix that made its LAYOUT correct shipped in **0.6.10** |
| `AE-T2` | Terminal INPUT — keystrokes reach the hosted shell | ⭐ **DONE 2026-08-07 (QEMU).** The hypothesis was right about the LAYER and wrong about the mechanism, and the answer was **one byte**: Enter encodes to **CR (13)** (`puka/src/input/evdev.cyr:74`, correctly — a terminal sends CR) and agnoshi's `read_line` terminates on **LF (10)** and nothing else (`agnsh.cyr:366`), so no line could ever complete. ⛔ Single-byte records were never the problem — `agnsh.cyr:363-375` accumulates across reads correctly. What was missing was CR→LF **and echo**, i.e. decomposition item **(iii)**, now `puka/src/line_discipline.cyr` (cooked: erase edits the line the child will get, so the screen cannot lie about what will run). Measured, repeatable and byte-identical across runs: **9 of 9 keys delivered**, echo **4991 → 5176** glyph px, **the shell answered 5176 → 6032**. 49/49 host asserts incl. a negative control that re-derives the CR bug from the production encoder. ⭐⭐ **IRON-PROVEN 2026-08-07** — `h-e-l-p-Enter` = 5 keys, none lost, then `line sent to the shell`, and agnsh's help rendered on the panel with a live prompt. ⚠ That burn also exposed the **missing ONLCR** (the output half): the child's bare LF moved down without returning the carriage, so the answer rendered as a staircase. Fixed and gated (correct = 6 text rows, staircase = 8), and the fix was **RE-FLASHED AND CONFIRMED ON IRON** — the kernel was byte-identical between the two flashes, so `/bin/puka` was the only variable. ⇒ the discipline is complete and hardware-validated in **both** directions: ICRNL + echo + erase + ONLCR |
| `AE-7` | Pointer input | ⛔ **not possible today** — agnos xhci matches only HID boot **keyboard**, protocol 0x01 |
| `AE-F` | Focus cycling (TAB) | ⭐ **FIXED 2026-08-07, UNRELEASED — and now PROVEN by `AE-T2`.** There was **no TAB handler at all**, while a comment in the frame loop asserted "a TAB in the input loop above" — so focus could never move and every key went to whichever client was added LAST, forever. A second client could not be typed into. Caught because puka's terminal received zero keys on a run where forwarding was working correctly — to the other window. The harness now types `tab` first and reaches the terminal, with `focus cycled by TAB` in the log; TAB is **consumed, not forwarded**, which is why the delivery count expects 9 and not 10 |
| `AE-8` | Glyphs off the CPU (`#92` op 0x03 GLYPH_1BPP) | unconsumed |
| `AE-9` | Batched `#92` (64 records/submission), `#88 gpu_fill_rect`, `#89` bytes +4..+31, `#91` GPU window-move | unconsumed |

**The GPU gap, quantified:** the desktop consumes **6 of 13** band numbers (`#82`–`#94`) and **1 of 14**
`#92` ops (`GPU_OP_SUPPORTED = 0x1FF1F`, `GPU_OP_NOTIMPL_MASK = 0`). Every unconsumed item above is
zero-kernel-change.

---

## 3. The client path — crab and puka

⭐ **Rewritten 2026-08-07: the compositor does not wait for a listener, because there is no listener.**
Per client the agnos arm mints a channel (`sys_chan_mint`), keeps the **peer** end and endows the other
(`sys_chan_endow`), writes `AGNOS_CHAN=<fd>` into the `#43` env blob, and `sys_spawn_path_env`s
`/bin/puka` then `/bin/crab` — each **born holding a connected end** (`src/main.cyr:321-361`). No port,
no dial, no accept, no window in which a client can connect to something that is not the compositor.

Two invariants at that call site, each of which produced a silent, plausible failure when violated:
- ⛔ **Endowment is a MOVE.** Once placed, the compositor's own claim on the endowed end goes inert, so it
  must keep `mint[1]` and endow `mint[0]`. Backwards hands the client the end the compositor needs and
  leaves **both sides mute**.
- ⛔ **Endow immediately before the spawn it is for.** The arm is per-CPU and one-shot, so minting both
  channels and then spawning both places the *second* endowment into the *first* child.

**The pair is still deliberately an instrument**: both present ⇒ client path and crab both work; probe
only ⇒ setu and two-proc are fine and crab is the variable; neither ⇒ the client path itself. Staging
only crab would conflate all three.

### crab — real, and working

The dhancha file manager (sadish raster + rekha text), dual-pane, alpha-255 clean throughout. Builds
`--agnos`, connects, presents. Nothing blocks it.

### puka — ⭐ real, and it hosts a shell. Both blockers are dead.

**Both of the problems this section used to lead with are FALSIFIED, and are recorded here in the past
tense only so they are not re-derived:**

- ⛔ ~~**(a) It does not build.**~~ **DEAD at puka 0.6.8.** The failure was real —
  `cyrius build --agnos src/main.cyr` died at `lib/mabda.cyr:4170: undefined variable 'SYS_IOCTL'`,
  because `cyrius build` **prepends every `[deps.*]` module** and mabda (3.2.11 pinned / 4.0.8 on disk)
  has **zero** `CYRIUS_TARGET_AGNOS` guards in its own `src/`; its native DRM/KMS and nouveau backends are
  built on `syscall(SYS_IOCTL, …)`, so a Linux-only backend nothing on agnos calls failed the whole
  consumer (same class as the kavach 3.9.3 breakage). **`[deps.mabda]` is simply not declared any more** —
  `puka/cyrius.cyml:36-38` says so in a comment at the point where a reader would otherwise re-add it. The
  whole codebase builds `--agnos` at **1,532,320 B**. The root-cause shim is still the right fix and is
  still unscheduled: **§5 D3**.
- ⛔ ~~**(b) There is no PTY on agnos, so there is nothing for a terminal to host.**~~ **DEAD at agnos
  1.56.40.** It was true — `pty|ptmx|devpts|termios|TIOC` across `agnos/kernel/` returned zero hits — and
  it stayed true right up to the `#97` band, which is exactly what D2 predicted: **the PTY and the socket
  were the same missing primitive seen twice.** A PTY is (i) a bidirectional local channel, (ii) a way to
  hand one end to a child at spawn, (iii) a line discipline. The band shipped (i) and (ii); puka 0.6.9
  mints a PTY on it, spawns `/bin/agnsh`, and renders it. ⭐ **(iii) is COMPLETE at puka 0.6.10** — ICRNL +
  echo + erase + ONLCR, all four hardware-validated. The decomposition held exactly as D2 predicted.

**Still true, and it trips people:** `agnos/scripts/burn/stage-tools.sh:342` stages **setu's
`present_probe`** under the name `puka` in the **shared** rootfs — the name is what the compositor spawns,
so the slot keeps it. `agnos/scripts/harness/puka-terminal-test.py:93-107` overrides that slot with the
real terminal **in its own seed only**, deliberately: `aethersafha-clients-test.py`'s oracle counts
present_probe's own colours, so replacing it globally would break the other harness's gate.

⚠ **`src/main.cyr` is the real entry now** (155 lines, the terminal — it was the 42-line headless stdout
demo this section used to warn about) and it is what the harness builds:
`cyrius build --agnos src/main.cyr build/puka_agnos`. The older `programs/puka_setu_probe.cyr` /
`puka_setu_term.cyr` are not the answer and are not the path under test.

⚠ **The `SYS_POLL = 7` hazard is real and now sits in THREE files**, not one:
`programs/puka_setu_term.cyr:29`, `programs/puka_term.cyr:27`, `programs/gpu_win_probe.cyr:24` each declare
a literal `SYS_POLL = 7`. **agnos syscall 7 is `open(name, namelen, flags)`**, so `syscall(7, &pfds, …)`
would attempt to open a 1-byte path out of a pollfd struct. None of the three is built `--agnos` today —
which is the only reason it has not fired.

---

## 4. Falsified / corrected — do not re-derive

- ⛔ **The "`spawn_path` cannot carry a ~2.5 MB child" ceiling is UNFOUNDED.** It is a code comment
  (`agnos/kernel/core/main.cyr:3217-3218`), not a measured failure, and it is **contradicted by a
  measured log**: `agnos/build/ae-jalwa-smoke-logs/serial.log` (2026-07-10, `smp: cpus online: 1`)
  records a **2,517,400 B** jalwa spawned via `#43` that connected and presented. The band has never
  been bisected. Do not treat 320 KB as a ceiling when sizing a client.
- ⛔ **"An alpha-0 surface VANISHES under `#92`" is WRONG**, and was asserted in three places. The
  kernel shader is `out = src + dst*(1 - src_a)`, so alpha 0 gives `out = src + dst` — an **additive
  over-bright ghost**, which is *harder* to spot than a missing window. Unfixed producers:
  `puka/src/render/pixfmt.cyr` (writes byte 3 = 0) and jalwa (`gui/draw.cyr` discards a real alpha
  byte from its `0xRRGGBBAA` theme). Irrelevant to the opaque `#87` path crab and the probe use today.
- ⛔ **"`render_frame` is the live path"** — it had zero callers for its whole life; the loop ran
  `render_desktop`. Merged 0.11.0.
- ⛔ **`bhumi_output_query` "returns 24"** — its agnos arm returned the kernel's **0** (`#38 fbinfo`
  is 0-ok) while callers tested `== BHUMI_FBINFO_SIZE`, so the compositor asked for the screen size
  and threw the answer away on **every agnos boot ever**, falling back to a hardcoded 1280×720 on an
  800×600 panel. Fixed in bhumi 1.1.3 by a pure, host-assertable `_bhumi_fbinfo_rc`; aethersafha now
  validates struct **contents**, not the return shape.
- ⛔ **`aethersafha-setu-smoke.sh` WAS RIGGED, AND EVERY GREEN IT EVER PRODUCED IS RETRACTED.**
  (Script, kernel hook and `build.sh` define all **DELETED 2026-08-03**; described here in the past
  tense so the trap is not rebuilt.) It launched the compositor from the `AETHERSAFHA_SETU_SELFTEST`
  kernel hook as a background proc — nobody launches a desktop that way — and, worse, that hook
  assigned `net_ip = 0x7F000001` in the kernel (`agnos/kernel/core/main.cyr`), making src == dst so
  `tcp_find_conn` matched. That was the *only* reason the smoke ever passed while no ordinary boot
  worked. **A smoke that passes on a path nobody uses is worse than no smoke** — and this one did
  not merely fail to prove the human path, it manufactured a pass for a transport that could not
  complete on an ordinary boot *at that time* (before `net_src_for`, agnos 1.56.34). ⭐ After that fix
  the transport DID connect un-rigged — see §`net_src_for` below and `aethersafha-clients-test.py`'s
  "connected: 2, presented: 2" (QEMU `-smp 1`, 2026-08-02). It is retired as the **wrong primitive**,
  not as a thing that never worked.
  ⛔ Every "green on agnos" in this repo's CHANGELOG from **0.8.0 through 0.9.1** rests on it and is
  retracted in place there. Do not cite any of them, and do not re-add a selftest hook that assigns
  a network address on the system's behalf.
- ⛔ **"archaemenid has a live NIC so loopback TCP goes to the wire"** — wrong. `net_tx`
  (`agnos/kernel/core/net.cyr`) tests `net_is_loopback(dst)` and queues to the `lo_ring` **before**
  consulting `nic_ready`. A live NIC does not divert 127.x.
- ⛔ **A SECOND RUN OF THE DESKTOP ON agnos HOSTED NOTHING AND LOOKED HEALTHY** (fixed in **0.12.0**,
  recorded because the failure signature is invisible). `sock_close(setu_sfd)` sat inside
  `#ifndef CYRIUS_TARGET_AGNOS`, so the one target whose compositor loop is designed to run forever —
  and therefore the one that actually gets quit and relaunched by hand — was the only target that never
  released port 7700. Run 1 prints `setu listener up` and launches both clients; **run 2 prints no
  listener line at all** and is otherwise indistinguishable from a working desktop: `desktop up —
  windows: 1`, geometry correct, Esc works. ⚠ Any burn log that shows a desktop hosting nothing must
  first be checked for *which run of the boot it was*. The guard had no reason to exist — `sock_close`
  is portable through `net.cyr` on both targets — and reads as an AF_UNIX-era leftover never revisited
  when the transport became TCP.
- ⛔ **THE ABSENCE OF A LINE IS NOT A SIGNAL** (0.12.0). `setu_srv_listen` and `spawn_path` were both
  reported only on success, so a compositor structurally incapable of hosting an app announced that by
  staying quiet — which is also exactly what a working listener with slow clients looks like. Anything
  worth printing on success is worth printing on failure, or the log cannot be read backwards.
- ⛔⛔ **AN ALPHA-0 SOURCE UNDER `#92` IS AN ADDITIVE GHOST, AND IT NEARLY PRODUCED A FALSE RED (2026-08-07).**
  The `--selftest` sentinels are `0x00C0FFEE`-shaped — byte 3 = **0**, correct for `#87` which ignores it.
  Turning that window premultiplied looks like a one-line change; under `#92` a == 0 gives
  `out = src + dst`, so every band would have failed and the obvious reading — *"`#92` is broken on this
  box"* — would have been false. ✅ Handled: the selftest source is painted **alpha 255** (trivially
  premultiplied, shader collapses to `out = src`, so every band expectation is reusable) and `ae_st_px`
  masks byte 3 because the shader writes `out_a = 255` where an opaque copy writes 0. The sentinels stay
  distinct in 24 bits; `tests/selftest.tcyr` asserts that alpha is dropped and colour is not.
  ⚠ The general rule: **`#92` and `#87` disagree on byte 3, so any pattern shared between them must be
  compared with alpha masked.**
- ⛔⛔ **"THE TERMINAL DOESN'T RESPECT WINDOW WRAPPING" IS WRONG — IT WAS A MISSING ONLCR (2026-08-07).**
  The iron burn rendered agnsh's `help` as a **staircase**: each line beginning where the previous ended,
  then breaking mid-word at the right edge. It is **not a wrapping bug and not a width bug**, and the width
  hypothesis was eliminated by MEASUREMENT before anything was changed — agnsh's longest help line is
  **77 of 80 columns**, so nothing on that screen should have wrapped. The real chain: agnsh emits a bare
  **LF** like every agnos program, because the agnos kernel console makes LF mean newline **and** carriage
  return (`agnos/kernel/arch/x86_64/fb_console.cyr:1051-1053`); puka's engine is a **correct VT100** where
  LF only moves down. ⇒ Line 2 started at column 53 and overflowed. Every fragment in that photo is
  arithmetic. Fixed where every Unix tty fixes it — **ONLCR in the line discipline's OUTPUT half**
  (`puka/src/line_discipline.cyr`, `ld_out_feed`). ⛔ **Do NOT fix it in the shell**: every agnos program
  emits bare LF for the same correct reason, so patching agnsh breaks owl/kriya/iam in a window and adds
  stray CRs to their console output.
  ⛔⛔ **AND THE INSTRUMENT LESSON: A PIXEL COUNT CANNOT SEE A LAYOUT DEFECT.**
  `puka-terminal-test.py` passed that build **before and after** the fix with byte-identical counts
  (4991 → 5176 → 6032), because a staircase draws **exactly the same characters** and only moves them. The
  gate was blind by construction; the operator's eye was the only oracle. It now counts occupied **text
  rows**, calibrated on both arms of one build: **correct 6 · staircase 8 · ceiling 7** — and the re-flash
  confirmed the panel shows what the 6-row arm predicted. ⭐ Those numbers transferred to hardware because
  they were derived from the MUTANT, not from a passing run.
- ⛔⛔ **"KEYSTROKES ARE BEING DROPPED" IS WRONG — THEY ARE NEVER SAMPLED (2026-08-07).** A key that goes
  missing on this desktop has not been dropped by the compositor, by setu, or by the client. **A USB HID
  keyboard reports STATE ON POLL and does not queue events**; agnos drains the xHCI HID ring only inside
  `kbscan #42`'s bounded `sti` window (`agnos/kernel/core/syscall.cyr:8746-8757`), and the compositor calls
  that **once per frame**. A key whose press and release both complete within one frame therefore never
  existed as far as the system is concerned. Measured on the QEMU CPU composite path (no AMD PCI device ⇒
  every GPU branch dead) at QEMU's ~100 ms default `sendkey` hold: **0 of 9 · 4 of 9 · 4 of 9** keys
  delivered; at a 500 ms hold **9 of 9**, twice, byte-identical. ⚠ **The 4-of-9 runs still completed a line
  and got an answer**, which is exactly what made this read as a line-discipline failure for two runs.
  ⭐ Two consequences worth keeping: **a human holds a key ~100 ms**, so this is a real usability defect and
  not a harness artifact — the levers are `AE-0a` (a cheaper frame) and IRQ-buffered HID reports, both
  system work; and **iron may not show it at all**, because the GPU frame is 6.40 ms against QEMU's CPU
  path, so the iron burn must count delivered keys rather than assume this reproduces.
- ⛔ **DO NOT ADD A CONNECT RETRY LOOP.** Tried; strictly worse. `sock_connect #47` holds preempt
  disabled for the whole attempt, so a retrying client **starves the compositor it is waiting for** —
  200 tries stretched a 30 s budget to **72 s** with still zero connections. A `sched_yield` between
  attempts does not help; the cost is *inside* the blocking syscall. The reason is recorded at the
  call site in `setu/src/client.cyr` so it is not re-attempted.

---

## 5. Open decisions

### D1 — the setu transport — ✅ **RULED 2026-08-03: TCP is a WRONG PREMISE. The answer is the kernel `chan_*` band on `#97`**

⛔ **This is no longer an open question and must never be reopened as one.** The operator ruled on
2026-08-03: TCP-on-loopback as the desktop/display transport is a wrong premise, and the corrupting
claims that made it look like a working path are being purged repo-wide. The architectural reasoning
below is kept because it is the diagnosis — not because anything here is still to be decided.

**TCP on loopback:7700 is the wrong primitive for a local display protocol.** It was reuse, not
design: setu 0.1.0 was AF_UNIX, agnos has no AF_UNIX, agnos did have a TCP stack. What rode along:
`net_ip` source-address semantics and 4-tuple matching · a **DHCP dependency for a local display
protocol** · a `net_ip == 0` case that is *unfixable from ring 3* because `net_is_loopback` excludes 0
· `sock_connect #47` blocking with preempt disabled · and a ~2 KB loopback window that the pixel path
already had to escape via `sys_shm`. **The pixels left TCP a month ago. Only the small control channel
— CREATE_SURFACE / ATTACH / COMMIT / INPUT — still drags the network stack behind it.**

The operator's framing is that the reason there is no UNIX socket is that agnos first needs a design
session on **what an agnos socket IS**, how it deliberately differs from AF_UNIX, and where agnos
improves on the concept.

⭐ **That session RAN and its design phase is COMPLETE — 2026-08-02.** Four surveys, **three fully-worked
candidate designs**, and **twelve judge verdicts** across sovereignty / correctness / generality /
increment. Its final synthesis agent died with the account and a second attempt to run that one phase
failed on an account limit, which left the decision itself outstanding for a day — **it has since
landed (below), so nothing here is open.** Everything is on disk and must not be re-derived.

⭐⭐ **THE SINGLE SOURCE OF TRUTH IS NOW agnos
[`docs/development/planning/ipc.md`](https://github.com/MacCracken/agnos/blob/main/docs/development/planning/ipc.md)** —
the constraints, the three designs, the verdict table with every fatal flaw, the landed synthesis
(§9), and **§10 — the TCP retirement inventory**. It is to the socket what `gpu.md` is to the GPU.
**Do not restart the design phase, and do not copy its tables here** — this section is a pointer by
design.

✅ **DECIDED 2026-08-03** — a kernel-owned channel on one syscall, where authority is re-derived per
operation so an **inherited handle is inert by construction**, a child is **placed** holding a connected
end rather than dialling, and **the batch IS the poll**. Full design, migration and kill criteria:
**ipc.md §9**.

⭐⭐ **UNBLOCKED 2026-08-05 — both operator decisions closed, and `#96` is NOT the number.**
- **Syscall `#97 chan_op`.** `#96` stays reserved for **`fork`** (agora's blocker, `roadmap.md:41`). The
  operator assigned both; this is no longer contested and must not be re-opened.
- ⛔ **It gets NO codename, and that is a rule.** The design carried a Sanskrit working name (`anu`) and
  ipc.md §9.7 spent one of its six operator decisions choosing it. Operator: *"it doesn't need a special
  name, it's in the kernel — unless we're splitting it out to its own repo."* The band is **`chan_*`**,
  VFS tag **`VFS_CHAN = 11`**, ops **`CH_*`** — the convention `pipe_*` / `shm_*` / `sock_*` / `net_*`
  already use. **A name is a distribution fact**: it exists so a thing can be found across a repo
  boundary, and a band that never leaves one kernel has no boundary to cross. Naming lanes apply to
  standalone repos only. If this is ever split out, *that* is when it earns a name.

⭐ **Sequencing is also unblocked**: ipc.md §9.7 item 3 made bites 4–9 wait on K1, and **K1 is done**
(2026-08-03, `EFER.NXE`). agnos **1.56.40** is the band.

**The only things this section asserts on its own:**
- ⛔ **THE CHANNEL BAND REPLACES TCP on agnos — it is not a second transport.** No fallback, no compile-time
  option, no runtime switch, no rollback to loopback:7700. At bite 6 setu's agnos TCP arm is **deleted**;
  at bite 7 `setu_srv_listen` and the accept block are **removed**. Revertibility comes from bite order
  — bites 0–5 land no consumer — not from keeping the rejected path alive.
- **Sequencing against K1:** ipc.md bites 0/1/2/10 are independent of the `-smp 4` fault; **bites 4–9
  should follow it**, because the cutover cannot be proven while `-smp 4` is broken.
- ⭐ **Bite 0 is one line of kernel and is worth landing regardless of the rest**: `epoll_wait`'s
  `found == 0` path runs a bare `hlt` with no `sti` inside an IF=0 handler, so an `epoll_wait` on an
  unexpired timerfd **hangs the box today**. The desktop never epolls, so this is free here.

Two facts that constrain any answer:
- agnos shm is **one global table with no owner field** (`agnos/kernel/core/syscall.cyr:746`:
  `SHM_MAX = 16`, `SHM_MAX_SIZE = 2097152`; `shm_slot_valid` checks bounds and a non-zero phys only).
  Any process may read or write any live slot — which is already how the compositor reads client
  pixel buffers. A shm-based control channel needs **zero new syscalls**; a 64-byte channel costing a
  2 MB page, and "any process may write any slot" as a security posture, are the two hard parts.
  ⚠ Measured: `shm_create` calls `pmm_alloc_2mb()` **unconditionally regardless of requested size**, and
  the desktop today runs 2 of 16 slots at **13.8% byte utilisation** — slot pressure is not the
  bottleneck, waste is.
- ⛔ **CORRECTED 2026-08-02 — "There is no fd-inheritance path to a spawned client today" was WRONG**,
  and it was asserted here. Three investigators caught it independently and it was then verified
  directly: `proc_create_user` calls **`vfs_fd_inherit(idx, proc_current_get())` unconditionally**
  (`agnos/kernel/core/proc.cyr:394`, whose own comment names it *"the single inheritance point for
  every user proc"*), `vfs_fd_inherit` byte-copies the creator's 32 entries into a fresh table
  (`vfs.cyr:167-175`), and **`elf_load_from_file` — the `#43` path — routes through it**
  (`elf.cyr:460`). A channel end minted before `sys_spawn_path` is **already in the child's table at
  the same index**. `exec_redirect #62` really is one-shot and `execwait #37`-only, but it is the
  narrower **placement** mechanism, not the transfer path. **What is missing is placement and
  announcement, not inheritance** — and announcement can ride the `KEY=VALUE` env blob `#43` already
  accepts and stages onto the child's init stack, which is exactly Wayland's `WAYLAND_SOCKET` trick.

✅ **Interim state RESOLVED 2026-08-02 — setu 0.7.3 + agnos 1.56.34.** setu's working tree had been
carrying an *unreleased* revert of the 0.7.2 `sys_net_ip()` workaround, in favour of the agnos kernel
fix `net_src_for` (`agnos/kernel/core/net.cyr:203-206`, source derived from destination) — so
`VERSION` said 0.7.2 while the code differed from the published 0.7.2 tag, and the kernel function had
**no CHANGELOG entry under any version**, leaving setu's own comments citing 1.56.34 and 1.56.35
inconsistently. Both closed: `net_src_for` is documented in agnos 1.56.34 (retroactively, at that
cycle's close, where it actually landed) and **setu 0.7.3** releases the revert with a stated floor of
**agnos >= 1.56.34**. ⚠ **The kernel floor is real** — a client built from 0.7.3 cannot connect at all
on an older kernel. 🔴 crab and puka cannot repin until the operator tags and pushes 0.7.3; a `path`
override is not a stopgap, it disables the tag as a test.

⛔ **Read this as a KEEP-THE-LIGHTS-ON interim, not as a transport under development.** The 2026-08-03
ruling stands over all of it: TCP-on-loopback is a wrong premise for the desktop and is being removed,
so `net_src_for`, the kernel floor and the 0.7.3 repin exist only to keep today's clients running until
the band lands. **Do not invest further in this path**, and do not read "RESOLVED" here as "the transport
works now" — the only agnos setu greens that ever existed on the *smoke* path were manufactured by the
`AETHERSAFHA_SETU_SELFTEST` hook (§4).

⚠ Also surfaced and deliberately **not** patched: `setu_read_blk` and `setu_poll_input` document
non-blocking semantics they do not have on agnos. A tagged socket fd's `sys_read` routes to the cyrius
stdlib's `_agnos_sock_recv_block`, which polls under a **30 s deadline** and returns 0 for both EOF and
timeout — so `r == 0` never means "would-block", and `setu_read_blk`'s outer 5001-iteration loop
multiplies a ~30 s call. **Ring 3 has no way to ask "is a byte waiting?" and get an immediate answer**,
so no userland change can honestly fix it; the comments were corrected in 0.7.3 and the gap is a
1.56.35 kernel item. Measured state: the desktop runs ~10 ms/frame in QEMU, so this is a latent hazard,
not an observed stall — **measure before designing.**

### D2 — what `/bin/puka` should BE — ✅ **DECIDED 2026-08-02, and ⭐ DELIVERED 2026-08-07 (QEMU): puka gets its own PTY, of sorts**

Operator call: *"puka would yes need its own pty of sorts."* So the answer is not to work around the
missing PTY forever — puka is a terminal, and a terminal without a controlling channel is a picture of
a terminal. **"Of sorts" is doing real work in that sentence: this is not a POSIX PTY port.**

⭐⭐ **THE PTY AND THE SOCKET ARE THE SAME MISSING PRIMITIVE, SEEN TWICE.** Decompose a PTY and it is
(i) a bidirectional local channel, (ii) a way to hand one end to a child at spawn, and (iii) a line
discipline over the top. agnos is missing **(i) and (ii)** — exactly what D1 is deciding. Only (iii) is
terminal-specific. So:

- **D2 is downstream of D1.** Whatever an agnos socket turns out to be, an agnos PTY should be that
  channel plus a line discipline plus a controlling-terminal notion — not a second, parallel IPC
  mechanism invented in the terminal's own repo.
- ⛔ **Do not design a PTY in isolation.** If D1 lands a channel that cannot be inherited by a
  `spawn_path #43` child, the PTY inherits that gap and the terminal still cannot host a shell. The
  inheritance question belongs to D1 and must be answered there.
- The shell side is already half-solved and worth mining rather than re-deriving: `exec_redirect #62`
  is a working save/swap/restore of a `vfs_table` entry around a run-to-completion child. It is
  one-shot and `execwait #37`-only, but it is the existing sovereign idiom for "point a child's stdio
  somewhere else", and the `#43` arm is already a named kernel item.

⭐⭐ **SUPERSEDED BY DELIVERY — the interim was never needed.** This section used to route through a
`programs/puka_setu_agnos.cyr` "resident puka surface" (the probe's no-PTY body plus a resident frame
loop, honest about not being a shell yet) *because the channel did not exist*. The channel landed first,
so puka went straight to the real thing: **`src/main.cyr` opens a setu window, mints a PTY on the `#97`
band, spawns `/bin/agnsh`, and renders it** — `AE-T`, proven by 4991 px of exact RGB(192,192,192) on the
panel against a measured 0-px control. The rung was skipped, not failed.

⛔ **Rejected then and still rejected: keep the `present_probe` stand-in as the answer.** It proves nothing
new. ⚠ It does still *occupy the slot in the shared rootfs* — see §3 for why that is deliberate and which
harness depends on it.

⚠ **The line discipline — decomposition item (iii) — is what is left**, and it is `AE-T2`: keystrokes reach
agnsh's stdin and the shell does not answer. D2 predicted exactly this split, so the remaining work is
where the design said it would be, not a surprise.

⚠ Whichever is chosen, if a real puka ever replaces the probe as `/bin/puka`, **three coupled edits
land in one change**: `agnos/scripts/burn/stage-tools.sh:319-320` (the duplicate-rootfs-name guard
fails the second row claiming a name), `agnos/scripts/burn/burn-prep.sh:1200`
(`puka) _src="../setu/build/puka_agnos"`), and — if the probe is kept as a third client —
`src/main.cyr:282-283` needs a third `sys_spawn_path` **with the correct length** (`"/bin/probe"` is
**10**, not 9).

### D3 — the mabda agnos shim — DECIDED (shape), unscheduled

Fix the root cause in mabda rather than working around it in each consumer: one function at the end of
`mabda/src/error.cyr` (already **first** in `cyrius.cyml`'s `modules` list, so no reorder is needed),
then a mechanical `syscall(SYS_IOCTL, ` → `mabda_ioctl(` rewrite across **42 sites** (backend_native_amdgpu 20,
backend_native_kms 12, backend_nvidia_nouveau 10 — all verified single-line 3-arg), then `cyrius distlib`.

```cyrius
fn mabda_ioctl(fd, req, arg): i64 {
    #ifdef CYRIUS_TARGET_AGNOS
    return 0 - 1;
    #endif
    #ifndef CYRIUS_TARGET_AGNOS
    return syscall(SYS_IOCTL, fd, req, arg);
    #endif
}
```

- ⛔ **NOT an enum shim.** `syscalls_x86_64_agnos.cyr:39` is `SYS_KILL = 16` — an
  `enum { SYS_IOCTL = 16 }` turns every DRM ioctl into a `kill()` **and compiles clean**.
- ⛔ **NOT a per-site `#ifdef` early-return** — that is a *runtime* branch and the undefined symbol
  still lands in the object. Only a shim whose `#ifdef`/`#ifndef` arms replace the whole body keeps it
  out. See `kavach/src/util.cyr:76-165`, which records that this exact distinction cost a build.
- ⛔ **Never patch `puka/lib/mabda.cyr`** — measured, `cyrius build` re-materializes it and silently
  discards the edit.

Measured: both 3.2.11 (31 sites) and 4.0.8 (42 sites) build `--agnos` with only this shim, Linux
unregressed. Needs a mabda tag (operator action) before puka can consume it.

**Interim** (no tag needed): drop `[deps.mabda]` from `puka/cyrius.cyml`. Casualty is exactly two
programs — `programs/gpu_probe.cyr:19` and `programs/gpu_win_probe.cyr:21` — which then fail on
`samvada_session_take_device`, because **samvada is mabda's transitive dep** and leaves with it. A
rescue manifest for those two must declare **both** `[deps.mabda]` and `[deps.samvada]`.

---

## 6. The agnos **1.56.35** cut — the desktop's kernel half

> ⭐ **SIX OF EIGHT ARE DONE — status as of 2026-08-05.** K1 + K6 closed in the 1.56.35 cut itself;
> **K3 / K4 / K5 landed at agnos 1.56.39**. Remaining: **K7** (the channel band, agnos **1.56.40**) and
> **K8** (the PTY, downstream of it). K2 is moot under the retired transport.
> The authoritative scope of each cut is agnos [`CHANGELOG.md`](https://github.com/MacCracken/agnos/blob/main/CHANGELOG.md);
> this table is the rationale behind it. ⛔ Whatever `build/agnos` is on disk at any moment is only
> flashable straight out of `burn-prep.sh` — any smoke or test run rebuilds it without the burn flags.

| # | Item | Why |
|---|---|---|
| ~~K1~~ ✅ | ~~**Root-cause the `-smp 4` PT_LOAD PDE-absent fault**~~ **DONE 1.56.35** — it was `EFER.NXE` never enabled on the APs (`smp.cyr:514`), so bit 63 of a paging entry was RESERVED and `proc_map_page_nx` set it on every W^X data page and user stack. Not a loader bug at all. | The arc's only remaining reproducible failure, and it reproduces in QEMU. Probe the PT_LOAD loop of `elf_load_from_file` (the **#43** path, not the in-memory `#3` path) reading each PDE back through `cr3 → PML4[0] → PDPT[0] → PD` right after `proc_map_page`. ⚠ `fmt_hex_buf` emits **zero characters for a zero value** — guard every hex print of a possibly-zero value and calibrate against a known answer, or the probe exposes itself and not the kernel |
| K2 | `net_tcp.cyr:395` `tcp_conn_count >= 8` → `>= 16`; surface `lo_dropped` / `lo_count` via `net_config #61` | A loopback connection consumes **two** slots, so listener + 2 clients = 5 and a **4th client fails**. Backing store is already 35 slots. `lo_dropped` is incremented and **never printed anywhere**, and a lo-ring drop costs a full 1 s TCP RTO. ⚠ Both may be moot depending on D1 |
| ~~K3~~ ✅ | ~~`spawn_path #43` fd-redirect arm~~ **DONE 1.56.39** — `spawn_redirect_apply(pid)`, apply-only into the child's **private** fd table (no restore: `#43` has no "afterwards"), refusing when `proc_fd_base[pid] == 0` because that is the global table. | #43 never calls `exec_redirect_apply` (#37 does), so the three desktop procs interleave on the console unserialised — this has **already corrupted a verdict** (`a11y nodes synced:run: exit 142` landed mid-line) |
| ~~K4~~ ✅ | ~~Raise the loader's user-image floor `0x200000` → `0x400000`~~ **DONE 1.56.39** — at **all four** sites, not the two named here: both loaders' `e_entry` *and* `PT_LOAD` checks, because `elf_load` (the in-memory `#3` path) carried the byte-identical hazard. Measured free: all **44** staged binaries already base at `0x400000` / `e_entry 0x4000b0`. | Today a segment could override PD[1], where the boot TSS RSP0 seeds live. Measured: **every** binary already bases its first PT_LOAD at 0x400000, so it costs nothing and retires the class |
| ~~K5~~ ✅ | ~~Make `pmm_kva_for_access` return `DIRECTMAP_BASE + phys` unconditionally~~ **DONE 1.56.39.** ⚠ The ordering warning here was wrong in detail and harmless in effect: the constraint is not `pmm_migrate_bitmap` but that the direct map installs at `main.cyr:232` while the earliest call site is `hda_probe` at `:661`. Also dropped the `vmm_map(phys, phys, 0x83)` side effect — verified call site by call site that every caller uses only the returned handle. | If the identity-VA class is closed at all, close it in **one** place — six call sites, versus patching one and leaving a byte-identical hazard twelve lines below. ⚠ `pmm_setup_directmap` runs at `main.cyr:228`, **after** `pmm_migrate_bitmap` (:219) |
| ~~K6~~ ✅ | ~~**CHANGELOG `net_src_for`**~~ **DONE** — documented under agnos 1.56.34, where it actually landed. | It exists in `net.cyr:203` and in the shipped `build/agnos`, with no entry under any version — and setu's client comments cite 1.56.34 and 1.56.35 inconsistently *because* of that gap. This is the half that decides what setu writes |
| K7 ⭐ | **The kernel `chan_*` band on `#97`** — agnos **1.56.40** | What D1 landed. ⭐ **Design, name, number and sequencing are ALL settled**; ipc.md §9 has the full design, the twelve-bite migration and the kill criteria, §9.6 the bite table, §10 the TCP removal inventory. `#96` is **`fork`**'s. Bites 0/1/2 are the unblocked prefix (the `epoll_wait` bare-`hlt` fix, shm owner+epoch warn-only, `proc_epoch[16]`). It ships **non-blocking-only** — a true blocked state needs a new proc state, a `sched_next` that skips it, a ready-edge, **and** a relaxation of `do_context_switch`'s unconditional ready-reset, which is a **scheduler arc** and should be named as one, not smuggled in here |
| K8 | **An agnos PTY, "of sorts"** (D2, operator-decided) | A terminal without a controlling channel is a picture of a terminal. ⭐ **Downstream of K7/D1, not parallel to it** — a PTY decomposes into (i) a local channel, (ii) inheritance by a `spawn_path #43` child, (iii) a line discipline. agnos is missing (i) and (ii), which is *exactly* what D1 decides; only (iii) is terminal-specific. Mine `exec_redirect #62` (a working save/swap/restore of a `vfs_table` entry around a child) as the existing sovereign idiom rather than porting POSIX |

---

## 7. Harness and proof hygiene

### `--clients` is the arc's instrument — read its exit code, not the log (0.12.0)

`run /bin/aethersafha --clients` runs the **production frame loop**, stops once both clients have
presented or the budget expires, and exits with a code that names the repo to look in:

| exit | meaning |
|---|---|
| **95** | both apps connected and presented |
| **94** | one did |
| **93** | neither, though both started — the client or the setu path |
| **92** | the display socket never opened |
| **91** | an app could not be started at all — the kernel's `spawn_path` or the image |

⚠ **93 and 91 point at different repos**, which is exactly the distinction the pre-0.12.0 burn could not
make: the log said `launched setu client #1` / `#2 (crab)` and then nothing, which is consistent with two
*opposite* states — the clients died on load, or they were still connecting.

⭐ Two corrections were folded in **before** the version was tagged, and both are general:
- **The budget is WALL TIME (30 s of `sys_uptime_ms`), not frames.** What a spawned client needs to load
  from ext2, start, and complete a loopback connect is seconds and scheduler slices; frame duration
  differs by more than an order of magnitude between QEMU and iron, so any single frame cap is far too
  short on one and far too long on the other. The 200000-frame cap survives only as a broken-clock
  backstop, deliberately far above anything the timer reaches.
- **A probe run IGNORES the quit key and COUNTS how many it ignored.** The first cut honoured
  `HID_ESC → IA_QUIT`, so the first iron `--clients` run ended at frame 192 of 5000 and still emitted a
  verdict: `quit on a key … 41 … 191` → `apps connected 0` → `exit 93`. **The code meant "nothing had
  connected yet" and the documentation said "nothing ever connected".** A diagnostic a keypress can
  truncate is not a diagnostic, and one whose docs overstate it is worse than none. Ignored presses are
  counted rather than dropped — on archaemenid a stray scancode is plausible (the console echoed
  `-0 -clients` for a line whose argv was demonstrably correct, the known flaky xHCI HID path).

- ⛔ **`build/agnos` is only flashable straight out of `scripts/burn/burn-prep.sh`.** Prep deletes the
  artifact and stamp up front so an abort leaves nothing to flash. **Any smoke run afterwards rebuilds
  it** — that is how a `DOOM_SELFTEST` kernel got flashed on 2026-08-02.
- ⛔ **Every `scripts/smoke/aethersafha-*.sh` clobbered `build/agnos` with a selftest kernel** — the
  setu smoke ran `env AETHERSAFHA_SETU_SELFTEST=1 sh scripts/build.sh`. Five scripts, all unmarked.
  ✅ **Resolved by deletion, 2026-08-03**: the `AETHERSAFHA_SETU_SELFTEST` hook, its `build.sh`
  define, and every smoke built on it (`aethersafha-setu-smoke.sh`, the doom-input and focus/input
  gates) are **gone**, because the hook did not merely clobber an artifact — it **manufactured the
  pass** by assigning `net_ip = 0x7F000001`. ⛔ Do not recreate any of them, and do not add a new
  selftest hook that sets system state (an address, a route, a device) on the tested code's behalf.
  ⭐ The one `aethersafha-*` smoke that survives is agnos `scripts/smoke/aethersafha-smoke.sh` — the
  "first light" RENDER proof, which uses no such hook and is honest.
- ✅ **`scripts/harness/aethersafha-clients-test.py` rebuilds nothing** — verified: no `build.sh`, no
  `stage-tools.sh`, no `cyrius build` anywhere in it. It is the desktop verification tool.
  ⛔ `AE_CLIENTS_MODE=both` is documented-broken in the file itself: the fg run orphans two spinning
  clients and the bg run then reports `launched: False`. **One mode per boot.**
- ⛔ **AN ARM THAT DID NOT RUN IS NOT AN ARM THAT FAILED — fixed 2026-08-05.** Because one mode runs
  per boot, one of `fg_code`/`bg_code` is `None` on almost every run *purely because that arm was
  never launched* — and the verdict block tested the codes alone. Measured that day against agnos
  1.56.39, while **both** arms independently reached exit 95 at `AE_CLIENTS_SMP=4`:
  `MODE=bg` printed *"Backgrounded (`&`) works; **FOREGROUND does not**"* and then invented a cause
  for it — *"⇒ agnsh's blocking execwait #37 frame prevents the spawned clients being scheduled"* —
  for a path it never launched; `MODE=fg` printed *"Both clients present on **BOTH** launch paths"*
  from one path. A false red and a false green out of the same block. It now records `ran_fg` /
  `ran_bg` **at the launch sites** (never re-derived from `MODE` at verdict time), prints
  `— (not run in this mode)` for an unrun arm, reports each arm that ran on its own, and gates the
  cross-path comparison on both having run. ⚠ `None` now means one thing only: **ran and produced no
  exit code** (stall/timeout), which is a failure. This is the same defect the `desktop` and `armed`
  guards were each added for, surviving in the one block those guards jump over.
- ✅ **THE FRAMEBUFFER IS NOW A GATE IN `desktop` MODE, not a suggestion — 2026-08-05.** This section
  used to say the counts were printed and not gated; the block itself only advised *"judge this on
  the FRAMEBUFFER counts"* while `rc` still rested entirely on the compositor's own serial claim.
  ⛔ **And the counts mean different things per mode, which was never stated.** `--clients` stops
  ~1.09 s in, so in `fg`/`bg`/`both`/`armed` the screendump lands *after* the run ended and is a
  picture of the **console** — zero client pixels there is the EXPECTED result and says nothing
  about the desktop. Every mode now prints which case it is in.
  ⚠ **The gate deliberately excludes dim-green.** Measured with both clients presented: dim-green
  **952,731 px** of a 2048×2048 capture (22.7% of the screen) — a client's 1-px border cannot be a
  fifth of the panel, so that count is dominated by compositor chrome in the same dark range, and
  gating on it would pass a desktop **hosting nothing** (the exact 0.12.0 leaked-listener state in
  §4). The gate is present_probe's own bars + bright border: **signal 3,500 px · console null 0 px**
  (measured twice), floor **200** via `AE_CLIENTS_FBMIN`. Negative control run: forcing the floor
  above the signal makes the harness **exit 1**, so the gate is wired to the exit code, not printed
  beside it. ⚠ Two honest gaps: bright-green reads **0 even on a passing run**, so the red bar
  carries this gate alone; and dim-green was excluded by reasoning from the pixel count, **not** from
  a measured hosting-nothing control.
- ⚠ The smokes consume **hyphen-named July artifacts** (`aethersafha-agnos`, `present_probe-agnos`,
  `crab-agnos`) that are byte-different from the underscore-named ones burn-prep flashes. Repoint them
  at `build/rootfs/bin/…`, which is what the newest harness already does and what `install-media.sh`
  actually flashes.
- ⚠ `burn-prep.sh:1243` derives its source-newer gate from a `sed` anchored on the nine GPU tests, then
  prints *"every --agnos build is newer than its source"*. **aethersafha, puka and crab are never
  source-checked.** Either extend the gate or narrow the message.
- ⚠ The clients harness prints framebuffer colour counts but **does not gate on them**. Its own comment
  argues the framebuffer is the external invariant and serial is a shared-premise oracle — so gate on
  it. See [[feedback_oracle_must_test_external_invariant]].

---

## 8. Pointers

- **What an agnos socket is** (D1's design session — surveys, three designs, twelve verdicts, and the
  outstanding decision) → agnos [`planning/ipc.md`](https://github.com/MacCracken/agnos/blob/main/docs/development/planning/ipc.md)
- **GPU registers, burn results, falsified GPU hypotheses** → agnos [`planning/gpu.md`](https://github.com/MacCracken/agnos/blob/main/docs/development/planning/gpu.md)
- **The SMP PDE fault** → agnos [`issues/2026-08-02-large-image-ptload-pde-absent-smp.md`](https://github.com/MacCracken/agnos/blob/main/docs/development/issues/2026-08-02-large-image-ptload-pde-absent-smp.md)
- **Next-session handoff** → agnosticos [`planning/desktop-arc-handoff.md`](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/desktop-arc-handoff.md)
- **Protocol design** → agnosticos [`planning/native-display-protocol.md`](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/native-display-protocol.md), this repo's ADR 0001
- **Theme system** → `designs/desktop_consolidated/theme-system.html`
- **Burns** → agnosticos [`iron-log.md`](https://github.com/MacCracken/agnosticos/blob/main/docs/development/iron-log.md)
- **Per-cut narrative** → this repo's `CHANGELOG.md`
