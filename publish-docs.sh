#!/usr/bin/env bash
# Build the API docs and publish them to the gh-pages branch.
#
# docs.rs cannot build these crates: they need the rust-z80 fork for the SM83
# target and the features the runtime uses. So the docs are built here, where
# that toolchain is, and pushed as a branch GitHub Pages serves.
#
# Usage: ./publish-docs.sh [--dry-run]

set -euo pipefail

BRANCH=gh-pages
ROOT_CRATE=gb
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKTREE="$REPO/target/gh-pages"

cd "$REPO"

if [ -n "$(git status --porcelain)" ]; then
    echo "working tree is dirty; commit or stash first" >&2
    exit 1
fi
REV="$(git rev-parse --short HEAD)"

# The library crates only; `--workspace` would pull in the examples. The four
# proc-macro crates document into the host tree instead and are left to
# docs.rs, which can build them without the fork.
cargo doc --all-features --no-deps
DOC="$(ls -d target/*/doc | head -1)"
[ -d "$DOC" ] || { echo "no doc output under target/" >&2; exit 1; }

# Pages serves the branch root, and rustdoc writes no index there.
cat > "$DOC/index.html" <<EOF
<!doctype html>
<meta charset="utf-8">
<meta http-equiv="refresh" content="0; url=$ROOT_CRATE/index.html">
<title>rust-gb</title>
<a href="$ROOT_CRATE/index.html">rust-gb documentation</a>
EOF

# Without this, Pages runs Jekyll, which drops rustdoc's `static.files` and
# anything else beginning with an underscore.
touch "$DOC/.nojekyll"

if [ "${1-}" = "--dry-run" ]; then
    echo "built $DOC for $REV; not publishing"
    exit 0
fi

rm -rf "$WORKTREE"
if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
    git worktree add -q "$WORKTREE" "$BRANCH"
else
    git worktree add -q --orphan -B "$BRANCH" "$WORKTREE"
fi

# Replace rather than merge, so a page for a removed item does not linger.
find "$WORKTREE" -mindepth 1 -maxdepth 1 ! -name .git -exec rm -rf {} +
cp -a "$DOC/." "$WORKTREE/"

cd "$WORKTREE"
git add -A
if git diff --cached --quiet; then
    echo "docs unchanged at $REV"
else
    git commit -q -m "docs: build from $REV"
    git push -q origin "$BRANCH"
    echo "published $REV to $BRANCH"
fi

cd "$REPO"
git worktree remove --force "$WORKTREE"
