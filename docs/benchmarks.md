# Benchmarks

The Cyrius port's benches for the series the Rust original benchmarked, with Rust's own record kept
beside them.

> ⛔ **Rust's numbers are a record, not a target.** They are two runs at commit `a460c3a`
> (2026-04-03), on a machine nobody wrote down. The port's numbers come from whatever machine ran
> `scripts/bench-history.sh`, which records it. Two columns from two machines do not support a speedup
> or slowdown claim in either direction. Read each column on its own, and compare a port run only with
> another port run on the same machine.

## Running

```sh
cyrius bench tests/compositor_bench.bcyr       # renderer, window stack, framebuffer, damage, blit
cyrius bench tests/screen_capture_bench.bcyr   # capture encode, region capture, pixel read, RGBA convert
./scripts/bench-history.sh                     # both, appended to the history, and the table below rebuilt
```

The benches are host programs. They render into an in-memory bhumi framebuffer and never open
`/dev/fb0`. `tests/aethersafha.bcyr` holds the render hot-path benches from the 1.56.36 burn (clear,
AE-0a band, chrome, and the surface copy at 800x600 and 2560x1440). Those benches have no Rust
counterpart, so the history script does not record them.

| File | What it is |
|---|---|
| [`benchmarks/rust-a460c3a.csv`](benchmarks/rust-a460c3a.csv) | Rust's `bench-history.csv`, copied byte for byte from `rust-old/bench-history.csv` at tag `0.17.5`: 64 rows, two runs at `a460c3a` |
| [`benchmarks/history.csv`](benchmarks/history.csv) | the port's history: one row per bench per run (`timestamp,commit,branch,benchmark,estimate_ns`), the same columns as Rust's |
| [`benchmarks/runs.csv`](benchmarks/runs.csv) | one row per port run: the machine (CPU model, clocksource) and the toolchain that produced it |

`estimate_ns` in Rust's file is criterion's point estimate, the centre of its `time: [lower estimate upper]`
line. In the port's file it is the mean cost of one op (`bench_avg_ps / 1000`).

## The series, and what the port runs for each

Each bench carries Rust's series name, so a row in one CSV lines up with its row in the other. Where the
port has no such operation (ADR 0003 has no scene graph and no damage region list), the row measures the
port's replacement and says so.

| Rust series | Rust ran | The port runs |
|---|---|---|
| `renderer/render_frame/{1,5,10,20}` | `DesktopRenderer::render_frame`, 1920x1080, N 400x300 surfaces with buffers | `render_desktop`, 1920x1080, CPU path, N windows with 400x300 content and a 400x300 client surface at the same origins. The background is cleared full-screen every frame (no damage trackers, like Rust's `back.clear`). The port's frame also draws the titlebar above the content, the shell panel, the launcher check and the cursor |
| `scene_graph/add_100_surfaces` | a fresh `SceneGraph`, 100 `add_surface` | a fresh compositor, 100 `comp_create_window`. There are no layers, and the title is a borrowed string |
| `scene_graph/z_order_rebuild_50_surfaces` | `surfaces_in_order()` over 50 surfaces | `comp_focus` on the bottom of 50 windows, which raises it to the top. This is the operation that changes the port's paint order |
| `framebuffer/create_1080p` | `Framebuffer::new(1920, 1080, fill)` | `bhumi_fb_new(1920, 1080)` plus its fill. bhumi has no per-framebuffer free, so this row runs 10 rounds |
| `framebuffer/clear_1080p` | `Framebuffer::clear` | `bhumi_fb_clear` |
| `framebuffer/blit_400x300` | `Framebuffer::blit` of an opaque 400x300 (its row-copy fast path) | `blit_rect`, the CPU surface copy |
| `damage_tracker/add_50_damage_regions` | a fresh `DamageTracker`, `flush`, 50 `add_damage` | the per-frame damage step over the same 50 rects: roll cur into prev, clear, fold the rects (border ring included), compute the band |
| `screen_capture/{png,bmp,raw}_encode/full_screen/{720p,1080p,4K}` | `ScreenCaptureManager::capture(FullScreen, fmt, None)` | `scap_mgr_capture(FULLSCREEN, fmt, agent 0)`: the pixel read, the encode, the result and its history entry, then the result is freed. The PNG uses stored zlib in both |
| `screen_capture/region_png/capture/{256x256,640x480,1280x720,1920x1080}` | a `Region` at (0, 0) from the 1080p compositor, PNG | the same region through `scap_mgr_capture` |
| `screen_capture/pixel_buffer/clone/{720p,1080p,4K}` | `fb.pixels.clone()` | `scap_capture_full_screen`, the capture's own read of the whole framebuffer, freed each op |
| `screen_capture/argb_to_rgba/convert/{720p,1080p,4K}` | a loop written in the bench: `Vec<u32>` to RGBA bytes | the same bench-local loop. The port's library does this conversion inside `scap_encode_png`, where it cannot be timed on its own |
| `screen_capture/blit/to_1080p/{256x256,640x480,1280x720}` | `Framebuffer::blit` at (0, 0) | `blit_rect` at (0, 0). The bench is in `compositor_bench.bcyr`, which links `render.cyr` |

Every bench checks that its op did the work before it reports. A frame must have the top window's
surface on screen. The damage band must have the expected extent. A blit must land its far corner, and a
capture must not have been refused. If a check fails, the bench exits non-zero and the history script
records nothing.

## The port's numbers, beside Rust's record

The table below is generated by `scripts/bench-history.sh`. It rewrites everything between the markers.
The Rust columns come from `benchmarks/rust-a460c3a.csv`. The port columns come from
`benchmarks/history.csv`: the first run, the one before the latest, and the latest. Deltas are computed
only between port runs, against the port's baseline. No delta is computed against Rust.

<!-- port-table:begin (generated by scripts/bench-history.sh; do not edit by hand) -->
Port runs in this table (UTC, from `docs/benchmarks/runs.csv`):

| Column | Run | Commit | Machine | Clocksource | cyrius |
|---|---|---|---|---|---|
| Port current | 2026-10-10T18:44:13Z | `eef1fbb+dirty` | AMD Ryzen 7 5800H with Radeon Graphics | hpet | 6.6.11 |

| Series | Rust `a460c3a` run 1 | Rust `a460c3a` run 2 | Port current |
|---|---:|---:|---:|
| `renderer/render_frame/1` | 490.6 µs | 352.2 µs | 2.7 ms |
| `renderer/render_frame/5` | 917.6 µs | 694.7 µs | 4.5 ms |
| `renderer/render_frame/10` | 1.6 ms | 1.2 ms | 6.7 ms |
| `renderer/render_frame/20` | 2.5 ms | 2.0 ms | 11.4 ms |
| `scene_graph/add_100_surfaces` | 16.4 µs | 14.7 µs | 9.9 µs |
| `scene_graph/z_order_rebuild_50_surfaces` | 614.9 ns | 558.9 ns | 140.5 ns |
| `framebuffer/create_1080p` | 188.3 µs | 125.5 µs | 2.4 ms |
| `framebuffer/clear_1080p` | 156.6 µs | 136.2 µs | 2.2 ms |
| `framebuffer/blit_400x300` | 77.9 µs | 64.2 µs | 290.9 µs |
| `damage_tracker/add_50_damage_regions` | 206.8 ns | 211.2 ns | 811.6 ns |
| `screen_capture/blit/to_1080p/256x256` | 49.8 µs | 36.4 µs | 160.5 µs |
| `screen_capture/blit/to_1080p/640x480` | 224.5 µs | 167.7 µs | 731.0 µs |
| `screen_capture/blit/to_1080p/1280x720` | 694.7 µs | 527.0 µs | 2.2 ms |
| `screen_capture/png_encode/full_screen/720p` | 41.2 ms | 40.7 ms | 227.7 ms |
| `screen_capture/png_encode/full_screen/1080p` | 94.0 ms | 92.9 ms | 520.6 ms |
| `screen_capture/png_encode/full_screen/4K` | 405.5 ms | 389.2 ms | 2.03 s |
| `screen_capture/bmp_encode/full_screen/720p` | 4.3 ms | 3.9 ms | 18.4 ms |
| `screen_capture/bmp_encode/full_screen/1080p` | 10.1 ms | 9.5 ms | 41.1 ms |
| `screen_capture/bmp_encode/full_screen/4K` | 78.9 ms | 74.8 ms | 158.6 ms |
| `screen_capture/raw_encode/full_screen/720p` | 990.4 µs | 718.9 µs | 18.4 ms |
| `screen_capture/raw_encode/full_screen/1080p` | 2.9 ms | 2.1 ms | 40.6 ms |
| `screen_capture/raw_encode/full_screen/4K` | 48.0 ms | 45.7 ms | 157.4 ms |
| `screen_capture/region_png/capture/256x256` | 3.2 ms | 3.0 ms | 16.8 ms |
| `screen_capture/region_png/capture/640x480` | 13.2 ms | 12.4 ms | 78.3 ms |
| `screen_capture/region_png/capture/1280x720` | 38.0 ms | 36.5 ms | 235.1 ms |
| `screen_capture/region_png/capture/1920x1080` | 83.6 ms | 81.5 ms | 521.0 ms |
| `screen_capture/pixel_buffer/clone/720p` | 142.5 µs | 77.7 µs | 12.6 ms |
| `screen_capture/pixel_buffer/clone/1080p` | 786.2 µs | 349.4 µs | 28.3 ms |
| `screen_capture/pixel_buffer/clone/4K` | 6.2 ms | 6.0 ms | 111.0 ms |
| `screen_capture/argb_to_rgba/convert/720p` | 4.1 ms | 3.8 ms | 6.4 ms |
| `screen_capture/argb_to_rgba/convert/1080p` | 10.2 ms | 8.6 ms | 14.5 ms |
| `screen_capture/argb_to_rgba/convert/4K` | 37.8 ms | 34.7 ms | 53.1 ms |
<!-- port-table:end -->

### What the first port run shows on its own

These notes compare rows of the same run (2026-10-10, the machine in the table above) with each other.
They do not compare the port with Rust.

- On this host the timer floor is about 1.3 µs per clock read, because the clocksource is `hpet` (see
  `lib/bench.cyr`). Every row batches enough ops for the floor to be subtracted without dominating it.
  The smallest rows are `z_order` (~140 ns) and the damage band (~810 ns).
- Most of a full-screen capture is the pixel read. `pixel_buffer/clone` (the read alone) is about 12.6
  ms of the ~18.4 ms `raw_encode` at 720p, and about 111 ms of the ~157 ms at 4K. That read is
  `scap_capture_full_screen`: one `bhumi_fb_get` per pixel into an 8-byte slot.
- `png_encode` is 12 to 13x `raw_encode` at every resolution. After the same pixel read, the PNG path
  builds the RGBA rows, copies them into stored zlib blocks and runs Adler-32 and CRC-32 over them, all
  one byte at a time.
- `render_frame` grows by about 0.46 ms per added window. Most of the 1-window frame (~2.7 ms) is the
  full 1080p clear (`clear_1080p`, ~2.2 ms).

## Rust's record

This is `rust-old/benchmarks.md` (tag `0.17.5`), the table Rust's `bench-history.sh` generated from its two runs. It is
kept here as written, apart from the heading levels, and the series names have been filled in where
Rust's table dropped the parameter. The machine is unknown, so this is a record and not a target.

Latest: **2026-04-03T09:04:10Z** — commit `a460c3a`

### renderer

| Benchmark | Baseline (`a460c3a`) | Current (`a460c3a`) |
|-----------|------|------|
| `render_frame/1` | 490.6 µs | 352.2 µs **-28%** |
| `render_frame/5` | 917.6 µs | 694.7 µs **-24%** |
| `render_frame/10` | 1.6 ms | 1.2 ms **-28%** |
| `render_frame/20` | 2.5 ms | 2.0 ms **-17%** |

### scene_graph

| Benchmark | Baseline (`a460c3a`) | Current (`a460c3a`) |
|-----------|------|------|
| `add_100_surfaces` | 16.4 µs | 14.7 µs **-11%** |
| `z_order_rebuild_50_surfaces` | 614.9 ns | 558.9 ns **-9%** |

### framebuffer

| Benchmark | Baseline (`a460c3a`) | Current (`a460c3a`) |
|-----------|------|------|
| `create_1080p` | 188.3 µs | 125.5 µs **-33%** |
| `clear_1080p` | 156.6 µs | 136.2 µs **-13%** |
| `blit_400x300` | 77.9 µs | 64.2 µs **-18%** |

### damage_tracker

| Benchmark | Baseline (`a460c3a`) | Current (`a460c3a`) |
|-----------|------|------|
| `add_50_damage_regions` | 206.8 ns | 211.2 ns |

### screen_capture

| Benchmark | Baseline (`a460c3a`) | Current (`a460c3a`) |
|-----------|------|------|
| `png_encode/full_screen/720p` | 41.2 ms | 40.7 ms |
| `png_encode/full_screen/1080p` | 94.0 ms | 92.9 ms |
| `png_encode/full_screen/4K` | 405.5 ms | 389.2 ms **-4%** |
| `bmp_encode/full_screen/720p` | 4.3 ms | 3.9 ms **-9%** |
| `bmp_encode/full_screen/1080p` | 10.1 ms | 9.5 ms **-6%** |
| `bmp_encode/full_screen/4K` | 78.9 ms | 74.8 ms **-5%** |
| `raw_encode/full_screen/720p` | 990.4 µs | 718.9 µs **-27%** |
| `raw_encode/full_screen/1080p` | 2.9 ms | 2.1 ms **-28%** |
| `raw_encode/full_screen/4K` | 48.0 ms | 45.7 ms **-5%** |
| `region_png/capture/256x256` | 3.2 ms | 3.0 ms **-5%** |
| `region_png/capture/640x480` | 13.2 ms | 12.4 ms **-6%** |
| `region_png/capture/1280x720` | 38.0 ms | 36.5 ms **-4%** |
| `region_png/capture/1920x1080` | 83.6 ms | 81.5 ms |
| `pixel_buffer/clone/720p` | 142.5 µs | 77.7 µs **-45%** |
| `pixel_buffer/clone/1080p` | 786.2 µs | 349.4 µs **-56%** |
| `pixel_buffer/clone/4K` | 6.2 ms | 6.0 ms **-3%** |
| `argb_to_rgba/convert/720p` | 4.1 ms | 3.8 ms **-7%** |
| `argb_to_rgba/convert/1080p` | 10.2 ms | 8.6 ms **-15%** |
| `argb_to_rgba/convert/4K` | 37.8 ms | 34.7 ms **-8%** |
| `blit/to_1080p/256x256` | 49.8 µs | 36.4 µs **-27%** |
| `blit/to_1080p/640x480` | 224.5 µs | 167.7 µs **-25%** |
| `blit/to_1080p/1280x720` | 694.7 µs | 527.0 µs **-24%** |

Both Rust runs are at the same commit, 35 minutes apart. Rust's "Baseline → Current" deltas therefore
measure run-to-run variance on that machine (up to 56 %), not a code change.
