# aethersafha — Current State

> **⛔ 60-LINE CAP. NOT A LOG.** History → [`../../CHANGELOG.md`](../../CHANGELOG.md), milestones →
> [`roadmap.md`](roadmap.md). It once reached 181 lines absorbing release narrative, and carried a version, a
> toolchain pin and eight dep versions all wrong. Over cap = cut prose, never facts. **Refreshed** 2026-08-16.

## TRUE — measured today

| Field | Value | Source |
|---|---|---|
| Version | **0.16.2** (2026-08-16) | [`VERSION`](../../VERSION) |
| Cyrius pin | **6.5.21** — matches `cycc` AND agnos, so kernel and desktop build on one language version (2026-08-16, `lib sync --full`, 107 stdlib files). ⭐ **MEASURED PROVENANCE-ONLY, like agnos**: cycc 6.5.20 and 6.5.21 build the agnos target to the same binary, `e6789468be0aa31b`. ⛔ This row first claimed the opposite — that the pin move changed codegen — from a hash that moved while THREE things changed (pin · `lib sync` · `deps` relock). The compiler was the one that made no difference; the delta was `lib/sandhi.cyr` 1.9.9 → 1.9.10. ⚠ The wrapper does not `--strict-pin`, so the pin never gated a build here either way | `cyrius.cyml [package].cyrius` |
| Vendored `lib/` | ⚠ **`lib/sigil.cyr` is 3.12.9 while cyrius 6.5.21 bundles 3.12.7 — EXPECTED, DO NOT "FIX" IT.** kavach 3.11.13 declares sigil 3.12.9 and `path` re-materialises it on every `deps`, so the newer transitive dep legitimately wins over the toolchain snapshot. Running `lib sync --full` to silence the warning would downgrade a declared requirement, and the next `deps` would undo it anyway | `cyrius lib sync` warning |
| Modules / tests | 27 `src/*.cyr` · **23 `.tcyr` suites** | `ls` |
| `--agnos` build | **GREEN** — 0.16.2 is **4,075,560 B · `e6789468`**, ⭐ **~10 MB smaller than the staged tool, and that is sigil 3.12.9 doing its job.** `build/rootfs/bin/aethersafha` is 14,069,784 B; the delta is `.bss` **11,545,776 → 1,551,568** with `.text` unchanged (2,472,800 → 2,473,648) — static data, not code. Cause: **sigil 3.12.9 (2026-08-14) unbanked the RSA sign path, "9.53 MiB of `.bss` goes with it"** — 9,993,748 B declared against 9,994,208 B measured here. It arrives transitively via kavach and landed AFTER the tool was staged at 09:52 that day. ⚠ **Restage anyway** — the staged entry is 0.16.1 at the old footprint. ⛔ **A SIZE DOES NOT IDENTIFY A BINARY** — 2026-08-12 three distinct artifacts shared one byte count (`1b8b3c57` · staged `690160b9` · rebuild `a692279f`). Quote a hash beside every size | `sha256sum` |
| Host (Linux) build | **GREEN**, 23/23 suites. ⭐ `--clients` reaches **exit 95** with two clients since **0.13.1** — it was structurally unreachable before (`running = 0` on first present). ⭐ 0.13.2 adds a **latched scanout-refused report**, negative-controlled: with `/dev/fb0` hidden it prints `drawing to nothing / -1`, with it present it is silent | measured |
| Backend | **bhumi 1.4.1** — ⭐⭐ **SCANOUT IS LIVE ON LINUX** (fbdev; bhumi ADR 0003). ⛔ 1.2.0's arm used `mmap` and displayed NOTHING on shadow-buffer fbdevs (`simpledrm`); 1.2.1 uses `pwrite`. **Found in QEMU — the dev box's amdgpu could not show it.** ⭐⭐ **KEYBOARD + POINTER LIVE on Linux** (bhumi **1.4.1**, evdev, one shared drain; QEMU-proven — cursor moved by exactly the injected delta, click routed, and usages 82/80/74/76/41 for Up/Left/Home/Del/Esc). ⭐ Extended keys included — a second table keyed on evdev's flat numbering, since Set-1's 0xE0-prefixed one cannot match here. GPU via the agnos ring-3 band `#82`-`#94` — **not** mabda; there is no `[deps.mabda]` | `bhumi/src/scanout.cyr` |
| Wallpaper | ⭐ **A FILE** — PNG/JPEG via `[deps.chitra]`, scaled to the screen (0.16.0). ⛔ 0.14.0 shipped a *colour source* under a "never varies in x" doctrine that is meaningless for an image. ✅ **THE UPLOAD CAP IS GONE — agnos 1.56.44 raised the `#86` slot 2 MB → 32 MB** and relocated the shm region to `0x90000000`, doubling it to 512 MB (16 × 32 MB, ending exactly at `GPU_RT_REGION_OFF`). A full-screen wallpaper and any single window now fit one slot up to and including **3840×2160** (33,177,600 B, 376 KB spare); 1920×1080, 2560×1440, 3440×1440 ultrawide and 5120×1440 all fit comfortably. ⛔ This line previously read *"fits one `#86` slot only up to ~800x600"* and *"bites any single window over ~724x724"* — **both are now false**; 5K/8K are the next tier and are roadmapped in agnos, blocked on a derived carveout layout. ⚠ Still **UNVERIFIED on iron**: QEMU has no AMD PCI device, so every GPU branch is dead there and a burn is the only thing that runs it. ⚠ agnos also fixed a 12× regression the bigger slot would otherwise have caused — `gpu_cp_dma_blit` issues one CP-DMA packet per row at ~91.5 µs/row, so a contiguous single-packet path was added for the full-width case | measured |
| Substrates | ⭐⭐⭐ **LINUX HOSTS A REAL CLIENT WINDOW, PHOTOGRAPHED IN QEMU** (`scripts/qemu-linux-desktop.sh`; compositor launches the client itself via `--client`; non-black 0.289 vs a 0.011 splash null). Also on iron: `screen size read from the kernel / 2560 / 1440` where it read `1280 / 720` that morning. **fbdev first, DRM/KMS later.** Sequencing → `roadmap.md` **M6** | measured |

**Deps** — ⭐ **ALL NINE RE-VERIFIED AGAINST THEIR SIBLING `VERSION` AND A REAL GIT TAG, 2026-08-16**: bhumi 1.4.1 · chitra 0.3.0 · rupa 0.1.2 · agnostik 1.3.4 · agnodrm 1.5.0 · kashi 1.0.4 · mehman 1.0.1 · **kavach 3.11.13** · setu 0.8.5. ⚠ kavach was declared **3.11.12** against a sibling already at 3.11.13 — the `path`-wins hazard firing a third time. ⛔ **MEASURED: the tag edit is INERT for the binary** — 3.11.12 and 3.11.13 build to the same hash, because `path` had been supplying 3.11.13 all along. The declaration was wrong, not the artifact, which is exactly why a green build is no evidence about the graph. Siblings: agnos **1.56.45 OPEN** (1.56.44 released 2026-08-16 — the kernel this desktop's op-0x06 path requires) · puka **0.6.12** · crab **0.4.7**.
⛔ **The path WINS over the tag** — the vendored copy tracks the local checkout whatever the tag says, which is why a green build here does not prove the declared graph builds.

## Proven, and by what

**Every `AE-` rung is shipped and iron-proven** (ladder → [`planning/desktop.md`](planning/desktop.md)). On archaemenid, on the FRAMEBUFFER not the serial: `--selftest` → **exit 95**; **two real clients** with distinct window ids; a live agnoshi in puka's window **answering typed keys**; a **titlebar drag** (`frames held: 7`); a cursor as **two `#92` op 0x03 masks on the shader cores**; `AE-9` — every frame layer on the GPU, `#39` not issued; four relaunches in one boot, two clients each.
⛔ Three needed a defect found by AUDIT, not a burn: the handshake must **drain** a channel that drops the OLDEST record and **resync**; `hid_mouse_take` re-seeded `buttons_seen` to the held LEVEL, making the drag-release edge unreachable; the seeded "Files" window had no client fd and **swallowed keys focused onto it** — that WAS the "puka got no keys" report.

⭐⭐ **M6-C3 IS DONE AND IRON-PROVEN (2026-08-16, agnos 1.56.45).** A see-through `crab` over a working `puka` terminal, chrome ON the GPU: `#92` **op 0x06** for the client surface, **op 0x02 BLEND_COV** with a constant-coverage mask for the alpha chrome. ⛔ **It took three burns and they were all one missing primitive** — `fill_rect_a` had no GPU route, so burn 1 lost every titlebar background into a framebuffer the skipped `#39` blit discarded, and burn 2's repair (chrome → CPU) reintroduced layer-order stacking, *"zlevel of topbar and content don't seem to be the same"*. op 0x02 has been advertised since it was `#93`. ⇒ **Check the op list before calling a gap structural.** ⚠ **Still not settled**: the rounding tie (agnos owns it); a measured OVERLAP (the operator's eye, not an oracle); and a NON-premultiplied surface, composited opaque at any opacity on both paths by design.
⚠ **MEASURED, and it is a ceiling**: the `#86` slot budget fell **16 → 15 → 14 → 13**, one per compositor run. setu's `present_probe` ignores `SETU_CLOSE` and survives holding its buffer — the shape `compositor.cyr:258` already names, now with a number. 16 launches per boot.

## Open

> ⛔ **CLOSED — a falsified premise left under "Open" gets re-derived as work.** ~~`#92` premultiplied never
> runs~~ · ~~`#88`/op 0x03 unconsumed~~ · ~~no pointer on iron~~ · ~~damage blit needs `union(cur, prev)`~~.
> ⚠ **Unconsumed list CORRECTED 2026-08-12**: `#89` **+0..+24 ARE read** (`gpu.cyr:108-113`) — **byte +28** (the `#92` op-support
> mask) is CONSUMED as of 0.14.1 — the desktop no longer probes ops by calling and reading the error. `#90` has a live consumer (`gpu.cyr:915`). ⛔ `#91` and batched `#92` are **falsified, not pending**.
- **`src/apps.cyr`/`screen_capture.cyr`/`screen_recording.cyr` are in NO build graph** — not "DCE'd on agnos by
  reachability" as this said; `src/main.cyr` never includes them, 0 callers repo-wide. `app_launch_terminal`
  should route to `sys_spawn_path` #43 *if* ever linked — M6-B5 is the first reason to.
- ⚠ **Two closed-2026-08-12 hazards, kept as RULES because the mechanisms are unchanged** (detail →
  CHANGELOG): **re-verify every dep tag against its sibling `VERSION` at every cut** — `lib/` re-materialises
  from sibling working trees, so `path` silently outran a stale tag for two cuts; and **a `dist/` header is
  only as fresh as the last `cyrius distlib` before the tag** — regenerate AT the cut.
- ⚠ **A compositor that WEDGES still writes no log.** `gpu_release_pid`'s `/klug.txt` spill (0.13.0,
  iron-proven) hangs off scanout-owner *exit*, so it covers a crash and a quit but not a hang — which is
  exactly how the 08-09 burn produced zero evidence. Watchdog or periodic spill; a design call, not a patch.
- ⚠ **`/klug.txt`'s CONTENT has never been read**, only its existence and byte-exactness verified.
- 🔴 **agnos 1.56.45 is OPEN** (operator 2026-08-16; 1.56.44 RELEASED and burned) and still carries two kernel defects: the **`VFS_CHAN` close leak**
  (`vfs_close_inner` has an arm for every tag but this one; endpoints hold until process death) and the
  **1.56.41 keystroke loss** (0/9, 4/9, 4/9 at a ~100 ms hold — ⛔ its recorded cause is contradicted by the
  code: `hid_poll` also runs from the timer ISR and the xHCI MSI-X handler, so it is **unexplained**).

## Pointers

Milestones (incl. **M6**) → [`roadmap.md`](roadmap.md) · the desktop arc → [`planning/desktop.md`](planning/desktop.md) ·
history → [`../../CHANGELOG.md`](../../CHANGELOG.md) · parity vs the frozen 27,207-line Rust oracle at `rust-old/`
→ [`parity-plan.md`](parity-plan.md) (⚠ stale: still headed "post-0.1.0") · GPU band → agnos `agnos-userland-abi.md`.
