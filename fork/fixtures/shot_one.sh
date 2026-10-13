#!/usr/bin/env bash
# shot_one.sh <run-dir>/<id> <refs-dir>/<id> — render both sides of one feature to PNG:
#   ref.png (reference PDF, all pages, cropped), oxide.png (headless Chrome on doc.html, trimmed),
#   pair.png (side by side: reference left, oxide right).
set -uo pipefail
o=$(cd "$1" && pwd); r=$2
export PATH="${TEXBIN:-/usr/local/texlive/2026/bin/universal-darwin}:$PATH"
FONT=${FONT:-/System/Library/Fonts/Supplemental/Arial.ttf}
CHROME=${CHROME:-"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"}
cap() { perl -e 'setpgrp(0,0); $SIG{ALRM}=sub{kill -9, 0}; alarm shift; system @ARGV' "$@" >/dev/null 2>&1; }
rm -f "$o"/ref*.png "$o"/oxide*.png "$o"/pair.png "$o"/shot.html
if [ -f "$r/doc.pdf" ]; then
  cap 60 pdfcrop --margins 8 "$r/doc.pdf" "$o/ref-crop.pdf"
  cap 60 pdftocairo -png -r 120 "$o/ref-crop.pdf" "$o/ref-p"
  magick "$o"/ref-p*.png -append -background white -flatten "$o/ref.png" 2>/dev/null
  rm -f "$o"/ref-p*.png "$o/ref-crop.pdf"
fi
if [ -s "$o/doc.html" ]; then
  # hide page furniture so the shot is the content alone
  sed 's|</head>|<style>.ltx_page_footer,.ltx_page_header,.ltx_page_logo{display:none!important}body{margin:0}.ltx_page_main{padding:8px!important}</style></head>|' "$o/doc.html" > "$o/shot.html"
  cap 40 "$CHROME" --headless=new --disable-gpu --hide-scrollbars --no-first-run --no-default-browser-check \
    --disable-extensions --blink-settings=preferredColorScheme=1 --force-device-scale-factor=1.5 --window-size=900,2400 --screenshot="$o/oxide-full.png" "file://$o/shot.html"
  [ -s "$o/oxide-full.png" ] && magick "$o/oxide-full.png" -trim +repage -bordercolor white -border 12 "$o/oxide.png" 2>/dev/null
  rm -f "$o/oxide-full.png" "$o/shot.html"
fi
lab() { magick \( -font "$FONT" -pointsize 18 -fill '#555' label:"$2" \) "$1" -background white -gravity west -append "$3" 2>/dev/null; }
[ -f "$o/ref.png" ] && lab "$o/ref.png" "reference (TeX)" "$o/l.png" || magick -background white -font "$FONT" -pointsize 18 -fill "#cf222e" label:"no reference" "$o/l.png"
[ -f "$o/oxide.png" ] && lab "$o/oxide.png" "latexml-oxide" "$o/r.png" || magick -background white -font "$FONT" -pointsize 18 -fill "#cf222e" label:"no oxide output" "$o/r.png"
magick "$o/l.png" "$o/r.png" -background white -gravity north +smush 30 -bordercolor '#ccc' -border 1 "$o/pair.png" 2>/dev/null
rm -f "$o/l.png" "$o/r.png"
