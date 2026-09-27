# A setu client outside the launcher registry cannot be started, and a spawned client gets no HOME

**Status:** ✅ **CLOSED in 0.16.26 (2026-09-27)** — all four asks answered; see **Resolution** at the end.
Archived. ⚠ The agnos-side code paths are unit-tested and build `--agnos` clean, but have not yet run on
QEMU or iron. **Originally:** open — filed by thoth (0.52.2) while planning its F8 arc (the AGNOS window
backend over setu).
**Severity:** blocks thoth's window on AGNOS entirely; nothing is broken for puka or crab.
**Consumer:** thoth `src/gui/` — a raw-wire Wayland window today, whose `gwl_win_*` seam mirrors puka's `win_*`
contract; the AGNOS backend would follow `puka/src/platform/setu/window_setu.cyr` over `setu/dist/setu.cyr` (0.8.9).

## What was found (read from source, aethersafha 0.16.25)

1. **A client cannot dial; it must be spawned by the compositor.** `ae_client_spawn` mints a channel, endows one end
   and spawns `path` with `AGNOS_CHAN=<fd>` (`src/main.cyr`, the `ae_client_spawn` banner), and setu's
   `setu_connect` refuses without `AGNOS_CHAN` (-9, `setu/src/client.cyr`). The launcher knows two programs:
   `lnch_register("/bin/puka", …)` and `lnch_register("/bin/crab", …)` (`src/main.cyr:1097-1098`). Any other client —
   thoth's window is `thoth gui` — has no way to be started, by the operator or by a harness.
2. **A spawned client's environment is `AGNOS_CHAN` and nothing else.** The agnos ABI says a parent-supplied env block
   *replaces* the default `HOME=/`, `PWD=/`, so the child has neither. thoth reads its global config layer from
   `$HOME/.thoth/config.cyml` (the only layer that may set authority keys: hooks, the t-ron policy, the tool-pin
   store) and works in a project directory (its file tree, git probe and `@file` resolution start from `$PWD`).

## What thoth needs (the full surface)

1. **A way to start a setu client that is not puka or crab.** Either an entry for thoth —
   `lnch_register("/bin/thoth gui", 14, "thoth")` (spawn splits argv from the path line) — or a registry that is data
   (a file the launcher reads), so a consumer does not need a compositor change to be launchable. A `--clients`-style
   pre-spawn for it, or a harness hook, so the window can be exercised under QEMU the way `crab-pointer-test.py`
   exercises crab.
2. **HOME and PWD for a spawned client** — inherited from the compositor's own, or set per registry entry.
3. **The keys a client can count on.** The 0.16.25 ruling (`2026-09-13-claimed-keys-never-reach-a-client.md`) makes
   Ctrl chords chrome and swallows them whole; thoth accepts that and will give its window's Ctrl+B / S / K / R / T a
   route that is not a chord on AGNOS. What it cannot find out is the rest of the contract: which bare keys are
   claimed (F2 and F3 today), whether Alt is forwarded, and whether that list is stable. A short table in the docs
   would do.
4. **The largest surface a client may attach.** Without a GPU carve-out (QEMU) a buffer slot is 2 MB, so thoth's
   960×600 default does not fit. A documented maximum, or a first `CONFIGURE` that states it, lets a client size to
   the slot instead of failing its attach.

## Not aethersafha's (recorded so the thread is whole)

- A channel fd cannot be waited on: `epoll_wait` reports signalfd, timerfd and TCP sockets only (agnos kernel), so a
  client idles in `sys_pause` the way crab does. A pollable channel would be the kernel's change.
- setu has no protocol version on the wire (`SETU_HELLO` is defined, but the handshake refuses anything before
  `CREATE_SURFACE`), and the names of pending wire work — modifier state on keys, damage on present, a pointer-motion
  opt-in — mean thoth holds its backend until the contract is declared stable.

## Resolution (0.16.26, 2026-09-27)

The whole contract a client can count on now lives in one place:
[`docs/architecture/001-setu-client-contract.md`](../../../architecture/001-setu-client-contract.md).

1. **Starting a client that is not puka or crab — two ways.**
   - **The launcher lists thoth** as `/bin/thoth gui` — `#43`'s line form splits the path line into argv
     `/bin/thoth`, `gui`. It is listed **only when `/bin/thoth` exists**, and **after** puka and crab. On an
     image without thoth the panel keeps its size and every row its index; the QEMU harnesses select by row,
     and `launcher-panel-test.py` recomputes the panel rect from `N_APPS = 2`. Chosen by the operator over a
     data-file registry: a further app is one `lnch_register` line in `src/main.cyr`.
   - **`--spawn NAME`** (repeatable) starts any registered app at boot, by name: `aethersafha --spawn thoth`.
     It is the harness hook, independent of `--clients`, which keeps its fixed puka + crab shape and verdict.
   - ⚠ **Not done here, and agnos's to do:** staging `/bin/thoth` on the rootfs (`scripts/burn/stage-tools.sh`
     has no thoth entry), and thoth's AGNOS backend itself.
2. **HOME and PWD — inherited.** A spawn blob *replaces* the kernel's default env, so a client got
   `AGNOS_CHAN` and nothing else. Every spawned client now also gets `HOME` and `PWD`, inherited from the
   compositor's own environment, and `/` when that is unset, empty, or over 255 bytes (`lnch_env_pack`,
   `src/launcher.cyr`; the exact bytes, and the kernel's env gate, are asserted in `tests/launcher.tcyr`).
3. **The keys a client can count on** — §3 of the contract. Every chrome key is a Ctrl chord, including, as
   of this cut, **Ctrl+F2** (launcher) and **Ctrl+F3** (theme), which were bare. So bare F2 and F3 reach a
   client now. **Alt is never claimed**: its edges are delivered like every modifier's. Anything pressed
   while Ctrl is held is swallowed, so thoth's Ctrl+B / S / K / R / T still need non-chord routes on AGNOS.
   The list changed at 0.16.25 and at 0.16.26; nothing further is planned, and a change will be marked ⛔ in
   the CHANGELOG.
4. **The largest surface — documented** (§4 of the contract), not a first `CONFIGURE`. A `#86` GPU slot is
   32 MB (8,388,608 BGRA pixels; 3840×2160 fits). Without a carve-out — QEMU — setu falls back to a `#71`
   slot of 2 MB (524,288 pixels). **960×600 does not fit that; 960×540 does.** Over the cap,
   `setu_client_present` returns −45 with nothing sent (setu ≥ 0.8.11), so a client can retry smaller.
