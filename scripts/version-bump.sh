#!/usr/bin/env bash
set -euo pipefail

# Bump the project version. VERSION is the single source of truth; cyrius.cyml's
# [package] version must match it — the release CI (.github/workflows/release.yml)
# rejects a tag unless VERSION == cyrius.cyml [package].version == the pushed tag.
# This does NOT touch the `cyrius = "X.Y.Z"` toolchain pin (a different key).
NEW_VERSION="${1:?Usage: $0 <new-version>}"

echo "$NEW_VERSION" > VERSION

# ⛔⛔ THE MANIFEST INTERPOLATES; THIS SCRIPT MUST NOT WRITE A LITERAL BACK INTO IT.
# It used to `sed` NEW_VERSION straight over the `version = "..."` line, which is what kept
# aethersafha the ONE repo in the stack still carrying a duplicated version literal — while the
# comment above that line, and release.yml's own comment, both asserted it was interpolated.
# Every sibling (puka / crab / setu / bhumi / agnoshi / kriya) uses "${file:VERSION}", and
# release.yml has resolved that form since it was written. A comment that describes the fix and a
# script that silently undoes it is worse than neither. [[feedback_audit_re_derive_dont_validate_comments]]
#
# ⇒ NORMALISE rather than assign: if the manifest still holds a stale literal, convert it to the
# interpolated form once and say so. After this the version lives in exactly one file.
CYML_RAW="$(grep -m1 '^version = ' cyrius.cyml | sed 's/version = "\(.*\)"/\1/')"
if [ "$CYML_RAW" != '${file:VERSION}' ]; then
    echo "note: cyrius.cyml carried a literal version (\"$CYML_RAW\") — converting to \${file:VERSION}"
    sed -i '0,/^version = ".*"/s//version = "${file:VERSION}"/' cyrius.cyml
fi

# Self-check, resolving EXACTLY the way .github/workflows/release.yml does — so a green run here
# means the tag will be accepted, rather than meaning this script agrees with itself.
FILE_V="$(tr -d '[:space:]' < VERSION)"
CYML_RAW="$(grep -m1 '^version = ' cyrius.cyml | sed 's/version = "\(.*\)"/\1/')"
if [ "$CYML_RAW" = '${file:VERSION}' ]; then CYML_V="$FILE_V"; else CYML_V="$CYML_RAW"; fi
if [ "$FILE_V" != "$NEW_VERSION" ] || [ "$CYML_V" != "$NEW_VERSION" ]; then
    echo "error: bump failed — VERSION=[$FILE_V] cyrius.cyml=[$CYML_RAW -> $CYML_V], expected [$NEW_VERSION]" >&2
    exit 1
fi

echo "Bumped to $NEW_VERSION (VERSION + cyrius.cyml [package].version; toolchain pin untouched)"
echo ""
echo "Next steps (git is yours to run):"
echo "  # finalize CHANGELOG.md: rename '## [Unreleased]' -> '## [$NEW_VERSION] - <date> — <title>'"
echo "  git add VERSION cyrius.cyml CHANGELOG.md"
echo "  git commit -m 'release: $NEW_VERSION'"
echo "  git tag $NEW_VERSION        # bare — NO leading 'v'; CI requires tag == VERSION"
echo "  git push origin main --tags"
