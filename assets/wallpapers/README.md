# Wallpapers

`aethersafha --wallpaper <file>` loads a PNG or JPEG through `[deps.chitra]` and scales it to the
screen. These assets ship with the desktop and are staged into the agnos rootfs by
`agnos/scripts/burn/stage-tools.sh`.

## `verify-2560x1440.png` — a VERIFICATION pattern, not a design

⛔ **This is an oracle, not a default.** Every element exists so a specific failure is visible by eye
rather than needing a screendump diff:

| element | what its absence or distortion means |
|---|---|
| 1px white frame on all four edges | the image is cropped, or the scale overruns the screen |
| four corner blocks — **red TL, green TR, blue BL, yellow BR** | the image is flipped, rotated, or the channel order is wrong (BGRA vs RGBA reads as red/blue swapped) |
| white cross through the exact centre | the scale is not centred |
| smooth horizontal + vertical gradient | tearing or a wrong stride shows as a **staircase**, which a flat colour would hide |

⭐ **Why 2560×1440**: it is archaemenid's panel, and scaled to XRGB it is a **14,745,600 B** upload —
far over the retired 2 MB `#86` slot cap and comfortably inside the 32 MB one agnos 1.56.44 raised it
to. That upload path has **never been verified on iron**: QEMU has no AMD PCI device, so every GPU
branch is dead there and only a burn runs it.

## Pending

⚠ **Designed default wallpapers are not here yet.** The operator's intent (2026-08-18) is that
defaults ship with the release; this file is the verification asset that arrived first. A designed
set wants the rupa palette (MUDRA / SHANTA × dark/light) so the backdrop and the theme agree.
