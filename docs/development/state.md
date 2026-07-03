# aethersafha — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**0.1.0** — ported from Rust (2026-07-03) via `cyrius port`. 27,207 lines of Rust
preserved at `rust-old/` as the frozen parity oracle.

## Toolchain

- **Cyrius pin**: `6.3.36` (in `cyrius.cyml [package].cyrius`)
- Build: `cyrius lib sync --full && cyrius deps && cyrius build src/main.cyr build/aethersafha`
  (the `lib sync --full` is required — the declared stdlib set exceeds the incremental pin).

## Source

- Rust reference: 27,207 lines at `rust-old/` (frozen, do not edit).
- Cyrius port: ~8,000 lines across 12 modules; compiles + runs on the bhumi seam.
  - **Core (M1)** — `geom`, `window`, `compositor`, `render` (software renderer over
    bhumi's XRGB framebuffer), `input` (bhumi HID → actions), `main` (backend →
    seed windows → poll/dispatch/render/present loop). Wired into the running binary.
  - **Leaf (M2)** — `theme_bridge`, `gestures`, `accessibility`, `ai_features`,
    `shell`, `security_ui`. Structural parity vs `rust-old/` (heap offset-accessor
    structs, prefixed symbols); compile individually + together (no collisions);
    smoke-tested. Standalone (not yet wired into the compositor). Deeper behavioral
    parity tests are follow-on.

## Tests

- `tests/aethersafha.tcyr` — **21 passed** (geom, window model, compositor CRUD).
- `tests/leaf_modules.tcyr` — **11 passed** (all 6 leaf modules coexist + construct).
- Run: `cyrius tests tests/` (or a single `cyrius test tests/<file>.tcyr`).

## Dependencies

Declared in `cyrius.cyml` (resolved into `lib/`):

- **stdlib** — syscalls, string, alloc, atomic, str, fmt, vec, slice, hashmap,
  fnptr, io, fs, process, args, ct, net, tagged, result, trait, bayan, chrono,
  math, assert, bench.
- **bhumi** 0.7.0 — platform backend (output/input/seat). Actively consumed.
- **mehman** 0.1.0 — foreign-surface backend (types-only). Wired, post-MVP.
- **agnostik** 1.3.2 — shared domain primitives. Wired.
- **agnodrm** 1.4.4 — udev/DRM device model (was `agnosys`). Wired.

Known: agnostik + agnodrm both bundle the shared `ERR_*` error module → benign
duplicate-symbol warnings ("last wins"). See roadmap "Known cleanup".

## Consumers

_None yet (top-level application, `publish = false`)._

## Next

M2 — leaf feature parity (shell, ai_features, security_ui, accessibility,
gestures, theme_bridge), driven by the parity workflow. See [`roadmap.md`](roadmap.md).
