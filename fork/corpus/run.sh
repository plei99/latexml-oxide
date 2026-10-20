#!/usr/bin/env bash
# run.sh <label> [id-regex]
# Convert every document of $CORPUS/corpus.tsv (id<TAB>abs path) into $CORPUS/runs/<label>/,
# JOBS at a time (default 6), then index the run (index.py). CORPUS defaults to <repo>/.corpus.
# OXIDE_PROTOCOL=raw (default) or native (run_one.sh: `localrawstyles`, TeX Live packages through bindings only);
# checked before anything is written, and recorded in the run directory's PROTOCOL file beside COMMIT and BINARY.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../.." && pwd)"
CORPUS=$(cd "${CORPUS:-$REPO/.corpus}" && pwd); label=$1; filt=${2:-.*}
case ${OXIDE_PROTOCOL:-raw} in raw|native) ;; *) echo "run.sh: OXIDE_PROTOCOL must be raw or native, not ${OXIDE_PROTOCOL}" >&2; exit 2 ;; esac
export OXIDE_PROTOCOL=${OXIDE_PROTOCOL:-raw}
export OUTROOT=$CORPUS/runs/$label; mkdir -p "$OUTROOT"
git -C "$REPO" rev-parse HEAD > "$OUTROOT/COMMIT"
echo "$OXIDE_PROTOCOL" > "$OUTROOT/PROTOCOL"
# Convert with a copy of the binary, so a rebuild during the run cannot swap it underneath (it has: a release
# build replaced the binary halfway through a figure review). BINARY records which build the run used.
BIN=${BIN:-$REPO/target/release/latexml_oxide}
"$REPO/fork/binary_info.sh" "$BIN" | tee "$OUTROOT/BINARY" || exit 2
cp -c "$BIN" "$OUTROOT/latexml_oxide" 2>/dev/null || cp "$BIN" "$OUTROOT/latexml_oxide" || exit 2
export BIN=$OUTROOT/latexml_oxide
grep -E "^($filt)	" "$CORPUS/corpus.tsv" | while IFS=$'\t' read -r i f; do printf '%s\0%s\0' "$i" "$f"; done \
  | xargs -0 -n2 -P"${JOBS:-6}" "$HERE/run_one.sh"
rm -f "$OUTROOT/latexml_oxide"
python3 "$HERE/index.py" "$OUTROOT"
