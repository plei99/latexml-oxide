#!/usr/bin/env bash
# run.sh <label> [id-regex]
# Convert every document of $CORPUS/corpus.tsv (id<TAB>abs path) into $CORPUS/runs/<label>/,
# JOBS at a time (default 6), then index the run (index.py). CORPUS defaults to <repo>/.corpus.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../.." && pwd)"
CORPUS=$(cd "${CORPUS:-$REPO/.corpus}" && pwd); label=$1; filt=${2:-.*}
export OUTROOT=$CORPUS/runs/$label; mkdir -p "$OUTROOT"
git -C "$REPO" rev-parse HEAD > "$OUTROOT/COMMIT"
grep -E "^($filt)	" "$CORPUS/corpus.tsv" | while IFS=$'\t' read -r i f; do printf '%s\0%s\0' "$i" "$f"; done \
  | xargs -0 -n2 -P"${JOBS:-6}" "$HERE/run_one.sh"
python3 "$HERE/index.py" "$OUTROOT"
