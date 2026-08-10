# aethersafha — Current State

> **⛔ 60-LINE CAP. NOT A LOG.** History belongs in [`../../CHANGELOG.md`](../../CHANGELOG.md), milestones in
> [`roadmap.md`](roadmap.md). This file reached 181 lines by absorbing release narrative, and carried a
> version, a toolchain pin and eight dependency versions that were all wrong. Over the cap means cut prose,
> never facts. **Last refresh** 2026-08-09.

## TRUE — measured today

| Field | Value | Source |
|---|---|---|
| Version | **0.12.9** | [`VERSION`](../../VERSION) |
| Cyrius pin | **6.5.13** | `cyrius.cyml [package].cyrius` |
| Modules / tests | 26 `src/*.cyr` · **23 `.tcyr` suites** | `ls` |
| `--agnos` build | **GREEN** — staged `/bin/aethersafha` 13,567,968 B, static ELF64 | `agnos/build/rootfs/bin/` |
| Backend | **bhumi** 1.1.5 (scanout + input + pointer). GPU via the agnos ring-3 band `#82`-`#94` — **not** mabda; there is no `[deps.mabda]` | `cyrius.cyml` |

**Deps** (all declare a tag **and** a `path` override): bhumi 1.1.5 · rupa 0.1.2 · agnostik 1.3.4 ·
agnodrm 1.5.0 · kashi 1.0.4 · mehman 1.0.1 · kavach 3.11.7 · setu 0.8.4.
⛔ **The path WINS over the tag.** The vendored copy tracks the local checkout whatever the tag says — that
is why this repo's `--agnos` build broke for a week with no change to this repo (kavach 3.9.1's Linux-only
backends), and why a green build here does not prove the declared graph builds.

## Proven, and by what

- **Composites on iron.** `--selftest` → **`exit 95`** (0.11.1): the client's sentinel words read back out of
  the GPU's own back buffer at its coordinates, margins and far frame clean. `#87`/`#90`/`#84` iron-proven
  for one opaque surface. **Hosts two real clients** (puka + crab, crab rendering real ext2 contents),
  verified on the FRAMEBUFFER not the serial; needed agnos 1.56.34 (syscall-kstack direct-map on `#37`).
- ⭐⭐ **CONFIRMED ON IRON 2026-08-08** (agnos 1.56.42, 0.12.6): **two** clients present with distinct window
  ids, and the terminal's shell **answers typed keys** — `forwarded a key to the focused client` →
  `puka: key received` x5 → `puka: line sent to the shell`, with a live agnoshi in puka's window answering
  `help`. ⛔ Getting there needed the placed-client handshake to **drain** a channel that drops the OLDEST
  record and **resync** past what loss removes; a one-record-per-frame consumer lost phase permanently
  against a client that presents every frame.
- ⭐⭐ **A real arrow ON THE SHADER CORES, ON IRON** (0.12.7): two `#92` op 0x03 masks, derived outline, true
  clipping to the last pixel — and it printed on that code's **first execution anywhere**.
- ⭐⭐⭐ **Titlebar DRAG works on iron** (0.12.8): `drag started` → `drag released ... frames held: 7`. That
  edge was UNREACHABLE — `hid_mouse_take` re-seeds `buttons_seen` to the held LEVEL, so `cur | seen` never
  saw the button rise. Found by AUDIT, not by a burn.
- ⭐ **0.12.8: desktop starts EMPTY, windows carry the app's name.** The seeded "Files" placeholder had no
  client fd, so it swallowed keys focused onto it — that WAS the "puka got no keys" report — and sat at TAB
  index 0. Titles ride from the spawn site; setu needed no protocol change.

## Open

> ⛔ **FOUR ENTRIES HERE BURNED GREEN AND ARE CLOSED** (2026-08-07/08), written out rather than deleted
> because each stated a *premise* now falsified, and a falsified premise under "Open" gets re-derived as
> work: ~~`#92` premultiplied never runs~~ (composites crab on iron) · ~~`#88` never called, op 0x03
> unconsumed~~ (`AE-9`: everything on the GPU, `#39` not issued) · ~~no pointer on iron, xhci matches only
> protocol 0x01~~ (`boot-mouse interfaces bound: 2`, composite same-slot included) · ~~damage blit needs
> `union(cur, prev)`~~ (right requirement; `AE-0a` burned PASS).
> ⚠ Genuinely unconsumed: `#92` batching, `#89` bytes +4..+31, `#90`/`#91`.
- **`src/apps.cyr` names `sys_fork`/`sys_dup2`/`sys_execve`** — DCE'd on agnos by reachability, not by a guard; `app_launch_terminal` should route to `sys_spawn_path` #43.

## Pointers

Ladder + milestones → [`roadmap.md`](roadmap.md) · per-release history → [`../../CHANGELOG.md`](../../CHANGELOG.md) ·
parity vs the frozen 27,207-line Rust oracle at `rust-old/` → [`parity-plan.md`](parity-plan.md) ·
GPU band contract → agnos `docs/development/agnos-userland-abi.md` · protocol → `setu`.
