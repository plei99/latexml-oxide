#!/usr/bin/env python3
"""Side-by-side review page for a fixture run: reference vs oxide per feature, with the recorded
verdict from STATUS.tsv and the run's error counts.

    gallery.py <run-dir> > gallery.html
"""
import csv, html, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
run = sys.argv[1].rstrip("/")
corpus = os.path.dirname(os.path.dirname(run))
desc = dict(l.rstrip("\n").split("\t", 1) for l in open(os.path.join(corpus, "features.tsv")))
status = {}
if os.path.exists(os.path.join(HERE, "STATUS.tsv")):
    with open(os.path.join(HERE, "STATUS.tsv")) as f:
        status = {r["id"]: r for r in csv.DictReader(f, delimiter="\t")}
docs = {}
if os.path.exists(os.path.join(run, "index", "docs.tsv")):
    with open(os.path.join(run, "index", "docs.tsv")) as f:
        docs = {r["id"]: r for r in csv.DictReader(f, delimiter="\t")}
COLOR = {"ok": "#1a7f37", "wrong": "#cf222e", "partial": "#9a6700"}
out = ["<!doctype html><meta charset=utf-8><title>fixtures: %s</title>" % html.escape(os.path.basename(run)),
       "<style>body{font:14px system-ui;margin:20px;color:#222}h2{margin-top:36px;border-bottom:1px solid #ddd}"
       ".f{margin:18px 0 28px}.v{font-weight:600;padding:1px 6px;border-radius:4px;color:#fff}"
       ".m{color:#666}img{max-width:100%;border:1px solid #eee}</style>",
       "<h1>Fixture review — %s</h1>" % html.escape(os.path.basename(run))]
pkg = None
for ident in desc:
    p = ident.split("__")[0]
    if p != pkg:
        pkg = p
        out.append(f"<h2>{html.escape(p)}</h2>")
    s = status.get(ident, {})
    v = s.get("verdict", "unreviewed")
    d = docs.get(ident, {})
    errs = f"errors {d.get('errors', '?')}, status {d.get('status', '?')}" if d else "not converted"
    out.append(f'<div class=f id="{ident}"><b>{html.escape(ident)}</b> '
               f'<span class=v style="background:{COLOR.get(v, "#888")}">{v}</span> '
               f'<span class=m>{html.escape(desc[ident])} · {errs}</span>'
               + (f"<br><i>{html.escape(s.get('note', ''))}</i>" if s.get("note") else "")
               + f'<br><img loading=lazy src="{ident}/pair.png"></div>')
print("\n".join(out))
