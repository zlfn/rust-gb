#!/usr/bin/env bash
# Build the API docs and publish them to the gh-pages branch.
# docs.rs cannot build these crates, since they need the rust-z80 toolchain.
#
# Usage: ./publish-docs.sh [--dry-run]

set -euo pipefail

BRANCH=gh-pages
ROOT_CRATE=gb
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="$REPO/target/doc-build"
DOC="$TARGET_DIR/sm83-nintendo-none-elf/doc"

cd "$REPO"

if [ -n "$(git status --porcelain)" ]; then
    echo "working tree is dirty; commit or stash first" >&2
    exit 1
fi
REV="$(git rev-parse --short HEAD)"

# A separate target dir, so the doc flags do not invalidate the ROM builds.
cargo doc-all --target-dir "$TARGET_DIR"

# Pages serves the branch root, where rustdoc writes no index.
cat > "$DOC/index.html" <<EOF
<!doctype html>
<meta charset="utf-8">
<meta http-equiv="refresh" content="0; url=$ROOT_CRATE/index.html">
<title>rust-gb</title>
<a href="$ROOT_CRATE/index.html">rust-gb documentation</a>
EOF

# Without this, Jekyll on Pages drops `static.files` and other `_` files.
touch "$DOC/.nojekyll"

if [ "${1-}" = "--dry-run" ]; then
    echo "built $DOC for $REV; not publishing"
    exit 0
fi

# Snapshot the doc tree through a scratch index, without a checkout.
INDEX="$TARGET_DIR/gh-pages.index"
rm -f "$INDEX"
GIT_INDEX_FILE="$INDEX" git -C "$DOC" --work-tree="$DOC" add -A -f -- . ':!.lock'
TREE="$(GIT_INDEX_FILE="$INDEX" git write-tree)"
rm -f "$INDEX"

# Commit on top of the published branch, so the push is a fast-forward.
if git fetch -q origin "$BRANCH" 2>/dev/null; then
    PARENT="$(git rev-parse FETCH_HEAD)"
    if [ "$(git rev-parse "$PARENT^{tree}")" = "$TREE" ]; then
        echo "docs unchanged at $REV"
        exit 0
    fi
    COMMIT="$(git commit-tree "$TREE" -p "$PARENT" -m "docs: build from $REV")"
else
    COMMIT="$(git commit-tree "$TREE" -m "docs: build from $REV")"
fi
git push -q origin "$COMMIT:refs/heads/$BRANCH"
echo "published $REV to $BRANCH"
