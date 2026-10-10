#!/usr/bin/env bash
# qemu-linux-input.sh — drive the compositor's LINUX build in QEMU with real key chords, and judge what
# its clients were TOLD.
#
# ⭐ WHY THIS EXISTS (0.17.0). `qemu-linux-desktop.sh` photographs a desktop that nobody touches, and the
# unit suites drive the compositor's functions, not its frame loop. Neither could have seen 0.16.29's
# three defects: focus changes that never reached clients, a minimized window that never came back, and a
# client that could crash the compositor with one bad ATTACH. This harness boots the same guest, sends
# Ctrl+Tab / Ctrl+F6 through QEMU's emulated keyboard (the guest's real evdev path), and reads the answer
# off the emulated display and the serial console — oracles the compositor cannot influence.
#
#   ./scripts/qemu-linux-input.sh            # every scenario
#   ./scripts/qemu-linux-input.sh focus      # focus notices, minimize and restore
#   ./scripts/qemu-linux-input.sh hostile    # a client attaching at stride 4
#   ./scripts/qemu-linux-input.sh sigint     # SIGINT from PID 1 runs the Ctrl+Q teardown; clients stay stoppable
#   ./scripts/qemu-linux-input.sh drag       # a USB mouse drags a window by its titlebar, then by its left edge
#   ./scripts/qemu-linux-input.sh silent     # clients that connect and stop talking; the desktop runs on (D12)
#   AE_INPUT_COMPOSITOR=/path/to/aethersafha ./scripts/qemu-linux-input.sh   # judge a prebuilt binary (A/B)
#
# The clients are `programs/focus_client.cyr` and its `-D AE_HOSTILE`, `-D AE_SILENT` and `-D AE_STALL`
# builds. Screendumps, a montage and the serial log land in build/qemu-input/<scenario>/. Boot setup
# (OVMF, `console=tty0`, no `quiet`) is the desktop harness's, for the reasons recorded there.
# Exit: 0 every scenario PASS · 1 at least one FAIL · 2 could not run.
set -uo pipefail

cd "$(dirname "$0")/.."
OUT=${AE_INPUT_OUT:-build/qemu-input}
KERNEL=${AE_QEMU_KERNEL:-/boot/vmlinuz-linux}
OVMF=${AE_QEMU_OVMF:-/usr/share/edk2/x64/OVMF.4m.fd}
SCENARIOS=${*:-focus hostile sigint drag silent}

[ -r "$KERNEL" ] || { echo "no kernel at $KERNEL (set AE_QEMU_KERNEL)"; exit 2; }
[ -r "$OVMF" ] || { echo "no OVMF firmware at $OVMF (set AE_QEMU_OVMF)"; exit 2; }
command -v qemu-system-x86_64 >/dev/null || { echo "no qemu-system-x86_64"; exit 2; }

mkdir -p "$OUT"
echo "== building guest binaries (static, host target) =="
if [ -n "${AE_INPUT_COMPOSITOR:-}" ]; then
    cp "$AE_INPUT_COMPOSITOR" "$OUT/aethersafha" || exit 2
    echo "   compositor: $AE_INPUT_COMPOSITOR (prebuilt)"
else
    cyrius build src/main.cyr "$OUT/aethersafha" >/dev/null 2>&1 || { echo "compositor build failed"; exit 2; }
fi
cyrius build programs/qemu_init.cyr "$OUT/init" >/dev/null 2>&1 || { echo "init build failed"; exit 2; }
cyrius build programs/focus_client.cyr "$OUT/focus_client" >/dev/null 2>&1 \
    || { echo "focus_client build failed"; exit 2; }
cyrius build -D AE_HOSTILE programs/focus_client.cyr "$OUT/hostile_client" >/dev/null 2>&1 \
    || { echo "hostile_client build failed"; exit 2; }
cyrius build -D AE_SILENT programs/focus_client.cyr "$OUT/silent_client" >/dev/null 2>&1 \
    || { echo "silent_client build failed"; exit 2; }
cyrius build -D AE_STALL programs/focus_client.cyr "$OUT/stall_client" >/dev/null 2>&1 \
    || { echo "stall_client build failed"; exit 2; }
for b in aethersafha init focus_client hostile_client silent_client stall_client; do
    file "$OUT/$b" | grep -q 'statically linked' || { echo "$OUT/$b is NOT static — the initramfs has no loader"; exit 2; }
done
echo "   compositor sha256 $(sha256sum "$OUT/aethersafha" | cut -c1-16)…"

rc=0
for s in $SCENARIOS; do
    echo
    echo "== scenario: $s =="
    AE_QEMU_KERNEL="$KERNEL" AE_QEMU_OVMF="$OVMF" python3 scripts/qemu-input.py "$OUT" "$OUT/$s" "$s"
    r=$?
    [ $r -gt $rc ] && rc=$r
done
echo
[ $rc -eq 0 ] && echo "== verdict: PASS ==" || echo "== verdict: $([ $rc -eq 1 ] && echo FAIL || echo 'could not run') =="
exit $rc
