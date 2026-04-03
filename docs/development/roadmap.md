# Development Roadmap

## v0.1.0 — Initial Extraction (complete)

- [x] Extract from agnosticos/userland/desktop-environment
- [x] Standalone Cargo.toml with path deps to agnostik + agnosys
- [x] All existing source, benchmarks, and tests carried over
- [x] Verify `cargo check --all-features` passes standalone
- [x] Verify `cargo test --all-features` passes standalone
- [x] First benchmark baseline via `./scripts/bench-history.sh`

## v0.2.0 — Standalone Hardening (complete)

- [x] P(-1) scaffold hardening pass
- [x] Full clippy + fmt + audit + deny clean
- [x] agnostik + agnosys as git deps with pinned tags
- [x] CI workflows (ci.yml, release.yml)
- [x] Integration tests in `tests/`
- [x] Example in `examples/`
- [x] Narrow blanket `#[allow(dead_code)]` — targeted per-module allows
- [x] 90.2% line coverage (target 80%+)

### P(-1) Audit Observations (deferred to future milestones)

**Compositor architecture:**
- Every field in `Compositor` is individually `Arc<RwLock<_>>` — no consistent
  lock ordering, latent deadlock risk under real concurrency. Restructure into
  fewer lock-protected groups before wiring real Wayland clients (v0.4.0).
- `wayland/protocol.rs` `ProtocolBridge` has `pub` fields that should be
  `pub(crate)` or private with accessors — tighten before API stabilisation.

**Unbounded growth:**
- `AccessibilityTree::announcements` grows without bound.
- `DamageTracker::regions` grows without bound between flushes.
- `ProtocolBridge::pending_actions` grows without bound between drains.
- All three need caps or ring buffers before production use (v0.4.0).

**Data structure upgrades:**
- `AccessibilityTree` uses `Vec<AccessibleNode>` with linear scan for
  lookup/removal — switch to `HashMap<Uuid, AccessibleNode>` when window
  counts matter (v0.4.0).
- `shell.rs` `search_launcher` lowercases every item on every keystroke —
  pre-compute lowercase names for responsive search (v0.5.0).

**Renderer:**
- `blit_clipped` still uses per-pixel blending (only `blit` got the row-level
  fast path) — extend the optimisation when damage-clipped blits are hot (v0.4.0).
- `screen_capture` clones entire pixel buffer (`fb.pixels.clone()`) for
  full-screen capture — encode directly from borrowed slice (v0.5.0).

**Concurrency / lifecycle:**
- `ai_features::start_hud_polling` spawns a tokio loop with no cancellation —
  multiple calls accumulate zombie tasks. Return + track `JoinHandle`, abort
  previous before starting new (v0.6.0).

## v0.3.0 — Independent Build

- [ ] Own Cargo.lock, independent of agnosticos workspace

## v0.4.0 — Real Compositor

- [ ] DRM/KMS backend (scanout to real displays)
- [ ] Wayland socket listener — accept client connections
- [ ] Wire xdg_shell, wl_compositor, wlr_layer_shell handlers
- [ ] XWayland launch and surface mapping
- [ ] GPU-accelerated rendering path (EGL/Vulkan)
- [ ] Input event routing from libinput
- [ ] Restructure `Compositor` locking (fewer Arc<RwLock<_>> groups)
- [ ] Cap unbounded vectors (announcements, damage regions, pending_actions)
- [ ] AccessibilityTree: Vec → HashMap for O(1) node lookup
- [ ] Extend blit_clipped with row-level fast path
- [ ] Tighten ProtocolBridge field visibility

## v0.5.0 — Desktop Integration

- [ ] Plugin host: dynamic loading (dlopen / WASM)
- [ ] Screen capture: real buffer source from compositor
- [ ] Screen capture: encode from borrowed slice (no full-buffer clone)
- [ ] Screen recording: encoder pipeline (ffmpeg / GStreamer)
- [ ] Gesture recognizer wired to libinput
- [ ] Theme bridge: live sync with Flutter shell
- [ ] App launcher: pre-computed lowercase for search

## v0.6.0 — AI Features

- [ ] Agent runtime integration (model inference)
- [ ] Context engine: real window/app tracking
- [ ] Agent HUD: live overlay rendering
- [ ] AI suggestion pipeline (context → model → UI)
- [ ] HUD polling: cancellable task with JoinHandle tracking

## v0.7.0 — Security Hardening

- [ ] Landlock sandbox for plugins (via agnosys)
- [ ] Seccomp filters for compositor process
- [ ] Per-agent permission enforcement (not just UI)
- [ ] Audit log persistence

## v1.0.0 Criteria

- [ ] All protocol handlers production-tested
- [ ] Plugin host API stable
- [ ] Screen capture/recording API stable
- [ ] Accessibility API stable
- [ ] 80%+ code coverage
- [ ] Independent ark package: `ark upgrade aethersafha`
