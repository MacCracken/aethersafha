# Desktop arc — handoff (2026-08-23)

## ⛔ READ FIRST — the slowness is NOT the clear, and my last handoff said it was

**Burned 2026-08-22. The measurement killed the hypothesis this document was built around.**

The previous handoff opened with *"the flicker fix made the desktop SLOW"* and named 0.16.18's
full-screen background clear as a shipped regression. **Iron says no.**

| window | mean frame | mean clear |
|---|---|---|
| run 1, frames 1-240 | **63,787 µs** | **0** |
| run 1, frames 241-480 (while typing) | **150,387 µs** | **0** |
| run 3, frames 1-240 (`--bandbg`) | **67,466 µs** | **0** |

`clear` measured **0 µs in every window**, and the operator's verdict on `--bandbg` was *"returns the
flicker to the background. No apparent speed improvements."* ⇒ **AE-0a's banded clear buys nothing on
the GPU path and 0.16.18 cost nothing.** The flicker fix is EXONERATED. Do not re-open it.

⚠ **A ZERO THERE MEANS NOT MEASURED, NOT FREE.** Since AE-9 the clear only ENQUEUES a `#88` rect, so
its cost lands at submit/flip where the clear timer cannot see it. The instrument was blind by
construction on the only path that matters — the same class of error as timing with a frozen clock.

⭐ **THE REAL NUMBER, AND IT IS THE FIRST ONE THIS ARC HAS EVER HAD: 63.8 -> 150.4 ms per frame, i.e.
7-15 fps, DOUBLING while keys are typed.** That is the standing "fullscreen keystrokes are slow"
report, quantified. It was never measurable before because `ae_now_ms` was frozen (see below).

**What the next burn must read.** 0.16.21 times the phases — `cumulative us (render, present, other)`
per window, and an unconditional summary AT EXIT. Host, 100 frames: frame 1805 µs = render 489 +
**present 1301** + other 15 — **present is 72%**. If present dominates on iron too, the cost is the
client-surface blit and the fix is the present protocol's missing damage rects (~28.8 MB/frame at
2560x1408 vs 983 KB at 80x24). ⛔ **Read the phase line before touching any renderer code.**

## Closed on iron this run

- ⭐⭐ **A LAUNCHED PROGRAM PRINTS IN puka.** `ls` returns content. Cause was `chan_auth` admitting
  only `chan_end_owner == proc_current_get()`, so a child inheriting the PTY fd by fd-table copy had
  every write refused. agnos 1.56.46 adds `chan_end_pty[]` and admits a DESCENDANT of the owner to a
  PTY-mode endpoint only. ⛔ `CH_ENDOW` still requires REAL ownership — endowment MOVES ownership, so
  descendant authority is a use-right, never a transfer-right, or a child could steal agnsh's stdio.
- ⭐ **The shell panel has producers** (0.16.20) and they are on screen.
- ⭐ **`#86` SLOT CEILING RE-MEASURED AND THE LEAK IS DEAD.** Budget read **16 on all three
  compositor runs in one boot**; it used to fall 16 -> 15 -> 14 -> 13, one per run. That open item
  is CLOSED.

## Observed on iron, open

- 🟡 **`ls` renders wrong in COLUMNS; `ls -l` is fine.** kriya's `_ls_term_width` asks agnos
  `#60 winsize`, which returns the **CONSOLE grid derived from the framebuffer**
  (`fb_width()/(8·scale)` ~ 320 cols at 2560), not puka's window — so a multi-column listing is
  formatted for the whole screen and wraps inside a narrower one. `-l` is one entry per line and is
  unaffected. ⇒ Wants a per-PTY window size; same gap as the missing TIOCSWINSZ
  (`pty_set_winsize` is a no-op returning 0).
- ⚠ **"Which folder is questionable" — it is `/`.** agnos has **no cwd and no chdir**; kriya's
  `k_getcwd` returns `/` there. The listing was correct, just not visibly anchored.
- ⚠ **NOT EXERCISED THIS BURN, still owed:** the **H5 media-key** stimulus (declare it in advance and
  watch the cursor) and **theme repaint on the GPU path** (F3 with a client open) — no theme lines in
  the log at all. The mouse bound (`boot-mouse interfaces bound: 1`, was 2) and `ptrscan` reached
  ring 3, but no motion or click was driven.

## Where it stands

| repo | version | what it owns |
|---|---|---|
| aethersafha | **0.16.20** (0.16.21 pending — the phase instrument, NOT cut) | the compositor |
| puka | **0.6.20** | terminal |
| crab | **0.4.14** | file manager |
| dhancha 0.9.12 · setu 0.8.7 | | toolkit · display wire |
| rupa 0.1.4 · sadish 0.5.2 · rekha 0.3.5 · kashi 1.0.6 | | tokens · raster · fonts |
| agnos | **1.56.46 OPEN** | the kernel |

⛔ **DO NOT CHASE THE PIN** ([`CLAUDE.md`](../../CLAUDE.md) Rules). cyrius moves constantly — it went
6.5.33 -> 6.5.34 -> 6.5.35 during one session — and repos legitimately sit on different pins.
aethersafha declares 6.5.33, the other consumers 6.5.28. That is normal, not drift, not a task.

⛔ **A PINNED SNAPSHOT DIRECTORY IS NOT IMMUTABLE, so "the pin makes the build reproducible" is FALSE
as stated.** `~/.cyrius/versions/<pin>/lib` is a mutable directory: two `lib sync --full` runs at the
same pin, minutes apart, gave different sources. 0.16.19 vendored `bayan` **1.5.2** from a 6.5.33
snapshot that transiently held 6.5.34 content — committing a stdlib file its own pin does not
provide, and inflating the binary by 215 KB of dead weight that was then wrongly attributed to the
pin bump. Corrected to 1.4.2; the bump was really **+4,296 B**. ⇒ **Re-sync and re-measure at every
cut; a same-pin re-sync is NOT a no-op.** 0.16.19's `8ee98f49` cannot be rebuilt — history only.

⭐ **`scripts/check-dep-tags.sh`** is the gate the manifest asked for twice and never got. Per
path-dep: tag == sibling `VERSION`, the tag exists locally AND on the remote, and every vendored
`lib/` file is byte-identical to that dep's module **at that tag**. 9/9 clean; three mutations proven
to fail it. ⛔ LOCAL ONLY (needs the siblings; cannot run in CI). Run before every cut and burn.
⚠ kavach's three-way split (manifest 3.11.14 / vendored 3.11.15 / sibling 3.12.2) is CLOSED — the
manifest declares 3.12.2 and the gate verifies the bytes.

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

⛔ **AND THE RULE CUTS BOTH WAYS — BUT NOT THE WAY THE LAST HANDOFF CLAIMED. CORRECTED 2026-08-22.**
This section previously ended *"…WHICH IS WHY THE DESKTOP IS NOW SLOW"*, asserting that discarding
AE-0a caused the slowness. **Iron falsified that**: `clear` measures **0 µs** and `--bandbg` changes
nothing, so AE-0a's saving does not exist on the GPU path and 0.16.18 cost nothing.
⇒ The surviving rule is narrower and still worth having: **retiring a measured optimisation owes the
same instrument that earned it** — 0.16.18 shipped on one operator glance recorded as *"not reported
as slow"*, and a glance is not a measurement in either direction. The instrument, once built, said
the trade was free. **Build the instrument; then believe it over the story, including mine.**

## What will bite you

- **`path` beats `tag`.** A vendored copy tracks the local checkout whatever the manifest says. Fired
  twice more this run (crab and puka both vendored dhancha 0.9.12 while declaring older tags), and a
  fourth time in aethersafha on 2026-08-21 — caused by an ordinary `cyrius build`, not by an edit.
  ⭐ **There is now a gate instead of a fourth comment: `scripts/check-dep-tags.sh`.** Per path-dep it
  checks tag == sibling VERSION, the tag exists locally AND on the remote, and — the one that matters —
  every vendored `lib/` file is **byte-identical to that dep's module at that tag**. Exit 0/1/2, three
  mutations proven to fail it. ⛔ **LOCAL ONLY — it needs the sibling checkouts, so it cannot run in CI.**
  Run it before every cut and before every burn.
- **The cyrius pin selects the STDLIB SNAPSHOT.** A re-vendor is a real change: 6.5.28 gave
  `lib/freelist.cyr` redzone poisoning (+4,112 B) and shifted DCE enough to break a test.
- **Unreachable is not absent.** `path_exists` was called at three sites and defined **nowhere**, in
  any stdlib version; it compiled for months because DCE removed the call sites.
- **A size does not identify a binary.** Quote a sha beside every size — it has collided three times.
- **`cyrius test` does not build the binary** (crab shipped a green suite with a broken `main.cyr`).
- **`/bin/puka` defaults to setu's `present_probe`.** An iron burn needs `PUKA_TERMINAL=1`.
- **`cyrius fmt` REWRITES IN PLACE since 6.5.28** (`--dry` is the old stdout form).
- ⛔ **`ae_now_ms` WAS FROZEN ON IRON FOR THE WHOLE ARC.** `#40 uptime_ms` reads `timer_ticks`, which
  the 100 Hz ISR increments — but a foreground `run` program starts with **IF CLEARED** (only
  `/bin/agnsh` gets IF=1), and the desktop IS `run /bin/aethersafha`. Every agnos timing readout was a
  constant. That is why no frame cost was ever measured: it was not measurable. Now `#95 uptime_us`
  (rdtsc, no interrupts), whose **-1 "calibration refused" must be PROPAGATED** — `-1 / 1000` is 0,
  which rebuilds the exact fail-plausible shape the kernel added the -1 to prevent. A frozen clock is
  worse than a dead one: 0 is obviously wrong, a constant looks like a working clock.
- ⛔ **A TRUNCATED SMOKE RUN INDICTS EVERY CASE IT NEVER RAN.** `edge-abi-smoke` went red with 10 FAILs
  — including the aethersafha chrome-text regression guard — and `gpo_validate` was never at fault:
  the battery had outgrown its **40 s boot dwell**, so the log stopped mid-battery and unreached cases
  were reported wrong. The tell is the companion line `AGNOS shell — boot did not reach shell`. **Read
  that first whenever a smoke goes red.** Dwell is now 180 s; the sweep is green.
- ⛔ **A NONZERO DELTA IS NOT EVIDENCE OF OUTPUT.** Chasing "ls prints nothing in puka", the first
  verdict was "a launched program DOES reach the screen" because the delta was 266 > 0. 266 was the
  PROMPT — a floor present for every command. Four commands with wildly different output all rendered
  exactly 266. **Test whether the measurement RESPONDS to the thing you are varying**, not whether it
  is nonzero.
- ⛔ **A SHORT RUN MUST STILL TESTIFY.** The frame instrument reported every 240 frames; the burn's
  no-flag CONTROL was quit at **198 frames** and produced nothing — the one measurement that burn
  existed to take. Cadence is now 120 **plus an unconditional summary at exit**. Any per-N-frames
  diagnostic owes an exit path.

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

0. **CUT 0.16.21 AND BURN IT — the phase instrument is written and UNCUT.** One run is enough: read
   `cumulative us (render, present, other)` and the AT EXIT summary. That single line decides item 1
   below. ⛔ Do NOT re-run the `--bandbg` A/B; it is settled and the clear is exonerated.
1. **Present protocol has no damage tracking — now the PRIME SUSPECT, not a guess.** A maximized
   2560x1408 terminal copies **~28.8 MB per frame** (client writes the `#86` slot, compositor re-reads
   it) vs 983 040 B at 80x24, and the frame DOUBLES to 150 ms while typing. Host phase split already
   puts **present at 72%**. The client must declare dirty rects — protocol work in setu.
2. **The shell is never told a resize, and it now has a VISIBLE symptom.** No TIOCSWINSZ on the `#97`
   band. `ls` in puka formats its columns for `#60 winsize`'s **console** grid (~320 cols from the
   framebuffer), not the window, so multi-column listings wrap; `ls -l` is fine. Fixing the window
   size fixes the listing.
3. ✅ **Producers — DONE (0.16.20) and on screen.** mem is real (oracle-checked against `free -b`);
   cpu and disk have **no syscall on agnos** and render `--%`, never a fabricated 0%. ⚠ What remains
   is the NOTIFICATION surface, which still has no producer.
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
- **Before a burn or a cut: `scripts/check-dep-tags.sh`** (local only; needs the siblings).
- Burn artifacts: `PUKA_TERMINAL=1 scripts/burn/stage-tools.sh --build`, then
  `scripts/burn/burn-prep.sh`, then **run nothing in agnos** before flashing —
  `check.sh`/`test.sh` rebuild `build/agnos` without the burn flags.
- The live burn rubric lives in `agnosticos/docs/development/iron-nuc-zen-log.md`, newest
  `#tracker-desktop-*`. That directory is **gitignored** — local only.
