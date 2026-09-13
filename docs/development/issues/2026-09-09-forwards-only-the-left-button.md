# aethersafha forwards only the left button, so no client can see a right-click

**Status:** ✅ **CLOSED in 0.16.24 (2026-09-12).** Every kernel button bit is forwarded; the numbering
is `wire = kernel_bit + 1` (**1 = left, 2 = right, 3 = middle**), named as `INPUT_BTN_LEFT` /
`INPUT_BTN_RIGHT` / `INPUT_BTN_MIDDLE` in `src/input.cyr`, X11 divergence pinned by the input suite.
Window management stays left-only structurally — the other buttons live in a forward-only loop that
contains no `deco_hit`, no focus and no drag. ⚠ Focus is still a left-button gesture (a right-click
reaches an unfocused window's client without focusing it) — deliberately not widened; a policy
decision, not the wire. ⚠ Not yet run on QEMU or iron; the first consumer's on-target run is the
end-to-end verdict. See `CHANGELOG.md` `[0.16.24]`.
⚠ **The blocker-behind-the-blocker below is HISTORY.** 0.16.23 migrated this repo to cyrius 6.6.2 and
the three disagreeing deps moved with it (sigil 3.12.17 · agnostik 1.6.1 · agnodrm 1.6.0); it builds.
The table and the pin-skew analysis are kept because the shape — a language change migrated in one
dep and not its peers — is the transferable part.
**Filed by:** crab, 2026-09-09, while closing the M6 interaction gaps (crab 0.8.3).
**Severity:** Medium — it does not lose data; it makes one whole class of gesture unreachable for
every client on the desktop, not just crab.

## What crab needs

crab's context menu has no pointer route: right-click runs the ordinary left-click path. That is not
a wiring gap in crab. **crab cannot tell a right-click from a left one, because the information does
not arrive.**

- The **kernel already has it**: `hid_mouse_btn` is a bitmap, documented `bit0 left, bit1 right,
  bit2 middle` (`agnos/kernel/arch/x86_64/usb/hid.cyr:311`), and `hid_mouse_take` publishes both the
  current bitmap and the OR-folded `buttons_seen` (`:398-404`).
- **bhumi passes it through intact**: `bhumi_button_state(ev) = ev & 0xFF`,
  `bhumi_button_seen(ev) = (ev >> 8) & 0xFF` (`aethersafha/lib/bhumi.cyr:866-867`).
- **setu's wire carries it**: `SETU_INPUT_PTR_BTN S->C id, button, state`, and `button` is a full
  i64 argument.
- **dhancha delivers it**: `POINTER_BTN` carries the code in `a` and the state in `b`.

⛔ **The single point of loss is aethersafha.** `src/main.cyr:528` reads
`input_btn_transitions(ae_ptr_btn, cur, seen, 1)` — mask `1`, commented *"left button only, for
now"* — and `:594` / `:614` then call `ae_ptr_forward(comp, 1, 1, pressed)` with the button number
**hardcoded to 1**. Every other button is discarded before it reaches the wire.

## The fix, which is small

`input_btn_transitions(believed_down, cur, seen, mask)` is **already per-button**: it takes a mask.
`ae_ptr_forward(comp, kind_is_btn, btn, pressed)` **already takes a button number**. What is missing
is only that `ae_ptr_btn` is a scalar tracking bit 0, and the two call sites pass a constant.

- Make `ae_ptr_btn` a real bitmask (its own comment already calls it *"last button bitmap"*), one
  believed-down bit per button.
- Derive transitions per button and forward the real number.
- ⛔⛔ **WINDOW MANAGEMENT MUST STAY LEFT-ONLY.** The press arm does click-to-focus, `deco_hit`
  close/maximize/minimize, and drag-start. A naive loop over all buttons would make a **right-click
  close a window**. Left keeps the whole existing path — including the `closed` guard, which exists
  because forwarding after a close re-derives `comp_window_at` and delivers the event to whatever was
  underneath. Right and middle forward *only*.
- ⚠ Keep the `closed` guard for the extra buttons too: a left-press that closed a window and a
  right transition in the same event must not deliver the right button to the window beneath.

## ⛔ THE NUMBERING IS A DECISION, AND NOBODY HAS MADE IT

No repository in the stack defines a button constant. `setu` and `dhancha` both carry `button` as an
opaque integer, and **aethersafha is its only producer** — the only fact in existence is that it
sends `1` for left.

⚠ **Do not default to X11.** X11 is 1=left, 2=middle, 3=right, and this stack has already
deliberately diverged from X11 on the neighbouring question: `setu` gives the wheel its **own kind**
rather than spending buttons 4/5 on detents, and says why — *"that would make `SETU_INPUT_PTR_BTN`'s
`button` arg mean two different things and silently turn a scroll into a click on any client that
range-checks buttons"*.

⇒ **Recommendation: mirror the kernel's own bit order**, so `wire = kernel_bit + 1`:
`1 = left, 2 = right, 3 = middle`. It agrees with the one existing fact (left = 1 = bit 0 + 1), it is
derivable rather than remembered, and it keeps a single source of truth for the ordering. Whichever
is chosen, **name it in constants and document the X11 divergence**, or the next reader will assume
X11 and put Delete on the middle button.

## ⛔⛔ WHY THIS COULD NOT BE FIXED: aethersafha DOES NOT BUILD

Attempted 2026-09-09. This is the blocker in front of the blocker.

| toolchain | result |
|---|---|
| `6.5.33` (its own pin) | **Not installed and cannot be installed** — `cyrius install 6.5.33` answers *"Package registry not yet available."* The toolchain also warns the pin *"carries the v6.5.36 enum Critical (constants >= 2^62 read back as -1). Re-pin to 6.5.36 or later."* |
| `6.5.36` (the prescribed floor) | Same — not installable. Only `6.6.0` and `6.6.1` exist locally. |
| `6.6.0` | **57 errors**, all in vendored `lib/` |
| `6.6.1` | **57 errors**, all in vendored `lib/` |

The 57 errors are **not** in aethersafha's own source, and **not** caused by re-resolving deps —
they reproduce against the committed `lib/`. They are dependency-versus-dependency conflicts:

- `a ': stack' enum returns two values — bind both` throughout `lib/agnodrm.cyr` and
  `lib/agnostik.cyr` (a language change these were never updated for);
- `duplicate fn 'result_print_err' disagrees about arity: this one takes 1, the one in
  lib/sigil.cyr takes 2 (last definition wins, so calls to the other arity would silently
  mis-bind)`;
- `'result_unwrap' expects 2 arguments, got 1`, `'err_code_of' expects 2 arguments, got 1`,
  `'result_unwrap_or' expects 3 arguments, got 2`.

### ⭐ THE ACTUAL CAUSE: A PIN SKEW ACROSS THREE REPOS, NOT A MYSTERY

⛔ **cyrius changed `: stack` enum functions to return TWO values** (`var tag, val = f();`).
`sigil` was migrated for it; `agnostik` and `agnodrm` were not. That is the whole disagreement.

| repo | cyrius pin | migrated for `: stack`? |
|---|---|---|
| `sigil` 3.12.16 | **6.6.0** | ✅ yes — hence its 2-arg `result_unwrap` / `result_print_err` |
| `agnostik` 1.5.1 | **6.5.35** | ❌ no |
| `agnodrm` 1.5.3 | **6.5.35** | ❌ no |

⚠ Every one is already at its repository's highest tag, so **there is nothing newer to pull** — the
skew is in the pins, not in the releases.

⇒ **The work is a migration, and it is mechanical.** Bumping `agnostik` to 6.6.1 and building it
reports **291 call sites** needing `var tag, val = f();`; `agnodrm` has its own set. Then
`cyrius distlib` in each so aethersafha resolves against fresh dists, then aethersafha's own pin
moves off the uninstallable `6.5.33`, and only then is the ten-line button fix above reachable.

⚠ **Order matters**: `agnostik` and `agnodrm` first (they are what disagree with `sigil`), dists
regenerated, then `aethersafha`. `path` overrides mean the sibling's `dist/` is what compiles, not
the vendored `lib/` — so a source fix with no `cyrius distlib` behind it changes nothing.

⛔ **crab deliberately shipped nothing rather than route around this.** The two things that would
have been available — hand-editing `lib/` (forbidden: vendored) or pushing an unverified change to a
repo that cannot be compiled — are both worse than the gap. The M6 context-menu pointer route stays
open and its reason is now written down.

## What crab does in the meantime

Nothing, and that is the point. The context menu keeps its keyboard route (the Menu key), which
works. crab does **not** ship a guessed button code, for the same reason it declined to ship a mount
probe before agnos minted `mountlist`#104: a guess that happens to work is indistinguishable from a
contract until the day it changes.
