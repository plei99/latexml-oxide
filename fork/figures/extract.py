#!/usr/bin/env python3
"""Extract every figure of the notes into its own mini-document.

    extract.py <corpus.tsv> <out-dir> [path-prefix]

For each document of <corpus.tsv> (id<TAB>abs path) under <path-prefix> (default ~/Documents/notes/), every
figure becomes `<out-dir>/src/<id>-<n>.tex`: the document's preamble, page furniture off, and the figure alone
(a `\\printpage` also gets the document's `sseqdata` blocks, which it draws from). The first line,
`% lxo-path: <document dir>`, lets ref_one.sh and run_one.sh resolve the document's own `\\input`s and `.sty`s
without writing into its directory. Writes `<out-dir>/corpus.tsv` (for the fork/corpus scripts) and
`<out-dir>/figures.tsv` (id, document id, kind, line).
"""
import os, re, sys

ENVS = ("tikzcd", "tikzpicture", "ytableau", "sseqpage", "circuitikz")
CMDS = ("ydiagram", "ytableaushort", "dynkin", "includegraphics", "printpage")


def strip_comments(text):
    """Blank `%` comments (not `\\%`), keeping line numbers and offsets."""
    return re.sub(r"(?<!\\)%[^\n]*", lambda m: " " * len(m.group(0)), text)


def env_end(text, env, start):
    """The offset just past the `\\end{env}` matching the `\\begin{env}` at `start`."""
    pat = re.compile(r"\\(begin|end)\{" + re.escape(env) + r"\}")
    depth = 0
    for m in pat.finditer(text, start):
        depth += 1 if m.group(1) == "begin" else -1
        if depth == 0:
            return m.end()
    return None


def group_end(text, i):
    """The offset just past the balanced `{…}` or `[…]` opening at `i`."""
    open_, close = text[i], "}" if text[i] == "{" else "]"
    depth = 0
    for j in range(i, len(text)):
        c = text[j]
        if c == "\\":
            continue
        if c == open_ and text[j - 1] != "\\":
            depth += 1
        elif c == close and text[j - 1] != "\\":
            depth -= 1
            if depth == 0:
                return j + 1
    return None


def cmd_end(text, start, name):
    """The end of a command's arguments: optional `[…]`s and `{…}`s, then for `\\dynkin` a bare `A4`-style spec."""
    i = start + 1 + len(name)
    while True:
        k = i
        while k < len(text) and text[k] in " \t":
            k += 1
        if k < len(text) and text[k] in "[{":
            e = group_end(text, k)
            if e is None:
                return i
            i = e
            continue
        if name == "dynkin" and i == start + 1 + len(name):
            m = re.match(r"\s*([A-Ga-g])\s*(\{[^}]*\}|\w+)", text[i:])
            if m:
                return i + m.end()
        return i


def figures(body):
    """(start, end, kind) of every top-level figure in `body`, in order."""
    found = []
    for env in ENVS:
        for m in re.finditer(r"\\begin\{" + env + r"\}", body):
            e = env_end(body, env, m.start())
            if e:
                found.append((m.start(), e, env))
    for cmd in CMDS:
        for m in re.finditer(r"\\" + cmd + r"(?![A-Za-z@])", body):
            found.append((m.start(), cmd_end(body, m.start(), cmd), cmd))
    found.sort()
    top, last = [], -1
    for s, e, k in found:
        if s >= last:  # skip what lies inside an earlier figure (a \dynkin in a tikzpicture)
            top.append((s, e, k))
            last = e
    return top


def main():
    corpus_tsv, out = sys.argv[1], os.path.abspath(sys.argv[2])
    prefix = sys.argv[3] if len(sys.argv) > 3 else os.path.expanduser("~/Documents/notes/")
    src = os.path.join(out, "src")
    os.makedirs(src, exist_ok=True)
    corpus, index = [], []
    for line in open(corpus_tsv):
        doc_id, path = line.rstrip("\n").split("\t")[:2]
        if doc_id == "id" or not path.startswith(prefix) or os.path.basename(path).startswith("_region_"):
            continue
        raw = open(path, errors="replace").read()
        if "\\begin{document}" not in raw:
            continue
        # Figures are found in the comment-free text and cut from the original, at the same offsets: a blanked
        # comment-only line would read as a blank line, a paragraph break, inside the figure.
        text = strip_comments(raw)
        cut = text.index("\\begin{document}")
        base = cut + len("\\begin{document}")
        pre, body = raw[:cut], text[base:]
        body = body.rsplit("\\end{document}", 1)[0]
        orig = raw[base:base + len(body)]
        sseqdata = "".join(orig[m.start():env_end(body, "sseqdata", m.start())] + "\n"
                           for m in re.finditer(r"\\begin\{sseqdata\}", body))
        for n, (s, e, kind) in enumerate(figures(body), 1):
            fig = orig[s:e]
            ident = f"{doc_id}-{n:03d}"
            lineno = raw.count("\n", 0, base + s) + 1
            lead = sseqdata if kind == "printpage" else ""
            new = (f"% lxo-path: {os.path.dirname(path)}\n{pre}\\begin{{document}}\\pagestyle{{empty}}"
                   f"\\thispagestyle{{empty}}\n{lead}{fig}\n\\end{{document}}\n")
            doc = os.path.join(src, ident + ".tex")
            if not os.path.exists(doc) or open(doc).read() != new:  # stable mtimes for refs.sh
                open(doc, "w").write(new)
            corpus.append(f"{ident}\t{doc}\n")
            index.append(f"{ident}\t{doc_id}\t{kind}\t{lineno}\n")
    open(os.path.join(out, "corpus.tsv"), "w").writelines(corpus)
    open(os.path.join(out, "figures.tsv"), "w").writelines(index)
    print(f"{len(corpus)} figures")


if __name__ == "__main__":
    main()
