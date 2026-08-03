# aethersafha — Current State

> **⛔ 60-LINE CAP. NOT A LOG.** History belongs in [`../../CHANGELOG.md`](../../CHANGELOG.md), milestones in
> [`roadmap.md`](roadmap.md). This file reached 181 lines by absorbing release narrative, and carried a
> version, a toolchain pin and eight dependency versions that were all wrong. Over the cap means cut prose,
> never facts. **Last refresh** 2026-08-02.

## TRUE — measured today

| Field | Value | Source |
|---|---|---|
| Version | **0.12.0** | [`VERSION`](../../VERSION) |
| Cyrius pin | **6.5.5** | `cyrius.cyml [package].cyrius` |
| Modules / tests | 25 `src/*.cyr` · **21 `.tcyr` suites** | `ls` |
| `--agnos` build | **GREEN** — staged `/bin/aethersafha` 15,592,080 B, static ELF64 | `agnos/build/rootfs/bin/` |
| Backend | **bhumi** 1.1.3 (scanout + input). GPU via the agnos ring-3 band `#82`-`#94` — **not** mabda; there is no `[deps.mabda]` | `cyrius.cyml` |

**Deps** (all declare a tag **and** a `path` override): bhumi 1.1.3 · rupa 0.1.2 · agnostik 1.3.4 ·
agnodrm 1.5.0 · kashi 1.0.4 · mehman 1.0.1 · kavach 3.11.0 · setu 0.7.2.
⛔ **The path WINS over the tag.** The vendored copy tracks the local checkout whatever the tag says — that
is why this repo's `--agnos` build broke for a week with no change to this repo (kavach 3.9.1's Linux-only
backends), and why a green build here does not prove the declared graph builds.

## Proven, and by what

- **Composites on iron.** `run /bin/aethersafha --selftest` → **`exit 95`** on archaemenid (AMD gfx90c /
  DCN2.1), 0.11.1: the client's sentinel words read back out of the GPU's own back buffer at the client's
  coordinates, margin ring clean, far frame clean. `#87` / `#90` / `#84` are iron-proven for **one opaque
  client surface**.
- **Hosts two real clients, foreground, in QEMU** (2026-08-02): `aethersafha` typed at the agnsh prompt —
  no `&` — reaches **connected 2, presented 2** with `/bin/puka` (setu's `present_probe`) and `/bin/crab`
  (the dhancha file manager) composited as windows, crab rendering real ext2 directory contents. Verified
  on the **framebuffer**, not the serial: the screendump carries crab's panes and the probe's own
  `0x00003000` / `0x00FF0000` bands.
  ⚠ This required an agnos kernel fix (1.56.34, syscall-kstack direct-map on the `#37` restore path) and
  agnoshi routing the foreground through `spawn_path #43` + a waitpid poll instead of `execwait #37`.
  **Not yet re-confirmed on iron.**

## Open

- **`#92` premultiplied compositing has never run** — on iron or in QEMU. No client sets
  `SETU_SURF_PREMULTIPLIED`, so every surface takes the opaque `#87` path. Backlogged in
  [`roadmap.md`](roadmap.md).
- **6 of 13 GPU band numbers consumed**, 1 of 14 `#92` ops. Unconsumed and zero-kernel-change: `#88`
  `gpu_fill_rect` (wrapper vendored, never called) · `#92` op 0x03 GLYPH_1BPP · `#92` batching · `#89`
  bytes +4..+31 · `#90`/`#91`.
- **No pointer input on iron** — agnos xhci matches only HID boot *keyboard* (protocol 0x01).
- **Damage-limited blit is not safe to enable with the obvious one-liner.** `#84 present` FLIPS the render
  target, so a single-frame damage rect leaves the other buffer two frames stale. Needs `union(cur, prev)`.
- **`src/apps.cyr` names `sys_fork`/`sys_dup2`/`sys_execve`** — DCE'd on agnos by reachability, not by a
  guard. `app_launch_terminal` should route to `sys_spawn_path` #43 there.

## Pointers

Ladder + milestones → [`roadmap.md`](roadmap.md) · per-release history → [`../../CHANGELOG.md`](../../CHANGELOG.md) ·
parity vs the frozen 27,207-line Rust oracle at `rust-old/` → [`parity-plan.md`](parity-plan.md) ·
GPU band contract → agnos `docs/development/agnos-userland-abi.md` · protocol → `setu`.
