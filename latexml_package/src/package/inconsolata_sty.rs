//! inconsolata.sty (inconsolata-zi4, v1.12 2019/05/17) — Inconsolata as the typewriter font.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(n). The package makes `zi4` the
//! `\ttdefault` (or, under `nott`, the `\rmdefault`, inconsolata.sty:31-32, :51-56), so
//! `\ttfamily`/`\texttt` text is typewriter (`zi4` is the typewriter family in the font model,
//! font.rs). Its glyph options pick character variants of the font (`var0` slashed zero, `varl`,
//! `varqu`; :61-70) and its spacing options set `\fontdimen`s of the loaded font through the `.fd`
//! files (:71-104, :113-119): the font is not loaded here, so they are read and kept in the package's
//! macros only. `\altzero` (:128-131) is the zero in the other style than the font's: the font's zero is
//! slashed unless `var0` (:61-64), so `\altzero` is the plain zero, and under `var0` the slashed one, U+0030
//! U+FE00 (Unicode's standardized "short diagonal stroke form" of DIGIT ZERO). With the default `upquote` behaviour the package loads
//! upquote at the document's start (:142-161), so a verbatim `'` is the straight quote.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // inconsolata.sty:11-34, :43
  RequirePackage!("textcomp");
  RequirePackage!("xkeyval");
  RawTeX!(r"\newcount\zifour@ocount \newif\ifzifour@altzero \newif\ifzifour@noupq \newif\ifzifour@mono \newif\ifzifour@hyphen
\def\zifour@sp{.5em}\def\zifour@stretch{.25em}\def\zifour@shrink{.167em}\def\zifour@extrasp{0em}
\def\zifour@spn{.45em}\def\zifour@stretchn{.225em}\def\zifour@shrinkn{.15em}\def\zifour@extraspn{0em}
\newcommand*{\zifour@default}{\renewcommand*{\ttdefault}{zi4}}
\newcommand\zifour@scaled{s*[1]}
\def\zifour@hyph{\hyphenchar\font=\m@ne}");
  // inconsolata.sty:36-84, as the package declares them (xkeyval options).
  RawTeX!(r"\DeclareOptionX{scaled}[1.0]{\renewcommand{\zifour@scaled}{s*[#1]}}
\DeclareOptionX{scale}[1.0]{\renewcommand{\zifour@scaled}{s*[#1]}}
\DeclareOptionX{hyphenate}{\def\zifour@hyph{}}
\DeclareOptionX{mono}{\zifour@monotrue}
\DeclareOptionX{nott}{\def\zifour@hyph{}\zifour@monofalse\def\zifour@extrasp{0em}\renewcommand*{\zifour@default}{\renewcommand*{\rmdefault}{zi4}}}
\DeclareOptionX{noupquote}{\zifour@noupqtrue}
\DeclareOptionX{var0}{\zifour@altzerotrue\advance\zifour@ocount\tw@}
\DeclareOptionX{varl}{\advance\zifour@ocount\@ne}
\DeclareOptionX{varqu}{\advance\zifour@ocount 4\relax}
\DeclareOptionX{narrow}{\gdef\zifour@narrow{n}}
\DeclareOptionX{spacing}[.5em]{\gdef\zifour@spx{#1}}
\DeclareOptionX{stretch}[.25em]{\gdef\zifour@stretchx{#1}}
\DeclareOptionX{shrink}[.167em]{\gdef\zifour@shrinkx{#1}}
\DeclareOptionX{extrasp}[.167em]{\gdef\zifour@extraspx{#1}}
\ProcessOptionsX
\zifour@default");
  // inconsolata.sty:86-127: the spacing the `.fd` files would apply, and the variant numbers.
  RawTeX!(r"\ifzifour@mono\def\zifour@stretch{0em}\def\zifour@shrink{0em}\def\zifour@extrasp{.5em}\def\zifour@stretchn{0em}\def\zifour@shrinkn{0em}\def\zifour@extraspn{.45em}\fi
\ifdefined\zifour@narrow\let\zifour@sp\zifour@spn\let\zifour@stretch\zifour@stretchn\let\zifour@shrink\zifour@shrinkn\let\zifour@extrasp\zifour@extraspn\fi
\ifdefined\zifour@spx\let\zifour@sp\zifour@spx\fi
\ifdefined\zifour@stretchx\let\zifour@stretch\zifour@stretchx\fi
\ifdefined\zifour@shrinkx\let\zifour@shrink\zifour@shrinkx\fi
\ifdefined\zifour@extraspx\let\zifour@extrasp\zifour@extraspx\fi
\edef\zifour@opt{\the\zifour@ocount}
\ifzifour@altzero\advance\zifour@ocount-\tw@\else\advance\zifour@ocount\tw@\fi
\edef\zifour@altopt{\the\zifour@ocount}");
  // inconsolata.sty:128-131: the other zero, in the typewriter family.
  DefMacro!("\\altzero", "{\\fontfamily{zi4}\\selectfont 0\\ifzifour@altzero\u{FE00}\\fi}");
  // inconsolata.sty:142-161: upquote at the document's start unless `noupquote`.
  RawTeX!(r"\ifzifour@noupq\else\AtBeginDocument{\usepackage{upquote}}\fi");
});
