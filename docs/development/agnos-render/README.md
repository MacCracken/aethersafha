# Rendering aethersafha on agnos — handoff to the agnos/ISO agent

This is the last step of **3a** (compositor scanout on agnos in QEMU). The
desktop side is done: `aethersafha` builds `--agnos` and its render path is clean
(foreign-guest hosting guarded off — it forks; the compositor loop runs
continuously on agnos, `while running`). What remains is two agnos-repo pieces +
the smoke, mirroring exactly how `doom-smoke.sh` renders cyrius-doom.

**These edit the agnos kernel — do them in the agnos repo when that track is
free** (the ISO agent was holding for tool fixes). Everything is a verbatim
mirror of the existing `DOOM_SELFTEST`.

## 1. `agnos/scripts/build.sh`

Next to the `DOOM_SELFTEST` line (~line 178), add:

```sh
[ -n "$AETHERSAFHA_SELFTEST" ] && echo '#define AETHERSAFHA_SELFTEST'
```

## 2. `agnos/kernel/core/main.cyr`

Next to the `#ifdef DOOM_SELFTEST` block (~line 2250), add — a verbatim mirror
(aethersafha parks in its compositor loop just like doom, so the boot stays here
while it renders; `sh_exec` runs the ELF from disk in ring 3):

```cyrius
#ifdef AETHERSAFHA_SELFTEST
# The sovereign compositor on agnos. /bin/aethersafha is seeded onto the ext2
# root by scripts/aethersafha-smoke.sh; run it from disk in ring 3. It opens the
# bhumi backend, renders its desktop (backdrop + windows + shell panel) to the
# framebuffer via fbinfo#38 + blit#39, and loops (the real compositor loop) — so
# the boot parks here while it renders; aethersafha-smoke.sh screendumps the live
# framebuffer. "exec: aethersafha returned" only prints if it exits.
kprintln("exec: running /bin/aethersafha", 30);
sh_exec("run /bin/aethersafha", 20);
kprintln("exec: aethersafha returned", 26);
#endif
```

(The `kprintln` byte-lengths — 30 / 20 / 26 — are exact for those strings, same
convention as the DOOM block.)

## 3. `agnos/scripts/aethersafha-smoke.sh`

Copy [`aethersafha-smoke.sh`](aethersafha-smoke.sh) (in this directory) into
`agnos/scripts/`. It stages `../aethersafha/build/aethersafha-agnos` as
`/bin/aethersafha`, builds the `AETHERSAFHA_SELFTEST` kernel, boots, and
screendumps.

## Run

```sh
cd /home/macro/Repos/aethersafha && cyrius build --agnos src/main.cyr build/aethersafha-agnos
cd /home/macro/Repos/agnos && sh scripts/aethersafha-smoke.sh
```

**Pass gates:** serial shows `aethersafha: bhumi backend up` + `aethersafha:
desktop up`, and the framebuffer screendump has > 8 distinct colors (the desktop
rendered — a blank/console screen is near-uniform). A PNG is saved to
`build/ae-smoke/aethersafha.png`.

## Notes / watch-fors

- **First real runtime of the compositor on agnos.** The build is clean but this
  is the first boot; watch the serial for `PANIC`/`FAULT`/`#PF`. Likely first
  suspects if it faults: bhumi backend open (cap/seat), or a leaf-manager init in
  `desktop_new` hitting an unexpected syscall.
- **No setu / no clients** — this is the compositor rendering *its own* desktop
  (the seeded Terminal/Files windows + shell panel). The client↔compositor setu
  path is 3b (needs an agnos transport; AF_UNIX fail-closes there).
- **kavach dep** carries agnos-readiness warnings (`sys_unlink`/`sys_rmdir`
  arity) in the now-unreachable foreign/sandbox path — cosmetic here; a future
  kavach cleanup if the swallow lane ever comes to agnos.
