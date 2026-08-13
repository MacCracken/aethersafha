#!/usr/bin/env python3
"""Send keys to a QEMU guest over QMP, then screendump and quit.

⛔ WHY A SECOND SCRIPT AND NOT A FLAG ON THE DUMPER. Input verification needs a strictly ordered
sequence — wait for the compositor, press, let it react, THEN read the device — and the dumper's job
is a single atomic capture. Folding a stimulus into it would make "did the picture change" and "did
the key arrive" share one failure mode.

⚠ QMP `send-key` takes qcode NAMES, not scancodes, and injects at the emulated INPUT device — so this
exercises the guest's real evdev path (atkbd -> /dev/input/eventN -> bhumi), not a shortcut into the
compositor. That is the whole point: a test that poked the compositor directly would prove nothing
about the layer being added.

Usage: qemu-sendkey.py <qmp-socket> <out.ppm> <settle-seconds> <key> [key ...]
"""
import json
import os
import socket
import sys
import time


def rpc(f, sock, cmd, **args):
    msg = {"execute": cmd}
    if args:
        msg["arguments"] = args
    sock.sendall((json.dumps(msg) + "\n").encode())
    while True:
        line = f.readline()
        if not line:
            return None
        try:
            obj = json.loads(line)
        except ValueError:
            continue
        if "return" in obj or "error" in obj:
            return obj


def main():
    if len(sys.argv) < 5:
        print(__doc__, file=sys.stderr)
        return 2
    path, out, settle = sys.argv[1], os.path.abspath(sys.argv[2]), float(sys.argv[3])
    keys = sys.argv[4:]

    sock = None
    for _ in range(200):
        try:
            sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            sock.connect(path)
            break
        except (FileNotFoundError, ConnectionRefusedError):
            sock = None
            time.sleep(0.1)
    if sock is None:
        print("qmp: could not connect to", path, file=sys.stderr)
        return 1

    sock.settimeout(30)
    f = sock.makefile("r")
    f.readline()
    if rpc(f, sock, "qmp_capabilities") is None:
        print("qmp: handshake failed", file=sys.stderr)
        return 1

    time.sleep(settle)
    for k in keys:
        # ⭐ `mouse:DX,DY` and `click` drive the POINTER through QEMU's own input layer
        # (`input-send-event`), so the path under test is emulated device -> Linux evdev -> bhumi,
        # not a shortcut into the consumer.
        if k.startswith("mouse:"):
            dx, dy = k[6:].split(",")
            rpc(f, sock, "input-send-event", events=[
                {"type": "rel", "data": {"axis": "x", "value": int(dx)}},
                {"type": "rel", "data": {"axis": "y", "value": int(dy)}},
            ])
            print("qmp: moved", dx, dy)
            time.sleep(0.3)
            continue
        if k == "click":
            for down in (True, False):
                rpc(f, sock, "input-send-event", events=[
                    {"type": "btn", "data": {"down": down, "button": "left"}}])
                time.sleep(0.15)
            print("qmp: clicked")
            time.sleep(0.3)
            continue
        # ⚠ hold-time matters and is not cosmetic. agnos measured 0-of-9 keys delivered at QEMU's
        # ~100 ms default because a USB HID keyboard reports STATE ON POLL and a press+release inside
        # one frame never existed. evdev is edge-driven so it does not have that failure, but a
        # generous hold costs nothing and keeps this harness honest against both.
        r = rpc(f, sock, "send-key", keys=[{"type": "qcode", "data": k}], **{"hold-time": 300})
        if r is None or "error" in r:
            print("qmp: send-key", k, "failed:", r, file=sys.stderr)
            return 1
        print("qmp: sent", k)
        time.sleep(0.4)

    time.sleep(settle)
    r = rpc(f, sock, "screendump", filename=out)
    if r is None or "error" in r:
        print("qmp: screendump refused:", r, file=sys.stderr)
        return 1
    for _ in range(100):
        if os.path.exists(out) and os.path.getsize(out) > 0:
            size = os.path.getsize(out)
            time.sleep(0.1)
            if os.path.getsize(out) == size:
                break
        time.sleep(0.1)
    rpc(f, sock, "quit")
    sock.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
