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

# Several crates gate their API on `--cfg` rather than on a cargo feature, and
# say so under `[package.metadata.docs.rs]`. Only docs.rs reads that, so gather
# it here too; without it those modules document as if they did not exist.
#
# The flags go to the compiler as well as to rustdoc. A facade documents its
# re-exports from the dependency's compiled metadata, so a cfg that reached only
# rustdoc would still leave `gb::pak` holding a fraction of `gb_pak`.
RUSTDOCFLAGS="$(cargo metadata --format-version 1 --no-deps | python3 -c '
import json, sys

args = []
for pkg in json.load(sys.stdin)["packages"]:
    rs = (pkg.get("metadata") or {}).get("docs", {}).get("rs", {})
    args += rs.get("rustdoc-args", [])

out, seen, i = [], set(), 0
while i < len(args):
    pair = tuple(args[i : i + 2]) if args[i] == "--cfg" else (args[i],)
    if pair not in seen:
        seen.add(pair)
        out += pair
    i += len(pair)
print(" ".join(out))')"
export RUSTDOCFLAGS
export RUSTFLAGS="$RUSTDOCFLAGS"

# The library crates only; `--workspace` would pull in the examples. The four
# proc-macro crates document into the host tree instead and are left to
# docs.rs, which can build them without the fork.
# Its own directory: these flags fingerprint differently from a ROM build, and
# sharing one would rebuild the world on either side of every docs run.
cargo doc --all-features --no-deps --target-dir "$REPO/target/doc-build"
DOC="$(ls -d "$REPO"/target/doc-build/*/doc | head -1)"
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
