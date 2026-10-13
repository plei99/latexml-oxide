#!/usr/bin/env bash
# review.sh [label] — the fixture loop: split → reference PDFs (only for changed features) →
# convert with the current binary → index → screenshots → gallery.html, and a comparison against
# the accepted fixture baseline when there is one. Label defaults to fx-<short HEAD>[-dirty].
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../.." && pwd)"
export CORPUS=${FIXTURES:-$REPO/.corpus/fixtures}
dirty=$(git -C "$REPO" diff --quiet HEAD -- . ':!fork' || echo -dirty)
label=${1:-fx-$(git -C "$REPO" rev-parse --short=8 HEAD)$dirty}
python3 "$HERE/split.py" "$CORPUS"
stale=$(while IFS=$'\t' read -r i f; do [ "$CORPUS/refs/$i/engine.tsv" -nt "$f" ] || printf '%s|' "$i"; done < "$CORPUS/corpus.tsv")
[ -n "$stale" ] && "$REPO/fork/corpus/refs.sh" "${stale%|}"
"$REPO/fork/corpus/run.sh" "$label"
run=$CORPUS/runs/$label
cut -f1 "$CORPUS/corpus.tsv" | while read -r i; do printf '%s\0%s\0' "$run/$i" "$CORPUS/refs/$i"; done \
  | xargs -0 -n2 -P1 "$HERE/shot_one.sh"   # headless Chrome instances collide when run in parallel
python3 "$HERE/gallery.py" "$run" > "$run/gallery.html" && echo "gallery: $run/gallery.html"
[ -e "$CORPUS/accepted" ] && python3 "$REPO/fork/corpus/compare.py" "$CORPUS/accepted" "$run" | tee "$run/compare.md"
exit 0
