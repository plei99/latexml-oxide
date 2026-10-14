#!/usr/bin/env bash
# ref_one.sh <id> <source.tex>   (OUTROOT env = refs directory)
# Typeset the reference PDF with the document's own engine into $OUTROOT/<id>/:
#   doc.pdf, ref.txt (pdftotext), engine.tsv: id engine latex_errors pages
# Engine: a `% !TEX program = X` magic comment wins; else xelatex for xeCJK/ctex,
# lualatex for fontspec/unicode-math, else pdflatex.
set -uo pipefail
id=$1; src=$2; out=$OUTROOT/$id
export PATH="${TEXBIN:-/usr/local/texlive/2026/bin/universal-darwin}:$PATH"
export SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-1767225600} FORCE_SOURCE_DATE=1
d=$(dirname "$src"); b=$(basename "$src" .tex)
rm -rf "$out"; mkdir -p "$out"
eng=$(grep -m1 -oiE '^%\s*!\s*TEX\s+(TS-)?program\s*=\s*[a-z]+' "$src" | sed -E 's/.*=[[:space:]]*//' | tr A-Z a-z)
case $eng in pdflatex|lualatex|xelatex) ;; *) eng= ;; esac
if [ -z "$eng" ]; then
  eng=pdflatex
  # Only uncommented lines: `% \usepackage{fontspec}` must not pick lualatex.
  grep -qE '^[^%]*\\usepackage(\[[^]]*\])?\{[^}]*(fontspec|unicode-math)' "$src" && eng=lualatex
  grep -qE '^[^%]*\\(usepackage|documentclass)(\[[^]]*\])?\{[^}]*(xeCJK|ctex)' "$src" && eng=xelatex
fi
typeset() {
  case $eng in pdflatex) flag=-pdf;; lualatex) flag=-lualatex;; xelatex) flag=-xelatex;; esac
  rm -rf "$out/aux"; mkdir -p "$out/aux"
  # \include'd files write .aux into matching subdirectories of the outdir
  (cd "$d" && find . -name '*.tex' -not -path '*/.*' -exec dirname {} \; | sort -u | while read -r s; do mkdir -p "$out/aux/$s"; done)
  (cd "$d" && perl -e 'setpgrp(0,0); $SIG{ALRM}=sub{kill -9, 0}; alarm 420; system @ARGV' \
    latexmk $flag -f -interaction=nonstopmode -outdir="$out/aux" "$b.tex" > "$out/latexmk.out" 2>&1)
}
typeset
# fontspec loaded from a local .sty is invisible to the source scan above: retry under LuaLaTeX
if [ ! -f "$out/aux/$b.pdf" ] && grep -q 'requires either XeTeX or' "$out/aux/$b.log" 2>/dev/null; then
  eng=lualatex; typeset
fi
[ -f "$out/aux/$b.pdf" ] && mv "$out/aux/$b.pdf" "$out/doc.pdf"
errs=$(grep -c "^!" "$out/aux/$b.log" 2>/dev/null); errs=${errs:-NA}
pages=NA; [ -f "$out/doc.pdf" ] && { pdftotext -enc UTF-8 "$out/doc.pdf" "$out/ref.txt"; pages=$(pdfinfo "$out/doc.pdf" | awk '/^Pages:/{print $2}'); }
cp "$out/aux/$b.log" "$out/doc.texlog" 2>/dev/null
rm -rf "$out/aux"
printf '%s\t%s\t%s\t%s\n' "$id" "$eng" "$errs" "$pages" > "$out/engine.tsv"
