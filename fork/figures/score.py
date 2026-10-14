#!/usr/bin/env python3
"""Score and lay out a figure run: reference (pdflatex) vs oxide per figure, worst first.

    score.py <figs-dir> <run-dir>

Reads <figs-dir>/figures.tsv (id, document id, kind, line) and, per figure, <run-dir>/<id>/{ref,oxide}.png (from
fork/fixtures/shot_one.sh) and the run's index. Writes <run-dir>/scores.tsv (id, doc, kind, line, errors, verdict,
phash, leaked, missing) and <run-dir>/gallery.html. `phash` is ImageMagick's perceptual-hash distance between the two renderings
(0 = identical; it tolerates scale, not a missing arrow); `verdict` is `no-ref`, `no-output`, `errors`, `text` (words
of 3+ letters in one rendering and not the other: `leaked`, `missing`), or `ok`. Sorted: no-output, errors, text, then
by distance, largest first.
"""
import csv, html, os, re, subprocess, sys
from html.parser import HTMLParser

figs, run = sys.argv[1].rstrip("/"), sys.argv[2].rstrip("/")
rows = [l.rstrip("\n").split("\t") for l in open(os.path.join(figs, "figures.tsv"))]
docs = {}
idx = os.path.join(run, "index", "docs.tsv")
if os.path.exists(idx):
    with open(idx) as f:
        docs = {r["id"]: r for r in csv.DictReader(f, delimiter="\t")}


SKIP = {"ltx_title_document", "ltx_authors", "ltx_dates", "ltx_abstract", "ltx_keywords", "ltx_classification",
        "ltx_page_footer", "ltx_page_header", "ltx_page_logo"}
WORD = re.compile(r"[A-Za-z]{3,}")


class FigureText(HTMLParser):
    """The text of a page's body, without the front matter a preamble's \\title renders without \\maketitle, the page
    header and footer, and math's `alttext`/`<annotation>` (TeX source, not what is shown)."""

    def __init__(self):
        super().__init__()
        self.stack, self.skip, self.parts = [], 0, []

    def handle_starttag(self, tag, attrs):
        if tag in ("meta", "link", "img", "br", "hr", "input", "mglyph", "mspace", "none", "mprescripts"):
            return
        classes = set((dict(attrs).get("class") or "").split())
        hide = bool(classes & SKIP) or tag in ("footer", "header", "annotation", "annotation-xml", "head", "style",
                                                 "script")
        self.stack.append(hide)
        self.skip += hide

    def handle_endtag(self, tag):
        if self.stack:
            self.skip -= self.stack.pop()

    def handle_data(self, data):
        if not self.skip:
            self.parts.append(data)


def words(text):
    return {w.lower() for w in WORD.findall(text)}


def text_delta(ref_txt, ox_html):
    """Words (3+ letters) in oxide's figure but not in the reference's text, and the reverse."""
    if not (os.path.exists(ref_txt) and os.path.exists(ox_html)):
        return set(), set()
    parser = FigureText()
    parser.feed(open(ox_html, errors="replace").read())
    ox = words(" ".join(parser.parts))
    ref = words(open(ref_txt, errors="replace").read())
    return ox - ref, ref - ox


def phash(a, b):
    p = subprocess.run(["magick", "compare", "-metric", "PHASH", a, b, "null:"], capture_output=True, text=True)
    try:
        return float(p.stderr.split()[0])
    except (IndexError, ValueError):
        return None


out = []
for ident, doc, kind, line in rows:
    d = os.path.join(run, ident)
    ref, ox = os.path.join(d, "ref.png"), os.path.join(d, "oxide.png")
    errors = int(docs.get(ident, {}).get("errors") or 0)
    if not os.path.exists(ref):
        verdict, dist = "no-ref", None
    elif not os.path.exists(ox):
        verdict, dist = "no-output", None
    else:
        dist = phash(ref, ox)
        verdict = "errors" if errors else "ok"
    leaked, missing = text_delta(os.path.join(figs, "refs", ident, "ref.txt"), os.path.join(d, "doc.html"))
    if verdict == "ok" and (leaked or missing):
        verdict = "text"
    out.append((ident, doc, kind, line, errors, verdict, dist, " ".join(sorted(leaked)), " ".join(sorted(missing))))

rank = {"no-output": 0, "errors": 1, "text": 2, "ok": 3, "no-ref": 4}
out.sort(key=lambda r: (rank[r[5]], -(r[6] if r[6] is not None else 0)))
with open(os.path.join(run, "scores.tsv"), "w") as f:
    f.write("id\tdoc\tkind\tline\terrors\tverdict\tphash\tleaked\tmissing\n")
    for r in out:
        f.write("\t".join("" if v is None else str(v) for v in r) + "\n")

COLOR = {"no-output": "#cf222e", "errors": "#9a6700", "text": "#bc4c00", "ok": "#1a7f37", "no-ref": "#888"}
page = ["<!doctype html><meta charset=utf-8><title>figures: %s</title>" % html.escape(os.path.basename(run)),
        "<style>body{font:14px system-ui;margin:20px;color:#222}.f{margin:18px 0 28px}"
        ".v{font-weight:600;padding:1px 6px;border-radius:4px;color:#fff}.m{color:#666}"
        "img{max-width:100%;border:1px solid #eee}</style>",
        f"<h1>Figure review — {html.escape(os.path.basename(run))}</h1>",
        "<p>Reference (pdflatex) left, latexml-oxide right; missing output and errors first, then by perceptual "
        "distance, largest first.</p>"]
for ident, doc, kind, line, errors, verdict, dist, leaked, missing in out:
    meta = f"doc {doc}, line {line}, {kind}" + (f", {errors} errors" if errors else "") + \
           (f", distance {dist:g}" if dist is not None else "") + \
           (f"; leaked: {leaked}" if leaked else "") + (f"; missing: {missing}" if missing else "")
    page.append(f'<div class=f id="{ident}"><b>{ident}</b> '
                f'<span class=v style="background:{COLOR[verdict]}">{verdict}</span> '
                f'<span class=m>{html.escape(meta)}</span><br><img loading=lazy src="{ident}/pair.png"></div>')
open(os.path.join(run, "gallery.html"), "w").write("\n".join(page))
counts = {}
for r in out:
    counts[r[5]] = counts.get(r[5], 0) + 1
print(counts)
