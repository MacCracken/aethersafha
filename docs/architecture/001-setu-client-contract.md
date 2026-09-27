# 001 — What a setu client can count on from aethersafha

> **As of 0.16.26.** Written because a client could not find this out anywhere but the source: thoth asked for it
> while planning its AGNOS window ([`../development/issues/archived/2026-09-17-a-setu-client-outside-the-registry-cannot-start.md`](../development/issues/archived/2026-09-17-a-setu-client-outside-the-registry-cannot-start.md)).
> Every row names the code that makes it true. **Where this page and the code disagree, the code is right and this
> page is the bug.** The wire itself is setu's (`setu/src/proto.cyr`); this page is what the compositor does with it.

## 1. How a client is started

**On agnos a client does not dial.** The compositor mints a `#97` channel, endows one end to the child, and spawns
it with `#43` in the line form. The child finds its end in `AGNOS_CHAN`; `setu_connect` refuses without it (−9).
`ae_client_spawn` in `src/main.cyr` is the one writer of that sequence.

| way | what it does | where |
|---|---|---|
| **The launcher** | **Ctrl+F2** opens it; Up / Down select, Enter launches, Esc closes. Rows, in order: `puka` (`/bin/puka`), `crab` (`/bin/crab`), then `thoth` (`/bin/thoth gui`) **only when `/bin/thoth` exists** | the `lnch_register` block in `main()` |
| **`--spawn NAME`** (repeatable) | starts a registered app at boot, by its launcher name. No exit verdict — the desktop runs normally. An unknown name is logged and nothing starts | same block; `lnch_find` |
| **`--clients`** | pre-spawns puka and crab and runs a bounded probe with an exit verdict. For harnesses; its shape is fixed | same block |

- **Adding an app is one `lnch_register` line**, appended and gated on the binary being present — so the panel's
  size and every existing row stay the same on an image that does not stage it (QEMU harnesses select by row).
- The line form is `#43`'s: `PATH arg arg`, split on spaces, **no quoting**, at most 127 bytes and 16 tokens.
- **Linux** is a different target, not a fallback: the compositor listens on an AF_UNIX `SOCK_SEQPACKET` socket
  and a client connects. `--client PATH` spawns one with `SETU_SOCKET=<socket>` as its entire environment.

## 2. The environment a spawned client gets (agnos)

A parent-supplied env blob **replaces** the kernel's default (`HOME=/`, `PWD=/`); it does not merge with it
(agnos ABI §4.6). So the client gets exactly these three — `lnch_env_pack` in `src/launcher.cyr`:

| variable | value |
|---|---|
| `AGNOS_CHAN` | the endowed channel fd |
| `HOME` | the compositor's own `HOME`; **`/`** if that is unset, empty, or over 255 bytes |
| `PWD` | the compositor's own `PWD`, same rule |

agnos has no cwd and no `chdir`, so `PWD` is only a variable. Nothing else is passed.

## 3. Keys

**Delivery.** `SETU_INPUT_KEY(id, usage, mods)`, to the **focused** window's client only. `usage` is a USB HID
Keyboard/Keypad-page (0x07) usage. By default a surface gets **presses only**, with `mods = 0`; a surface created
with `SETU_SURF_FULL_KEYS` (CREATE_SURFACE flag `1`) gets presses (`mods = 1`) **and** releases (`mods = 0`).
(`setu_srv_forward_key`, `src/setu_dispatch.cyr`.)

**Claimed by the compositor — never delivered.** Every chrome key is a **Ctrl chord**
(`input_map` and `input_chrome_key`, `src/input.cyr`):

| keys | does |
|---|---|
| Ctrl+Q | quit the desktop |
| Ctrl+Tab | focus the next window |
| Ctrl+F2 | open the application launcher |
| Ctrl+F3 | cycle the desktop theme |
| Ctrl+F4 · Ctrl+F5 · Ctrl+F6 | close · maximize · minimize the focused window |
| Ctrl+F7 … Ctrl+F10 | move the focused window ← → ↑ ↓ |
| **any other key while Ctrl is held** | **swallowed**, press and release: the wire carries no modifier state, so a forwarded Ctrl+R would reach the client as a bare R |
| every key **press** while the launcher panel is open | the panel's own (Esc, Up, Down, Enter); releases still reach the client |

**Delivered bare: everything else** — Esc, Tab, F1–F12, arrows, Enter, letters, digits.

**Modifiers.** The eight modifier usages `0xE0`–`0xE7` (LCtrl LShift LAlt LGui RCtrl RShift RAlt RGui) are
delivered as ordinary key events, Ctrl's own included. A FULL_KEYS client can track Shift, Alt and Gui from those
edges; a press-only client sees only their presses. **Alt is never claimed**: Alt+X arrives as an Alt press, then
an X. On agnos RCtrl arrives as `0xE0`, like LCtrl.

⚠ **One boundary.** Release Ctrl *before* the chorded key and that key's **release** is forwarded bare, so a
FULL_KEYS client sees a release with no press. Every client shipped treats a stray release as a no-op.

**Stability.** The claimed set changed twice: 0.16.25 put the window keys on Ctrl, 0.16.26 moved F2 and F3.
No further change is planned, and any change will be marked ⛔ in `CHANGELOG.md`.

**Pointer, briefly.** `SETU_INPUT_PTR_BTN` numbers buttons **1 = left, 2 = right, 3 = middle** — the kernel's bit
order plus one, **not X11**. The wheel is its own kind (`SETU_INPUT_PTR_SCROLL`, 12). Focus, decorations and drags
are left-button only; the other buttons are forwarded and do nothing else.

## 4. The largest surface a client may attach

A client's pixels live in a kernel shared-memory slot the **client** creates. setu's `setu_client_present` tries
`#86` first and falls back to `#71`:

| slot | when | bytes per slot | BGRA pixels | examples |
|---|---|---|---|---|
| `#86 shm_create_gpu` | a GPU carve-out exists (AMD hardware) | **32 MB** (`GPU_SHM_SLOT_SIZE`, agnos `kernel/core/gpu_regs.cyr`) | 8,388,608 | 3840×2160 fits |
| `#71 shm_create` | no carve-out — **QEMU**, or `#86` refused | **2 MB** (one 2 MB page) | 524,288 | 1024×512 and 960×540 fit; **960×600 does not** |

- **16 slots system-wide** (`SHM_MAX`), shared by every client. A resize holds two for one present (setu 0.8.11).
- **Over the cap**, the create fails and `setu_client_present` returns **−45** with nothing sent (setu ≥ 0.8.11), so a
  client can retry smaller.
- The compositor sends **no initial `SETU_CONFIGURE`** stating a cap. It sends `SETU_CONFIGURE(id, w, h, state)` when
  it resizes a window (maximize, a grip drag); a client that cannot allocate the new size keeps presenting its old one.

## Not the compositor's (recorded so the picture is whole)

- A channel fd cannot be waited on: `epoll_wait` covers signalfd, timerfd and TCP only (agnos kernel). Clients poll
  and `sys_pause`, as crab does.
- setu has no protocol version on the wire, and three pieces of wire work are pending: modifier state on key events,
  damage rectangles on present, and a per-surface opt-in for pointer motion.
