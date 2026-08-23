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
Open puka, type for ~20 s, quit. Read BOTH:
- `aethersafha: cumulative us (render, present, other)` — printed every 120 frames
- `aethersafha: frame cost us AT EXIT (frames, frame, render, present, other, dropped)`

Host baseline, 100 frames: frame 1805 = render 489 + **present 1301** + other 15 (**present 72%**).
- **present dominates on iron** ⇒ the cost is the client-surface blit ⇒ the fix is damage rects in the
  present protocol (~28.8 MB/frame at 2560x1408 vs 983 KB at 80x24). Go there next.
- **render dominates** ⇒ the cost is in the compositor's own drawing; instrument inside `render_desktop`.
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
- **Theme repaint on the GPU path.** Press F3 with a client open. No theme lines appeared in the last
  log at all, so this remains unreproduced rather than fixed.

## 4. Known-open, expect to see it
`ls` multi-column output wraps in puka (`#60 winsize` returns the ~320-col CONSOLE grid, not the
window); `ls -l` renders correctly. Not a regression — the missing per-PTY window size.
