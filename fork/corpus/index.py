#!/usr/bin/env python3
"""Index one corpus run: the per-document facts compare.py diffs between runs.

    index.py <run-dir>            (run-dir/<id>/{doc.html,doc.log,verdict.tsv})

Writes <run-dir>/index/:
  docs.tsv      id status errors warnings fatals secs ltx_errors leaks toc maths svgs src_pics recall missing extra
  errors.tsv    id class count            (class = `Error:<category>:<object>` / `Fatal:...`)
  pictures.tsv  id n width height digest  (one row per top-level <svg>, in document order)

`recall` is upstream's pdf_recall measure (words of 4+ letters, as the s3 corpus audit: shorter runs are
mostly math that pdftotext glues together) against the reference PDF in $CORPUS/refs/<id>/doc.pdf
(refs.sh): the share of distinct PDF words that reach the HTML. `extra` counts distinct HTML words
absent from the PDF (leaked macro names, duplicated text). `src_pics` counts picture environments in
the source (tikzcd, tikzpicture, circuitikz, forest, axis, sseq, ytableau, \\tikz) for comparison with `svgs`.
`leaks` counts math-parser token names rendered as text (`<mo>CLOSE</mo>`, `OPEN`); `toc` counts table-of-
contents entries.
"""
import collections, hashlib, html, importlib.util, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
spec = importlib.util.spec_from_file_location("pdf_recall", os.path.join(REPO, "tools/perfect_kernel/pdf_recall.py"))
pdf_recall = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pdf_recall)

PIC_ENVS = ("tikzcd", "tikzpicture", "circuitikz", "forest", "axis", "sseqpage", "sseqdata", "ytableau")
INPUT = re.compile(r"\\(?:input|include|subfile)\{([^}]+)\}")


def source_text(path, seen):
    if path in seen or not os.path.exists(path):
        return ""
    seen.add(path)
    t = re.sub(r"(?<!\\)%.*", "", open(path, errors="replace").read())
    d = os.path.dirname(path)
    for m in INPUT.findall(t):
        t += source_text(os.path.join(d, m if m.endswith(".tex") else m + ".tex"), seen)
    return t


def source_pictures(src):
    body = source_text(src, set()).split("\\begin{document}", 1)[-1]
    n = sum(len(re.findall(r"\\begin\{%s\}" % e, body)) for e in PIC_ENVS)
    return n + len(re.findall(r"\\tikz\b(?!set|style|picture|cd)", body))


def top_level_svgs(doc):
    """Yield each outermost <svg>…</svg> (an SVG may nest <svg> inside foreignObject)."""
    depth, start = 0, None
    for m in re.finditer(r"<(/?)svg\b[^>]*>", doc):
        if m.group(1):
            depth -= 1
            if depth == 0 and start is not None:
                yield doc[start:m.end()]
                start = None
        elif not m.group(0).endswith("/>"):
            if depth == 0:
                start = m.start()
            depth += 1


def svg_digest(svg):
    # ids/hrefs are document-global counters: a picture earlier in the document shifts them
    s = re.sub(r'\s(?:id|xml:id)="[^"]*"', "", svg)
    s = re.sub(r'url\(#[^)]*\)|"#[^"]*"', "#", s)
    s = re.sub(r"\s+", " ", s)
    return hashlib.sha1(s.encode()).hexdigest()[:12]


def attr(tag, name):
    m = re.search(r'\b%s="([^"]*)"' % name, tag)
    return m.group(1) if m else ""


def index_doc(run, ident, src, refs):
    d = os.path.join(run, ident)
    v = open(os.path.join(d, "verdict.tsv")).read().split("\t")
    status, errors, warnings, fatals, secs = v[1], v[3], v[4], v[5], v[6].strip()
    log = open(os.path.join(d, "doc.log"), errors="replace").read() if os.path.exists(os.path.join(d, "doc.log")) else ""
    classes = collections.Counter(m.group(1) for m in re.finditer(r"^((?:Error|Fatal):\S+)", log, re.M))
    hp = os.path.join(d, "doc.html")
    doc = open(hp, errors="replace").read() if os.path.exists(hp) else ""
    pics = []
    for n, svg in enumerate(top_level_svgs(doc)):
        head = svg[: svg.index(">") + 1]
        pics.append((n, attr(head, "width"), attr(head, "height"), svg_digest(svg)))
    recall = missing = extra = ""
    pdf = os.path.join(refs, ident, "doc.pdf")
    if doc and os.path.exists(pdf):
        xw = pdf_recall.normalize(pdf_recall.xml_text(hp), 4, compounds=True)
        pw = pdf_recall.normalize(pdf_recall.pdf_text(pdf), 4, rejoin=True)
        miss = pw - xw
        recall = "%.1f" % (100.0 * (len(pw) - len(miss)) / len(pw)) if pw else "100.0"
        missing, extra = len(miss), len(xw - pw)
    leaks = len(re.findall(r">(?:OPEN|CLOSE)</m", doc))
    row = (ident, status, errors, warnings, fatals, secs, doc.count('class="ltx_ERROR'), leaks,
           doc.count("ltx_tocentry"), doc.count("<math"),
           len(pics), source_pictures(src), recall, missing, extra)
    return row, classes, pics


def main():
    run = sys.argv[1].rstrip("/")
    corpus = os.environ.get("CORPUS", os.path.join(REPO, ".corpus"))
    refs = os.path.join(corpus, "refs")
    sources = dict(l.rstrip("\n").split("\t") for l in open(os.path.join(corpus, "corpus.tsv")))
    out = os.path.join(run, "index")
    os.makedirs(out, exist_ok=True)
    docs, errs, pics = [], [], []
    for ident in sorted(os.listdir(run)):
        if ident in sources and os.path.exists(os.path.join(run, ident, "verdict.tsv")):
            row, classes, p = index_doc(run, ident, sources[ident], refs)
            docs.append(row)
            errs += [(ident, c, n) for c, n in sorted(classes.items())]
            pics += [(ident,) + x for x in p]
    hdr = {"docs": "id\tstatus\terrors\twarnings\tfatals\tsecs\tltx_errors\tleaks\ttoc\tmaths\tsvgs\tsrc_pics\trecall\tmissing\textra",
           "errors": "id\tclass\tcount", "pictures": "id\tn\twidth\theight\tdigest"}
    for name, rows in (("docs", docs), ("errors", errs), ("pictures", pics)):
        with open(os.path.join(out, name + ".tsv"), "w") as f:
            f.write(hdr[name] + "\n")
            f.writelines("\t".join(map(str, r)) + "\n" for r in rows)
    st = collections.Counter(r[1] for r in docs)
    print(f"indexed {len(docs)} docs: status {dict(sorted(st.items()))}; {len(pics)} svgs; "
          f"{sum(int(r[2]) for r in docs)} errors")


if __name__ == "__main__":
    main()
