# 2026-09-26 — the `--agnos` build fails: vendored `lib/kavach.cyr` uses `O_NOFOLLOW` and `sys_ftruncate`

**Status:** 🟡 **OPEN**
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
