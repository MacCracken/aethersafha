# A setu client outside the launcher registry cannot be started, and a spawned client gets no HOME

**Status:** open — filed by thoth (0.52.2) while planning its F8 arc (the AGNOS window backend over setu).
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
