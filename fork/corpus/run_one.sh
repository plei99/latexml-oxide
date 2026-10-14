#!/usr/bin/env bash
# run_one.sh <id> <source.tex>   (OUTROOT env = run directory)
# Convert one corpus document under the perfect-kernel raw protocol into $OUTROOT/<id>/:
#   doc.html     HTML5 output (deterministic for a fixed binary + SOURCE_DATE_EPOCH)
#   doc.log      ANSI-free conversion log (carries `Status:conversion:N`)
#   verdict.tsv  id status exit errors warnings fatals seconds
# status: cortex codes (3 fatal, 2 error, 1 warning, 0 clean); 4 = no output (timeout/crash).
set -uo pipefail
id=$1; src=$2; out=$OUTROOT/$id
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
BIN=${BIN:-$REPO/target/release/latexml_oxide}
TIMEOUT_S=${TIMEOUT_S:-300}
export PATH="${TEXBIN:-/usr/local/texlive/2026/bin/universal-darwin}:$PATH"
export SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-1767225600}
d=$(dirname "$src")
# A figure mini-document (fork/figures/extract.py) names its document's directory on its first line, so the
# document's own \input files and packages resolve; it is searched after the snippet's own.
extra=$(sed -n '1s/^% lxo-path: //p' "$src")
# The engine persona, chosen as ref_one.sh chooses the reference engine: a `% !TEX program` magic comment, else
# uncommented xeCJK/ctex → xelatex, fontspec/unicode-math → lualatex. oxide emulates pdfTeX unless told otherwise.
eng=$(grep -m1 -oiE '^%\s*!\s*TEX\s+(TS-)?program\s*=\s*[a-z]+' "$src" | sed -E 's/.*=[[:space:]]*//' | tr A-Z a-z)
if [ -z "$eng" ]; then
  grep -qE '^[^%]*\\usepackage(\[[^]]*\])?\{[^}]*(fontspec|unicode-math)' "$src" && eng=lualatex
  grep -qE '^[^%]*\\(usepackage|documentclass)(\[[^]]*\])?\{[^}]*(xeCJK|ctex)' "$src" && eng=xelatex
fi
case $eng in xelatex) persona=xetex, ;; lualatex) persona=luatex, ;; *) persona= ;; esac
rm -rf "$out"; mkdir -p "$out"; cd "$out" || exit 1
t0=$(perl -MTime::HiRes=time -e 'print time')
perl -e 'setpgrp(0,0); $SIG{ALRM}=sub{kill -9, 0}; alarm shift; system @ARGV; exit($? & 127 ? 128+($? & 127) : $?>>8)' \
  $((TIMEOUT_S + 30)) "$BIN" --preload="[${persona}rawstyles,rawclasses]latexml.sty" --timeout="$TIMEOUT_S" \
  --path="$d" ${extra:+--path="$extra"} --sourcedirectory="${extra:-$d}" --log="$out/doc.log" --dest="$out/doc.html" "$src" > stdout.txt 2>&1
rc=$?
t1=$(perl -MTime::HiRes=time -e 'print time')
log=doc.log; [ -s "$log" ] || { sed 's/\x1b\[[0-9;]*m//g' stdout.txt > doc.log; }
e=$(grep -c '^Error:' $log); w=$(grep -c '^Warning:' $log); f=$(grep -c '^Fatal:' $log)
st=$(grep -o '^Status:conversion:[0-9]' $log | tail -1 | cut -d: -f3)
[ -s doc.html ] || st=4
[ -n "$st" ] || st=4
printf '%s\t%s\t%s\t%s\t%s\t%s\t%.1f\n' "$id" "$st" "$rc" "$e" "$w" "$f" "$(perl -e "print $t1-$t0")" > verdict.tsv
rm -f stdout.txt
