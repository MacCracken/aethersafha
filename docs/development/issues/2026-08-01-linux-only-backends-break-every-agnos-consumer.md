# The Firecracker and OCI backends call the LINUX syscall wrappers unguarded — every `--agnos` consumer fails to compile

**Discovered:** 2026-08-01, building aethersafha `--agnos` during the desktop-arc GPU work
**Severity:** High — it is a hard compile error, and it blocks the whole consumer, not just the backend
**Affects:** kavach 3.9.3 (`src/backend_firecracker.cyr`, `src/backend_oci.cyr`)

## Summary

`sys_unlink` and `sys_rmdir` have **different arities on Linux and agnos**:

| target | signature |
|---|---|
| Linux (`lib/syscalls_x86_64_linux.cyr`) | `fn sys_unlink(path)` · `fn sys_rmdir(path)` |
| agnos (`lib/syscalls_x86_64_agnos.cyr`) | `fn sys_unlink(path, pathlen)` · `fn sys_rmdir(path, pathlen)` |

kavach's Firecracker and OCI backends call the **one-argument Linux form**, with no
`#ifdef CYRIUS_TARGET_AGNOS` anywhere in either file:

- `src/backend_firecracker.cyr:115` — `sys_unlink(cfg_path);`
- `src/backend_firecracker.cyr:116` — `sys_rmdir(workdir);`
- `src/backend_oci.cyr:178` — `sys_unlink(err_path);`
- `src/backend_oci.cyr:180` — `sys_unlink(log_path_pre);`
- `src/backend_oci.cyr:264` — `sys_unlink(path);`

plus `sys_mount` (agnos takes **0** arguments, the call passes 5) and a bare `SYS_CHDIR`, which
does not exist in the agnos enum at all.

Because `cyrius build` auto-prepends every `[deps.*]` module into the compilation unit, this is
**not** contained to consumers that use the OCI or Firecracker backends. Any project that declares
`[deps.kavach]` and builds `--agnos` fails, even if it only ever touches `sandbox_*`.

## Why it is newly fatal

Cyrius used to **warn** on an arity mismatch and compile anyway — which is its own hazard (the
agnos kernel has a documented case where a warned-through arity mismatch made a function's first
statement a wild kernel store). Arity is now a hard **error**, so what was silently-wrong code is
now a build stop. That is the correct direction; this issue is the backlog it exposes.

## Reproduction

```bash
cd /home/macro/Repos/aethersafha
cyrius build --agnos src/main.cyr build/aethersafha_agnos
```

```
error:lib/kavach.cyr:7429: 'sys_rmdir' expects 2 arguments, got 1
error:lib/kavach.cyr:7808: 'sys_unlink' expects 2 arguments, got 1
error:lib/kavach.cyr:7930: 'sys_mount' expects 0 arguments, got 5
error:lib/kavach.cyr:7934:26: undefined variable 'SYS_CHDIR' (missing include or enum?)
FAIL
```

Reproduces identically under the pinned 6.4.78 toolchain and under 6.5.5, so it is not toolchain
drift on the consumer's side.

## Suggested fix

These backends are **inherently Linux-only** — agnos has no Firecracker VMM, no OCI runtime, and
no `fork`. So the fix is a guard, not a port:

1. Wrap the Linux-only bodies in `#ifdef CYRIUS_TARGET_AGNOS` / `#ifndef` so the agnos build gets
   a clean "backend unavailable" stub, exactly as `backend_is_available` already models at runtime.
2. Or, if any of these paths must survive on agnos, use the two-argument form there with an
   explicit length.

⚠ **Please do not fix this in a consumer's `lib/`.** That directory is materialized output; an edit
there is erased by the next `cyrius deps` and hides the defect from every other consumer.

## A second, separable problem this surfaced — in the consumer

aethersafha's manifest declares:

```toml
[deps.kavach]
tag = "3.7.0"
path = "../kavach"
```

The `path` override wins, so `cyrius deps` materialized the **local working tree at 3.9.3**, not the
3.7.0 the manifest names. `lib/kavach.cyr` is byte-identical to `/home/macro/Repos/kavach/dist/kavach.cyr`.

That means a consumer's vendored dependency silently tracks whatever a sibling checkout happens to
contain, and the declared tag documents an intention that is not enforced. The consumer's build was
working on 2026-07-25 and stopped working with no change to the consumer — which is the failure mode
worth naming: **the pin did not hold, and nothing said so.** Filed here because it is the same
incident; it belongs to the consumer to decide (drop the path override for releases, or bump the tag
to match reality).
