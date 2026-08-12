#!/usr/bin/env python3
"""Take a QEMU screendump over QMP and power the guest off.

⛔ WHY THIS IS NOT A ONE-LINE `nc`. This box has neither `socat` nor `nc`, and the QEMU *monitor*
protocol is a chatty human REPL that is awkward to drive blind. QMP is the machine interface, it is
one unix socket, and python3's `socket` module is already here — so the harness depends on nothing
that has to be installed.

⚠ QMP requires the capabilities handshake (`qmp_capabilities`) before ANY other command; skip it and
every request comes back `CommandNotFound`, which reads like a QEMU version problem rather than a
protocol mistake.

Usage: qemu-screendump.py <qmp-socket> <output.ppm>
Exit:  0 the dump was written · 1 QMP refused or the file never appeared
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
    # Read until we see a reply to *this* command — QMP interleaves asynchronous events with
    # replies, and taking the first line back would often hand you a STOP/RESUME event instead.
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
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    path, out = sys.argv[1], os.path.abspath(sys.argv[2])

    # The guest is booting while we connect; retry rather than racing it.
    sock = None
    for _ in range(100):
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

    sock.settimeout(20)
    f = sock.makefile("r")
    f.readline()                      # the greeting banner
    if rpc(f, sock, "qmp_capabilities") is None:
        print("qmp: handshake failed", file=sys.stderr)
        return 1

    r = rpc(f, sock, "screendump", filename=out)
    if r is None or "error" in r:
        print("qmp: screendump refused:", r, file=sys.stderr)
        return 1

    # ⚠ `screendump` returns as soon as it is ACCEPTED, not when the file is complete. Returning here
    # would hand the caller a truncated PPM that ImageMagick reports as a 1-colour image — i.e. a
    # FALSE "nothing was drawn", which is the exact verdict this harness exists to make trustworthy.
    for _ in range(100):
        if os.path.exists(out) and os.path.getsize(out) > 0:
            size = os.path.getsize(out)
            time.sleep(0.1)
            if os.path.getsize(out) == size:
                break
        time.sleep(0.1)

    rpc(f, sock, "quit")
    sock.close()
    if not os.path.exists(out) or os.path.getsize(out) == 0:
        print("qmp: screendump produced no file", file=sys.stderr)
        return 1
    print("qmp: screendump ->", out, os.path.getsize(out), "bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
