//! abstract.sty (v1.2a, 2009/06/08) — configurable abstracts.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(y). The package redefines the
//! `abstract` environment (abstract.sty:139-185) to typeset its heading (`\abstractname` in
//! `\abstractnamefont`, placed by `\absnamepos`, `\abstitleskip` below it, or run in with
//! `\abslabeldelim` under `runin`) above a list of the text in `\abstracttextfont`, indented by
//! `\absleftindent`/`\absrightindent`. The abstract stays the kernel's frontmatter `ltx:abstract`,
//! named by `\abstractname`, as the class's own abstract is: the fonts, indents and skips are the
//! print layout of that block, as the kernel already treats article's `\small` abstract. The
//! configuration commands and lengths are defined as the package defines them, so a document's
//! `\renewcommand{\abstractnamefont}…`/`\setlength{\absleftindent}…` works. What changes the
//! document beyond layout is kept: under `number` the heading is a `\section` (or `\chapter`), so
//! the abstract takes that sectional unit's number, shown in its name ("1 Abstract"), and the next
//! one is numbered after it (:116-119);
//! `onecolabstract` is the abstract (:186-188); `\saythanks` prints the `\thanks` notes in two-column
//! layouts where `\maketitle` cannot (:197-206), and those notes are already in the frontmatter here,
//! so it adds nothing.
//! Witnesses (errors in the default configuration, before → after): 2610.18646 (8 → 0).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // abstract.sty:26-49: options.
  RawTeX!(r"\newif\if@bsonecol \@bsonecoltrue \newif\ifadd@bstotoc \add@bstotocfalse
\newif\ifnumber@bs \number@bsfalse \newif\if@bsrunin \@bsruninfalse \newif\if@bsstyle \@bsstylefalse");
  DeclareOption!("original", { RawTeX!(r"\@bsonecolfalse"); });
  DeclareOption!("addtotoc", { RawTeX!(r"\add@bstotoctrue"); });
  DeclareOption!("number", { RawTeX!(r"\number@bstrue"); });
  DeclareOption!("runin", { RawTeX!(r"\@bsrunintrue"); });
  DeclareOption!("style", { RawTeX!(r"\@bsstyletrue"); });
  ProcessOptions!();
  RawTeX!(r"\if@bsstyle\number@bsfalse\add@bstotocfalse\fi
\if@bsrunin\number@bsfalse\@bsstylefalse\fi");

  // abstract.sty:51-103: the configuration macros and lengths with the package's defaults per
  // layout (title page, two columns, `original`).
  RawTeX!(r"\newcommand{\abstractnamefont}{\normalfont\small\bfseries}
\newcommand{\abstracttextfont}{\normalfont\small}
\newcommand{\absnamepos}{center}
\newlength{\abstitleskip}\setlength{\abstitleskip}{-0.5em}
\newlength{\absleftindent}\newlength{\absrightindent}\newlength{\absparindent}\newlength{\absparsep}
\newcommand{\abslabeldelim}[1]{\def\@bslabeldelim{#1}}\abslabeldelim{}
\newcommand{\@bsrunintitle}{\hspace*{\abstitleskip}{\abstractnamefont\abstractname\@bslabeldelim}}
\if@titlepage
  \setlength{\absleftindent}{\z@}\renewcommand{\abstractnamefont}{\normalfont\bfseries}
  \renewcommand{\abstracttextfont}{\normalfont}\setlength{\abstitleskip}{0em}
\else\if@twocolumn\if@bsonecol\setlength{\absleftindent}{\leftmargin}
  \else\setlength{\absleftindent}{\z@}\renewcommand{\abstractnamefont}{\normalfont\Large\bfseries}
    \renewcommand{\abstracttextfont}{\normalfont}\renewcommand{\absnamepos}{flushleft}\setlength{\abstitleskip}{0em}\fi
  \else\setlength{\absleftindent}{\leftmargin}\fi\fi
\setlength{\absrightindent}{\absleftindent}
\AtBeginDocument{\setlength{\absparindent}{\parindent}\setlength{\absparsep}{\parskip}}
\newcommand*{\abstitlestyle}[1]{#1}");

  // abstract.sty:116-119 (`\num@bs`): under `number` the heading is the sectional unit's, so the
  // abstract steps that counter (a chapter where the class has chapters).
  RawTeX!(r"\newcommand{\put@bsintoc}{}
\newcommand{\num@bs}{\@ifundefined{chapter}{\refstepcounter{section}\edef\lx@bs@number{\thesection}}{\refstepcounter{chapter}\edef\lx@bs@number{\thechapter}}\def\lx@abstract@name{\format@title@abstract{\lx@bs@number\space\abstractname}}}");
  DefMacro!(T_CS!("\\begin{abstract}"), None, "\\par\\ifnumber@bs\\num@bs\\fi\\lx@begin@abstract");
  // abstract.sty:186-188
  DefMacro!(T_CS!("\\begin{onecolabstract}"), None, "\\begin{abstract}");
  DefMacro!(T_CS!("\\end{onecolabstract}"), None, "\\end{abstract}");

  // abstract.sty:189-196: the package's utility macro, as it defines it.
  RawTeX!(r"\providecommand{\appendiargdef}[2]{\begingroup\toks@\expandafter{#1{##1}#2}\edef\@bsx{\endgroup \def\noexpand#1####1{\the\toks@}}\@bsx}
\let\@bs@thanks\@empty");
  // abstract.sty:203-206: the `\thanks` notes are already the frontmatter's.
  DefMacro!("\\saythanks", None);
});
