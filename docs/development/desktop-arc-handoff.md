# Desktop arc — handoff (2026-08-21)

## ⛔ READ FIRST — the item this handoff dropped

**The flicker fix made the desktop SLOW, and it shipped that way.** Operator, on iron after 0.16.18:
the whole-screen refresh is **slower**. The 2026-08-19 handoff recorded the flicker as closed and did
not carry the cost forward — so "every defect is closed" below is true of the flicker and **false of
the frame budget**. This is the next work. Full mechanism and the fix order → [`state.md`](state.md).

⇒ In one line: `desk_bg_clear_is_full` returns 1 whenever the chrome is on the GPU, which since
**AE-9 is every iron frame**, so the background clear is **full-screen forever** on the only path that
matters — handing back AE-0a's 3.83 → 2.46 ms of a 6.40 ms frame. ⛔ **Measure the GPU-path frame cost
before touching it**; no burn in this arc ever has. The durable fix is **per-buffer damage**, not a
blunt full clear.

## Where it stands

**Every flicker defect from iron burns 2–8 is closed and confirmed on iron.** Nine burns in this run.
⚠ **Closed ≠ free** — see the section above.

| repo | version | what it owns |
|---|---|---|
| aethersafha | **0.16.18** | the compositor |
| puka | **0.6.19** | terminal |
| crab | **0.4.14** | file manager |
| dhancha 0.9.12 · setu 0.8.7 | | toolkit · display wire |
| rupa 0.1.4 · sadish 0.5.2 · rekha 0.3.5 · kashi 1.0.6 | | tokens · raster · fonts |
| agnos | **1.56.46 OPEN** | the kernel |

The four consumers declare **cyrius 6.5.28**; the libraries still declare 6.5.27.
⛔ **THE "BYTE-IDENTICAL" CLEARANCE HAS EXPIRED — RE-MEASURED 2026-08-21.** This paragraph used to end
*"the wrapper is now 6.5.29 … measured byte-identical for these artifacts"*. **The wrapper is 6.5.33
now, and it is not byte-identical.** The pin does not bind the compiler — `cyrius build` warns about
the drift and compiles anyway — so the burned `264d1701` (4,112,704 B) rebuilds today as **`b033fdfc`
(4,116,816 B)** from the same clean tree. ⇒ **The next burn flashes a binary that has never been on
iron.** Re-pin, or restage and re-measure, before flashing. (Output-path length ruled out: long and
short paths give a byte-identical result.)

⚠ **kavach carries three numbers** — manifest **3.11.14**, vendored `lib/kavach.cyr` **3.11.15**,
sibling **3.12.2**. The other eight deps match their siblings.

## Closed this run, each with the evidence that closed it

| defect | cause | proof |
|---|---|---|
| wallpaper never loaded (`rc -1`) | `sys_open` on agnos is `(name, **namelen**, flags)`, not Linux's `(path, flags, mode)` — namelen 0 | QEMU harness, pre/post |
| crab showed 32 of 114 entries | hardcoded `sys_readdir` cap | QEMU, 32/45 → 45/45 |
| crab slow after the cap grew | one console line + one `alloc(24)` **per entry, per readdir** | iron |
| puka fullscreen grew the frame, not the grid | `SETU_CONFIGURE` unconsumed at three layers | QEMU, 80x24 → 256x126 |
| …and still did not display | compositor stopped reading a client's channel after its first present (`chan_served` was also the poll gate) + `setu_hs_action` treated state 3 as terminal | iron |
| theme-switch flicker | a full-screen invalidation held for ONE frame against a FLIPPED buffer | iron |
| background flicker below a window | the background clear was band-limited on a path where `#84` flips | iron, `--clearbg` |

## ⛔ The two things that decided every one of them

**1. `#84` FLIPS, so anything painted for one frame lands in one buffer and alternates forever.**
Five instances now: a closed window (`comp_retire`), a minimized one, the theme background
(`ae_inval_owed = 2`), rows leaving the band (`0.16.16` carry), and the background clear itself
(`0.16.18`). When something blinks, count frames of coverage before anything else.

**2. QEMU cannot see any of it.** No GPU ⇒ no `#84` ⇒ no flip ⇒ one buffer. A green suite and a green
QEMU run are both consistent with iron flickering. ⇒ **Ship a diagnostic FLAG and let a burn decide.**
That pattern closed the last two defects in three burns after code-reading had failed for several:

> `--fullclear` (blunt: confirm the cause is outside the band) → `--clearbg` (narrow: name WHICH
> producer). One broad switch to confirm, one narrow switch to attribute.

Both flags are still in the binary. Keep them.

## ⚠ An optimisation outlives the conditions it was measured under

`AE-0a` banded the clear: 3.83 ms → 2.46 ms of a 6.40 ms frame — measured on a **CPU store loop into a
SINGLE framebuffer**. `AE-9` then moved the clear onto the GPU behind a **flipping** buffer, and the
premise silently vanished. The number stayed true and the conclusion stopped being. **Re-read what a
measurement was taken against before trusting it.**

⛔ **AND THE SAME RULE CUTS BACK — THIS SECTION WAS USED TO JUSTIFY DISCARDING AE-0a, WHICH IS WHY THE
DESKTOP IS NOW SLOW.** "The premise vanished" licenses re-measuring the optimisation; it does not
license deleting it unmeasured. 0.16.18 traded a **measured** 1.37 ms saving for an **unmeasured**
full-screen clear, and the only evidence on the other side was one operator glance recorded as *"not
reported as slow"*. A glance is not a measurement either. ⇒ Retiring an optimisation owes the same
instrument that earned it.

## What will bite you

- **`path` beats `tag`.** A vendored copy tracks the local checkout whatever the manifest says. Fired
  twice more this run (crab and puka both vendored dhancha 0.9.12 while declaring older tags).
- **The cyrius pin selects the STDLIB SNAPSHOT.** A re-vendor is a real change: 6.5.28 gave
  `lib/freelist.cyr` redzone poisoning (+4,112 B) and shifted DCE enough to break a test.
- **Unreachable is not absent.** `path_exists` was called at three sites and defined **nowhere**, in
  any stdlib version; it compiled for months because DCE removed the call sites.
- **A size does not identify a binary.** Quote a sha beside every size — it has collided three times.
- **`cyrius test` does not build the binary** (crab shipped a green suite with a broken `main.cyr`).
- **`/bin/puka` defaults to setu's `present_probe`.** An iron burn needs `PUKA_TERMINAL=1`.
- **`cyrius fmt` REWRITES IN PLACE since 6.5.28** (`--dry` is the old stdout form).

## Tests: read this before trusting one

⚠ **Mutation-test anything load-bearing.** Repeatedly in this arc a suite passed a mutation that
removed the behaviour. Worse, twice a test **asserted the defect**: `render.tcyr` asserted the
one-frame repaint as correct, and `setu_handshake.tcyr` asserted state 3 as terminal.

⚠ **A harness must not score a test it did not perform.** `ae-theme-repaint-test.py` had never once
run its experiment (the launcher swallows F3, so it delivered zero switches and exited INCONCLUSIVE
every time). `puka-resize-test.py` asserted only the CLIENT's report and passed while iron failed.

Harnesses in `agnos/scripts/harness/`: `ae-wallpaper-load-test.py`, `crab-listing-cap-test.py`,
`puka-resize-test.py`, `ae-theme-repaint-test.py`, `ae-resize-fault-test.py`. All exit **INCONCLUSIVE
(2)** rather than passing when the scene did not set up.

## Next — in the order I would take them

0. **THE FRAME BUDGET — the regression above.** Instrument the GPU-path frame cost, then replace the
   full-screen clear with per-buffer damage. Everything below is slower to notice while this stands.
1. **Present protocol has no damage tracking.** A maximized 2560x1408 terminal copies **~28.8 MB per
   frame** (client writes the `#86` slot, compositor re-reads it) vs 983 040 B at 80x24. That is the
   "fullscreen keystrokes are slow" report. The client must declare dirty rects — protocol work.
2. **The shell is never told a resize.** No TIOCSWINSZ on the `#97` channel band, so puka's grid
   reflows and agnsh keeps its original width.
3. **Producers, not renderers.** The notification surface and the disk/agent gauges render correctly
   and **nothing feeds them** — `shell_new()` seeds zeros. Same pattern as everything else in this
   arc: state that exists with no consumer.
4. **crab is thin.** No fullscreen/maximize view; needs real work to feel like a file browser.
5. **agnos aarch64 does not build** — 30 reachable undefined functions, visible only since the
   compiler probe was fixed (it had been dead since cyrius v6.1.0 behind a swallowed error).
6. **C4a's present half** — undecided, and needs a measurement nobody has taken: *does aethersafha
   redraw from a live buffer without a commit?*

**M6-B (Linux)** is on hold by operator ruling. **murrahir** stays do-not-start, gated behind C4a.

## Operating rules that are not negotiable

- The operator runs **all** git operations. Never commit, never push.
- **Never use `gh`** — `curl` to the GitHub API if needed.
- **Never file an issue without being asked.**
- Do not modify `rust-old/` (the parity oracle) or `lib/` by hand.
- Burn artifacts: `PUKA_TERMINAL=1 scripts/burn/stage-tools.sh --build`, then
  `scripts/burn/burn-prep.sh`, then **run nothing in agnos** before flashing —
  `check.sh`/`test.sh` rebuild `build/agnos` without the burn flags.
- The live burn rubric lives in `agnosticos/docs/development/iron-nuc-zen-log.md`, newest
  `#tracker-desktop-*`. That directory is **gitignored** — local only.
