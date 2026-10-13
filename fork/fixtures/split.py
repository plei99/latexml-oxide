#!/usr/bin/env python3
"""Split each fixture into one mini-document per feature.

    split.py <out-dir>

A fixture `<pkg>.tex` is a preamble, `\\begin{document}`, then features each opened by a line
`%% feature: <slug> | <what it exercises>`. Every feature becomes `<out-dir>/src/<pkg>__<slug>.tex`
(the shared preamble + that body alone, page furniture off so the cropped reference is the content), listed in `<out-dir>/corpus.tsv` (id<TAB>path) for the
fork/corpus scripts, with descriptions in `<out-dir>/features.tsv`. `*.bib` files are copied along.
"""
import glob, os, re, shutil, sys

here = os.path.dirname(os.path.abspath(__file__))
out = os.path.abspath(sys.argv[1])
src = os.path.join(out, "src")
os.makedirs(src, exist_ok=True)
for bib in glob.glob(os.path.join(here, "*.bib")):
    shutil.copy(bib, src)
corpus, features = [], []
for path in sorted(glob.glob(os.path.join(here, "*.tex"))):
    pkg = os.path.basename(path)[:-4]
    text = open(path).read()
    pre, body = text.split("\\begin{document}", 1)
    body = body.rsplit("\\end{document}", 1)[0]
    parts = re.split(r"^%% feature: *([\w-]+) *\| *(.*)$", body, flags=re.M)
    for slug, desc, chunk in zip(parts[1::3], parts[2::3], parts[3::3]):
        ident = f"{pkg}__{slug}"
        doc = os.path.join(src, ident + ".tex")
        new = f"{pre}\\begin{{document}}\\pagestyle{{empty}}\\thispagestyle{{empty}}\n{chunk.strip()}\n\\end{{document}}\n"
        if not os.path.exists(doc) or open(doc).read() != new:  # keep mtimes stable for refs.sh
            open(doc, "w").write(new)
        corpus.append(f"{ident}\t{doc}\n")
        features.append(f"{ident}\t{desc.strip()}\n")
open(os.path.join(out, "corpus.tsv"), "w").writelines(corpus)
open(os.path.join(out, "features.tsv"), "w").writelines(features)
print(f"{len(corpus)} features from {len(glob.glob(os.path.join(here, '*.tex')))} fixtures")
