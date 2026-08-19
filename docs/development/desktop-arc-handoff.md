# Desktop arc — handoff (2026-08-18)

> **Who this is for**: whoever picks the desktop up next, including me after a context reset.
> Live numbers → [`state.md`](state.md) · sequencing → [`roadmap.md`](roadmap.md) · history → the CHANGELOGs.
> This file is the things you cannot recover by reading the code.

## Where it stands

**The M7 phase is complete (A–F), and everything below is released.**

| repo | version | what it owns |
|---|---|---|
| aethersafha | **0.16.9** | the compositor |
| dhancha | **0.9.12** | client-side widget toolkit |
| setu | **0.8.7** | the display wire |
| crab | **0.4.12** · puka **0.6.17** | the two reference apps |
| rupa 0.1.4 · sadish 0.5.2 · rekha 0.3.5 · kashi 1.0.6 | | theme tokens · 2D raster · fonts · system font |
| agnos | **1.56.46 OPEN** | the kernel |

All ten declare **cyrius 6.5.27**. Every dep tag was verified against its sibling's `VERSION`: 0 stale.

## ⛔ The pattern that dominated this arc

Nearly every rung turned out to be **state that already existed with no consumer** — not a missing
feature. In order found:

`WS_MAXIMIZED` and `DECO_RESIZE_*` (machinery, unread) · `DH_W_SEL` (LIST stored a selection it never
painted) · the whole `Notification` record (title/body/priority modelled, only the COUNT drawn) ·
`shell_status_disk` and `_agents` (zero readers) · rupa's four themes (the panel was the one surface
theme-switching could not reach) · kashi's library face (**unconsumable** — a root-relative include
that died on vendoring, so four apps used the kernel's escape hatch) · puka's terminal engine (already
clean of the app layer, never bundled) · **`SETU_CONFIGURE`** (in the protocol with a constructor,
**zero senders and zero handlers**, while the compositor resized windows and never said so).

⇒ **Before building a new model, grep for readers of the one that exists.** The fix is almost never a
new model.

## What will bite you

- **`path` beats `tag`.** A vendored copy tracks the local checkout whatever the manifest says. It fired
  again on 2026-08-17: 0.16.6 shipped `lib/kavach.cyr` at 3.11.14 while declaring 3.11.13. **A green
  build is no evidence about the declared graph.** Re-verify every tag at every cut.
- **The cyrius pin selects the STDLIB SNAPSHOT**, not just documentation. Moving it swaps the library
  sources compiled in. A note in this repo claimed otherwise and cost a wrong prediction.
- **`sd_surface_pixel_at` drops alpha by contract.** No test written against it can catch a missing
  alpha byte — and a zero alpha under `#92` op 0x01 renders as an *additive over-bright ghost*, not a
  black rectangle. Read the raw 32-bit word. This stack paid that bill three times.
- **QEMU has no AMD device**, so every GPU branch is dead there. QEMU proves the CPU path only; a GPU
  claim needs iron. QEMU key delivery is also lossy (once-per-frame HID drain) — repetition beats
  patience, and count-based A/B does not work.
- **`cyrius fmt <file>` prints to stdout and does NOT rewrite in place**, and `--check` exits 1 in
  silence. agnos has 9 files flagged; deferred pending 6.5.28, which is expected to accept 4-space
  continuations. Do not churn them before then.

## Tests: what is actually covered

⚠ **Mutation-test anything you rely on.** Repeatedly in this arc a suite passed a mutation that removed
the behaviour it claimed to test. Concretely caught here: a label check that scanned **past the
framebuffer's right edge** (out-of-bounds reads return black, which is never the background, so an
off-screen label "passed"); a neighbour's label bleeding into the sampled column; a QEMU harness that
scored a run as *survived* when the client had never spawned.

⛔ **Known gap, stated so nobody reads it as coverage**: the 0.16.9 launcher-damage fix is **not
unit-covered**. A discriminating test needs a live damage tracker that only `main.cyr` allocates, and a
mutation removing the call passed the check written for it. Iron verifies it: F2, watch for flashing.

## Reproducing the two burn defects

`agnos/scripts/harness/ae-resize-fault-test.py` — boots the desktop, spawns a client through the
launcher, drives the sequence. `AE_BIN=<binary>` chooses the compositor; `SEQ=f5` isolates maximize.
Four minutes, versus a reboot of the operator's only machine.

`agnos/scripts/harness/ae-theme-repaint-test.py` — per-region screendump comparison across a theme
switch.

⚠ Both **exit INCONCLUSIVE rather than passing** when the scene did not set up. That guard exists
because the first version reported a pass for a run it never performed.

## Next

1. **Re-burn.** The fault fix (0.16.8) and the launcher damage (0.16.9) are QEMU/code-verified only.
   Also unverified on iron: the 32 MB `#86` slot and a full-screen wallpaper.
2. **Theme repaint on the GPU path** — reported on iron, measured working on the CPU path in QEMU. Most
   likely the same defect as the flashing. Re-test before treating it as separate.
3. **Producers, not renderers.** The notification surface, disk and agent gauges render correctly and
   **nothing in the running desktop feeds them** — `shell_new()` seeds zeros and nothing updates it.
   That is the same pattern as above, one layer up, and it is now mine rather than inherited.
4. **C4a's present half** — crab writes a live shared buffer with no per-frame protocol;
   `dh_client_present` sends ATTACH + COMMIT every frame. Undecided, and it needs a measurement nobody
   has taken: *does aethersafha redraw from a live buffer without a commit?*
5. **M6-B (Linux)** is on hold by operator ruling. **murrahir** stays do-not-start, gated behind C4a.
