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
| `compositor.rs` | 2565 / 99 | `compositor`, `window`, `geom`, `input`, routing in `main` | driven | PORT | routing + drag/resize out of `main.cyr` with its 9 tests; workspaces (4 defaults, queries, context setter); L / BL resize grips; lifecycle and workspace tests. (0.17.1: close updates `WSP_ACTIVE`; close and move drop every entry, as `retain` did) | .4 .5 |
| `main.rs` | 630 / 26 | `main`, `desktop`, `sysprobe`, `flags`, `sigint` | driven | ✅ **CLOSED 0.17.2** — the strict flag parser with `--secure` / `--help` / `--version`, the welcome notice, the 5 s tick with `UserPresent`, cpu / mem / disk as Rust read them, SIGINT through the Ctrl+Q teardown; divergences in ADR 0003 | — | .2 |
| `renderer.rs` | 1656 / 49 | `render`, `window`, `desktop`, `shell_render`, `wallpaper`, `theme`, `cursor` | driven | PORT | CPU premultiplied blend + opacity (CPU frames differ from iron today); border outline; tests for overlap and minimized-not-drawn. (0.17.1: title truncation; the dead asserts run. 0.17.2: UTF-8 text, text width in scalars, the `…` glyph) | .4 .5 |
| `shell.rs` | 1396 / 60 | `shell`, `shell_render` | panel + cards painted | PORT | 6 missing + 4 partial tests; `toggle_panel` honoured. (0.17.1: the registry's `terminal` entry; launch and search tests. 0.17.2: the net dot reads UNKNOWN, the agent count is fed, cards cut to their width) | .5 |
| `accessibility.rs` | 729 / 30 | `accessibility` | mirrored live | ✅ **CLOSED 0.17.2** — a node and an announcement on create / close, focus sync, an idempotent reconcile, the 3 tests; divergences in ADR 0003 | — | .2 |
| `ai_features.rs` | 1707 / 86 | `ai_features` | constructed only | PORT | `poll_agents` against daimon 2.4.3; 13 tests. (0.17.1: the `"terminal"` context match; stable suggestion sort; null-checked hour) | .3 |
| `security_ui.rs` | 1858 / 77 | `security_ui` | constructed only | PORT | audit-log writer; SIGHUP / SIGKILL; Linux-only kill effects; 6 tests incl. 4 eviction. (0.17.1: the dashboard maximum over active alerts; minted ids from 2⁶²+1, ADR 0003) | .3 |
| `plugin_host.rs` | 848 / 31 | `plugin_host` | constructed only | PORT | `PluginMessage` JSON (both directions `return 0`) and its 8 tests; delete the `plug_ipc_*` stubs (Rust had no transport) | .3 |
| `screen_capture.rs` | 1299 / 31 | `screen_capture` (not linked) | library | PORT as library | alpha 0xFF; refuse w / h ≤ 0 (a null write today); decode round-trip over 64 KB; per-capture memory | .3 |
| `screen_recording.rs` | 938 / 22 | `screen_recording` (not linked) | library | PORT as library | the ring frees what it evicts | .3 |
| `gestures.rs` | 569 / 17 | `gestures` | constructed only | KEEP dormant (ADR 0004) | 1 test (`touch_point_fields`); the rotate wrap and the floor-sqrt boundary are recorded in ADR 0003 | .5 |
| `apps.rs` | 2986 / 173 | — | dead in Rust too | ✅ **CLOSED 0.17.0** — retired by ADR 0004, `src/apps.cyr` deleted | — | .0 |
| `theme_bridge.rs` | 372 / 18 | — | dead | ✅ **CLOSED 0.17.0** — retired by ADR 0004, deleted with `DSK_THEME` | — | .0 |
| `shell_integration.rs` | 516 / 26 | — | one tray item at boot | ✅ **CLOSED 0.17.0** — retired by ADR 0004, deleted with the desktop tray | — | .0 |
| quick settings, `KeyboardNavConfig` (in `shell.rs`, `accessibility.rs`) | — | — | never read | ✅ **CLOSED 0.17.0** — retired by ADR 0004, deleted | — | .0 |
| `hud/*` | 1987 / 67 | — | never ported | ✅ **CLOSED 0.17.0** — retired by ADR 0004, which keeps the widget specs | the agent-list poll is ported under `ai_features.rs` | .0 .3 |
| `wayland/*` | 4763 / 181 | setu | — | ✅ **CLOSED 0.17.0** — ADR 0001 + ADR 0002 (what is dropped, what is owed a native design) | — | .0 |
| `xwayland.rs` | 823 / 34 | mehman, `foreign` | — | ✅ **CLOSED 0.17.0** — ADR 0001 + ADR 0002 (what mehman owes) | — | .0 |
| `system_tests.rs` | 1477 / 40 | — | tests | re-express | 24 covered · 13 partly · 3 not | .5 |
| `tests/*.rs` | 422 / 29 | — | tests | re-express | compositor 7 / 13 covered, renderer 6 / 12, security 4 / 4 | .5 |
| benches, `bench-history.csv`, `benchmarks.md` | 378 / — | `tests/aethersafha.bcyr` | — | PRESERVE the CSV | the CSV moves to `docs/benchmarks.md` ("machine unknown, not a target"); Cyrius benches for the same series | .5 |
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

As of 2026-10-09, for the PORT rows. Names are Rust's with any `test_` / module prefix dropped, so grep
for the suffix; *partly* means a `.tcyr` suite covers the behaviour but not every assertion the Rust test
made. Tests that only check enum variants, Debug or Clone output,
or error text close under ADR 0004 and are not listed.

- **compositor.rs** —
  lifecycle: `close_nonexistent_window`, `close_window_double_close_fails`,
  `close_middle_window_preserves_others`, `create_multiple_windows_tracking`;
  state: `set_window_state_nonexistent`, `set_all_window_states`, `window_state_transitions_full_cycle`;
  workspaces: `has_4_default_workspaces`, `move_nonexistent_window`, `move_window_to_same_workspace`,
  `move_window_to_last_workspace`, the four `get_active_windows*`, `windows_across_workspaces`,
  `workspace_active_window_after_multiple_creates`, and the four workspace-context tests;
  focus: `close_updates_focus`, `cascade_placement`, `with_resolution`;
  routing, nine tests: the `route_input_*` set, `drag_workflow`, `end_interactive_clears_state`,
  `mouse_move_no_drag`, `right_click_on_window`.
- **tests/compositor_integration.rs** — `close_nonexistent_window_fails`,
  `workspace_switching_hides_windows`, `closing_focused_window_moves_focus`, `custom_resolution`,
  `many_windows_cascade`; partly `window_appears_in_active_workspace`, `window_state_transitions`,
  `input_routing_click_on_empty_is_none`.
- **renderer.rs** — `blend_semi_transparent` (input rule changes with the CPU premultiplied blend),
  framebuffer `new` / `out_of_bounds`, `draw_rect_outline`, hit-test
  `fullscreen` (returns with fullscreen, ADR 0002) and `resize_edge_variants`, `overlapping_surfaces`,
  `z_order`. Damage `flush` / `region_count` / `has_damage` close with the region list (ADR 0003).
- **ai_features.rs** — `update_context_command_executed`, `dismiss_nonexistent_suggestion`,
  `unregister_nonexistent_agent_no_panic`, `desktop_context_equality`; partly
  `ai_suggestion_custom`, `resource_metrics`, the three `analyze_context*`,
  `update_context_file_opened`, `_file_opened_cap_at_10`, `multiple_file_open_events_ordering`,
  `update_agent_hud_updates_last_activity`.
- **security_ui.rs** — `alert_eviction_at_max_capacity`, `alert_eviction_resolved_first`,
  `permission_request_eviction`, `override_request_eviction`; partly `grant_permission_lockdown`,
  `permission_definitions_initialized`.
- **plugin_host.rs** — the seven `message_*_roundtrip` and `message_from_invalid_json` (vacuous until
  the JSON stub is real: a parse failure and the stub both return 0).
- **shell.rs** — `notification_default`, `launcher_item`, `launcher_action`,
  `dismiss_multiple_notifications`, `notification_with_all_priorities`,
  `notification_eviction_prefers_non_action`; partly `notification_custom`, `multiple_notifications`,
  `lock_unlock_round_trip`, `update_system_status_all_fields`. `initial_system_status` is the UNKNOWN
  seed (ADR 0003).
- **gestures.rs** — `touch_point_fields`.
- **screen_capture.rs** — partly `full_screen_capture_png` (asserted on 4×4, not 800×600) and
  `grant_and_revoke_permission` (list length after revoke). **screen_recording.rs** — partly
  `one_recording_per_agent` (the existing recording's id is not returned).
- **system_tests.rs** (all `test_e2e_*` except the first) — `workspace_context_management`,
  `screen_lock_blocks_interactions`; partly `workspace_switch`, `multi_window_workspace_management`,
  `full_desktop_startup_sequence`, `full_teardown_sequence`, `security_and_ai_combined`,
  `notification_flood_and_dismiss`, `ai_context_with_multiple_agents`, `compositor_window_operations`,
  `multi_app_concurrent_operations`, `emergency_kill_switch_flow`, `system_status_propagation`. The apps,
  HUD and quick-settings tests close with ADR 0004.
