# aethersafha — parity matrix

> **The gate for deleting `rust-old/`.** This is the 0.17.x arc's burn-down list; the sequence is
> [`roadmap.md`](roadmap.md) § 0.17.x. Measured 2026-10-09 against the code by seven read-only audits, one
> per module group; every defect named here was re-read before it was written down. A row is **CLOSED**
> when its Rust behaviour is ported and its Rust tests are re-expressed in `.tcyr`, or when an ADR retires
> it. This page replaces the post-0.1.0 bite plan that used to live here, whose bites are done or
> superseded. It is deleted with `rust-old/`; the ADRs carry what outlives it.

## The fact that sizes the arc

**The Rust binary never created a window or rendered a frame.** `main.rs` built `DesktopEnvironment`, ran a
5 s status tick (`main.rs:136-185`), honoured `--secure`, and exited on a signal. Compositor depth, the
renderer, apps, capture and the HUD were library code reached only by tests. So parity is **library
behaviour plus that tick**. A module that was library-only in Rust and is library-only here is at parity;
wiring it into the running desktop is new work and is not part of the deletion gate.

## Matrix

| Rust | Lines / tests | Cyrius home | At runtime | Verdict | Open before CLOSED | Bite |
|---|---|---|---|---|---|---|
| `compositor.rs` | 2565 / 99 | `compositor`, `window`, `geom`, `input`, `route` | driven | ✅ **CLOSED 0.17.4** — routing and drag / resize in `src/route.cyr` with Rust's 9 routing tests, four default workspaces with the queries and context setter, the L / BL grips, the lifecycle / state / workspace / focus tests; divergences in ADR 0003 | — | .1 .4 |
| `main.rs` | 630 / 26 | `main`, `desktop`, `sysprobe`, `flags`, `sigint` | driven | ✅ **CLOSED 0.17.2** — the strict flag parser with `--secure` / `--help` / `--version`, the welcome notice, the 5 s tick with `UserPresent`, cpu / mem / disk as Rust read them, SIGINT through the Ctrl+Q teardown; divergences in ADR 0003 | — | .2 |
| `renderer.rs` | 1656 / 49 | `render`, `window`, `desktop`, `shell_render`, `wallpaper`, `theme`, `cursor` | driven | ✅ **CLOSED 0.17.4** — the CPU blend is the kernel's `#92` arithmetic (CPU and GPU frames word-identical), window opacity, the border, the overlap / z-order / minimized tests. `hit-test fullscreen` returns with fullscreen (ADR 0002). (0.17.1: title truncation. 0.17.2: UTF-8 text) | — | .1 .2 .4 |
| `shell.rs` | 1396 / 60 | `shell`, `shell_render` | panel + cards painted | ✅ **CLOSED 0.17.5** — every Rust test re-expressed. `toggle_panel` flips the flag as Rust's did; Rust's renderer never read it either, so honouring it on screen is desktop work, not parity. (0.17.1: the `terminal` entry. 0.17.2: UNKNOWN net dot, fed agent count, cards cut to width) | — | .1 .2 .5 |
| `accessibility.rs` | 729 / 30 | `accessibility` | mirrored live | ✅ **CLOSED 0.17.2** — a node and an announcement on create / close, focus sync, an idempotent reconcile, the 3 tests; divergences in ADR 0003 | — | .2 |
| `ai_features.rs` | 1707 / 86 | `ai_features` | constructed only | ✅ **CLOSED 0.17.5** — every Rust test re-expressed (`desktop_context_equality` as a field-wise compare: the port has no `==`, ADR 0004's derive rule). (0.17.1: `"terminal"`, stable sort. 0.17.3: `poll_agents` against daimon 2.4.3) | — | .1 .3 .5 |
| `security_ui.rs` | 1858 / 77 | `security_ui` | constructed only | ✅ **CLOSED 0.17.3** — the audit-log writer, SIGHUP on revoke, SIGKILL and the daimon DELETE on kill, the eviction tests; the cgroup removal is retired (ADR 0004). (0.17.1: the dashboard maximum, minted ids) | — | .3 |
| `plugin_host.rs` | 848 / 31 | `plugin_host` | constructed only | ✅ **CLOSED 0.17.3** — `PluginMessage` JSON as serde writes and reads it, its 8 tests; the `plug_ipc_*` stubs deleted | — | .3 |
| `screen_capture.rs` | 1299 / 31 | `screen_capture` (not linked) | library | ✅ **CLOSED 0.17.3** — alpha 0xFF, w / h ≤ 0 refused, a chitra round-trip past 64 KB, memory freed | — | .3 |
| `screen_recording.rs` | 938 / 22 | `screen_recording` (not linked) | library | ✅ **CLOSED 0.17.3** — the ring frees what it evicts; `one_recording_per_agent` complete | — | .3 |
| `gestures.rs` | 569 / 17 | `gestures` | constructed only | ✅ **CLOSED 0.17.5** — dormant (ADR 0004); `touch_point_fields` re-expressed in integer pixels; the rotate wrap and the floor-sqrt boundary are in ADR 0003 | — | .5 |
| `apps.rs` | 2986 / 173 | — | dead in Rust too | ✅ **CLOSED 0.17.0** — retired by ADR 0004, `src/apps.cyr` deleted | — | .0 |
| `theme_bridge.rs` | 372 / 18 | — | dead | ✅ **CLOSED 0.17.0** — retired by ADR 0004, deleted with `DSK_THEME` | — | .0 |
| `shell_integration.rs` | 516 / 26 | — | one tray item at boot | ✅ **CLOSED 0.17.0** — retired by ADR 0004, deleted with the desktop tray | — | .0 |
| quick settings, `KeyboardNavConfig` (in `shell.rs`, `accessibility.rs`) | — | — | never read | ✅ **CLOSED 0.17.0** — retired by ADR 0004, deleted | — | .0 |
| `hud/*` | 1987 / 67 | — | never ported | ✅ **CLOSED 0.17.0** — retired by ADR 0004, which keeps the widget specs | the agent-list poll is ported under `ai_features.rs` | .0 .3 |
| `wayland/*` | 4763 / 181 | setu | — | ✅ **CLOSED 0.17.0** — ADR 0001 + ADR 0002 (what is dropped, what is owed a native design) | — | .0 |
| `xwayland.rs` | 823 / 34 | mehman, `foreign` | — | ✅ **CLOSED 0.17.0** — ADR 0001 + ADR 0002 (what mehman owes) | — | .0 |
| `system_tests.rs` | 1477 / 40 | — | tests | ✅ **CLOSED 0.17.5** — every e2e test re-expressed in `tests/system.tcyr` or a module suite; the apps, HUD-overlay and quick-settings e2e tests close with ADR 0004 | — | .5 |
| `tests/*.rs` | 422 / 29 | — | tests | ✅ **CLOSED 0.17.5** — compositor 13 / 13, renderer 12 / 12 (damage on the band, the blend on the kernel's rule, ADR 0003), security 4 / 4 | — | .4 .5 |
| benches, `bench-history.csv`, `benchmarks.md` | 378 / — | `tests/compositor_bench.bcyr`, `tests/screen_capture_bench.bcyr` | — | ✅ **CLOSED 0.17.5** — Rust's CSV kept as `docs/benchmarks/rust-a460c3a.csv` ("machine unknown, a record"), Cyrius benches for every series beside it in `docs/benchmarks.md`, `scripts/bench-history.sh` on `cyrius bench` | — | .5 |
| `lib.rs`, `examples/`, Cargo / Makefile / deny / codecov / toolchain | — | — | — | OBSOLETE | map every `pub use` in `lib.rs:21-88` to its Cyrius home or its ADR, then drop | .6 |

There are 1,096 Rust tests. The retired rows' tests (181 in `wayland/`, 34 in `xwayland.rs`, and those of
`apps`, `theme_bridge`, `shell_integration` and the HUD) close by ADR. The PORT rows' open tests are listed
by name under *Rust tests still to re-express* below.

## The ADRs (written at 0.17.0)

- [ADR 0002](../adr/0002-retire-the-rust-protocol-layer.md) retires the Rust protocol layer and lists what it
  carried: covered natively, dropped, owed a native design (setu), owed by mehman, never present.
- [ADR 0003](../adr/0003-the-ports-divergences-from-rust.md) records the divergences that stay: substrate,
  representation, and accepted behaviour. A port row's Rust tests are re-expressed under its mappings.
- [ADR 0004](../adr/0004-the-retired-surface.md) records the retired surface, with the HUD widget specs and
  what changed in daimon. Every CLOSED-by-retirement row above cites it.

## Rust tests still to re-express

✅ **Empty since 0.17.5**: every Rust test of every PORT row is re-expressed in a `.tcyr` suite, under ADR
0003's mappings, each naming its Rust test in a comment. Tests that only check enum variants, Debug or
Clone output, or error text close under ADR 0004. What is left closes by ADR, not by a test:

- **renderer.rs** — hit-test `fullscreen` returns with fullscreen (ADR 0002). Damage `flush` /
  `region_count` / `has_damage` close with the region list (ADR 0003).
