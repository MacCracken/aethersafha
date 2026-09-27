# Iron burn card — NEXT (supersedes burn-card-2026-08-22.md)

⛔ **ONE QUESTION DECIDES THE NEXT MONTH OF WORK: where does the frame go?**
The 2026-08-22 burn measured **63.8 -> 150.4 ms per frame (7-15 fps), doubling while typing**, with
`clear` at **0 µs**. The clear is exonerated; nothing else is known.

## Before flashing
- `scripts/check-dep-tags.sh` (local only — needs the siblings)
- `PUKA_TERMINAL=1 scripts/burn/stage-tools.sh --build` — ⚠ without `PUKA_TERMINAL=1`, `/bin/puka` is
  setu's `present_probe` and there is no terminal to test
- `scripts/burn/burn-prep.sh` — must exit 0 (sweep green)
- then **run nothing in agnos** — `check.sh` / `test.sh` rebuild `build/agnos` without the burn flags
- flash: `sudo ./scripts/install-media.sh --update-all` — ⚠ NOT `--update`; the oracle is
  `run /bin/<tool>`, and an ESP-only refresh pairs a new kernel with a STALE tool, silently

## 1. THE PHASE LINE — the whole point of this burn
Open puka (Ctrl+F2, Enter), type for ~20 s, quit (Ctrl+Q). Read ALL of these — every 120 frames, and
again AT EXIT (so a short run still testifies):
- `aethersafha: frame cost us THIS WINDOW (frames, frame, clear, clear pct, dropped, period)`
- `aethersafha: cumulative us (render, present, other)`
- ⭐ **0.16.27:** `aethersafha: cumulative us (client blit, #84 flip, input, yield, period)` — and
  `aethersafha: AT EXIT, cumulative us (client blit, #84 flip, input, yield, period)`

⛔ **Before 0.16.27, `present` also held the `sys_sched_yield` and the vsync-paced `#84` wait, and input
and the loop period were not timed at all**, so "present dominates" could not have said WHY. Blit and flip
are now inside present; input, yield and period outside the frame. `-1` = not measured on that path.
- **client blit dominates** ⇒ the per-window composite of a whole surface every frame ⇒ damage /
  change-signal in the present protocol (~28.8 MB/frame at 2560x1408 vs 983 KB at 80x24). Go there next.
- **`#84` flip dominates** ⇒ the frame is WAITING for vblank — work is pushing each flip past a vblank
  (iron's 150,387 us is 9.0 × 16.7 ms, 67,466 is 4.0). The fix is pacing, not blit volume.
- **yield dominates** ⇒ the clients' own work (puka re-rendering its grid) while the compositor waits.
- **input dominates** ⇒ the kbscan/ptrscan drain or the key/pointer dispatch.
- **render dominates** ⇒ the compositor's own drawing; instrument inside `render_desktop`.
Host baseline (0.16.27, no clients, 240 frames): period 1,090 = input 4 + frame 1,084 (render 345 +
present 728 + other 11); blit, flip and yield `-1` — those paths are agnos-only.
- **other dominates** ⇒ the cost is client polling / setu dispatch / input, none of which is timed yet.
⛔ Do NOT re-run the `--bandbg` A/B. It is settled.
⚠ `dropped` must be 0. Nonzero means `#95` refused calibration and NOTHING here is trustworthy.

## 2. Regressions to confirm still fixed
- `ls` in puka returns content (the `#97` PTY-descendant fix).
- Panel: **mem** shows a real %, **cpu** and **disk** show `--%` — never a fabricated 0%.
- `#86` slot budget prints **16 on every compositor run of the boot** (it read 16/16/16 last time;
  it used to fall 16 -> 15 -> 14 -> 13).

## 3. STILL OWED — not exercised on 2026-08-22
- **H5 media key.** Declare it in advance, press a Keychron media key, watch the cursor. Open since
  2026-08-16. The mouse bound (`boot-mouse interfaces bound: 1`) and `ptrscan` reached ring 3, but no
  motion or click was ever driven.
- **Theme repaint on the GPU path.** Press **Ctrl+F3** with a client open — bare F3 is the client's since
  0.16.26. No theme lines appeared in the last log at all, so this remains unreproduced rather than fixed.

## 4. Known-open, expect to see it
`ls` multi-column output wraps in puka (`#60 winsize` returns the ~320-col CONSOLE grid, not the
window); `ls -l` renders correctly. Not a regression — the missing per-PTY window size.
