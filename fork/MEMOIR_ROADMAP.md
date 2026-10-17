# memoir roadmap (fork)

A living worklist for full memoir support. memoir is the fork's first area that upstream does not drive: Perl
LaTeXML has no `memoir.cls.ltxml`, and arXiv papers rarely use the class, while this fork's author writes in it.
Take the top unblocked row of the current milestone; when a row lands, mark it done here and flip its
`fork/fixtures/STATUS.tsv` rows.

## What "full support" means

A memoir document is supported when its HTML:

1. **loses no content.** Every word in the pdfLaTeX PDF that is not page furniture (running heads, folios, trim
   marks) is in the HTML.
2. **has the right structure and numbers.** Headings, their numbers, ToC/LoF/LoT entries, captions, notes and
   cross-references agree with the PDF.
3. **uses semantic markup** for memoir's constructs: notes are `ltx:note`, poems keep their lines, epigraphs are
   quotations, extra footnote series stay notes.
4. **reports only what pdfLaTeX reports.** memoir's own class warnings (titlesec/fancyhdr "not recommended",
   deprecated `\fref`, `\rm`) are parity; anything else is a defect.
5. **carries presentation where HTML can** (chapter styles, ToC fonts, caption and verse layout). This tier is
   beyond Perl and fork-only. It comes after 1–4.

Out of scope: page geometry, running heads and footers (`\makepagestyle`, `\makeoddhead`), trims, page breaking and
`\pref`'s page numbers. These commands must stay silent no-ops, which they already are.

## Where it stands

Measured 2026-10-17, binary `a20379aa1a1a` (checkout `2997f6c60`), with `fork/oxide`.

- **Binding.** `latexml_contrib/src/memoir_cls.rs` loads the real `memoir.cls` raw (12315 lines, TL 2026) and
  overrides four things on top: the three standard lists go to LaTeXML's `\lx@tableofcontents` family, the
  auto-tables (`\autorows`, `\autocols`, `{ctabular}`) reduce to `\tabular`, `\title`/`\author` define
  `\thetitle`/`\theauthor`, and `\ch@ngetext` keeps one column. The file is 11 lines ahead of upstream (d3e175a1e)
  and upstream still edits it (batches 46, 54, 62za).
- **Private corpus.** 32 of 264 documents are memoir. All convert with status 1, 0 errors, mean recall 97.1 (range
  93.1–99.7). They share one template, so they use few memoir commands: `\chapterstyle`, `\settocdepth`,
  `\setsecnumdepth` and the `\cftchapter*` ToC commands (31 documents each), the layout setters and `titlingpage`
  (29), `abstract` (2), `\frontmatter`/`\mainmatter` (1). Most also load titlesec (31), fancyhdr (23), parskip (29),
  enumitem, booktabs, titling and mdframed (32). Every memoir document carries one spurious
  `Warning:misdefined:\@iffirstamp` (from `memoir.cls:5454` `\newif \@iffirstamp`).
- **Public manuals** (TeX Live's `doc/latex/memoir` and neighbours; 61 memoir-class `.tex` files exist there):

  | Document | Errors | Warnings |
  |---|---:|---:|
  | memman (517 pages in pdfLaTeX's first pass) | 7 | 28 |
  | memexsupp | 0 | 1 |
  | MemoirChapStyles | 0 | 2 |
  | titlepages | 0 | 3 |

  memman's 7 errors are five `undefined:\Margi`…`\Margv` (memman.tex:31300; defined only inside memoir's
  `\memsecinfo` hook), one `malformed:ltx:text` and one `malformed:ltx:_CaptureBlock_`. Against the full
  pdfLaTeX build (625 pages: three passes, bibtex and the three makeindex runs of its `Makeidxglo`), its text
  comes through. Upstream's `pdf_recall` gives 97.9 over 7,647 distinct words; 115 of the 163 misses are the PDF
  splitting a styled first letter off a word, and most of the other 48 are text inside embedded PDF figures,
  hyphen splits and roman folios, so the recall that counts is about 99.4. The counts agree: 118/118
  bibliography entries, ~3,880 index entries for the `.ind`'s 3,862 items, the command summary (glossary),
  sections 171/173, tables 53/55, all 31 typeset examples. The defects are structural and visual. The 7
  appendices are headed "Chapter A"…; the chapter-style showcase (Appendix B) becomes 26 extra chapters, which
  look identical and fill the ToC; the "List of typeset examples" (`\newlistof`) is missing; and the five
  `\Margi` errors print as `ltx_ERROR` text.
- **Feature probe.** Of about 60 features probed across nine groups, roughly half are ok. The rest fail in one of
  four ways, which the milestones below follow: content is lost, numbers or structure are wrong, semantics are
  flattened, or presentation is ignored.

## The central design question

LaTeXML replaces memoir's sectioning, caption and ToC machinery with its own constructors, because those build
the document structure. memoir's *hooks* run inside the replaced code, so they are silently skipped:

- `\memsecinfo` is called from memoir's `\M@sect` (`memoir.cls:3757-3761`), which never runs. That is memman's
  five `\Margi` errors.
- `\printchaptername`, `\printchapternum`, `\printchaptertitle` and `\chapterstyle` are called from
  `\@makechapterhead` (`memoir.cls:3092`), so a redefined hook's literal text disappears.
- `\chapter`'s second optional argument (`\@m@mchapter`, `memoir.cls:3001-3004`) is not read by the native
  `\chapter` (`latexml_engine/src/latex_constructs/sect04.rs:179`), so the heading becomes `[` and the rest
  leaks into the body.
- `\settocdepth` writes `\changetocdepth` into the ToC (`memoir.cls:7631-7647`, `:7739`) through
  `\addtocontents`, which `sect04.rs:511` turns into a no-op.
- `\addcontentsline`'s title is discarded (`sect04.rs:525`), so `\cftaddtitleline`, custom entries and
  `\newlistof` lists come out empty.
- sidecaption steps its counter (`memoir.cls:6575`) and the native caption steps it again
  (`sect09.rs:134`): the figure numbers drift.
- memoir allocates its index streams under its own names (`memoir.cls:7882`, `:8168`), but the index fallback
  only looks for the kernel's (`latex_constructs_rust_only.rs:1339`).

Each defect needs one of two fixes. Either the native constructor calls memoir's hook at the point memoir would,
or the binding reroutes the hook's information into the native construct (as `memoir_cls.rs` already does for
`\thetitle`). Settle this per mechanism: the first fix generalizes to other classes that hook their sectioning
(KOMA, `scrbook`); the second stays local. Default to the first when the hook is a documented class interface.

## Decisions

- **D1. Raw class plus targeted overrides.** Keep loading `memoir.cls` raw. A hand-written binding cannot keep up
  with 12k lines, and the raw load is what makes the manuals error-free. Override only where a native construct
  must own the structure.
- **D2. Emulated packages.** memoir marks 27 packages loaded (`memoir.cls:12211-12253`, through
  `\EmulatedPackage`, `:135-152`), but the oxide loader ignores `\ver@<pkg>.sty` (`docs/SYNC_STATUS.md` K13), so a
  later `\usepackage{X}` loads oxide's X binding over memoir's definitions. A probe of each package under memoir
  sorted them as follows:

  | Effect of the explicit load | Packages | Policy |
  |---|---|---|
  | binding is better | changepage, chngpage, enumerate, makeidx, framed, titling, setspace | keep loading the binding |
  | memoir's raw is better | appendix (hides `\appendixpage`), ccaption (renumbers `\contcaption`, numbers legends), titleref (number instead of title), parskip | skip the binding under memoir |
  | breaks | epigraph (`\epigraphhead[70]` prints `70`), tocloft (literal `*` after starred lists), abstract (`undefined:\if@titlepage`) | skip the binding under memoir |
  | harmless | the rest | either |

  Skip a binding per package from `memoir_cls.rs`, not by honouring `\ver@` globally: the global change broke
  abntex2's `\listoffigures*` (2605.08402) because the bindings carried semantics the emulation lacked. Separately,
  where memoir emulates a package whose binding is better and the document does *not* load it, memoir's own
  version must work. Today `\begin{enumerate}[(a)]` prints `[(a)]` and numbers 1., 2.
- **D3. Upstream or fork.** Kernel defects that memoir only exposes (`\addtocontents`, `\addcontentsline` titles,
  the caption double step, index stream names, a second optional chapter argument) are general and go upstream,
  one fix per PR, with the user's go-ahead each time (`fork/AGENTS.md`). Presentation (milestone 5) is fork-only: it
  goes in a separate fork file (for example `latexml_contrib/src/memoir_fork.rs` plus a fork CSS resource) reached
  by one call from `memoir_cls.rs`, so upstream merges conflict on a single line. Its divergences are recorded under
  `fork/`, not in upstream's `docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md`.
- **D4. Font encoding.** memoir defaults to T1 (`\memfontenc`, `memoir.cls:658-679`).
  `OXIDIZED_DESIGN_DIVERGENCES.md` still defers memoir's T1 because `memoir_cls.rs` was "a minimal stub"; it no
  longer is. Decide it as part of the verse `\\>` row: the `>` typesets as `¿`, which is the OT1 slot.

## Milestones

### M0. Measurement

- [ ] Turn the probe documents (`/tmp/memoir_probe/0[1-9]_*.tex`; `/tmp` does not last) into
      `fork/fixtures/memoir_*.tex` with `%% feature:` markers, one fixture per group, and a `STATUS.tsv` row per
      feature. The existing `memoir__chapters` note is stale: "Contents" is a heading now (an `<h6>`).
- [ ] A public benchmark script under `fork/` that converts memman, memexsupp, MemoirChapStyles and titlepages and
      compares their errors, warnings and word recall against pdfLaTeX. memman needs `trims-example.pdf` built
      first (memman.tex:190).
- [ ] For the 32 corpus documents, split the missing words between memoir constructs and everything else, so
      the corpus measures memoir progress and not math or TikZ.
- [ ] Remove the spurious `\@iffirstamp` warning: `\newif` names starting `\@if` are legal (latex.ltx's `\newif`
      strips two characters from any name).

### M1. What the corpus uses

- [ ] `\settocdepth` in the preamble and mid-document. Root-cause first: memoir's default `tocdepth` is 1
      (`memoir.cls:12126`), yet `memoir__chapters` loses section 1.1 under `\settocdepth{section}`. Mid-document
      changes need per-entry depth in the ToC, which LaTeXML's post-processed ToC does not have yet.
- [ ] memoir with titlesec, and memoir with fancyhdr: confirm titlesec's `\titleformat` still yields the right
      headings and numbers after memoir's sectioning, and that fancyhdr stays silent. 31 and 23 documents depend
      on this.
- [ ] `abstract`: `\abstractintoc` (no ToC entry today), `\abstractnum` (unnumbered, and the next chapter takes 1),
      `\abstractrunin` (a heading, not run-in).

### M2. No content loss

- [ ] `\chapter[toc][head]{title}`: read memoir's second optional argument (the running-head title) in the
      native `\chapter`.
- [ ] `\addcontentsline` titles reach the ToC: custom entries, `\cftaddtitleline`, `\newlistof` with
      `\newlistentry`.
- [ ] Chapter-head hooks: redefined `\printchaptername`/`\printchapternum`/`\printchaptertitle` text, and
      `\memsecinfo` (memman's `\Margi`…`\Margv`).
- [ ] `\pretitle`/`\posttitle` text, `\prebibhook` text, `\epigraphhead` (a quotation set in the chapter head).
- [ ] memoir's indexes: `\makeindex[x]`, `\index[x]`, `\printindex[x]`, `\specialindex`, and
      `\makeglossary`/`\glossary`/`\printglossary`. All are empty without a prepared `.ind`/`.gls` today.
- [ ] Verse line numbers (`\linenumberfrequency`), and the `¿` from `\\>` (see D4).
- [ ] `\book` reaches the ToC and the document structure. Today its title is two paragraphs.
- [ ] memman's `malformed:ltx:text` and `malformed:ltx:_CaptureBlock_`.

### M3. Numbers and structure

- [ ] sidecaption's double counter step.
- [ ] Appendices: "Appendix A", not "Chapter A", and `\appendixpage`'s ToC entry.
- [ ] The `article` class option: no "Chapter" name.
- [ ] `\begin{enumerate}[(a)]` with memoir's own emulation (D2).
- [ ] D2's policy: skip the appendix, ccaption, titleref, parskip, epigraph, tocloft and abstract bindings under
      memoir, with a guard for each package.
- [ ] LoF/LoT entries: the number runs into the title.
- [ ] `\contcaption` and `\bionenumcaption` as captions, not paragraphs; `\chapterprecis` into the ToC.

### M4. Semantics

- [ ] Extra footnote series (`\newfootnoteseries`, `\footnoteA`), `\sidefootnote`, `\sidepar` and `\sidebar` as
      `ltx:note` with a role.
- [ ] `\printpagenotes` grouped under its chapter headings, as in the PDF.
- [ ] `\PoemTitle` as a numbered heading with a ToC entry.
- [ ] `{shaded}` and `{leftbar}` as framed blocks; `\plainbreak`/`\fancybreak`/`\pfbreak` as one consistent
      anonymous-division marker.

### M5. Presentation (fork-only)

Each row needs the `surpass-perl` evaluation and a fork divergence record (D3).

- [ ] `\chapterstyle`: a class on the chapter's heading per named style, with CSS for the styles in memman and
      MemoirChapStyles. Start with the style the corpus uses.
- [ ] `\setsecheadstyle` and its siblings; `\cftchapterfont` and its siblings in the ToC.
- [ ] Caption styles (`\captionstyle`, `\hangcaption`), `\hangpara`, `altverse`/`patverse` indentation,
      `\tightlists`/`\firmlists` spacing, `\footmarkstyle`, `dcolumn` alignment, `tabularx` width.

### M6. Hardening

- [ ] memman at zero errors, with recall measured against pdfLaTeX.
- [ ] All 61 memoir-class documents in TeX Live's doc tree, each with errors tallied and every error classified
      as parity or defect.
- [ ] Every memoir fixture row at `ok`, or `partial` with a reason that falls under "out of scope".
- [ ] `fork/corpus/check.sh` clean after every milestone.

## Working rules for this area

- Read `memoir.cls` (`kpsewhich memoir.cls`) and the manual (`texmf-dist/doc/latex/memoir/memman.pdf`) first.
  There is no Perl oracle, so pdfLaTeX's output is the reference, and a fix cites `memoir.cls:line`.
- A guard asserts the fixed output (an entry, a number, a note element), not only that the error is gone, and
  pins expected memoir warnings by name and count.
- Re-convert memman and the corpus memoir documents before landing any change to sectioning, captions or the ToC.
