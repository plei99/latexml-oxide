#!/usr/bin/env bash
# accept.sh <label> — make runs/<label> the accepted baseline that check.sh compares against,
# and record its index in the $CORPUS git history (index/ + BASELINE), one commit per acceptance.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../.." && pwd)"
CORPUS=$(cd "${CORPUS:-$REPO/.corpus}" && pwd); label=$1; run=$CORPUS/runs/$label
[ -f "$run/index/docs.tsv" ] || { echo "no index for $run (run index.py first)" >&2; exit 1; }
cd "$CORPUS"
[ -d .git ] || { git init -q; printf 'runs/\nrefs/\naccepted\n*.out\nfixtures/\nsrc/\n' > .gitignore; }
ln -sfn "runs/$label" accepted
rm -rf index; cp -R "$run/index" index
# BASELINE: label, oxide commit, protocol (a run without a PROTOCOL file predates the knob: raw)
printf '%s\t%s\t%s\n' "$label" "$(cat "$run/COMMIT")" "$(cat "$run/PROTOCOL" 2>/dev/null || echo raw)" > BASELINE
git add -A . && git commit -qm "accept $label (oxide $(cut -c1-8 "$run/COMMIT"))" && git log --oneline -1
