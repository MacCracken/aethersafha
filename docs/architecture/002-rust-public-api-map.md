# 002 — Where the Rust original's public API lives now

> **As of 0.17.6**, the release that deletes `rust-old/`. Written because the deletion gate asked for it
> (roadmap § 0.17.x, gate item 3): once the directory is gone, the question "where did `X` go?" has no
> answer in the tree unless one is written down. Tag **`0.17.5`** is the last that carries the Rust
> original; read any file with `git show 0.17.5:rust-old/<path>`. A citation `rust-old/<path>[:<line>]`, or
> a bare `<name>.rs:<line>` in a parity comment, means that path at that tag. Every Cyrius name below was
> checked against `src/` when this page was written. **Where this page and the code disagree, the code is
> right and this page is the bug.**

The port has no `pub`: Cyrius has no module visibility, so a "public type" becomes a heap object (a
named-offset `enum` plus a `*_new` constructor and accessors), a plain `enum`, or an `i64` code. ADR 0003
lists the representation mappings (`Uuid` → `i64`, `f32` → permille, `Option` → a sentinel, error enums →
`i64` codes, getters hand out the live object); they apply to every row below without being repeated.

## 1. The modules (`lib.rs:3-19`)

Lines are `wc -l` of the Rust file; tests count `#[test]` and `#[tokio::test]`. The closing column is the
parity matrix's verdict, kept here because the matrix (`docs/development/parity-plan.md`) is deleted with
`rust-old/`.

| `lib.rs` | Lines / tests | Cyrius home | Closed |
|---|---|---|---|
| `pub mod accessibility` | 729 / 30 | `src/accessibility.cyr` | 0.17.2, ported; mirrored live. `KeyboardNavConfig` retired (ADR 0004) |
| `mod ai_features` | 1707 / 86 | `src/ai_features.cyr`, `src/daimon_client.cyr` (the agent-list poll) | 0.17.5, ported |
| `mod apps` | 2986 / 173 | — | 0.17.0, retired (ADR 0004); `src/apps.cyr` deleted |
| `mod compositor` | 2565 / 99 | `src/compositor.cyr`, `src/window.cyr`, `src/geom.cyr`, `src/input.cyr`, `src/route.cyr` | 0.17.4, ported |
| `pub mod gestures` | 569 / 17 | `src/gestures.cyr` | 0.17.5, ported; dormant (ADR 0004) |
| `pub mod hud` | 1987 / 67 | — | 0.17.0, retired (ADR 0004, which keeps the widget specs); the agent-list poll is ported under `ai_features` |
| `pub mod plugin_host` | 848 / 31 | `src/plugin_host.cyr` | 0.17.3, ported |
| `pub mod renderer` | 1656 / 49 | `src/render.cyr`, `src/window.cyr`, `src/desktop.cyr`, `src/shell_render.cyr`, `src/wallpaper.cyr`, `src/theme.cyr`, `src/cursor.cyr` | 0.17.4, ported |
| `pub mod screen_capture` | 1299 / 31 | `src/screen_capture.cyr` (library code, not linked into the binary) | 0.17.3, ported |
| `pub mod screen_recording` | 938 / 22 | `src/screen_recording.cyr` (library code, not linked) | 0.17.3, ported |
| `mod security_ui` | 1858 / 77 | `src/security_ui.cyr` | 0.17.3, ported; the cgroup removal retired (ADR 0004) |
| `mod shell` | 1396 / 60 | `src/shell.cyr`, `src/shell_render.cyr` | 0.17.5, ported; quick settings retired (ADR 0004) |
| `pub mod shell_integration` | 516 / 26 | — | 0.17.0, retired (ADR 0004) |
| `mod system_tests` | 1477 / 40 | `tests/system.tcyr` and the module suites | 0.17.5, re-expressed; the apps, HUD-overlay and quick-settings scenarios close by ADR 0004 |
| `pub mod theme_bridge` | 372 / 18 | — | 0.17.0, retired (ADR 0004); `rupa` is the theme source |
| `pub mod wayland` | 4763 / 181 | setu (`src/setu_server.cyr`, `src/setu_dispatch.cyr`, the `setu` dependency) | 0.17.0, refused (ADR 0001) and retired with its inventory (ADR 0002) |
| `pub mod xwayland` | 823 / 34 | mehman (`src/foreign.cyr`, the `mehman` dependency) | 0.17.0, refused (ADR 0001); what mehman owes is in ADR 0002 |

Outside `lib.rs`: the binary `main.rs` (630 / 26) is `src/main.cyr`, `src/desktop.cyr`, `src/sysprobe.cyr`,
`src/flags.cyr` and `src/sigint.cyr`, closed at 0.17.2. `tests/*.rs` (422 / 29):
`compositor_integration.rs` (13) is in `tests/compositor.tcyr` and `tests/route.tcyr`,
`renderer_integration.rs` (12) in `tests/render.tcyr`, and `security_integration.rs` (4) is covered by
`tests/security_ui.tcyr`'s show / dismiss alert, request / grant / deny, human override and agent
permissions groups, which name the module tests they port rather than these four. That is 1,096 Rust tests
in all. The retired rows' tests close by ADR; every test of every ported row is re-expressed in a `.tcyr`
suite under ADR 0003's mappings.

## 2. Every `pub use` (`lib.rs:21-88`)

### `compositor` (`lib.rs:21-24`)

| Rust | Cyrius |
|---|---|
| `Compositor` | `comp_new` and the `CompOff` record, `src/compositor.cyr` |
| `CompositorBackend` (trait: initialize, handle_input, render, shutdown) | ADR 0003, substrate: the `bhumi` backend; the frame loop in `src/main.cyr` |
| `CompositorError` | ADR 0003, error enums → `i64` codes: the `comp_*` calls return 0 or −1, for an unknown window (`WindowNotFound`) or an invalid workspace (Rust's `DisplayServerError` in `switch_workspace` and `get_workspace_context`). `PermissionDenied` was only ever constructed in a test |
| `ContextType` | `enum CtxType` (`CTX_*`), `src/compositor.cyr`; `comp_workspace_context`, `comp_set_workspace_context` |
| `InputAction` (the routing result) | `enum RouteAct` (`RA_*`) and the outcome list `route_out_*`, `src/route.cyr` (`route_pointer`, `route_key`). ⚠ `src/input.cyr`'s own `enum InputAction` is the port's keyboard chrome actions (`IA_*`), not this type |
| `InputEvent` | bhumi's event records (motion, button, scroll, key), fed to `route_pointer` / `route_key`. `TouchEvent` is retired (ADR 0004, `Touch*` input events) |
| `Rectangle` | `rect_new` and `enum RectLayout`, `src/geom.cyr` |
| `SurfaceId` (= `Uuid`) | an `i64` window id, `win_id`, from 1 (ADR 0003) |
| `WaylandBackend` | ADR 0003 (substrate row) and ADR 0002: it bridged the Wayland protocol layer, which is retired |
| `Window` | `win_new` and the `WinOff` record, `src/window.cyr` |
| `WindowState` | `enum WinState` (`WS_*`), `src/window.cyr`; `comp_set_window_state` refuses FULLSCREEN (ADR 0002) and FLOATING (ADR 0004) |
| `Workspace` | `wsp_new` and the `WspOff` record, `src/compositor.cyr` |

Test-only `ClipboardManager` and `tile_windows` are retired (ADR 0004).

### `shell` (`lib.rs:26-29`)

| Rust | Cyrius (`src/shell.cyr`) |
|---|---|
| `AppCategory` | `enum SH_AppCategory` |
| `AppEntry` | `shell_app_new`, `SH_AppOff`; the registry is `shell_registry_put` / `_get` / `_count` |
| `DesktopShell` | `shell_new`, `SH_ShellOff`; owned by the desktop aggregate (`desktop_new`, `src/desktop.cyr`) |
| `LauncherAction` | `shell_action_new`, `SH_ActionOff`, `enum SH_LauncherActionKind` |
| `LauncherItem` | `shell_item_new`, `SH_ItemOff` |
| `NetworkStatus` | `enum SH_NetStatus`, plus the port's `SH_NET_UNKNOWN` (ADR 0003) |
| `Notification` | `shell_notif_new`, `SH_NotifOff`; drawn by `render_notifications`, `src/shell_render.cyr` |
| `NotificationId` (= `Uuid`) | a monotonic `i64` |
| `NotificationPriority` | `enum SH_NotifPriority` |
| `QuickSetting` | retired (ADR 0004) |
| `ShellError` | `enum SH_Err` |
| `SystemStatus` | `shell_status_new`, `SH_StatusOff`; fed by `src/sysprobe.cyr`, drawn by `render_shell_panel` |

### `ai_features` (`lib.rs:31-35`)

| Rust | Cyrius (`src/ai_features.cyr`) |
|---|---|
| `AIDesktopFeatures` | `ai_new`, `AI_FeatOff` |
| `AIFeatureError` | `enum AI_FeatureError` |
| `AISuggestion` | `ai_suggestion_new`, `AI_SugOff` |
| `ActivityLevel` | `enum AI_ActivityLevel` |
| `AgentHUDState` | `ai_hud_new`, `AI_HudOff`; fed by the agent-list poll (`src/daimon_client.cyr`, ADR 0003) |
| `AgentStatus` | `enum AI_AgentStatus` |
| `ContextEvent` | `ai_event_new`, `AI_EvtOff` |
| `ContextEventType` | `enum AI_ContextEventType` |
| `ContextType as AIContextType` | `enum AI_ContextType` (`AI_CT_*`), distinct from the compositor's `CtxType` as in Rust |
| `DesktopContext` | `AI_CtxOff`, `ai_context_default` and the `ai_context_*` accessors |
| `ResourceMetrics` | `ai_res_new`, `AI_ResOff` |
| `SuggestionType` | `enum AI_SuggestionType` |
| `TimeOfDay` | `enum AI_TimeOfDay` |

### `apps` (`lib.rs:37-41`)

`AgentInfo`, `AgentManagerApp`, `AppError`, `AppType`, `AppWindow`, `AuditEntry`, `AuditFilters`,
`AuditViewerApp`, `DesktopApplications`, `FileManagerApp`, `ModelInfo`, `ModelManagerApp`, `TerminalApp`,
`TimeRange`: **all retired by ADR 0004** (the app layer). Every window is a real client process now:
puka is the terminal, crab the file manager, and the launcher (`src/launcher.cyr`) starts them.

### `renderer` (`lib.rs:43-46`)

| Rust | Cyrius |
|---|---|
| `DamageTracker` | `rend_dmg_new` and the `rend_dmg_*` box, plus the damage band (`rend_band_*`), `src/render.cyr`. Rust's region list with `flush` / `region_count` / `has_damage` closes with ADR 0003's damage-band row |
| `DecorationHit` | `enum Deco` (`DECO_*`), `src/window.cyr`; `deco_hit`, `src/render.cyr` |
| `DesktopRenderer` | no object (ADR 0003, small representation notes): a bhumi framebuffer, the kernel's `#84` flip, and `render_desktop` (`src/desktop.cyr`) over `render_window` (`src/render.cyr`) |
| `Framebuffer` | bhumi's framebuffer (`bhumi_fb_new`, `lib/bhumi.cyr`), XRGB8888 (ADR 0003); a zero size is refused |
| `Layer` | retired (ADR 0004, renderer remnants); paint order is the window vector (ADR 0003) |
| `Pixel` (ARGB `u32`) | an XRGB8888 word (ADR 0003); capture writes alpha 0xFF |
| `ResizeEdge` | the grips `DECO_RESIZE_R` / `_B` / `_BR` / `_L` / `_BL` in `enum Deco`; T / TL / TR retired (ADR 0004) |
| `SceneGraph` | the compositor's window vector (`comp_windows`), ordered with raise-on-focus (ADR 0003) |
| `SceneSurface` | the window's client buffer (`win_pix`, `win_set_client_buf`) and opacity (`win_opacity`), `src/window.cyr` |

Hit-testing a fullscreen window returns with fullscreen, which setu owes (ADR 0002). `render_hud_overlay`
and `blit_clipped` are retired (ADR 0004).

### `security_ui` (`lib.rs:48-51`)

| Rust | Cyrius (`src/security_ui.cyr`) |
|---|---|
| `AgentPermission` | `sec_agentperm_new`, `SecAgentPermOff` |
| `OverrideRequest` | `sec_override_new`, `SecOverrideOff` |
| `PermissionCategory` | `enum SecPermCategory` |
| `PermissionDefinition` | `sec_permdef_new`, `SecPermDefOff` |
| `PermissionRequest` | `sec_permreq_new`, `SecPermReqOff` |
| `SecurityAlert` | `sec_alert_new`, `SecAlertOff` |
| `SecurityDashboard` | `sec_dash_new`, `SecDashOff`; `sec_ui_get_security_dashboard` |
| `SecurityLevel` | `enum SecLevel` |
| `SecurityUI` | `sec_ui_new`, `SecUIOff` |
| `SecurityUIError` | `enum SecErr` |
| `ThreatLevel` | `enum SecThreatLevel` |

The module's other public types, not re-exported by `lib.rs`, are ported too: `TrustLevel`,
`SandboxStatus` and `BadgeIcon` (`enum SecTrustLevel`, `SecSandboxStatus`, `SecBadgeIcon`),
`WindowOwnerBadge` (`sec_badge_new`) and `WindowBadgeManager` (`sec_badge_mgr_new`).

### `plugin_host` (`lib.rs:53-56`)

| Rust | Cyrius (`src/plugin_host.cyr`) |
|---|---|
| `PluginCapability` | `enum PLG_Capability` |
| `PluginHost` | `plug_host_new`, `PLG_HostOff` |
| `PluginHostError` | `enum PLG_Err` |
| `PluginInfo` | `plug_info_new`, `PLG_InfoOff` |
| `PluginMessage` | `enum PLG_MsgKind` and `PLG_MsgOff`; serde's JSON through `plug_msg_to_json` / `plug_msg_from_json` (ADR 0003) |
| `PluginSandboxProfile` | `plug_sandbox_new`, `PLG_SandboxOff` (data only, as in Rust) |
| `PluginState` | `enum PLG_State` |
| `PluginType` | `enum PLG_Type` |

### `accessibility` (`lib.rs:58-61`)

| Rust | Cyrius (`src/accessibility.cyr`) |
|---|---|
| `AccessibilityRole` | `enum AxRole` |
| `AccessibilityState` | `a11y_state_new`, `AxStateOff` (an `Option<bool>` is `enum AxTristate`) |
| `AccessibilityTree` | `a11y_tree_new`, `AxTreeOff`; owned by the compositor and mirrored live (ADR 0003) |
| `AccessibleAction` | `enum AxAction` |
| `AccessibleNode` | `a11y_node_new`, `AxNodeOff` |
| `HighContrastTheme` | `AxThemeOff`, `a11y_theme_make`, `a11y_theme_default_high_contrast`, `a11y_theme_default_dark_high_contrast` |
| `KeyboardNavConfig` | retired (ADR 0004) |

### `shell_integration` (`lib.rs:63-67`)

`ExternalNotification`, `NotificationBridge`, `ShellIntegrationError`, `ShellIntegrationManager`,
`SystemTrayItem`, `TrayAction`, `TrayMenuItem`, `Urgency`, `WindowManagementRequest`,
`WindowManagementResult`: **all retired by ADR 0004**. If they return, it is as setu message kinds.

### `theme_bridge` (`lib.rs:69-72`)

`FlutterThemeData`, `PlatformChannelMessage`, `ThemeBridge`, `ThemeOverrides`, `color_hex_to_u32`,
`color_u32_to_hex`: **all retired by ADR 0004**. There is no Flutter, and `rupa` is the theme source.

### `screen_capture` (`lib.rs:74-77`)

| Rust | Cyrius (`src/screen_capture.cyr`) |
|---|---|
| `CaptureError` | `enum ScapErr` |
| `CaptureFormat` | `enum ScapFmt` |
| `CaptureHistoryEntry` | `scap_hist_new`, `ScapHistOff` |
| `CaptureId` (= `Uuid`) | a monotonic `i64` |
| `CapturePermission` | `scap_perm_new`, `ScapPermOff` |
| `CaptureResult` | `scap_result_new`, `ScapResultOff`; freed by `scap_result_free` |
| `CaptureTarget` | `scap_target_new`, `ScapTargetOff` |
| `CaptureTargetKind` | `enum ScapKind` |
| `ScreenCaptureManager` | `scap_mgr_new`, `ScapMgrOff`; `scap_capture_full_screen` / `_window` / `_region` read a framebuffer the caller passes (ADR 0003) |

### `screen_recording` (`lib.rs:79-82`)

| Rust | Cyrius (`src/screen_recording.cyr`) |
|---|---|
| `RecordedFrame` | `srec_frame_new`, `SrecFrameOff` |
| `RecordingConfig` | `srec_cfg_new`, `srec_cfg_default`, `SrecCfgOff` |
| `RecordingError` | `enum SrecErr` |
| `RecordingId` (= `Uuid`) | a monotonic `i64` |
| `RecordingSession` | `srec_sess_new`, `SrecSessOff` |
| `RecordingState` | `enum SrecState` |
| `ScreenRecordingManager` | `srec_mgr_new`, `SrecMgrOff`, and the `srec_mgr_*` calls |

### `hud` (`lib.rs:84-88`)

`crew_status::{CrewEntry, CrewRunStatus, CrewStatusRenderData, CrewStatusWidget}`,
`domain_filter::{DomainAgentEntry, DomainFilterRenderData, DomainFilterWidget, DomainGroup}` and
`gpu_status::{GpuDeviceState, GpuStatusRenderData, GpuStatusWidget, MetricBand}`: **all retired by ADR
0004**, which keeps each widget's data source, parsing rules and thresholds, and what changed in daimon
since.

### `gestures`, `wayland`, `xwayland` (public modules with no `pub use`)

- `gestures` (`src/gestures.cyr`): `TouchPoint` → `gest_point`; `SwipeDirection` → `enum G_SwipeDir`;
  `GestureType` → `enum G_GestureKind`; `GestureEvent` → `gest_event_new`; `GestureConfig` →
  `gest_config_new`; `TouchTracker` → `gest_tracker_new`; `GestureRecognizer` → `gest_recognizer_new`.
  Distances are integer pixels and angles milliradians (ADR 0003).
- `wayland` and `xwayland`: nothing is ported. ADR 0002 is the inventory of what they carried: what setu
  covers natively, what is dropped, what is owed a native design and what mehman owes.

## 3. `examples/`

| Rust | Where it is now |
|---|---|
| `examples/headless_compositor.rs` (65 lines) | `t_headless` in `tests/compositor.tcyr` (0.17.4), as assertions: three windows at 1920×1080, the browser maximized, the active and agent windows, focus, and every window closed in creation order, plus the teardown edge cases. Its `render_to_vec` step has no counterpart in that test: rendering a desktop frame is covered by `tests/render.tcyr`, `tests/desktop.tcyr` and the `renderer/render_frame/*` benches. Its `tracing` output and `println!` listing are not carried over. |

## 4. The Rust build files

| Rust | Replaced by, or why it is gone |
|---|---|
| `Cargo.toml` | `cyrius.cyml`: name, version (`${file:VERSION}`), license, the toolchain pin and `[deps.*]`. Its features (`wayland`, `gpu` = `mabda`) are gone: Wayland is refused (ADR 0001) and the GPU path is the agnos band by direct syscall (ADR 0003). Its crates have no one-for-one counterpart: the Cyrius stdlib (with `bayan` for JSON) and the declared deps (bhumi, rupa, kashi, mehman, kavach, chitra, setu) do that work. |
| `Cargo.lock` | `cyrius.lock`, which pins each dep's commit. Two Rust versions are still load-bearing, and the code that matches them cites them: serde 1.0.228 / serde_json 1.0.149 (`src/plugin_host.cyr`, `tests/plugin_host.tcyr`) and chrono 0.4.44 (`src/ai_features.cyr`, `tests/ai_features.tcyr`). `src/security_ui.cyr` cites `Cargo.toml` / `Cargo.lock` for serde_json's missing `preserve_order` feature, which is why the audit line's keys are sorted. |
| `Makefile` | `cyrius` subcommands: `cargo fmt --check` → `cyrius fmt --check`; `cargo test` → `cyrius test`; `cargo build` → `cyrius build` (and `cyrius build --agnos`); `cargo llvm-cov` → `cyrius coverage --min`; `bench` → `scripts/bench-history.sh` on `cyrius bench`. CI (`.github/workflows/ci.yml`) gates the format check, build, test and coverage floor. `clippy`, `cargo audit` and `cargo doc` have no counterpart. |
| `deny.toml` | `scripts/check-dep-tags.sh` (each dep's bytes against its tag) and `cyrius deps --verify` (the vendored `lib/` against the lock). The crates.io license and advisory checks have no crates to check. |
| `codecov.yml` (80 % project and patch) | CI's **Coverage floor** step (`cyrius coverage --min`), with the v1.0 criterion of 80 % in `docs/development/roadmap.md`. |
| `rust-toolchain.toml` (stable, rustfmt, clippy) | `[package].cyrius` in `cyrius.cyml`. |
| `LINES_OF_RUST.txt` (`27207`) | `README.md` and `docs/development/state.md`, with its definition: 27,207 lines of `src/**/*.rs`, 28,072 counting `tests/`, `benches/` and `examples/` (recounted at 0.17.5). |
| `bench-history.csv`, `benchmarks.md`, `benches/*.rs` | `docs/benchmarks/rust-a460c3a.csv` (byte-identical), `docs/benchmarks.md` (Rust's table, as a record), `tests/compositor_bench.bcyr` and `tests/screen_capture_bench.bcyr` (0.17.5). |
| `.gitignore`'s Cargo lines (`/target/`, `/target/criterion/`, `*.rs.bk`, the Rust coverage artefacts) | removed at 0.17.6. |
