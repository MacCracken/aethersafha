# aethersafha — Current State

> **⛔ 60-LINE CAP. NOT A LOG.** History → [`../../CHANGELOG.md`](../../CHANGELOG.md), milestones →
> [`roadmap.md`](roadmap.md). It once reached 181 lines absorbing release narrative, and carried a version, a
> toolchain pin and eight dep versions all wrong. Over cap = cut prose, never facts. **Refreshed** 2026-08-18.

## TRUE — measured today

| Field | Value | Source |
|---|---|---|
| Version | **0.16.18** (2026-08-19) | [`VERSION`](../../VERSION) |
| Cyrius pin | **6.5.28** — stack-wide sweep 2026-08-17: all ten repos (incl. agnos) declare ONE toolchain, where pins had drifted across 6.5.5 / 6.5.20 / 6.5.21. ⛔ **THE PIN IS NOT DOCUMENTATION.** It selects the stdlib snapshot under `~/.cyrius/versions/<pin>/lib`, so moving it swaps the library sources compiled in — measured, the bump ALONE moved kashi/rupa/sadish/rekha by ~8 KB each. A prior entry here claimed the opposite and it cost a wrong prediction | `cyrius.cyml [package].cyrius` |
| Vendored `lib/` | ⚠ **`ganita` 1.0.4 and `patra` 1.13.0 sit below the 6.5.27 bundle (1.1.0 / 1.13.8) — a `lib/`-TRACKED repo, so this is a dep decision, not housekeeping.** Neither is referenced anywhere in `src/`; they arrive transitively. Deferred to the next dep sweep rather than swept in with a toolchain bump | `cyrius lib sync` warning |
| Modules / tests | 28 `src/*.cyr` · **25 `.tcyr` suites, 1,778 assertions** | `ls` · `cyrius test` |
| `--agnos` build | **GREEN** — 0.16.18 is **4,112,704 B · `264d1701`**. ⛔ **A SIZE DOES NOT IDENTIFY A BINARY**: 0.16.10 and 0.16.11 both measured 4,107,920 B under different hashes, and 0.16.17/0.16.18 both measure 4,112,704 B. Quote a hash beside every size | `sha256sum` |
| Host (Linux) build | **GREEN**, 25/25 suites (1,778 assertions). ⚠ The host cannot see the defects that matter most here: no GPU means no `#84`, no flip, one buffer — three separate flicker bugs were invisible in QEMU and on the host, and were each settled by an iron burn with a diagnostic flag | measured |
| Backend | **bhumi 1.4.1** — ⭐⭐ **SCANOUT LIVE ON LINUX** (fbdev; bhumi ADR 0003). ⭐⭐ **KEYBOARD + POINTER LIVE on Linux** (evdev, one shared drain; QEMU-proven). GPU via the agnos ring-3 band `#82`-`#94` — **not** mabda; there is no `[deps.mabda]` | `bhumi/src/scanout.cyr` |
| Wallpaper | ⭐ **A FILE** — PNG/JPEG via `[deps.chitra]`, scaled to the screen (0.16.0). ✅ **THE UPLOAD CAP IS GONE — agnos 1.56.44 raised the `#86` slot 2 MB → 32 MB** (region 512 MB). A full-screen wallpaper and any single window fit one slot to **3840×2160**; 5K/8K are the next tier, roadmapped in agnos. ⚠ Still **UNVERIFIED on iron**: QEMU has no AMD PCI device, so every GPU branch is dead there and a burn is the only thing that runs it. | measured |
| Substrates | ⭐⭐⭐ **LINUX HOSTS A REAL CLIENT WINDOW, PHOTOGRAPHED IN QEMU** (`scripts/qemu-linux-desktop.sh`; non-black 0.289 vs a 0.011 splash null). **fbdev first, DRM/KMS later.** Sequencing → `roadmap.md` **M6** | measured |

**Deps** — ⭐ **ALL NINE RE-VERIFIED tag-by-tag against their sibling `VERSION`, 0 stale (2026-08-18)**: bhumi 1.4.2 · chitra 0.3.1 · rupa 0.1.4 · agnostik 1.3.5 · agnodrm 1.5.1 · kashi 1.0.6 · mehman 1.0.2 · kavach 3.11.14 · setu 0.8.7. Siblings: agnos **1.56.46 OPEN** · puka **0.6.17** · crab **0.4.12** · dhancha **0.9.12**.
⛔ **The path WINS over the tag** — the vendored copy tracks the local checkout whatever the tag says, so a green build here does not prove the declared graph builds. ⚠ It fired again 2026-08-17: 0.16.6 shipped `lib/kavach.cyr` at 3.11.14 while declaring 3.11.13, because kavach's tree was mid-flight. Re-verify at EVERY cut.

## Proven, and by what

**Every `AE-` rung is shipped and iron-proven** (ladder → [`planning/desktop.md`](planning/desktop.md)). On archaemenid, on the FRAMEBUFFER not the serial: `--selftest` → **exit 95**; **two real clients**; a live agnoshi in puka's window answering typed keys; a **titlebar drag**; a GPU cursor (`#92` op 0x03); `AE-9` — every frame layer on the GPU.
⛔ Three needed a defect found by AUDIT, not a burn: the handshake must **drain** a channel dropping the OLDEST record and **resync**; `hid_mouse_take` re-seeded `buttons_seen` to the held LEVEL, making the drag-release edge unreachable; the seeded "Files" window had no client fd and **swallowed keys focused onto it** — that WAS the "puka got no keys" report.
⭐⭐ **M6-C3 DONE, IRON-PROVEN (2026-08-16, agnos 1.56.45).** A see-through `crab` over a working `puka` terminal, chrome ON the GPU: `#92` **op 0x06** for the client surface, **op 0x02 BLEND_COV** with a constant-coverage mask for the alpha chrome. ⛔ Three burns, all one missing primitive — `fill_rect_a` had no GPU route. op 0x02 had been advertised since it was `#93`. ⇒ **Check the op list before calling a gap structural.** ⚠ Unsettled: the rounding tie (agnos owns it) and a measured OVERLAP.
⚠ **`#86` SLOT LEAK — CAUSE FIXED, CEILING UNRE-MEASURED.** The budget fell **16 → 15 → 14 → 13**, one per compositor run: setu's `present_probe` ignored `SETU_CLOSE` and survived holding its buffer. setu 0.8.6 fixed the probe and crab 0.4.12 now exits on EOF too. ⛔ The 16-per-boot ceiling has NOT been re-measured since — treat it as open until a burn counts it again.

⭐⭐ **2026-08-18 BURN — M7 SHIPPED AND TWO DEFECTS FOUND.** Launcher (F2), LIST panes, clipping, both event shapes and puka's widget tree all reached iron. ⭐ **puka renders OPAQUE** — 0.6.15's alpha fix holds, and that defect was invisible off-target by construction. Fixed after: the compositor acted on BOTH key edges (three F3 presses → six theme switches; `input_handle` filtered releases at its own door but three chrome handlers ran before it), and **maximize faulted the desktop** (`cr2=0x12600000`, exit 142).
⛔ **ONE ROOT CAUSE FOR TWO REPORTS.** The GPU blit took its extent from the WINDOW while the client's buffer stayed its own size, and `ae_gpu_window_admissible` compared the window only to the SCREEN — which a maximized window passes trivially. `win_resize_by` diverges identically: that was the "drag destroyed crab's FB" report. Fixed in 0.16.8 — record the attached size (`win_set_client_buf` carries it so it cannot be forgotten), refuse/clamp an unbacked window, and SEND `SETU_CONFIGURE`, a message that sat in setu's protocol with **zero senders and zero handlers**. Reproduced + verified in QEMU: `agnos/scripts/harness/ae-resize-fault-test.py`, `SEQ=f5`.

## Open

> ⛔ **CLOSED — a falsified premise left under "Open" gets re-derived as work.** ~~`#92` premultiplied never runs~~ · ~~`#88`/op 0x03 unconsumed~~ · ~~no pointer on iron~~ · ~~damage blit needs `union(cur, prev)`~~.
> ⚠ **CORRECTED 2026-08-12**: `#89` +0..+28 and `#90` all have live consumers. `#91` and batched `#92` are **falsified, not pending**.
- **`apps.cyr`/`screen_capture.cyr`/`screen_recording.cyr` are in NO build graph** — `main.cyr` never
  includes them, 0 callers repo-wide.
- ⚠ **RULES, mechanisms unchanged**: re-verify every dep tag against its sibling `VERSION` at every cut, and
  regenerate `dist/` AT the cut — a `dist/` header is only as fresh as the last `cyrius distlib`.
- ⚠ **A compositor that WEDGES still writes no log.** The `/klug.txt` spill hangs off scanout-owner *exit*,
  so it covers a crash and a quit but not a hang — how the 08-09 burn produced zero evidence.
- 🟡 **OPEN, NOT REPRODUCED — theme repaint on the GPU path.** 2026-08-18: *"theme switching works for the
  titlebar but not background or desktop top bar."* Measured in QEMU per-region: **top bar 55/55 and
  background 39/39 pixels change — both repaint.** ⚠ QEMU has no AMD device, so that covers only the CPU
  path. ⭐ Likeliest: the SAME defect as the flashing (0.16.9) — F2 was pressed first, and a partially-copied
  frame reads as "the theme did not reach it". **Re-test before treating it as separate.**
  ⚠ And the launcher-damage fix (0.16.9) is **NOT unit-covered** — a discriminating test needs a live damage
  tracker only `main.cyr` allocates, and a mutation removing the call PASSED the check written for it.
- 🔴 **agnos 1.56.46 is OPEN** and still carries two kernel defects: the **`VFS_CHAN` close leak**
  (`vfs_close_inner` has an arm for every tag but this one; endpoints hold until process death) and the
  **1.56.41 keystroke loss** (0/9, 4/9, 4/9 at a ~100 ms hold — ⛔ its recorded cause is contradicted by the
  code: `hid_poll` also runs from the timer ISR and the xHCI MSI-X handler, so it is **unexplained**).

## Pointers

Milestones → [`roadmap.md`](roadmap.md) · desktop arc → [`planning/desktop.md`](planning/desktop.md) · **handoff →
[`desktop-arc-handoff.md`](desktop-arc-handoff.md)** · history → [`../../CHANGELOG.md`](../../CHANGELOG.md) · parity
→ [`parity-plan.md`](parity-plan.md) (⚠ stale: headed "post-0.1.0") · GPU band → agnos `agnos-userland-abi.md`.
