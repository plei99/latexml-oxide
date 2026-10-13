# Fork tooling (plei99/latexml-oxide)

This fork tracks `upstream/perfect_kernel` and adds what is needed to use oxide on one author's
notes and papers. Everything fork-specific lives under `fork/`, so merges from upstream never
touch it. Personal sources and outputs never enter git: they live in `.corpus/`, which is listed
in `.git/info/exclude`.

Every conversion here uses upstream's perfect-kernel protocol: raw loading of `.sty`/`.cls`
via `--preload=[rawstyles,rawclasses]latexml.sty`, a fixed `SOURCE_DATE_EPOCH`, and the release
binary. Build the binary with `cargo build --release`. Rebuild the dumps after a TeX Live
upgrade or an upstream merge with `PROFILE=release tools/make_formats.sh`.

## Two indexes

| | `fork/fixtures/` (public) | `fork/corpus/` over `.corpus/` (private) |
|---|---|---|
| Content | Synthetic feature snippets for each package in use, modelled on how the corpus uses them | The author's actual documents (`.corpus/corpus.tsv`: id, path) |
| Answers | What correct output looks like for each feature, and whether oxide produces it | Did a change break any real document? |
| Reference | Each feature's own pdfLaTeX PDF | Each document's PDF, typeset with its own engine (`refs.sh`) |
| Verdict | `fixtures/STATUS.tsv`: ok / partial / wrong, reviewed side by side | `compare.py` against the accepted baseline |

## Fixture loop

`fork/fixtures/<pkg>.tex` holds one preamble and a series of features. Each feature starts with
`%% feature: <slug> | <description>`. `review.sh` does the following:

1. splits each fixture into one mini-document per feature;
2. typesets references for the features that changed;
3. converts every feature and screenshots both sides;
4. writes `.corpus/fixtures/runs/<label>/gallery.html`, with the reference on the left, oxide on
   the right, and the recorded verdict;
5. compares the run against the accepted fixture run.

```sh
fork/fixtures/review.sh                    # label fx-<HEAD>
CORPUS=.corpus/fixtures fork/corpus/accept.sh fx-<HEAD>
```

When a fix lands, flip the feature's row in `STATUS.tsv` and accept the new run. A feature
listed as `ok` that changes picture digests or loses words is a regression.

## Corpus loop

```sh
fork/corpus/refs.sh                        # reference PDFs; rerun after editing documents or upgrading TeX Live
fork/corpus/run.sh base-<sha>              # convert everything; JOBS=6 by default, about 1 h
fork/corpus/accept.sh base-<sha>           # make it the baseline, recorded in .corpus git history
fork/corpus/check.sh                       # after a change: convert again and compare; exits 1 on regression
fork/corpus/check.sh '' '0(0[1-9]|1.)'     # the same over a subset (id regex)
```

`compare.py` treats any of the following as a regression:

- a worse status;
- more errors or a new error class;
- more `ltx_ERROR` marks;
- fewer `<math>` or `<svg>` elements;
- recall against the PDF down by more than 0.2 points;
- lost visible words.

A changed SVG digest on its own is reported as a change. Look at it before accepting.

Per-document facts are written to `runs/<label>/index/{docs,errors,pictures}.tsv`. Their
meanings are documented in `corpus/index.py`.
