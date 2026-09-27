# 2026-09-26 — the `--agnos` build fails: vendored `lib/kavach.cyr` uses `O_NOFOLLOW` and `sys_ftruncate`

**Status:** ✅ **CLOSED in 0.16.26 (2026-09-27)** — `cyrius build --agnos` succeeds; see **Resolution** at
the end. Archived. ⚠ agnos's own `aethersafha-smoke` on the new binary is agnos's to run and has not been.
**Filed by:** agnos 1.57.10 (its `scripts/burn/stage-tools.sh --build`, which builds each tool for the agnos target).
**Checked against:** aethersafha **0.16.25** (`d370d74`), cyrius pin of that tree.
**Severity:** the agnos rootfs cannot pick up a new aethersafha; agnos keeps staging the 2026-09-08 binary (3,834,136 B).

## What happens

`cyrius build --agnos` stops with:

```
error:lib/kavach.cyr:522:67:   undefined variable 'O_NOFOLLOW'   (O_WRONLY | O_CREAT | O_TRUNC | O_EXCL | O_NOFOLLOW)
error:lib/kavach.cyr:8440:66:  undefined variable 'O_NOFOLLOW'   (file_open(meta_path, O_WRONLY | O_TRUNC | O_NOFOLLOW, 0))
error:lib/kavach.cyr:11070:51: undefined variable 'O_NOFOLLOW'   (file_open(path, O_RDONLY | O_NOFOLLOW, 0))
warning: undefined function 'sys_ftruncate'
FAILED (compiler exit 1)
```

The agnos target of the pinned cyrius stdlib defines no `O_NOFOLLOW` and no `sys_ftruncate`, so the kavach module vendored
through the mehman → kavach dependency no longer compiles for agnos.

## Fix (owner to decide)

Either kavach guards those uses for the agnos target (agnos has no symlinks in its VFS, so `O_NOFOLLOW` can be 0 there, and
`ftruncate` has no agnos syscall), or cyrius's agnos platform layer defines `O_NOFOLLOW` (as 0) and a `sys_ftruncate` that
returns an error. Then re-vendor `lib/kavach.cyr` here. Gate: `cyrius build --agnos` succeeds; agnos's `aethersafha-smoke`
passes on the new binary.

## Resolution (0.16.26, 2026-09-27)

**Reproduced first, at 0.16.25's pin (6.6.2) on `0fdde56`:** the same three `O_NOFOLLOW` errors at
`lib/kavach.cyr:522:67`, `8440:66` and `11070:51`, plus `sys_ftruncate`. ⚠ **The host build failed too**,
which this filing could not see from an agnos-only build: six reachable undefined stdlib functions
(`proc_timeout_ms`, `_proc_read_pipe`, `_proc_wait_deadline`, `_proc_child_guard`, `sys_nanosleep`,
`sys_ftruncate`). So `0fdde56` built on **neither** target.

**Root cause — not kavach, and not a missing agnos peer so much as a skew.** Those line numbers are kavach's
*unreleased* HEAD (three commits past 3.13.1, written against cyrius 6.6.6). aethersafha declared kavach
3.12.5 and pinned 6.6.2, but its live `path = "../kavach"` override vendored whatever the checkout held, and
`0fdde56` committed it. The 6.6.2 stdlib predates everything that code needs:

| needed | where the toolchain added it |
|---|---|
| `O_NOFOLLOW` (and `O_DIRECTORY`) for agnos | cyrius **6.6.4** — `lib/io.cyr`, mapped to `AO_NOFOLLOW` 0x1000 in `file_open` |
| `sys_ftruncate` and `sys_nanosleep`, both targets | cyrius **6.6.5** — `lib/syscalls_linux_common.cyr` and the agnos peer, where `sys_ftruncate` is an explicit −ENOSYS stub (no truncate family) |
| `proc_timeout_ms` and the `_proc_*` capture helpers | cyrius **6.6.6** — `lib/process.cyr` |

⇒ The second remedy the filing offered — cyrius's agnos layer defining both — had already landed upstream,
and better than proposed: `O_NOFOLLOW` is mapped to agnos's real `AO_NOFOLLOW` rather than defined as 0.

**The fix (0.16.26):**
1. **cyrius pin 6.6.2 → 6.6.6** (the operator asked, and it is also required: agnodrm 1.6.2
   needs ≥ 6.6.5, kavach 3.13.1 needs 6.6.6).
2. **kavach resolved from its tag, 3.13.1**, not the checkout: every `path` line in `cyrius.cyml` is now
   dormant, so this cannot recur by a build, and `scripts/check-dep-tags.sh` fails on a live one.

**Gate:** `cyrius build --agnos` → **OK, 4,316,280 B** (sha256 `9114c5fd…`); host 4,392,488 B; 27/27 suites,
1,969 assertions. The declared graph at 0.16.25 (resolved from its own tags at 6.6.2) had built fine on both
targets — 4,175,440 B host, 4,099,584 B agnos, both matching `state.md` — so the tags were never broken, only
the committed `lib/`.
