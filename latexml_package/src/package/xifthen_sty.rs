//! xifthen.sty (v1.4.0, 2015/11/05) — more tests for ifthen's `\ifthenelse`.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(u). The package adds tests to
//! ifthen's evaluator: each test is a macro that `\TE@throw`s TeX conditional code into the
//! expression ifthen's `\TE@eval` builds (xifthen.sty:72-171): `\cnttest{a}{op}{b}` and
//! `\dimtest{a}{op}{b}` (calc expressions, `<` `=` `>` `<=` `>=`), `\isnamedefined{name}`,
//! `\isempty{text}` (blank per ifmtarg), `\isequivalentto{\a}{\b}` (`\ifx`), `\isin{a}{b}`,
//! `\endswith{a}{b}`; `\newtest\name[n]{expr}` names an expression (:172-191); and it redefines
//! `\ifthenelse` to accept `\or`/`\and`/`\not` for `\OR`/`\AND`/`\NOT` and to install the tests
//! (`\TE@hook`, :192-237). ifthen's evaluator is ifthen.sty's own (ifthen_sty.rs, as Perl's
//! ifthen.sty.ltxml loads it), so these definitions are the package's, over it.
//! Witnesses (errors in the default configuration, before → after): 2610.17646 (61 → 7 with its calligra, quiver and dynkin-diagrams).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // xifthen.sty:44-46
  RequirePackage!("calc");
  RequirePackage!("ifthen");
  InputDefinitions!("ifmtarg", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // xifthen.sty:51-71 `\TE@repl`: replace a token in `\toks@`.
  RawTeX!(r"\def\TE@repl#1#2{\long\def\@tempc##1#1##2{\def\@tempb{\@tempc}\begingroup\toks@{##2}\edef\@tempa{\the\toks@}\expandafter\endgroup\ifx\@tempa\@tempb\toks@\expandafter{\the\toks@##1}\expandafter\@gobble\else\toks@\expandafter{\the\toks@##1#2}\expandafter\@tempc\fi##2}\toks@\expandafter{\expandafter}\expandafter\@tempc\the\toks@#1\@tempc}");
  // xifthen.sty:72-125: `\cnttest`, `\dimtest`.
  RawTeX!(r##"\newcommand*{\TE@cnttest}[4]{\TE@throw\noexpand\calc@assign@count{\@tempcnta}{\unexpanded{#1}}\noexpand\calc@assign@count{\@tempcntb}{\unexpanded{#3}}\noexpand\if\noexpand\TE@cnttest@{#2}#4}
\newcommand*{\TE@cnttest@}[1]{\ifcsname TE@cnttest@@#1\endcsname\csname TE@cnttest@@#1\expandafter\expandafter\expandafter\endcsname\expandafter\@gobble\else\expandafter\@firstofone\fi{FF\PackageError{xifthen}{Not a valid comparison operator "#1"}{Valid comparisons are <, =, >, <=, and =>.}}}
\@namedef{TE@cnttest@@<}{\ifnum\@tempcnta<\@tempcntb AA\else AB\fi}
\@namedef{TE@cnttest@@>}{\ifnum\@tempcnta>\@tempcntb AA\else AB\fi}
\@namedef{TE@cnttest@@=}{\ifnum\@tempcnta=\@tempcntb AA\else AB\fi}
\@namedef{TE@cnttest@@<=}{\ifnum\@tempcnta<\@tempcntb AA\else\ifnum\@tempcnta=\@tempcntb AA\else AB\fi\fi}
\@namedef{TE@cnttest@@>=}{\ifnum\@tempcnta>\@tempcntb AA\else\ifnum\@tempcnta=\@tempcntb AA\else AB\fi\fi}
\newcommand*{\TE@dimtest}[4]{\TE@throw\noexpand\calc@assign@skip{\@tempskipa}{\unexpanded{#1}}\noexpand\calc@assign@skip{\@tempskipb}{\unexpanded{#3}}\noexpand\if\noexpand\TE@dimtest@{#2}#4}
\newcommand*{\TE@dimtest@}[1]{\ifcsname TE@dimtest@@#1\endcsname\csname TE@dimtest@@#1\expandafter\expandafter\expandafter\endcsname\expandafter\@gobble\else\expandafter\@firstofone\fi{FF\PackageError{xifthen}{Not a valid comparison operator "#1"}{Valid comparisons are <, =, >, <=, and =>.}}}
\@namedef{TE@dimtest@@<}{\ifdim\@tempskipa<\@tempskipb AA\else AB\fi}
\@namedef{TE@dimtest@@>}{\ifdim\@tempskipa>\@tempskipb AA\else AB\fi}
\@namedef{TE@dimtest@@=}{\ifdim\@tempskipa=\@tempskipb AA\else AB\fi}
\@namedef{TE@dimtest@@<=}{\ifdim\@tempskipa<\@tempskipb AA\else\ifdim\@tempskipa=\@tempskipb AA\else AB\fi\fi}
\@namedef{TE@dimtest@@>=}{\ifdim\@tempskipa>\@tempskipb AA\else\ifdim\@tempskipa=\@tempskipb AA\else AB\fi\fi}"##);
  // xifthen.sty:126-171: the other tests.
  RawTeX!(r"\newcommand*{\TE@isnamedefined}[2]{\TE@throw\noexpand\ifcsname #1\endcsname#2}
\newcommand*{\TE@isempty}[2]{\TE@throw\noexpand\if FF\noexpand\@nameuse{fi}\noexpand\@ifmtarg{\unexpanded{#1}}{\noexpand\@nameuse{iftrue}}{\noexpand\@nameuse{iffalse}}#2}
\newcommand*{\TE@isequivalentto}[3]{\TE@throw\noexpand\ifx\noexpand#1\noexpand#2#3}
\newcommand*{\TE@isin}[3]{\TE@throw\unexpanded{\in@{#1}{#2}}\noexpand\ifin@#3}
\newcommand*{\TE@endswith}[3]{\TE@throw\unexpanded{\in@{#2\@nil}{#1\@nil}}\noexpand\ifin@#3}");
  // xifthen.sty:172-191: `\newtest`.
  RawTeX!(r"\newcommand*{\newtest}[1]{\@ifnextchar[{\@newtest{#1}}{\@newtest{#1}[0]}}
\newcommand*{\@newtest}{}
\def\@newtest#1[#2]#3{\toks@={\(#3\)}\TE@repl\or\TE@or\TE@repl\and\TE@and\TE@repl\not\TE@neg\TE@repl\OR\TE@or\TE@repl\AND\TE@and\TE@repl\NOT\TE@neg\edef\@tempa{\unexpanded{\newcommand*{#1}[{#2}]}{\the\toks@}}\@tempa}");
  // xifthen.sty:192-237: the tests installed in `\ifthenelse`, which also reads `\or`/`\and`/`\not`.
  RawTeX!(r"\newcommand*{\TE@hook}{\let\isequivalentto=\TE@isequivalentto\let\terminateswith=\TE@endswith\let\endswith=\TE@endswith\let\isnamedefined=\TE@isnamedefined\let\isempty=\TE@isempty\let\cnttest=\TE@cnttest\let\dimtest=\TE@dimtest\let\isin=\TE@isin}
\renewcommand{\ifthenelse}[1]{\toks@{#1}\TE@repl\or\TE@or\TE@repl\and\TE@and\TE@repl\not\TE@neg\TE@repl\OR\TE@or\TE@repl\AND\TE@and\TE@repl\NOT\TE@neg
\begingroup\let\protect=\@unexpandable@protect\def\@setref##1##2##3{\ifx##1\relax\z@\else\expandafter##2##1\fi}\def\value##1{\the\csname c@##1\endcsname}\let\equal=\TE@equal\let\(=\TE@lparen\let\)=\TE@rparen\let\isodd=\TE@odd\let\lengthtest=\TE@length\let\isundefined=\TE@undef\TE@hook
\begingroup\let\@tempa=\relax\let\@tempb=\relax\xdef\@gtempa{\expandafter\TE@eval\the\toks@\TE@endeval}\endgroup\@gtempa\expandafter\endgroup\ifTE@val\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi}");
});
