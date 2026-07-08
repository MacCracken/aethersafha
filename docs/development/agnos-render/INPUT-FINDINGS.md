# Keyboard input on the agnos desktop — findings (2026-07-08)

Item **3a-input**: make the agnos compositor respond to the keyboard. Status:
**decoder DONE + proven; interactive capture blocked on an agnos-kernel xHCI-HID
reliability issue** (handoff to the agnos agent below).

## What works — the bhumi Set-1 decoder (PROVEN on real agnos)

Before this, bhumi's agnos input path was **fundamentally wrong**: `bhumi_input_poll`
decoded 8-byte **USB HID reports**, but agnos `kbscan`#42 delivers raw **Set-1
scancodes** (it is DOOM's raw-scancode pipe — see `agnos kernel/core/syscall.cyr`
num==42). So agnos keyboard input through bhumi never produced correct events.

Fix (bhumi `src/kbscan.cyr`): a pure `bhumi_scancode_process` that decodes Set-1
make/break (+ 0xE0-extended) into the same normalized HID-usage events the
compositor already consumes. `bhumi_input_poll` now branches:
`#ifdef CYRIUS_TARGET_AGNOS` → scancode decode; host → the HID path (a no-op, no
keyboard). Tables derived from the AT/XT Set-1 layout + USB HID Usage Table.

Proven:
- **Host**: `cyrius test` — 217/217, incl. a "Set-1 scancode decode" group
  (Tab/Esc/F4-F6, a letter, an arrow via 0xE0, multi-key, break codes).
- **Real agnos (QEMU)**: with a debug build, `sendkey tab` produced key event
  **usage 0x2B (43)** in the compositor loop, and `comp_focused` mutated **1→0**
  — i.e. the full chain works: QEMU usb-kbd → xHCI → `hid_poll` → `kb_buf` →
  `kbscan`#42 (Set-1) → `bhumi_scancode_process` → HID usage → compositor
  `input_map`/`input_apply`. Decode + delivery + state mutation all confirmed.

## What's blocked — agnos xHCI-HID multi-key reliability (agnos-kernel territory)

Under the compositor's frame loop, **only ~1 of 3 injected keys arrives**, with
high latency:
- Busy loop (no sleep): `sendkey tab; sendkey f4; sendkey esc` → only the tab
  pair (press+release) was drained; f4 + esc were dropped. The one key also
  arrived late (after the screendumps), so the focus change wasn't captured.
- `sleep_ms(16)` per frame made it **worse (0 keys)** — confirming `sleep_ms`#41
  does **not** drain the xHCI USB ring (it waits for the 100 Hz tick / PS/2
  IRQ); it only reduces `kbscan`/`hid_poll` frequency and starves USB input.

So reliable USB-HID input needs `kbscan`#42 / `hid_poll` to service the xHCI ring
robustly across multiple keystrokes under a ring-3 render loop. `doom-input-test.py`
(w then q) works, so agnos *can* do multiple keys — the difference is likely the
interrupt-endpoint re-arm / transfer-ring advance timing under aethersafha's
heavier loop. **This is not the bhumi decoder** (proven above).

### Handoff to the agnos agent
1. Investigate `kbscan`#42 / `hid_poll` (`kernel/arch/x86_64/usb/hid.cyr`)
   multi-key reliability + latency when the ring-3 caller polls in a tight loop.
2. Consider whether `sleep_ms`#41 should also drive `hid_poll` (so a paced
   compositor loop can both pace *and* capture USB input), or provide a
   blocking/at-tick input-drain primitive.
3. Acceptance test (ready): `scripts/aethersafha-input-smoke.py` (uses KVM +
   `qemu-xhci`+`usb-kbd`, mirrors `doom-input-test.py`). It sends tab→f4→esc and
   asserts a framebuffer change + a clean `esc`-quit. It PASSES once multi-key
   delivery is reliable. The `AETHERSAFHA_SELFTEST` kernel hook + the render
   smoke are in this directory's README.

The compositor's own key map is already wired (`aethersafha/src/input.cyr`):
Tab = focus-next, F4 = close, F5/F6 = maximize/minimize, Esc = quit.
