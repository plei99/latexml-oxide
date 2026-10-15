#!/usr/bin/env bash
# fork/worktree.sh <branch> [path] [base] — a checkout of its own for one writer agent. Writers sharing a working
# tree and target/ have broken one another: one writer's half-done edit failed another's build, a stray edit
# from one fix dropped every table of contents in another's review, and a rebuild swapped the binary under a
# running review. Creates <branch> from [base] (default HEAD) at [path] (default <repo>/../latexml-wt/<branch>),
# or reuses the branch if it exists, and supplies what git does not carry: LaTeXML/ (the Perl source, linked),
# resources/dumps/ (copied; build.rs embeds them) and .corpus/ (linked; runs are labelled per commit).
# The worktree builds into its own target/ from scratch. Remove it with `git worktree remove <path>`.
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
branch=${1:?usage: fork/worktree.sh <branch> [path] [base]}
wt=${2:-$REPO/../latexml-wt/$branch}; base=${3:-HEAD}
if git -C "$REPO" show-ref -q --verify "refs/heads/$branch"; then
  git -C "$REPO" worktree add -q "$wt" "$branch"
else
  git -C "$REPO" worktree add -q -b "$branch" "$wt" "$base"
fi
wt=$(cd "$wt" && pwd)
ln -sfn "$REPO/.upstream" "$wt/LaTeXML"
[ -d "$REPO/.corpus" ] && ln -sfn "$REPO/.corpus" "$wt/.corpus"
mkdir -p "$wt/resources/dumps" && cp "$REPO"/resources/dumps/* "$wt/resources/dumps/"
echo "$wt ($branch at $(git -C "$wt" rev-parse --short HEAD))"
