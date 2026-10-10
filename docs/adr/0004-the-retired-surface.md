# 0004 — The retired surface: what the port does not carry over from Rust

**Status**: Accepted
**Date**: 2026-10-09

## Context

The 0.17.x arc closes Rust parity and then deletes `rust-old/`. The 2026-10-09 parity audit established
that **the Rust binary never created a window or rendered a frame**. `main.rs` built `DesktopEnvironment`,
ran a 5 s status tick, honoured `--secure`, and exited on a signal. Much of the Rust tree was library code
reached only by tests. Several of those modules have since been superseded by the sovereign desktop's real
programs, and some had no reader in either codebase. The operator decided which of them retire
(2026-10-09). This ADR is the record, so their Rust tests close here instead of being re-expressed.

## Decision

**These are retired: not ported, and where they had been ported, deleted at 0.17.0.**

| Retired | Reason | If it comes back |
|---|---|---|
| **The app layer** (`apps.rs`): `DesktopApplications`, `AppWindow`, `open_*`, FileManager, the Command Palette, 8 web-browser launchers and Shruti, the Agent / Audit / Model managers. Ported as `src/apps.cyr`, never linked into the binary; deleted. | Every window on the desktop is a real process now: puka, crab and thoth are separate setu clients, started by the launcher. `open_*` made window records with no surface, rendering or input, in Rust too. The browsers launched Wayland clients with Wayland environment variables into a compositor that does not speak Wayland. | As client programs. The managers' filesystem, socket and HTTP effects belong in a process, not in the compositor's frame loop. |
| **`theme_bridge`** (AGNOS → Flutter theme translation). Deleted with `DSK_THEME` and `desk_bg_color`. | There is no Flutter. `rupa` is the theme source for the compositor, dhancha and every client; the high-contrast palette becomes a rupa variant. | — |
| **`shell_integration`**: tray items, window-management requests, external notifications. Deleted with the desktop tray. | Rust's `process_window_request` never reached a compositor, and agnos has no tray / StatusNotifier source. The one tray item at boot was never drawn. | As setu message kinds. |
| **Quick settings** (`shell.rs`) and **`KeyboardNavConfig`** (`accessibility.rs`). Deleted. | No backing service and no reader in either codebase; the Rust callbacks only logged. | With a service to toggle and a reader. |
| **The HUD widgets** (`hud/{crew_status,gpu_status,domain_filter}.rs`, 1,987 lines, 67 tests). Never ported. | Never used in Rust (`lib.rs` re-exported them; nothing polled), and the daimon they polled has changed under them (below). | Only if daimon gains the tools again. The agent-list poll IS ported (0.17.3), against daimon 2.4.3, to feed the panel's agent gauge. GPU device status belongs in `gpu.cyr`'s local probe. |
| **Test-only Rust features**: `ClipboardManager`, `tile_windows`, `render_hud_overlay`. | No caller outside tests. | Clipboard: in setu (ADR 0002). |
| **Inert flags**: `--backend`, `--kiosk`, `--no-ai`, `--high-contrast`, `--accessibility`. | Logged or stored, never acted on. `--high-contrast` stored a theme the renderer never read. The build target chooses the backend. | `--secure` is not retired; it acted, and returned at 0.17.2 (`src/flags.cyr`). The retired flags are now unknown-flag errors. |
| **SIGHUP reload** and the goodbye notification. | Reload only re-ran the status tick; the goodbye was never drawn. | — |
| **The emergency kill's cgroup removal** (`/sys/fs/cgroup/agnos/<uuid>`). | Nothing creates that path: daimon 2.4.3 makes and removes its own `daimon-<pid>/agent-<id>-<run>` cgroups, and stopping the agent through its DELETE is what clears them. agnos has no cgroups. Retired at 0.17.3. | If daimon stops owning its cgroups. |
| **`Touch*` input events.** | The Rust compositor ignored touch, and bhumi has no touch event kind. | With a touch path in bhumi. |
| **The floating layer for agent windows** (`WS_FLOATING`, `agent_aware_mode`, `created_at`, `app_id`). | Written and never read in Rust; nothing can create an agent window over setu today. | With agent-owned surfaces (ADR 0001). |
| **Renderer remnants**: `blit_clipped`, the `Layer` type, ResizeEdge T / TL / TR. | `blit_clipped` had no caller. Paint order is the window vector (ADR 0003). The top edges were unreachable in Rust too. | — |
| **Rust derive behaviour**: Clone, Debug, PartialEq, `serde` derives, and the tests that only check them. | Language facts, not behaviour; nothing serialized those types. | — |

**Kept dormant, not retired: `gestures`.** It is ported and tested library code. No touch path exists in
bhumi or agnos, so it is constructed and never fed. It stays at parity until there is input to recognise.

**The HUD widget specifications**, kept because the code that held them leaves with `rust-old/`:

| Widget | Source in Rust | Behaviour |
|---|---|---|
| `crew_status` (607 lines, 23 tests) | MCP tool `agnostic_list_crews` via `POST http://localhost:8090/v1/mcp/tools/call`, 5 s timeout | Reads `result.content[0].text` as JSON: `{crews: [...]}` or a bare array. Invalid JSON gives an empty list. Defaults: `id` or `crew_id` → `"unknown"`, `name` → `"unnamed crew"`. Status, case-insensitive: running/active → Running, completed/done/finished → Completed, failed/error → Failed. An empty task becomes none. |
| `gpu_status` (691 lines, 24 tests) | MCP tool `agnos_gpu_status`, same endpoint; polled every 3 s in the example; 5 s timeout | `{devices: [...]}` or a bare array, with field aliases `vram_used(_mib)`, `vram_total(_mib)`, `(compute_)utilization(_pct)`, `temperature(_c)`. A missing index falls back to list position, then devices are sorted. Utilization and VRAM: ≥ 80 Warning, ≥ 95 Critical. Temperature: ≥ 75 °C Warning, ≥ 90 °C Critical. No sensor reads Normal; total VRAM 0 gives fraction 0. Reports devices under pressure, peak VRAM and peak temperature. |
| `domain_filter` (661 lines, 20 tests) | `GET http://localhost:8090/v1/agents`, 5 s timeout | Domain from `domain`, then `metadata.domain`, else `"general"`. A bad UUID becomes a random v4. Groups are sorted; `all_domains` is computed before filtering. Active means status ∈ {`running`, `active`, `Acting`}, case-sensitive. |

None of the three ever errors on a shape it does not expect: a 2xx response with another shape becomes an
empty list with `last_fetch_ok = true`.

**What changed in daimon** (2.4.3, still on `127.0.0.1:8090`, checked 2026-10-09):
- `POST /v1/mcp/tools/call` now answers 405, because that path prefix only accepts DELETE. Calls moved to
  `POST /v1/mcp/call`, which returns a bare `{"content": [...]}`, so Rust's `result` lookup would find nothing.
- The tools are gone. daimon's built-ins are five libro tools, `web_fetch`, `web_search` and six nein tools.
  `agnostic_list_crews` survives only in secureyeoman's TypeScript MCP server, and nothing implements
  `agnos_gpu_status`.
- `GET /v1/agents` returns `{agents: [{id: int, name, type, status: int 0–6, pid, exit_code}], count}`, with
  no domain, task or heartbeat field.

## Consequences

- **Positive**:
  - 2,141 lines of ported source with no reader and 757 lines of their tests are gone from the tree. The
    compositor binary is 25,912 B smaller on the host and 21,816 B smaller on agnos.
  - The deletion gate shrinks to modules that do something.
  - The Rust tests of every row above close by this ADR. That includes 173 in `apps.rs`, 67 in the HUD and
    the clipboard/tiling/HUD-overlay tests in `compositor.rs`.
- **Negative**:
  - The work sunk into porting `apps.cyr` (C1+C2, 133 assertions) and `theme_bridge` is discarded.
  - A future tray, settings panel or HUD starts from these notes, not from code.
- **Neutral**:
  - Each "if it comes back" column is a pointer for the roadmap, not a commitment.

## Alternatives considered

- **Port everything and link nothing**, so `rust-old/` can go with every Rust test re-expressed. Rejected.
  It closes the gate with tests of code nobody runs, and the app layer alone is 173 tests of behaviour real
  clients replaced.
- **Keep the ported-but-unwired modules as dormant library code**, as `gestures` is kept. Rejected for these
  rows: `gestures` waits on an input path that will exist, while these wait on nothing.
