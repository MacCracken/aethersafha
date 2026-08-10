# aethersafha — Current State

> **⛔ 60-LINE CAP. NOT A LOG.** History belongs in [`../../CHANGELOG.md`](../../CHANGELOG.md), milestones in
> [`roadmap.md`](roadmap.md). This file reached 181 lines by absorbing release narrative, and carried a
> version, a toolchain pin and eight dependency versions that were all wrong. Over the cap means cut prose,
> never facts. **Last refresh** 2026-08-10.

## TRUE — measured today

| Field | Value | Source |
|---|---|---|
| Version | **0.13.0** | [`VERSION`](../../VERSION) |
| Cyrius pin | **6.5.13** | `cyrius.cyml [package].cyrius` |
| Modules / tests | 26 `src/*.cyr` · **23 `.tcyr` suites** | `ls` |
| `--agnos` build | **GREEN** — staged `/bin/aethersafha` 13,584,728 B, static ELF64. ⚠ NOT the burned binary (13,568,328) — the 0.13.0 build re-materialised stale `lib/agnodrm` + `lib/agnostik`; sources unchanged, growth is all in the *unreachable* count | `agnos/build/rootfs/bin/` |
| Backend | **bhumi** 1.1.5 (scanout + input + pointer). GPU via the agnos ring-3 band `#82`-`#94` — **not** mabda; there is no `[deps.mabda]` | `cyrius.cyml` |

**Deps** (all declare a tag **and** a `path` override): bhumi 1.1.5 · rupa 0.1.2 · agnostik 1.3.4 · agnodrm 1.5.0 · kashi 1.0.4 · mehman 1.0.1 · kavach 3.11.7 · setu 0.8.4.
⛔ **The path WINS over the tag.** The vendored copy tracks the local checkout whatever the tag says — why
this repo's `--agnos` build broke for a week with no change here (kavach 3.9.1's Linux-only backends), and
why a green build here does not prove the declared graph builds.

## Proven, and by what

- **Composites on iron.** `--selftest` → **`exit 95`** (0.11.1): the client's sentinel words read back out of
  the GPU's own back buffer, margins and far frame clean. `#87`/`#90`/`#84` iron-proven for one opaque
  surface. **Hosts two real clients**, verified on the FRAMEBUFFER not the serial.
- ⭐⭐ **CONFIRMED ON IRON 2026-08-08** (agnos 1.56.42, 0.12.6): **two** clients present with distinct window ids, and the terminal's shell **answers typed keys** — a live agnoshi in puka's window answering `help`.
  ⛔ Needed the handshake to **drain** a channel that drops the OLDEST record and **resync** past the loss.
- ⭐⭐ **A real arrow ON THE SHADER CORES, ON IRON** (0.12.7): two `#92` op 0x03 masks, derived outline, true clipping — printed on that code's **first execution anywhere**.
- ⭐⭐⭐ **Titlebar DRAG works on iron** (0.12.8): `drag started` → `drag released ... frames held: 7`. That
  edge was UNREACHABLE (`hid_mouse_take` re-seeds `buttons_seen` to the held LEVEL). Found by AUDIT.
- ⭐ **0.12.8: desktop starts EMPTY, windows carry the app's name.** The seeded "Files" placeholder had no
  client fd, so it swallowed keys focused onto it — that WAS the "puka got no keys" report.

## Open

> ⛔ **FOUR ENTRIES HERE BURNED GREEN AND ARE CLOSED** (2026-08-07/08), written out rather than deleted
> because a falsified premise under "Open" gets re-derived as work:
> ~~`#92` premultiplied never runs~~ (composites crab on iron) · ~~`#88` never called, op 0x03
> unconsumed~~ (`AE-9`: all on the GPU) · ~~no pointer on iron~~ (`boot-mouse interfaces bound: 2`) ·
> ~~damage blit needs `union(cur, prev)`~~ (`AE-0a` burned PASS).
> ⚠ Genuinely unconsumed: `#92` batching, `#89` bytes +4..+31, `#90`/`#91`.
- **`src/apps.cyr` names `sys_fork`/`sys_dup2`/`sys_execve`** — DCE'd on agnos by reachability, not a guard; `app_launch_terminal` should route to `sys_spawn_path` #43.
- ⭐ **FIXED 2026-08-10 (agnos side): a failed desktop run is readable now.** It cost the 08-09 burn its
  evidence — `println` lands on the console this program drew over. `gpu_release_pid` now spills the ring to
  `/klug.txt`; iron-proven on all four exits and the file **verified byte-exact** off the partition.
  ⚠ A compositor that WEDGES never reaches the hook and still writes nothing.
- ⭐⭐⭐ **FIXED + IRON-PROVEN 2026-08-10, shipped in 0.13.0: `comp_close_all_clients()` at exit.** There was no
  agnos session teardown at all (`sys_close(setu_sfd)` is `#ifndef CYRIUS_TARGET_AGNOS`); **F4 reaped a
  client, Esc reaped nothing**, leaking 3 procs per relaunch into a **16-slot** table until #4 found it full.
  QEMU broke at #4 → 8/8 clean; **iron ran four launches in one boot, two clients every time**.
  ⭐ `clients told to close:` read **2·2·0·2** — the 0 being the run F4 had already cleared, so it counts
  LIVE clients, not a constant.

## Pointers

Ladder + milestones → [`roadmap.md`](roadmap.md) · history → [`../../CHANGELOG.md`](../../CHANGELOG.md) ·
parity vs the frozen 27,207-line Rust oracle at `rust-old/` → [`parity-plan.md`](parity-plan.md) · GPU band
contract → agnos `docs/development/agnos-userland-abi.md` · protocol → `setu`.
