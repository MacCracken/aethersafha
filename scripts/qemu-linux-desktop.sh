#!/usr/bin/env bash
# qemu-linux-desktop.sh — run the compositor's LINUX build inside QEMU and screendump the result.
#
# ⛔⛔ WHY THIS EXISTS. bhumi 1.2.0 made Linux a real display target, and the only way to see that
# working on a dev box is to let the compositor take over the screen the operator is using. This
# harness gives it a framebuffer nobody is looking at.
#
# ⭐ THE ORACLE IS OUTSIDE THE COMPOSITOR, WHICH IS THE ONLY KIND WORTH HAVING HERE. QEMU's monitor
# `screendump` reads the emulated VGA device's own memory — it is not something aethersafha or bhumi
# can influence, so it cannot report a picture that is not there. A compositor's own "presented ok"
# log line proves nothing; this is the same reasoning that made `programs/fbdev-probe.cyr` re-open
# /dev/fb0 on a second fd instead of reading back through its own mapping.
#
# The guest userland is TWO static binaries and nothing else: `qemu_init` (PID 1, mounts /proc + /sys,
# forks the compositor, reaps it, powers off) and `aethersafha`. No busybox, no distro image.
#
#   ./scripts/qemu-linux-desktop.sh              # 240-frame desktop run
#   ./scripts/qemu-linux-desktop.sh --clients    # the verdict-ladder probe
#   AE_QEMU_FRAMES=600 ./scripts/qemu-linux-desktop.sh
#
# Exit: 0 = the guest booted, got a framebuffer, and the screendump shows a drawn desktop.
set -uo pipefail

cd "$(dirname "$0")/.."
ROOT=$(pwd)
OUT=${AE_QEMU_OUT:-build/qemu-linux}
FRAMES=${AE_QEMU_FRAMES:-600}
KERNEL=${AE_QEMU_KERNEL:-/boot/vmlinuz-linux}
TIMEOUT=${AE_QEMU_TIMEOUT:-90}
SETU=${AE_QEMU_SETU:-../setu}
GUEST_ARGS=${*:-}

mkdir -p "$OUT"
rm -f "$OUT"/serial.log "$OUT"/screen.ppm "$OUT"/screen.png

[ -r "$KERNEL" ] || { echo "no kernel at $KERNEL (set AE_QEMU_KERNEL)"; exit 2; }

echo "== building guest binaries (static, host target) =="
cyrius build src/main.cyr           "$OUT/aethersafha" >/dev/null 2>&1 || { echo "compositor build failed"; exit 2; }
cyrius build programs/qemu_init.cyr "$OUT/init"        >/dev/null 2>&1 || { echo "init build failed"; exit 2; }
# ⭐ Stage a real setu CLIENT too, so the guest shows a hosted window and not just chrome. The
# compositor launches it itself via `--client` (M6-B5); before that existed a client had to be started
# by hand in another shell, which is impossible inside a one-shot guest.
CLIENT_ARG=""
if [ -d "$SETU" ] && (cd "$SETU" && cyrius build programs/present_probe.cyr "$ROOT/$OUT/present_probe" >/dev/null 2>&1); then
    CLIENT_ARG="--client /present_probe"
    echo "   staged setu present_probe as the guest client"
else
    echo "   ⚠ no setu at $SETU — the guest will run with NO client (chrome only)"
fi
[ -n "$GUEST_ARGS" ] || GUEST_ARGS="--frames $FRAMES $CLIENT_ARG"
for b in "$OUT/aethersafha" "$OUT/init"; do
    file "$b" | grep -q 'statically linked' || { echo "$b is NOT static — the initramfs has no loader"; exit 2; }
done

echo "== assembling initramfs =="
IRD=$(mktemp -d); trap 'rm -rf "$IRD"' EXIT
mkdir -p "$IRD"/{proc,sys,dev}
cp "$OUT/init" "$IRD/init"; cp "$OUT/aethersafha" "$IRD/aethersafha"
chmod +x "$IRD/init" "$IRD/aethersafha"
[ -f "$OUT/present_probe" ] && { cp "$OUT/present_probe" "$IRD/present_probe"; chmod +x "$IRD/present_probe"; }
( cd "$IRD" && find . -print0 | cpio --null -o --format=newc 2>/dev/null | gzip -1 ) > "$OUT/initramfs.gz"
echo "   initramfs: $(stat -c%s "$OUT/initramfs.gz") bytes"

QMP=$(mktemp -u /tmp/ae-qemu-qmp.XXXX)
OVMF=${AE_QEMU_OVMF:-/usr/share/edk2/x64/OVMF.4m.fd}

# ⛔⛔ BOOT VIA OVMF, AND THE REASON IS THE PIXEL DEPTH — not a preference for UEFI.
# The first cut used `-vga std` with `vga=792`, expecting 1024x768x32. The guest came up at
# **bits_per_pixel = 24** (`simpledrmdrmfb`, stride 3072 = 1024*3), and bhumi's fbdev arm correctly
# REFUSED it: the whole pixel contract is XRGB8888 and a 24bpp packed surface is not that. The arm was
# right and the harness was wrong. VBE's 0x318 is a 24-bit mode; there is no `vga=` number that
# reliably yields a 32bpp linear framebuffer on a modern kernel.
# ⇒ OVMF's GOP hands over a 32-bit BGRX framebuffer, which built-in `simpledrm` binds with no module
# loader — and 32bpp BGRX is exactly what `BhumiFb` stores, so the blit is a raw copy.
# ⚠ Needs `CONFIG_EFI_STUB=y` in the guest kernel so OVMF can load `-kernel` directly.
#
# ⛔⛔ `console=tty0` IS LOAD-BEARING AND COST THE LONGEST DETOUR IN THIS HARNESS. With only
# `console=ttyS0`, **nothing a userspace process writes to /dev/fb0 ever reaches the screen** — every
# syscall succeeds, bhumi reports a clean present, and the screendump keeps showing the firmware
# splash. The reason is that `simpledrmdrmfb` is DRM **fbdev emulation over a shadow buffer**: writes
# land in the shadow and are flushed to scanout by the deferred-I/O damage worker, which only runs
# while the fbdev is bound and active. No tty0 console ⇒ fbcon never binds fb0 ⇒ nothing ever flushes.
# ⭐ Proven, not guessed: `qemu_init --fbfill` paints the top half via `pwrite` and the bottom half via
# `mmap`. With `console=ttyS0` alone BOTH halves stayed at the splash (0.010 non-black). Adding
# `console=tty0` made the same binary produce mean RGB **(1,0,0)** top and **(0,0,1)** bottom — a
# full-screen, exactly-correct picture. So neither the write path nor bhumi was ever at fault.
# ⛔⛔ AND `quiet` MUST NOT BE ON THE CMDLINE, for the same underlying reason — this cost a second
# round of the same confusion. `console=tty0` alone is not enough: with `quiet loglevel=3` the kernel
# prints nothing, fbcon never does its first draw, the fbdev is never driven, and userspace writes
# are still never flushed. Measured with ONE variable changed and everything else identical:
# `--fbfill` painted 0.010 non-black (the splash) WITH `quiet`, and a full-screen (1,0,0)/(0,0,1)
# WITHOUT it. Kernel boot text on the framebuffer is harmless here — the compositor repaints every
# pixel of the screen on its first frame.
# ⚠ Carry this to real hardware: bhumi's fbdev arm needs an ACTIVE, DRIVEN VT. On a box whose console
# has been taken over by a display server there is no fbcon doing the flushing, which is a second
# reason not to run this against a live desktop session.
# ⚠ `-display none` keeps QEMU off the operator's screen; `screendump` reads the DEVICE, not a window.
[ -r "$OVMF" ] || { echo "no OVMF firmware at $OVMF (set AE_QEMU_OVMF)"; exit 2; }

echo "== booting (OVMF/UEFI, for a 32bpp GOP framebuffer) =="
timeout "$TIMEOUT" qemu-system-x86_64 \
    -enable-kvm -m 512 -smp 2 \
    -bios "$OVMF" \
    -kernel "$KERNEL" -initrd "$OUT/initramfs.gz" \
    -append "console=tty0 console=ttyS0 rdinit=/init -- --hold $GUEST_ARGS" \
    -vga std -display none \
    -serial "file:$OUT/serial.log" \
    -qmp "unix:$QMP,server,nowait" \
    -no-reboot &
QPID=$!

# ⛔⛔ SYNCHRONISE ON THE LOG, DO NOT SLEEP A GUESS. The first cut slept a fixed 9 s and captured
# nothing at all: OVMF's firmware phase plus the boot took ~1 s, the compositor drew its frames, the
# guest powered off at **1.3 s**, and the QMP socket was gone long before the sleep expired. The
# opposite guess is just as wrong — dump too early and you photograph the firmware splash and call it
# "nothing was drawn". A screendump has to happen INSIDE a window whose length nobody knows in advance.
# ⇒ Wait for the compositor to announce itself on the serial line, then dump immediately. The frame
# count is sized so there is a real window to hit rather than to make a guess more likely to land.
# ⛔ DUMP ON AN OBSERVABLE EVENT, NOT A TIMER. Two earlier cuts got this wrong in both directions:
# a fixed 9 s sleep photographed nothing (the guest had already powered off at 1.3 s), and syncing on
# "desktop up" photographed the FIRMWARE SPLASH, because that line is printed before the frame loop
# starts — and the colour gate happily called 65 colours of TianoCore logo a drawn desktop.
# ⇒ `--hold` makes the init pause AFTER the compositor has exited and print a marker. Nothing redraws
# during the hold, so what the device holds is exactly the compositor's last frame.
echo "== waiting for the compositor's last frame =="
for _ in $(seq 1 1200); do
    grep -q 'holding the framebuffer' "$OUT/serial.log" 2>/dev/null && break
    kill -0 $QPID 2>/dev/null || break
    sleep 0.05
done
python3 scripts/qemu-screendump.py "$QMP" "$OUT/screen.ppm" || true
wait $QPID 2>/dev/null
rm -f "$QMP"

echo
echo "== serial =="
sed -n '1,40p' "$OUT/serial.log" 2>/dev/null

echo
echo "== verdict =="
rc=0
grep -q 'qemu_init: PID 1 up' "$OUT/serial.log" 2>/dev/null \
    && echo "  ✅ guest booted (PID 1 is ours)" || { echo "  ❌ guest never reached init"; rc=1; }

# ⛔ THE TWO FAILURE MODES MUST BE NAMED SEPARATELY. "No pixels" because the guest had no framebuffer
# is a harness problem; "no pixels" with a framebuffer present is a compositor problem. They point at
# different repos and the pre-0.13.2 version of this check could not tell them apart.
if grep -q 'NO /dev/fb0' "$OUT/serial.log" 2>/dev/null; then
    echo "  ❌ guest had NO framebuffer — harness/kernel-driver problem, not a compositor problem"; rc=1
else
    echo "  ✅ guest has /dev/fb0"
fi
grep -q 'screen size read from the kernel' "$OUT/serial.log" 2>/dev/null \
    && echo "  ✅ compositor read geometry from the kernel (not its fallback)" \
    || echo "  ⚠  compositor used its built-in fallback — geometry did not come from the framebuffer"
if grep -q 'setu client presented surface' "$OUT/serial.log" 2>/dev/null; then
    echo "  ✅ a real setu client connected and presented a window"
else
    if grep -q 'launched client' "$OUT/serial.log" 2>/dev/null; then
        echo "  ❌ a client was launched but never presented"
        grep -q 'buf_create failed' "$OUT/serial.log" 2>/dev/null \
            && echo "     — 'buf_create failed': the guest is missing /dev/shm, where setu's buffers live"
        rc=1
    else
        echo "  ⚠  no client was launched this run (chrome only)"
    fi
fi

if [ -f "$OUT/screen.ppm" ]; then
    if command -v magick >/dev/null || command -v convert >/dev/null; then
        IM=$(command -v magick || command -v convert)
        "$IM" "$OUT/screen.ppm" -resize 60% "$OUT/screen.png" 2>/dev/null
        K=$("$IM" "$OUT/screen.ppm" -format %k info: 2>/dev/null)
        # ⛔⛔ THE GATE IS THE NON-BLACK FRACTION, **NOT** THE COLOUR COUNT — and the colour count is
        # printed only because it is interesting, never because it decides anything.
        # An earlier cut gated on ">= 8 distinct colours" and returned a confident
        # "✅ DRAWN DESKTOP" for a photograph of the **OVMF TianoCore splash**, which has 65 colours.
        # A count cannot tell a logo on black from a desktop; it is blind to layout by construction,
        # exactly as `puka-terminal-test`'s pixel count was blind to a staircase.
        # ⭐ CALIBRATED ON BOTH ARMS, which is what makes the threshold meaningful rather than a guess:
        #      firmware splash (the null)  0.011   <- measured, twice
        #      drawn desktop  (the signal) 0.225   <- measured
        # A desktop fills the screen; a splash is a small logo on black. 0.10 sits an order of
        # magnitude above the null and less than half the signal.
        NB=$("$IM" "$OUT/screen.ppm" -fill white -fuzz 5% +opaque black -colorspace Gray -format '%[fx:mean]' info: 2>/dev/null)
        echo "  screendump: $OUT/screen.png — $K colours, non-black fraction $NB"
        if awk "BEGIN{exit !($NB >= 0.10)}" 2>/dev/null; then
            echo "  ✅ the framebuffer shows a DRAWN DESKTOP (non-black $NB >= 0.10)"
        else
            echo "  ❌ the framebuffer is essentially empty (non-black $NB < 0.10)"
            echo "     — 0.011 is the OVMF splash, i.e. the compositor drew nothing that reached scanout"
            rc=1
        fi
    else
        echo "  screendump: $OUT/screen.ppm (install imagemagick for the gate)"
    fi
else
    echo "  ❌ no screendump captured"; rc=1
fi
exit $rc
