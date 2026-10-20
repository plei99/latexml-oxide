#!/usr/bin/env bash
# fork/worktree.sh <branch> [base] — a checkout of its own for one writer agent, on the OrbStack machine `agents`.
# Writers sharing a working tree and target/ have broken one another: one writer's half-done edit failed another's
# build, a stray edit from one fix dropped every table of contents in another's review, and a rebuild swapped the
# binary under a running review.
#
# Creates <branch> from [base] (a commit of the invoking checkout, default its HEAD) in the guest clone, at
# /home/patrick/agent-work/worktrees/latexml/<branch with / as ->, or reuses the branch if the guest clone has it (a
# name that only the Mac checkout has is refused), and supplies what git does not carry: LaTeXML/ (a link to the Mac
# checkout's .upstream, the Perl source, which resolves on both hosts), and Cargo.lock and resources/dumps/ (copied
# from the invoking checkout, else the Mac checkout). It prints
# the Linux path and its macOS view. Bind a T3 thread to it with T3's workspace controls (a shell `cd` does not), and
# build and test in it with fork/linux/run.sh. The corpus loop (fork/oxide, .corpus/) stays on the Mac checkout.
# Remove it with `git worktree remove <path>` in the guest.
set -euo pipefail
MACHINE=agents
MAC_REPO=${MAC_REPO:-/Users/patrick/.t3/projects/latexml}
GUEST_CLONE=/home/patrick/agent-work/repos/latexml
WT_ROOT=/home/patrick/agent-work/worktrees/latexml
MAC_VIEW=/Users/patrick/OrbStack/$MACHINE   # the guest's / as macOS sees it

usage() { echo "usage: fork/worktree.sh <branch> [base]   (base: a commit of this checkout, default HEAD)" >&2; exit 2; }
[ $# -ge 1 ] && [ $# -le 2 ] || usage
case $1 in -*) usage ;; esac
branch=$1

if [ "$(uname -s)" = Darwin ]; then
  here=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
  base=$(git -C "$here" rev-parse --verify -q "${2:-HEAD}^{commit}") || { echo "fork/worktree.sh: ${2:-HEAD} is not a commit of $here" >&2; exit 2; }
  # The guest sees the Mac's /Users under the same path, and its own files outside the OrbStack view.
  case $here in "$MAC_VIEW"/*) here=${here#"$MAC_VIEW"} ;; esac
  orb -m "$MACHINE" true >/dev/null 2>&1 || { echo "fork/worktree.sh: machine $MACHINE is not running (orb start $MACHINE)" >&2; exit 2; }
  # The script travels as an argument; the guest runs it with the invoking checkout and the resolved commit.
  exec orb -m "$MACHINE" env MAC_REPO="$MAC_REPO" WT_FROM="$here" bash -c "$(cat "$0")" worktree.sh "$branch" "$base"
fi

if [ -n "${WT_FROM:-}" ]; then
  here=$WT_FROM base=$2   # from the Mac side: a commit already
else
  here=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
  base=$(git -C "$here" rev-parse --verify -q "${2:-HEAD}^{commit}") || { echo "fork/worktree.sh: ${2:-HEAD} is not a commit of $here" >&2; exit 2; }
fi
[ -d "$GUEST_CLONE/.git" ] || { echo "fork/worktree.sh: no guest clone at $GUEST_CLONE (fork/linux/setup.sh)" >&2; exit 2; }
if ! git -C "$GUEST_CLONE" cat-file -e "$base^{commit}" 2>/dev/null; then
  # A commit of the Mac checkout reaches the guest clone through the remote `mac`: by itself (a detached HEAD
  # is on no branch), else with the Mac's branches, as fork/linux/check.sh fetches.
  git -C "$GUEST_CLONE" fetch -q mac "$base" 2>/dev/null || git -C "$GUEST_CLONE" fetch -q mac
  git -C "$GUEST_CLONE" cat-file -e "$base^{commit}" 2>/dev/null \
    || { echo "fork/worktree.sh: the guest clone cannot fetch $base from $MAC_REPO" >&2; exit 2; }
fi
base=$(git -C "$GUEST_CLONE" rev-parse "$base^{commit}")

wt=$WT_ROOT/${branch//\//-}
[ ! -e "$wt" ] || { echo "fork/worktree.sh: $wt already exists" >&2; exit 2; }
if ! git -C "$GUEST_CLONE" show-ref -q --verify "refs/heads/$branch" \
  && git -C "$GUEST_CLONE" ls-remote --exit-code -q mac "refs/heads/$branch" >/dev/null 2>&1; then
  # Not a second branch of the same name, started from elsewhere.
  echo "fork/worktree.sh: $branch is a branch of $MAC_REPO, not of the guest clone; pass its commit as base under another name" >&2
  exit 2
fi
if git -C "$GUEST_CLONE" show-ref -q --verify "refs/heads/$branch"; then
  git -C "$GUEST_CLONE" worktree add -q --relative-paths "$wt" "$branch"
else
  git -C "$GUEST_CLONE" worktree add -q --relative-paths -b "$branch" "$wt" "$base"
fi

if [ -d "$MAC_REPO/.upstream" ]; then ln -sfn "$MAC_REPO/.upstream" "$wt/LaTeXML"; else
  echo "fork/worktree.sh: no Perl source at $MAC_REPO/.upstream; LaTeXML/ not linked" >&2; fi
for from in "$here" "$MAC_REPO"; do
  [ -f "$from/Cargo.lock" ] && { cp "$from/Cargo.lock" "$wt/"; break; }
done
for from in "$here" "$MAC_REPO"; do
  if compgen -G "$from/resources/dumps/*.dump.txt" >/dev/null; then
    mkdir -p "$wt/resources/dumps" && cp "$from"/resources/dumps/* "$wt/resources/dumps/"
    break
  fi
done
[ -f "$wt/Cargo.lock" ] || echo "fork/worktree.sh: no Cargo.lock copied; cargo will resolve one" >&2
compgen -G "$wt/resources/dumps/*.dump.txt" >/dev/null \
  || echo "fork/worktree.sh: no dumps copied; run fork/linux/run.sh tools/make_formats.sh" >&2
echo "$wt ($branch at $(git -C "$wt" rev-parse --short HEAD))"
echo "macOS view: $MAC_VIEW$wt"
