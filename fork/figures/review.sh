#!/usr/bin/env bash
# review.sh [label] — the figure loop over the notes: extract every figure into a mini-document →
# reference PDFs (only for changed figures) → convert with the current binary → screenshots →
# scores.tsv + gallery.html, worst first. Label defaults to figs-<short HEAD>[-dirty].
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../.." && pwd)"
export CORPUS=${FIGURES:-$REPO/.corpus/figs}
dirty=$(git -C "$REPO" diff --quiet HEAD -- . ':!fork' || echo -dirty)
label=${1:-figs-$(git -C "$REPO" rev-parse --short=8 HEAD)$dirty}
python3 "$HERE/extract.py" "$REPO/.corpus/corpus.tsv" "$CORPUS"
stale=$(while IFS=$'\t' read -r i f; do [ "$CORPUS/refs/$i/engine.tsv" -nt "$f" ] || printf '%s|' "$i"; done < "$CORPUS/corpus.tsv")
[ -n "$stale" ] && "$REPO/fork/corpus/refs.sh" "${stale%|}"
"$REPO/fork/corpus/run.sh" "$label"
run=$CORPUS/runs/$label
cut -f1 "$CORPUS/corpus.tsv" | while read -r i; do printf '%s\0%s\0' "$run/$i" "$CORPUS/refs/$i"; done \
  | FIG_ONLY=1 xargs -0 -n2 -P"${SHOTS:-4}" "$REPO/fork/fixtures/shot_one.sh"
python3 "$HERE/score.py" "$CORPUS" "$run" && echo "gallery: $run/gallery.html"
