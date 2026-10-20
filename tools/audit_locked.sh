#!/usr/bin/env bash
# Audit `locked => 1` attribute parity between Perl and Rust ports.
#
# For each Perl package/contrib .ltxml file with at least one active
# (non-commented) `locked => 1`, compare the count to the paired Rust
# .rs file, counting both macro-argument syntax (`locked => true`) and
# struct-literal syntax (`locked: true`), and skipping `//` comments.
#
# Prints DRIFT lines where the counts diverge. A drift is not always
# a bug — sometimes Perl has a benign double-binding (e.g. nicematrix
# declares `\endNiceTabular` twice, once for the unstarred env and once
# for the `*` env) that Rust collapses to one, or Rust is more paranoid
# (more locked marks than Perl). Investigate each drift to classify as
# (a) genuine missing `locked` in Rust, (b) benign Perl redundancy,
# or (c) Rust defensive-locking not mirrored in Perl.
set -euo pipefail

cd "$(dirname "$0")/.."
# LaTeXML/ (the Perl oracle) is gitignored and absent from a fresh checkout or worktree; without it every count
# below is zero and the sweep would report no drift.
if [ ! -d LaTeXML/lib/LaTeXML/Package ]; then
  echo "$(basename "$0"): no Perl source at $PWD/LaTeXML/lib/LaTeXML/Package; link LaTeXML/ to a brucemiller/latexml checkout" >&2
  exit 2
fi

for pf in $(grep -rl 'locked *=> *1' LaTeXML/lib/LaTeXML/Package/ ar5iv-bindings/bindings/ 2>/dev/null); do
  name=$(basename "$pf")
  stem="${name%.ltxml}"
  rstem=$(echo "$stem" | sed -E 's/\.(sty|cls|tex|def|ltx)$/_\1/' \
    | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9_' '_' | sed 's/_*$//')
  rfile=""
  for root in latexml_package/src/package latexml_package/src/engine latexml_contrib/src; do
    c="$root/${rstem}.rs"
    [ -f "$c" ] && rfile="$c" && break
  done
  [ -z "$rfile" ] && continue
  plocked=$(grep -E 'locked *=> *1' "$pf" | grep -vcE '^\s*#')
  rlocked=$(grep -E 'locked *(=>|:) *true' "$rfile" | grep -vcE '^\s*//')
  if [ "$plocked" != "$rlocked" ]; then
    printf 'DRIFT\t%s\tperl=%s\trust=%s\t%s\n' "$rstem" "$plocked" "$rlocked" "$rfile"
  fi
done
