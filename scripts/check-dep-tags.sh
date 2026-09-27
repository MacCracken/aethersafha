#!/bin/sh
# check-dep-tags.sh — the gate `[deps.kavach]` asked for twice in comments and never got.
#
# ⛔ WHY THIS EXISTS. `path` WINS OVER `tag`: while a `path` line is live, every `cyrius build`
# re-materialises the vendored `lib/<dep>.cyr` from the local sibling checkout, whatever the manifest
# declares. The build stays green, `cyrius.lock` moves, and the DECLARED graph is the only thing that
# is wrong — silently. It fired four times on kavach (3.11.7, 3.11.10, 3.11.12, 3.11.14), each caught
# by a human reading the manifest, then five tags at once (2026-09-12, caught by this gate), then in a
# COMMIT (`0fdde56`, 2026-09-26) that neither target could build. A comment cannot fix a mechanism.
#
# ⭐ 0.16.26: THE MECHANISM IS CLOSED — every `path` line in `cyrius.cyml` is DORMANT (`# path = …`),
# so each dep resolves from its published tag and `cyrius.lock` pins the commit it resolved to. This
# gate now reads the dormant line as the sibling's LOCATION (the oracle it checks against) and FAILS
# on a live one: uncommenting a `path` for cross-repo work is fine, committing it is not.
#
# Per declared [deps.*] with a live or dormant `path`:
#   0. the `path` line is dormant                                   (live → FAIL)
#   1. declared tag vs sibling VERSION                              (differ → ⚠ note, not a failure)
#   2. that tag exists in the sibling's local git
#   3. that tag exists on the remote     (skip with --no-remote; needs network)
#   4. every vendored module is byte-identical to the sibling's copy AT THAT TAG
#   5. `cyrius.lock` pins that dep to the commit the tag names
# (4) is the one that matters: the others can all pass while `lib/` holds something else entirely.
# (5) is what proves the resolve came from the tag — a `path` resolve pins no commit at all.
# ⚠ (1) is a note since 0.16.26, not a failure. It used to be the drift detector — with `path` live, a
# sibling VERSION ahead of the tag meant `lib/` had moved. With `path` dormant, (4) and (5) catch drift
# directly, and a sibling that is ahead is only a newer release to take when this project chooses to.
#
# Exit: 0 all clean · 1 at least one mismatch · 2 could not run the check (never silently "pass").
set -u
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd) || exit 2
MAN="$ROOT/cyrius.cyml"
LOCK="$ROOT/cyrius.lock"
[ -r "$MAN" ] || { echo "check-dep-tags: cannot read $MAN" >&2; exit 2; }
[ -r "$LOCK" ] || { echo "check-dep-tags: cannot read $LOCK — run \`cyrius deps\` first" >&2; exit 2; }

REMOTE=1
[ "${1:-}" = "--no-remote" ] && REMOTE=0

TMP=$(mktemp -d) || exit 2
trap 'rm -rf "$TMP"' EXIT
fail=0; checked=0; indet=0

# Parse [deps.<name>] blocks: name, path, tag, modules, declared git url, and whether `path` is live.
# ⛔ THE HEADER RULE MUST EMIT THE PREVIOUS BLOCK BEFORE RESETTING. awk evaluates rules in order, so a
# `[deps.X]` line matches the header pattern FIRST — an emit rule placed after it never sees a block
# that is followed by another block. The first cut had exactly that shape and yielded ONE dep (the
# last, flushed by END) while reporting "clean", which is this project's oldest failure mode: a
# harness that scores a run it did not perform. The count guard below is the belt to this braces.
# ⚠ The dormant pattern is `#`, spaces, then `path =` — so prose that MENTIONS path (`# ⚠ \`path\`
# added …`, `# path-wins …`) does not match. A live line wins over a dormant one in the same block.
awk '
  function flush(   ) { if (name != "" && path != "") print name"\t"path"\t"tag"\t"mods"\t"url"\t"live }
  /^\[deps\.[A-Za-z0-9_-]+\]/ { flush(); name=$0; sub(/^\[deps\./,"",name); sub(/\]$/,"",name);
                                path=""; tag=""; mods=""; url=""; live=0; next }
  /^\[/                        { flush(); name=""; next }
  name!="" && /^[ \t]*git[ \t]*=/     { u=$0; sub(/^[^"]*"/,"",u); sub(/".*$/,"",u); url=u;  next }
  name!="" && /^[ \t]*path[ \t]*=/    { p=$0; sub(/^[^"]*"/,"",p); sub(/".*$/,"",p); path=p; live=1; next }
  name!="" && /^[ \t]*#[ \t]*path[ \t]*=[ \t]*"/ {
                                        if (live == 0) { p=$0; sub(/^[^"]*"/,"",p); sub(/".*$/,"",p); path=p }
                                        next }
  name!="" && /^[ \t]*tag[ \t]*=/     { t=$0; sub(/^[^"]*"/,"",t); sub(/".*$/,"",t); tag=t;  next }
  name!="" && /^[ \t]*modules[ \t]*=/ { m=$0; gsub(/^[^=]*=[ \t]*\[/,"",m); gsub(/\].*$/,"",m);
                                        gsub(/"/,"",m); gsub(/[ \t]/,"",m); mods=m; next }
  END { flush() }
' "$MAN" > "$TMP/deps" || exit 2

# ⛔ THE GATE MUST NOT SCORE A SUBSET. Count the [deps.<name>] headers that carry a `path` (live or
# dormant) directly from the manifest and demand the parser produced exactly that many. A parser that
# silently drops blocks reports "all clean" over work it never did — which is how the first cut passed.
# ⚠ Counted with the SAME flush-first shape as the parser above, and for the same reason: written as
# a plain `/^\[/` rule it is shadowed by the `[deps.<name>]` rule and counts 1. It did, on the first
# try — the guard caught its own counter, which is the only reason this line is right.
want=$(awk '/^\[deps\.[A-Za-z0-9_-]+\]/                      { if (inb && has) n++; inb=1; has=0; next }
            /^\[/                                            { if (inb && has) n++; inb=0; next }
            inb && /^[ \t]*path[ \t]*=/                      { has=1 }
            inb && /^[ \t]*#[ \t]*path[ \t]*=[ \t]*"/        { has=1 }
            END { if (inb && has) n++; print n+0 }' "$MAN")
got=$(wc -l < "$TMP/deps" | tr -d ' ')
if [ "$want" != "$got" ]; then
    echo "check-dep-tags: parsed $got path-deps but the manifest declares $want — parser is wrong, refusing to report" >&2
    exit 2
fi

[ -s "$TMP/deps" ] || { echo "check-dep-tags: parsed 0 path-deps from cyrius.cyml — parser or manifest changed" >&2; exit 2; }

# ⚠ A [deps.<name>] block with NO path line (live or dormant) has no sibling to check against, so it
# is not scored — say so by name rather than let "N deps clean" read as all of them.
unchecked=$(awk '/^\[deps\.[A-Za-z0-9_-]+\]/                      { if (inb && !has) print nm; nm=$0; sub(/^\[deps\./,"",nm); sub(/\]$/,"",nm); inb=1; has=0; next }
                 /^\[/                                            { if (inb && !has) print nm; inb=0; next }
                 inb && /^[ \t]*path[ \t]*=/                      { has=1 }
                 inb && /^[ \t]*#[ \t]*path[ \t]*=[ \t]*"/        { has=1 }
                 END { if (inb && !has) print nm }' "$MAN" | tr '\n' ' ')
[ -n "$unchecked" ] && echo "check-dep-tags: ⚠  NOT checked (no \`path\` line, live or dormant, to locate a sibling): $unchecked"

while IFS='	' read -r name path tag mods url live; do
    checked=$((checked + 1))
    sib="$ROOT/$path"
    label=$(printf '%-10s' "$name")

    # 0. a live `path` means `lib/` tracks the checkout, not the tag.
    if [ "$live" = 1 ]; then
        echo "$label ⛔ \`path = \"$path\"\` is LIVE — lib/ tracks the checkout, not tag $tag; make it dormant before a commit"
        fail=1
    fi

    [ -d "$sib/.git" ] || { echo "$label ⛔ no sibling checkout at $path"; fail=1; continue; }
    [ -n "$tag" ]      || { echo "$label ⚠  no tag declared — skipped"; continue; }

    # 1. tag vs sibling VERSION — a note, not a failure (see the header).
    ver=$(tr -d ' \n' < "$sib/VERSION" 2>/dev/null)
    if [ "$ver" != "$tag" ]; then
        if ! git -C "$sib" rev-parse -q --verify "refs/tags/$ver" >/dev/null 2>&1; then
            echo "$label ⚠  sibling is at $ver, not tagged yet — nothing newer to take (declared $tag)"
        elif [ "$(printf '%s\n%s\n' "$tag" "$ver" | sort -V | tail -1)" = "$ver" ]; then
            echo "$label ⚠  newer release $ver is tagged — declared $tag"
        else
            echo "$label ⚠  sibling checkout is at $ver, behind the declared $tag"
        fi
    fi

    # 2. tag exists locally
    if ! git -C "$sib" rev-parse -q --verify "refs/tags/$tag" >/dev/null 2>&1; then
        echo "$label ⛔ tag $tag does not exist in $path"; fail=1; continue
    fi

    # 3. tag exists on the remote.
    # ⛔ USE THE MANIFEST'S DECLARED `git =` URL, NOT the sibling's origin. They differ: origin here is
    # often SSH (git@github.com:...) with no key in CI or a sandbox, while the declared URL is the https
    # one the declared graph would actually fetch. The first cut of this gate read origin, got
    # "Permission denied (publickey)", and reported setu 0.8.7 ABSENT — a tag that is demonstrably
    # present. ⇒ A check that cannot look must say so, never vote. Same rule as the QEMU harnesses:
    # INCONCLUSIVE is a distinct outcome from FAIL, because a false accusation costs more than a gap.
    if [ "$REMOTE" = 1 ] && [ -n "$url" ]; then
        rl=$(timeout 25 git ls-remote --tags "$url" "refs/tags/$tag" 2>"$TMP/lserr")
        if [ -n "$rl" ]; then
            :
        elif [ -s "$TMP/lserr" ]; then
            echo "$label ⚠  remote unreachable ($(head -1 "$TMP/lserr" | cut -c1-48)) — tag NOT checked"
            indet=1
        else
            echo "$label ⛔ tag $tag is not on the remote — the declared graph is unfetchable"; fail=1
        fi
    fi

    # 4. THE ONE THAT MATTERS: vendored bytes == sibling's module at that tag.
    #    A vendored file is lib/<basename of module>, which is how `cyrius deps` lays it down.
    if [ -n "$mods" ]; then
        echo "$mods" | tr ',' '\n' | while read -r m; do
            [ -n "$m" ] || continue
            # ⚠ TWO VENDORING SHAPES, and assuming one costs four false accusations. `dist/<dep>.cyr`
            # lands as `lib/<dep>.cyr`, but a non-dist module is NAMESPACED on the way in:
            # kashi's `src/font_data.cyr` -> `lib/kashi_font_data.cyr`, mehman's `src/types.cyr` ->
            # `lib/mehman_types.cyr` (otherwise `types.cyr` from two deps would collide in one flat dir).
            # Try the plain name, then the namespaced one, before calling anything missing.
            base=$(basename "$m")
            vend="$ROOT/lib/$base"
            [ -f "$vend" ] || vend="$ROOT/lib/${name}_${base}"
            [ -f "$vend" ] || { echo "$label ⛔ vendored lib/$base (or lib/${name}_${base}) missing"; echo x >> "$TMP/fail"; continue; }
            if ! git -C "$sib" show "$tag:$m" > "$TMP/at_tag" 2>/dev/null; then
                echo "$label ⛔ $m does not exist at tag $tag"; echo x >> "$TMP/fail"; continue
            fi
            if ! cmp -s "$vend" "$TMP/at_tag"; then
                echo "$label ⛔ $(basename "$vend") DIFFERS from $m at tag $tag — not what the tag resolves to"
                echo x >> "$TMP/fail"
            fi
        done
    fi

    # 5. the lock pins the tag's commit. Line shape: `commit<TAB>sha<TAB>name<TAB>url<TAB>tag`.
    #    A `path` resolve writes no commit line, so a missing one means `lib/` did not come from the tag.
    want_sha=$(git -C "$sib" rev-parse -q --verify "refs/tags/$tag^{commit}" 2>/dev/null)
    lock_line=$(awk -F'\t' -v n="$name" '$1 == "commit" && $3 == n' "$LOCK")
    if [ -z "$lock_line" ]; then
        echo "$label ⛔ cyrius.lock pins no commit — this dep did not resolve from its tag (a live path, or a stale lock)"
        fail=1
    else
        lock_sha=$(printf '%s\n' "$lock_line" | awk -F'\t' 'NR == 1 { print $2 }')
        lock_tag=$(printf '%s\n' "$lock_line" | awk -F'\t' 'NR == 1 { print $5 }')
        if [ "$lock_tag" != "$tag" ]; then
            echo "$label ⛔ cyrius.lock pins tag $lock_tag, the manifest declares $tag — re-run \`cyrius deps\`"
            fail=1
        elif [ "$lock_sha" != "$want_sha" ]; then
            echo "$label ⛔ cyrius.lock pins $lock_sha for $tag, but the tag names $want_sha — a moved tag or a stale lock"
            fail=1
        fi
    fi
done < "$TMP/deps"

[ -f "$TMP/fail" ] && fail=1
if [ "$fail" = 0 ]; then
    if [ "$indet" = 1 ]; then
        echo "check-dep-tags: $checked deps clean on every check that RAN — but at least one remote"
        echo "                could not be reached, so remote-tag existence is UNVERIFIED, not confirmed."
        exit 0
    fi
    echo "check-dep-tags: $checked deps clean — path dormant, tag exists local+remote, vendored bytes match the tag, lock pins its commit"
    exit 0
fi
echo "check-dep-tags: FAILED — see above. Re-verify before any cut or burn." >&2
exit 1
