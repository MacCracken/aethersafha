#!/usr/bin/env python3
"""Drive the compositor's Linux build in QEMU with real key chords, and judge what clients were TOLD.

usage: qemu-input.py <bindir> <outdir> <scenario>
  <bindir> holds static `aethersafha`, `init` (programs/qemu_init.cyr), `focus_client` and
  `hostile_client` (programs/focus_client.cyr built with -D AE_HOSTILE). scripts/qemu-linux-input.sh
  builds them.
  scenario `focus`   — two focus_clients; screendumps after: start, Ctrl+Tab, Ctrl+F6, Ctrl+Tab, Ctrl+Tab
  scenario `hostile` — a hostile_client (stride 4) and one focus_client; one screendump
Exit: 0 PASS · 1 FAIL · 2 could not run.

⭐ THE ORACLES ARE OUTSIDE THE COMPOSITOR. QEMU's `screendump` reads the emulated VGA device, and each
focus_client draws its border BRIGHT green (0,255,0) only after the compositor has sent it
SETU_INPUT_FOCUS(1), DIM (0,48,0) after (0) — so a picture shows which client was TOLD it has focus.
Every client also logs each change to the serial console with its window id.
⛔ EVERY CHECK BELOW FAILS ON 0.16.29 (verified when this was written, 2026-10-09): after Ctrl+Tab the
window that lost focus kept its bright border, Ctrl+F6 told nobody anything, the minimized window never
came back, and the stride-4 client got the compositor killed by SIGSEGV. A check that passes on the
broken build is not a check.
⚠ The Linux build also hosts a mehman demo guest ("echo (foreign)") at boot, so Ctrl+Tab cycles THREE
windows here. The checks are written against invariants, not window positions, so they do not care.
"""
import gzip, json, os, socket, subprocess, sys, tempfile, time

KERNEL = os.environ.get("AE_QEMU_KERNEL", "/boot/vmlinuz-linux")
OVMF = os.environ.get("AE_QEMU_OVMF", "/usr/share/edk2/x64/OVMF.4m.fd")
BRIGHT, DIM = b"\x00\xff\x00", b"\x00\x30\x00"
PANEL_ROWS = 40          # the shell panel's gauges change on their own; never judge pixels up there


def cpio_newc(entries):
    """A minimal `newc` cpio writer, so the harness needs no `cpio` binary. entries: (name, mode, data)."""
    out = bytearray()
    for ino, (name, mode, data) in enumerate(entries + [("TRAILER!!!", 0, b"")], start=1):
        data = data or b""
        nb = name.encode() + b"\0"
        hdr = "070701" + "".join("%08X" % v for v in (
            ino, mode, 0, 0, 1, int(time.time()), len(data), 0, 0, 0, 0, len(nb), 0))
        out += hdr.encode() + nb + b"\0" * ((4 - (len(hdr) + len(nb)) % 4) % 4)
        out += data + b"\0" * ((4 - len(data) % 4) % 4)
    return bytes(out)


def build_initramfs(bindir, path):
    ents = [("proc", 0o40755, None), ("sys", 0o40755, None), ("dev", 0o40755, None)]
    for name in ("init", "aethersafha", "focus_client", "hostile_client"):
        ents.append((name, 0o100755, open(os.path.join(bindir, name), "rb").read()))
    with gzip.open(path, "wb", compresslevel=1) as f:
        f.write(cpio_newc(ents))


class QMP:
    def __init__(self, path):
        self.s = None
        for _ in range(300):
            try:
                s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                s.connect(path)
                self.s = s
                break
            except OSError:
                time.sleep(0.05)
        if self.s is None:
            raise RuntimeError("QMP socket never appeared")
        self.s.settimeout(30)
        self.f = self.s.makefile("r")
        self.f.readline()
        self.cmd("qmp_capabilities")

    def cmd(self, name, **args):
        msg = {"execute": name}
        if args:
            msg["arguments"] = args
        self.s.sendall((json.dumps(msg) + "\n").encode())
        while True:
            line = self.f.readline()
            if not line:
                return None
            try:
                obj = json.loads(line)
            except ValueError:
                continue
            if "return" in obj or "error" in obj:       # skip asynchronous events
                return obj

    def chord(self, *keys):
        # ⭐ One send-key with several qcodes presses them TOGETHER: a chord through the guest's evdev.
        self.cmd("send-key", keys=[{"type": "qcode", "data": k} for k in keys], **{"hold-time": 250})

    def dump(self, path):
        if os.path.exists(path):
            os.remove(path)
        self.cmd("screendump", filename=path)
        for _ in range(100):
            if os.path.exists(path) and os.path.getsize(path) > 0:
                n = os.path.getsize(path)
                time.sleep(0.1)
                if os.path.getsize(path) == n:
                    return True
            time.sleep(0.05)
        return False


def read_ppm(path):
    data = open(path, "rb").read()
    parts, i = [], 0
    while len(parts) < 4:                               # magic, width, height, maxval
        while data[i:i + 1].isspace():
            i += 1
        j = i
        while not data[j:j + 1].isspace():
            j += 1
        parts.append(data[i:j])
        i = j
    return int(parts[1]), int(parts[2]), data[i + 1:]


def pixels_of(img, rgb):
    """The set of pixel indices with exactly this colour, below the panel."""
    w, _, px = img
    found, start = set(), PANEL_ROWS * w * 3
    while True:
        k = px.find(rgb, start)
        if k < 0:
            return found
        if k % 3 == 0:
            found.add(k // 3)
            start = k + 3
        else:
            start = k + 1


def changed(a, b):
    """Pixel indices (below the panel) whose colour differs between two shots of one size."""
    w, h, pa = a
    pb = b[2]
    return {i for i in range(PANEL_ROWS * w, w * h) if pa[3 * i:3 * i + 3] != pb[3 * i:3 * i + 3]}


def same_fraction(a, b, region):
    if not region:
        return 0.0
    pa, pb = a[2], b[2]
    return sum(1 for i in region if pa[3 * i:3 * i + 3] == pb[3 * i:3 * i + 3]) / len(region)


def montage(shots, path):
    pngs = []
    for p in shots:
        png = p[:-4] + ".png"
        if subprocess.run(["magick", p, "-crop", "560x420+0+0", "+repage", png],
                          capture_output=True).returncode == 0:
            pngs.append(png)
    if pngs:
        subprocess.run(["magick"] + pngs + ["+append", path], capture_output=True)


def main():
    bindir, outdir = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
    scenario = sys.argv[3]
    os.makedirs(outdir, exist_ok=True)
    ird = os.path.join(outdir, "initramfs.gz")
    build_initramfs(bindir, ird)
    serial = os.path.join(outdir, "serial.log")
    if os.path.exists(serial):
        os.remove(serial)
    if scenario == "focus":
        gargs = "--frames 0 --client /focus_client --client /focus_client"
    elif scenario == "hostile":
        gargs = "--frames 0 --client /hostile_client --client /focus_client"
    else:
        print("unknown scenario", scenario)
        return 2

    def log():
        return open(serial, errors="replace").read() if os.path.exists(serial) else ""

    qmp_path = tempfile.mktemp(prefix="ae-input-qmp.")
    q = subprocess.Popen([
        "qemu-system-x86_64", "-enable-kvm", "-m", "512", "-smp", "2", "-bios", OVMF,
        "-kernel", KERNEL, "-initrd", ird,
        "-append", "console=tty0 console=ttyS0 rdinit=/init -- " + gargs,
        "-vga", "std", "-display", "none", "-serial", "file:" + serial,
        "-qmp", "unix:%s,server,nowait" % qmp_path, "-no-reboot"],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    shots, marks, failures = [], [], []
    try:
        qmp = QMP(qmp_path)
        t0 = time.time()
        while time.time() - t0 < 40:                 # both clients announce their present
            if log().count(" presented\n") >= 2 or q.poll() is not None:
                break
            time.sleep(0.1)
        if log().count(" presented\n") < 2:
            print("  ⚠  the clients never both presented — cannot judge (see %s)" % serial)
            return 2
        time.sleep(2.0)                              # let the compositor composite a few frames

        def shot(name):
            p = os.path.join(outdir, name + ".ppm")
            try:
                ok = qmp.dump(p)
            except OSError:
                ok = False
            marks.append(len(log()))
            shots.append(p if ok else None)

        steps = [("1-start", None)]
        if scenario == "focus":
            steps += [("2-ctrl-tab", ("ctrl", "tab")), ("3-ctrl-f6", ("ctrl", "f6")),
                      ("4-ctrl-tab", ("ctrl", "tab")), ("5-ctrl-tab", ("ctrl", "tab"))]
        for name, keys in steps:
            if keys:
                try:
                    qmp.chord(*keys)
                except OSError:
                    pass
                time.sleep(1.5)
            shot(name)
        try:
            qmp.chord("ctrl", "q")
            time.sleep(2.0)
            qmp.cmd("quit")                          # usually already powered off by Ctrl+Q
        except OSError:
            pass
    except RuntimeError as e:
        print("  ⚠ ", e)
        return 2
    finally:
        try:
            q.wait(timeout=20)
        except subprocess.TimeoutExpired:
            q.kill()
        if os.path.exists(qmp_path):
            os.remove(qmp_path)

    full = log()
    deltas, prev = [], 0
    for m in marks:                                  # the serial lines each step produced
        deltas.append(full[prev:m])
        prev = m
    killed = "KILLED by signal" in full

    def check(ok, what):
        print("  %s %s" % ("✅" if ok else "❌", what))
        if not ok:
            failures.append(what)

    check(not killed, "the compositor was not killed by a signal")
    if scenario == "focus":
        if not all(shots):
            check(False, "every screendump was taken (the guest died mid-run)")
        else:
            img = [read_ppm(p) for p in shots]
            start_bright = pixels_of(img[0], BRIGHT)
            check(len(start_bright) > 0 and len(pixels_of(img[0], DIM)) > 0,
                  "start: one client is told it has focus (bright), the other is not (dim)")
            still = len(start_bright & pixels_of(img[1], BRIGHT))
            check(still <= len(start_bright) // 10,
                  "Ctrl+Tab: the client that lost focus is TOLD so — its border goes dim (%d of %d still bright)"
                  % (still, len(start_bright)))
            check("focus GAINED" in deltas[2] or "focus lost" in deltas[2],
                  "Ctrl+F6: minimizing the focused window moves focus, and a client is told")
            vacated = changed(img[1], img[2])
            check(len(vacated) > 1000, "Ctrl+F6: the minimized window left the screen (%d px changed)"
                  % len(vacated))
            back = same_fraction(img[4], img[1], vacated)
            check(back >= 0.90, "Ctrl+Tab x2: the minimized window is BACK where it was (%.0f%% of its "
                  "pixels match the shot before it was minimized)" % (100 * back))
        montage([p for p in shots if p], os.path.join(outdir, "montage.png"))
    else:
        check("HOSTILE attach" in full, "the hostile client ran (stride 4 on a 240-wide surface)")
        check("surface refused" in full, "the compositor refused its surface")
        check(bool(shots and shots[0]) and len(pixels_of(read_ppm(shots[0]), BRIGHT)) > 0,
              "the desktop runs on, with the honest client told it has focus")
    print("  --- serial (clients and verdict lines) ---")
    for line in full.splitlines():
        if any(k in line for k in ("focus_client", "presented surface", "surface refused",
                                   "qemu_init: aethersafha", "KILLED")):
            print("    " + line.rstrip())
    print("  %s %s (screendumps and montage in %s)" % (
        "PASS" if not failures else "FAIL", scenario, outdir))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
