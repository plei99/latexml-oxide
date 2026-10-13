#!/usr/bin/env bash
# refs.sh [id-regex] — typeset reference PDFs for $CORPUS/corpus.tsv into $CORPUS/refs/ (JOBS, default 4).
# References depend only on the sources and TeX Live: rebuild after editing a document or upgrading TL.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../.." && pwd)"
CORPUS=$(cd "${CORPUS:-$REPO/.corpus}" && pwd); filt=${1:-.*}
export OUTROOT=$CORPUS/refs; mkdir -p "$OUTROOT"
grep -E "^($filt)	" "$CORPUS/corpus.tsv" | while IFS=$'\t' read -r i f; do printf '%s\0%s\0' "$i" "$f"; done \
  | xargs -0 -n2 -P"${JOBS:-4}" "$HERE/ref_one.sh"
