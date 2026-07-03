#!/usr/bin/env bash
set -euo pipefail

# Bump the project version. VERSION is the single source of truth; cyrius.cyml's
# [package] version must match it — the release CI (.github/workflows/release.yml)
# rejects a tag unless VERSION == cyrius.cyml [package].version == the pushed tag.
# This does NOT touch the `cyrius = "X.Y.Z"` toolchain pin (a different key).
NEW_VERSION="${1:?Usage: $0 <new-version>}"

echo "$NEW_VERSION" > VERSION

# Update the [package] version line — the FIRST `version = "..."` in the manifest —
# leaving the `cyrius = "..."` toolchain pin (a different key) untouched.
sed -i "0,/^version = \".*\"/s//version = \"$NEW_VERSION\"/" cyrius.cyml

# Self-check: both must now equal NEW_VERSION, or the release CI would reject the tag.
FILE_V="$(tr -d '[:space:]' < VERSION)"
CYML_V="$(grep -m1 '^version = ' cyrius.cyml | sed 's/version = "\(.*\)"/\1/')"
if [ "$FILE_V" != "$NEW_VERSION" ] || [ "$CYML_V" != "$NEW_VERSION" ]; then
    echo "error: bump failed — VERSION=[$FILE_V] cyrius.cyml=[$CYML_V], expected [$NEW_VERSION]" >&2
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
