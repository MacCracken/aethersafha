# aethersafha — Current State

> **⛔ 60-LINE CAP. NOT A LOG.** History → [`../../CHANGELOG.md`](../../CHANGELOG.md), milestones →
> [`roadmap.md`](roadmap.md). It once reached 181 lines absorbing release narrative, and carried a version, a
> toolchain pin and eight dep versions all wrong. Over cap = cut prose, never facts. **Refreshed** 2026-08-12.

## TRUE — measured today

| Field | Value | Source |
|---|---|---|
| Version | **0.13.1** | [`VERSION`](../../VERSION) |
| Cyrius pin | **6.5.20** — matches `cycc`; bumped from 6.5.13 on 2026-08-12 with `lib sync --full` (17 stdlib files) | `cyrius.cyml [package].cyrius` |
| Modules / tests | 26 `src/*.cyr` · **23 `.tcyr` suites** | `ls` |
| `--agnos` build | **GREEN** — 0.13.1 is **13,584,832 B · `c00d9d6b`** at pin 6.5.20 (13,584,728 at 6.5.13; +104 B is the toolchain). ⛔ **A SIZE DOES NOT IDENTIFY A BINARY** — 2026-08-12 **three distinct artifacts shared the old byte count** (`1b8b3c57` · staged `690160b9` · rebuild `a692279f`). Quote a **hash** beside every size. ⚠ The staged rootfs is now version- AND toolchain-stale (M6-A1) | `sha256sum` |
| Host (Linux) build | **GREEN**, 23/23 suites. ⭐ `--clients` reaches **exit 95** with two clients since **0.13.1** — it was structurally unreachable before (`running = 0` on first present) | measured |
| Backend | **bhumi** 1.1.5 — scanout + input + pointer **on agnos only**; the Linux arms of `_bhumi_kfbinfo`/`_bhumi_kblit`/`_bhumi_kbscan`/`_bhumi_ptrscan` are `-1`/`0` stubs. GPU via the agnos ring-3 band `#82`-`#94` — **not** mabda; there is no `[deps.mabda]` | `bhumi/src/scanout.cyr:67-71` |
| Substrates | ⭐⭐ **LINUX IS A DECLARED DISPLAY TARGET** (operator 2026-08-12), superseding bhumi ADR 0001 and `planning/desktop.md:53-57`. **fbdev first, DRM/KMS later.** Sequencing → `roadmap.md` **M6** | operator |

**Deps** — ⭐ **ALL EIGHT VERIFIED AGAINST THEIR SIBLING `VERSION` AND A REAL GIT TAG, 2026-08-12**: bhumi 1.1.5 · rupa 0.1.2 · agnostik 1.3.4 · agnodrm 1.5.0 · kashi 1.0.4 · mehman 1.0.1 · **kavach 3.11.10** · setu 0.8.4. Siblings: agnos **1.56.43** · puka **0.6.11** · crab **0.4.6**.
⛔ **The path WINS over the tag** — the vendored copy tracks the local checkout whatever the tag says, which is
why this repo's `--agnos` build broke for a week with no change here, and why a green build here does not prove
the declared graph builds.

## Proven, and by what

**Every `AE-` rung is shipped and iron-proven** (ladder → [`planning/desktop.md`](planning/desktop.md)). On archaemenid, on the FRAMEBUFFER not the serial: `--selftest` → **exit 95**; **two real clients** with distinct window ids; a live agnoshi in puka's window **answering typed keys**; a **titlebar drag** (`frames held: 7`); a cursor as **two `#92` op 0x03 masks on the shader cores**; `AE-9` — every frame layer on the GPU, `#39` not issued; four relaunches in one boot, two clients each.
⛔ Three needed a defect found by AUDIT, not a burn: the handshake must **drain** a channel that drops the OLDEST record and **resync**; `hid_mouse_take` re-seeded `buttons_seen` to the held LEVEL, making the drag-release edge unreachable; the seeded "Files" window had no client fd and **swallowed keys focused onto it** — that WAS the "puka got no keys" report.

## Open

> ⛔ **CLOSED — a falsified premise left under "Open" gets re-derived as work.** ~~`#92` premultiplied never
> runs~~ · ~~`#88`/op 0x03 unconsumed~~ · ~~no pointer on iron~~ · ~~damage blit needs `union(cur, prev)`~~.
> ⚠ **Unconsumed list CORRECTED 2026-08-12**: `#89` **+0..+24 ARE read** (`gpu.cyr:108-113`) — only **byte
> +28**, the `#92` op-support mask, is not, so the desktop still probes ops by calling and reading the error
> (M6-A3). `#90` has a live consumer (`gpu.cyr:915`). ⛔ `#91` and batched `#92` are **falsified, not pending**.
- **`src/apps.cyr`/`screen_capture.cyr`/`screen_recording.cyr` are in NO build graph** — not "DCE'd on agnos by
  reachability" as this said; `src/main.cyr` never includes them, 0 callers repo-wide. `app_launch_terminal`
  should route to `sys_spawn_path` #43 *if* ever linked — M6-B5 is the first reason to.
- ✅ **CLOSED 2026-08-12 — the graph is honest again.** `lib/` re-materialises from sibling WORKING TREES on
  any local build, so a plain `cyrius build` had moved `lib/kavach.cyr` 3.11.8 → 3.11.10 while the manifest
  still declared tag **3.11.7** — the `path`-wins hazard above, firing in practice, wrong in committed state
  for two cuts. Manifest bumped to **3.11.10** (verified a real tag on a clean tree), pin **6.5.13 → 6.5.20**,
  `lib sync --full` (17 files), `deps` relocked. Shadow-lib and pin-drift warnings both gone; 23/23 green on
  both targets. ⚠ The mechanism is unchanged — **re-verify tag vs sibling `VERSION` at every cut**.
- ⚠ **bhumi ships a stale header at its own tag**: `bhumi/dist/bhumi.cyr` is committed reading
  `# Version: 1.1.4` at tag **1.1.5** (its working tree has the fix, uncommitted). A bhumi-side commit.
- ⚠ **A compositor that WEDGES still writes no log.** `gpu_release_pid`'s `/klug.txt` spill (0.13.0,
  iron-proven) hangs off scanout-owner *exit*, so it covers a crash and a quit but not a hang — which is
  exactly how the 08-09 burn produced zero evidence. Watchdog or periodic spill; a design call, not a patch.
- ⚠ **`/klug.txt`'s CONTENT has never been read**, only its existence and byte-exactness verified.
- 🔴 **agnos 1.56.44 is OPEN** (operator 2026-08-12) for two kernel defects: the **`VFS_CHAN` close leak**
  (`vfs_close_inner` has an arm for every tag but this one; endpoints hold until process death) and the
  **1.56.41 keystroke loss** (0/9, 4/9, 4/9 at a ~100 ms hold — ⛔ its recorded cause is contradicted by the
  code: `hid_poll` also runs from the timer ISR and the xHCI MSI-X handler, so it is **unexplained**).

## Pointers

Milestones (incl. **M6**) → [`roadmap.md`](roadmap.md) · the desktop arc → [`planning/desktop.md`](planning/desktop.md) ·
history → [`../../CHANGELOG.md`](../../CHANGELOG.md) · parity vs the frozen 27,207-line Rust oracle at `rust-old/`
→ [`parity-plan.md`](parity-plan.md) (⚠ stale: still headed "post-0.1.0") · GPU band → agnos `agnos-userland-abi.md`.
