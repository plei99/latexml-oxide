#!/usr/bin/env bash
# check.sh [label] [id-regex] — convert the corpus with the current binary and compare against the
# accepted baseline. Label defaults to check-<short HEAD>[-dirty]. Exit 1 on any regression.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; REPO="$(cd "$HERE/../.." && pwd)"
CORPUS=$(cd "${CORPUS:-$REPO/.corpus}" && pwd)
dirty=$(git -C "$REPO" diff --quiet HEAD -- . ':!fork' || echo -dirty)
label=${1:-check-$(git -C "$REPO" rev-parse --short=8 HEAD)$dirty}
"$HERE/run.sh" "$label" "${2:-.*}"
python3 "$HERE/compare.py" "$CORPUS/accepted" "$CORPUS/runs/$label" | tee "$CORPUS/runs/$label/compare.md"
exit "${PIPESTATUS[0]}"
