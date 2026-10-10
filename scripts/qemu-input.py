#!/usr/bin/env python3
"""Drive the compositor's Linux build in QEMU with real key chords, and judge what clients were TOLD.

usage: qemu-input.py <bindir> <outdir> <scenario>
  <bindir> holds static `aethersafha`, `init` (programs/qemu_init.cyr), `focus_client` and
  `hostile_client` (programs/focus_client.cyr built with -D AE_HOSTILE). scripts/qemu-linux-input.sh
  builds them.
  scenario `focus`   — two focus_clients; screendumps after: start, Ctrl+Tab, Ctrl+F6, Ctrl+Tab, Ctrl+Tab
  scenario `hostile` — a hostile_client (stride 4) and one focus_client; one screendump
  scenario `sigint`  — two focus_clients, then SIGINT from PID 1 instead of Ctrl+Q: the teardown must run
    (0.17.1 fails it: KILLED by signal 2; with the child's mask reset removed, the clients read SigBlk 2)
  scenario `drag`    — two focus_clients and a USB mouse; the focused window is dragged by its titlebar,
    then pressed on its left edge (0.17.4: pointer routing moved to src/route.cyr)
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
DRAG_DX, DRAG_DY = 120, 70   # the `drag` scenario's titlebar drag: small enough that no clamp is reached
LEFT_DX = -60            # ...and its drag of the window's left edge, outwards
HOME = -4000             # a relative move this far left and up parks the cursor on (0, 0): it clamps there
TITLEBAR_H = 30          # src/window.cyr: the titlebar is the rows ABOVE a window's content rect


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

    # ⭐ The pointer goes through QEMU's own input layer too (`input-send-event`, as qemu-sendkey.py does):
    # the emulated USB mouse -> the guest's evdev -> bhumi. ⚠ It is RELATIVE, so a position is reached by
    # parking the cursor at (0, 0) first (`HOME`) and moving from there.
    def rel(self, dx, dy):
        self.cmd("input-send-event", events=[{"type": "rel", "data": {"axis": "x", "value": dx}},
                                             {"type": "rel", "data": {"axis": "y", "value": dy}}])

    def btn(self, down):
        self.cmd("input-send-event", events=[{"type": "btn", "data": {"down": down, "button": "left"}}])

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


def bbox(img, rgb):
    """(x0, y0, x1, y1), inclusive, of the pixels of this colour below the panel; None when there are none."""
    w = img[0]
    pts = pixels_of(img, rgb)
    if not pts:
        return None
    xs, ys = [i % w for i in pts], [i // w for i in pts]
    return min(xs), min(ys), max(xs), max(ys)


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


def same_shifted(a, b, x0, y0, w, h, dx):
    """The fraction of the w x h rect at (x0, y0) in shot `a` that reappears in shot `b` dx columns over."""
    wa, ha, pa = a
    pb = b[2]
    n = same = 0
    for y in range(y0, y0 + h):
        for x in range(x0, x0 + w):
            if 0 <= x < wa and 0 <= x + dx < wa and 0 <= y < ha:
                n += 1
                i, j = 3 * (y * wa + x), 3 * (y * wa + x + dx)
                same += pa[i:i + 3] == pb[j:j + 3]
    return same / n if n else 0.0


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
    if scenario in ("focus", "drag"):
        gargs = "--frames 0 --client /focus_client --client /focus_client"
    elif scenario == "hostile":
        gargs = "--frames 0 --client /hostile_client --client /focus_client"
    elif scenario == "sigint":
        # ⭐ `--sigint-port` is qemu_init's: a line on the guest's ttyS1 makes PID 1 report the masks and
        # kill(compositor, SIGINT) — from OUTSIDE the compositor, like `kill -INT <pid>`. (A tty Ctrl+C
        # would signal the whole foreground group, clients included; this models the compositor's half.)
        gargs = "--sigint-port --frames 0 --client /focus_client --client /focus_client"
    else:
        print("unknown scenario", scenario)
        return 2

    def log():
        return open(serial, errors="replace").read() if os.path.exists(serial) else ""

    qmp_path = tempfile.mktemp(prefix="ae-input-qmp.")
    sig_path = tempfile.mktemp(prefix="ae-input-sig.")
    extra = ["-serial", "unix:%s,server,nowait" % sig_path] if scenario == "sigint" else []
    if scenario == "drag":
        # ⚠ A USB mouse, because the guest kernel builds usbhid in and psmouse as a module the initramfs
        # does not carry. Only this scenario gets one, so the others boot exactly as they always have.
        extra = ["-device", "qemu-xhci,id=xhci", "-device", "usb-mouse,bus=xhci.0"]
    q = subprocess.Popen([
        "qemu-system-x86_64", "-enable-kvm", "-m", "512", "-smp", "2", "-bios", OVMF,
        "-kernel", KERNEL, "-initrd", ird,
        "-append", "console=tty0 console=ttyS0 rdinit=/init -- " + gargs,
        "-vga", "std", "-display", "none", "-serial", "file:" + serial,
        "-qmp", "unix:%s,server,nowait" % qmp_path, "-no-reboot"] + extra,
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

        def gesture(x, y, dx, dy):
            # From the parked cursor to (x, y): press, move by (dx, dy), release, and park it again.
            # ⚠ A pause after each, so no two land in one bhumi poll: a press folded into the same poll as
            # the motion after it is a press at the wrong place.
            for step in ((qmp.rel, x, y), (qmp.btn, True), (qmp.rel, dx, dy), (qmp.btn, False),
                         (qmp.rel, HOME, HOME)):
                try:
                    step[0](*step[1:])
                except OSError:
                    pass
                time.sleep(0.8)

        if scenario == "drag":
            try:
                qmp.rel(HOME, HOME)                  # park the cursor at (0, 0), clear of every window
            except OSError:
                pass
            time.sleep(1.0)
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
        if scenario == "drag":
            # ⭐ AIM FROM THE PICTURE. The focused client's bright border is its content rect, so its
            # titlebar is the 30 rows above it and its left strip the first columns of it.
            box = bbox(read_ppm(shots[0]), BRIGHT) if shots[0] else None
            if box:
                gesture(box[0] + 60, box[1] - 15, DRAG_DX, DRAG_DY)
                shot("2-dragged")
                box2 = bbox(read_ppm(shots[1]), BRIGHT) if shots[1] else None
                if box2:
                    gesture(box2[0] + 2, box2[1] + 80, LEFT_DX, 0)
                    shot("3-left-edge")
        if scenario == "sigint":
            try:
                s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                s.connect(sig_path)
                s.sendall(b"INT\n")
                t1 = time.time()
                while time.time() - t1 < 20 and "qemu_init: powering off" not in log():
                    time.sleep(0.1)
                s.close()
                qmp.cmd("quit")
            except OSError:
                pass
        else:
            try:
                qmp.chord("ctrl", "q")
                time.sleep(2.0)
                qmp.cmd("quit")                      # usually already powered off by Ctrl+Q
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
        for p in (qmp_path, sig_path):
            if os.path.exists(p):
                os.remove(p)

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
    elif scenario == "sigint":
        sent = full.find("qemu_init: sent SIGINT")
        quit_at = full.find("aethersafha: quit on SIGINT")
        check("SIGINT is read per frame" in full, "the compositor set up its SIGINT signalfd")
        check(sent >= 0 and quit_at > sent, "PID 1 sent SIGINT, and the compositor quit on it (a log line names SIGINT)")
        tail = full[quit_at:] if quit_at >= 0 else ""
        for line in ("at exit — clients told to close", "frame cost us AT EXIT", "frame loop ok"):
            check(line in tail, "the Ctrl+Q teardown ran after it: %r" % line)
        # ⚠ `exit 0` alone is not enough: a signal death prints it too (WEXITSTATUS is 0), then KILLED.
        check("qemu_init: aethersafha exit 0" in full and not killed, "a clean exit 0, not a signal death")
        blk = {}
        for line in full.splitlines():
            if line.startswith("qemu_init: ") and " SigBlk:" in line:
                who, pid, _, mask = line[len("qemu_init: "):].split()[:4]
                blk.setdefault(who, []).append(int(mask, 16))
        check(len(blk.get("compositor", [])) == 1 and blk["compositor"][0] & 2 == 2,
              "the compositor itself has SIGINT blocked (read from /proc, not from its log)")
        kids = blk.get("child", [])
        check(len(kids) >= 2 and all(m & 2 == 0 for m in kids),
              "every spawned client has SIGINT UNBLOCKED, so Ctrl+C can still stop it (%s)"
              % ", ".join("%x" % m for m in kids))
        # ⚠ The compositor's own signalfd is the detector's positive control: a scan that finds nothing
        # anywhere proves nothing about the clients.
        holds = [l.split()[1] for l in full.splitlines()
                 if l.startswith("qemu_init: ") and "HOLDS A SIGNALFD" in l]
        check("compositor" in holds, "the fd scan sees the compositor's own signalfd (the detector works)")
        check("child" not in holds, "no client inherited the compositor's signalfd")
    elif scenario == "drag":
        img = [read_ppm(p) if p else None for p in shots]
        a = bbox(img[0], BRIGHT) if img and img[0] else None
        b = bbox(img[1], BRIGHT) if len(img) > 1 and img[1] else None
        check(a is not None, "start: the focused client is on screen, told it has focus (bright)")
        want = (a[0] + DRAG_DX, a[1] + DRAG_DY, a[2] + DRAG_DX, a[3] + DRAG_DY) if a else None
        check(b is not None and b == want,
              "a titlebar drag moved the window by exactly the pointer's travel, +%d +%d (%s -> %s)"
              % (DRAG_DX, DRAG_DY, a, b))
        flat = full.replace("\r", "")
        check("drag started -- a titlebar press was caught" in flat, "the compositor caught the titlebar press")
        check("drag released -- the window stopped, frames held:" in flat, "  ...and the release ended the drag")
        check("a grip press was caught, region:\n9\n" in flat,
              "a press on the window's left edge began a LEFT resize (region 9, DECO_RESIZE_L)")
        # ⭐ JUDGED ON THE COMPOSITOR'S OWN CHROME, not the client: focus_client never re-attaches at the new
        # size, so its surface says nothing about the window's. The titlebar does — its buttons sit at the
        # RIGHT edge and its title at the LEFT, and both are in the TITLEBAR_H rows above the content.
        c = img[2] if len(img) > 2 else None
        if b and c:
            held = same_shifted(img[1], c, b[2] - 70, b[1] - TITLEBAR_H, 71, TITLEBAR_H, 0)
            moved = same_shifted(img[1], c, b[0], b[1] - TITLEBAR_H, 100, TITLEBAR_H, LEFT_DX)
            check(held >= 0.99, "the left-edge drag held the RIGHT edge: the titlebar buttons did not move "
                  "(%.0f%% unchanged)" % (100 * held))
            check(moved >= 0.99, "  ...and moved the LEFT edge with the pointer: the title is %d px further "
                  "left (%.0f%% match)" % (-LEFT_DX, 100 * moved))
        else:
            check(False, "every screendump was taken (the guest died mid-run, or the window was not found)")
        montage([p for p in shots if p], os.path.join(outdir, "montage.png"))
    else:
        check("HOSTILE attach" in full, "the hostile client ran (stride 4 on a 240-wide surface)")
        check("surface refused" in full, "the compositor refused its surface")
        check(bool(shots and shots[0]) and len(pixels_of(read_ppm(shots[0]), BRIGHT)) > 0,
              "the desktop runs on, with the honest client told it has focus")
    print("  --- serial (clients and verdict lines) ---")
    for line in full.splitlines():
        if any(k in line for k in ("focus_client", "presented surface", "surface refused",
                                   "qemu_init: aethersafha", "KILLED", "SIGINT", "SigBlk", "SIGNALFD",
                                   "drag started", "drag released", "resize started", "pointer ")):
            print("    " + line.rstrip())
    print("  %s %s (screendumps and montage in %s)" % (
        "PASS" if not failures else "FAIL", scenario, outdir))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
