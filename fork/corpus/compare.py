#!/usr/bin/env python3
"""Diff two indexed corpus runs and rank what changed.

    compare.py <old-run> <new-run> [--show N]

Per document, a REGRESSION is any of: status worse (3 fatal > 2 error > 1 warning > 0; 4 = no
output), more errors or a new error class, more `ltx_ERROR` marks or leaked parser tokens, fewer <math>, <svg> or TOC entries, recall
against the reference PDF down by more than 0.2 points, or visible words lost (content_diff.py's
multiset rule, on rendered text). The mirror images are IMPROVEMENTS. A picture whose SVG digest
changed with nothing else worse is a CHANGE: geometry moved and needs eyes (pictures.py renders
both sides). Exit status 1 when any regression is found.
"""
import collections, csv, importlib.util, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
spec = importlib.util.spec_from_file_location("pdf_recall", os.path.join(REPO, "tools/perfect_kernel/pdf_recall.py"))
pdf_recall = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pdf_recall)
WORD = re.compile(r"[^\W_]+", re.UNICODE)


def table(run, name, key):
    rows = collections.defaultdict(list)
    with open(os.path.join(run, "index", name + ".tsv")) as f:
        for r in csv.DictReader(f, delimiter="\t"):
            rows[r[key]].append(r)
    return rows


def words(run, ident):
    p = os.path.join(run, ident, "doc.html")
    if not os.path.exists(p):
        return collections.Counter()
    return collections.Counter(w.casefold() for w in WORD.findall(pdf_recall.xml_text(p)))


def num(x, default=0.0):
    try:
        return float(x)
    except (TypeError, ValueError):
        return default


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    show = int(sys.argv[sys.argv.index("--show") + 1]) if "--show" in sys.argv else 6
    if "--show" in sys.argv:
        args.remove(str(show))
    old, new = (a.rstrip("/") for a in args)
    od, nd = table(old, "docs", "id"), table(new, "docs", "id")
    oe, ne = table(old, "errors", "id"), table(new, "errors", "id")
    op, np_ = table(old, "pictures", "id"), table(new, "pictures", "id")
    reg, imp, chg = [], [], []
    tsv = os.path.join(os.path.dirname(os.path.dirname(os.path.realpath(new))), "corpus.tsv")
    where = dict(l.rstrip("\n").split("\t") for l in open(tsv)) if os.path.exists(tsv) else {}
    home = os.path.expanduser("~/")
    skipped = len(set(od) - set(nd))  # a subset run (run.sh id-regex) judges only what it converted
    for ident in sorted(nd):
        if ident not in od:
            chg.append((ident, ["new document"]))
            continue
        o, n = od[ident][0], nd[ident][0]
        bad, good, moved = [], [], []

        def cmp(field, label, higher_is_worse=True, tol=0.0):
            a, b = num(o[field]), num(n[field])
            if abs(b - a) <= tol:
                return
            (bad if (b > a) == higher_is_worse else good).append(f"{label} {o[field]}→{n[field]}")

        cmp("status", "status")
        cmp("errors", "errors")
        cmp("ltx_errors", "ltx_ERROR")
        cmp("leaks", "leaked tokens")
        cmp("toc", "toc entries", higher_is_worse=False)
        cmp("maths", "maths", higher_is_worse=False)
        cmp("svgs", "svgs", higher_is_worse=False)
        if o["recall"] and n["recall"]:
            cmp("recall", "recall%", higher_is_worse=False, tol=0.2)
        oc = {r["class"]: int(r["count"]) for r in oe.get(ident, [])}
        nc = {r["class"]: int(r["count"]) for r in ne.get(ident, [])}
        if set(nc) - set(oc):
            bad.append("new " + ", ".join(f"{c}×{nc[c]}" for c in sorted(set(nc) - set(oc))))
        if set(oc) - set(nc):
            good.append("gone " + ", ".join(sorted(set(oc) - set(nc))))
        ow, nw = words(old, ident), words(new, ident)
        lost, gained = ow - nw, nw - ow
        if lost:
            bad.append(f"lost {sum(lost.values())} words ({' '.join(w for w, _ in lost.most_common(show))})")
        if gained:
            moved.append(f"gained {sum(gained.values())} words ({' '.join(w for w, _ in gained.most_common(show))})")
        opics = {r["n"]: r["digest"] for r in op.get(ident, [])}
        npics = {r["n"]: r["digest"] for r in np_.get(ident, [])}
        diff = sorted((k for k in set(opics) & set(npics) if opics[k] != npics[k]), key=int)
        if diff:
            moved.append(f"{len(diff)} pictures changed (n={','.join(diff[:20])}{'…' if len(diff) > 20 else ''})")
        if bad:
            reg.append((ident, bad + good + moved))
        elif good:
            imp.append((ident, good + moved))
        elif moved:
            chg.append((ident, moved))
    print(f"# {os.path.basename(old)} → {os.path.basename(new)}: "
          f"{len(reg)} regressed, {len(imp)} improved, {len(chg)} changed, "
          f"{len(set(od) & set(nd)) - len(reg) - len(imp) - len(chg)} identical"
          + (f" ({skipped} baseline documents not in this run)" if skipped else ""))
    for title, rows in (("REGRESSIONS", reg), ("IMPROVEMENTS", imp), ("CHANGES", chg)):
        if rows:
            print(f"\n## {title}")
            for ident, notes in rows:
                src = where.get(ident, "").replace(home, "~/")
                print(f"{ident}  " + (f"[{src}]  " if src else "") + "; ".join(notes))
    sys.exit(1 if reg else 0)


if __name__ == "__main__":
    main()
