//! manyfoot.sty (ncctools v1.11) — further series of footnotes.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(k). `\DeclareNewFootnote[style]{S}[fmt]`
//! (manyfoot.sty:230-278) makes a footnote series `S` with its own insert (`\footins<S>`, its own
//! place at the page foot, :192-207), counter `footnote<S>` printed by `\@<fmt>` (arabic, roman,
//! alph, fnsymbol, gobble …) and commands `\footnote<S>[n]{text}`, `\footnotemark<S>[n]`,
//! `\footnotetext<S>[n]{text}`, `\Footnote<S>{mark}{text}`, `\Footnotemark<S>{mark}`,
//! `\Footnotetext<S>{mark}{text}`; `\newfootnote[style]{S}` makes only the insert and
//! `\Footnotetext<S>`. Here a series is a note type: each note is the kernel's `ltx:note` with role
//! `footnote<S>` and that counter's mark (the kernel's `\lx@note`/`\lx@notemark`/`\lx@notetext` are
//! parameterised by the note type, latex_constructs/sect03.rs), so the series keep apart and are
//! numbered as the package numbers them. `\SetFootnoteHook{code}` puts `code` at the start of every
//! note of the next series (plain style, :183-190). The rules (`\SelectFootnoteRule`,
//! `\extrafootnoterule`, :173-182), the `para` style's run-in layout (:58-163), `ruled` and the page
//! resets of `perpage` (:169-171) are page-foot layout, read and kept in their macros. The hand-marked
//! notes are nccfoots' (nccfoots_sty.rs), which manyfoot loads (:28).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // manyfoot.sty:28-35, :58, :164, :169: options.
  RawTeX!(r"\newif\ifMFL@para\newif\ifMFL@perpage
\newcommand\extrafootnoterule{}\newcommand\defaultfootnoterule{}\newcommand\footnoterulepriority{1}");
  DeclareOption!("ruled", { RawTeX!(r"\def\extrafootnoterule{\defaultfootnoterule}"); });
  DeclareOption!("para", { RawTeX!(r"\MFL@paratrue"); });
  DeclareOption!("para*", { RawTeX!(r"\MFL@paratrue"); });
  DeclareOption!("perpage", { RawTeX!(r"\MFL@perpagetrue"); });
  ProcessOptions!();
  RawTeX!(r"\AtBeginDocument{\let\defaultfootnoterule\footnoterule}");
  // manyfoot.sty:58-163: the `para` style's commands exist only under the option.
  RawTeX!(r"\ifMFL@para
\def\footglue{\hskip 1em plus .5em minus .3em}
\newcommand\ExtraParaSkip[1]{}\@onlypreamble\ExtraParaSkip
\long\def\SplitNote{}
\fi");

  // manyfoot.sty:28: nccfoots, whose hand-marked notes the `\Footnote<S>` commands extend.
  RequirePackage!("nccfoots");

  // manyfoot.sty:183-190: the hook of the next series.
  RawTeX!(r"\newcommand\SetFootnoteHook[1]{\def\MFL@hook{#1}}\SetFootnoteHook{}\@onlypreamble\SetFootnoteHook
\newcommand\SelectFootnoteRule[2][0]{\@ifnextchar[{\MFL@rule@action{#1}{#2}}{\MFL@rule@action{#1}{#2}[]}}
\def\MFL@rule@action#1#2[#3]{\def\MFL@rule{#2}}\@onlypreamble\SelectFootnoteRule");

  // manyfoot.sty:192-207 `\newfootnote[style]{S}`: the insert and `\Footnotetext<S>`; a style other
  // than `plain` (or `para` under the option) is an error.
  RawTeX!(r"\newcommand\newfootnote[2][plain]{%
\def\MFL@style{#1}\def\MFL@plain{plain}\def\MFL@paras{para}%
\ifx\MFL@style\MFL@plain\else\ifMFL@para\ifx\MFL@style\MFL@paras\else\MFL@badstyle{#1}\fi\else\MFL@badstyle{#1}\fi\fi
\expandafter\newinsert\csname footins#2\endcsname
\expandafter\let\csname ext@footnote#2\endcsname\@empty
\expandafter\def\csname footnote#2typerefname\endcsname{\footnotetyperefname}%
\expandafter\let\csname MFL@hook@#2\endcsname\MFL@hook
\expandafter\long\expandafter\def\csname Footnotetext#2\endcsname##1##2{\lx@MFL@notetext{footnote#2}{##1}{\csname MFL@hook@#2\endcsname##2}}%
\SetFootnoteHook{}}
\def\MFL@badstyle#1{\PackageError{manyfoot}{Unknown footnote style #1}\@ehc}
\@onlypreamble\newfootnote");

  // manyfoot.sty:209-278 `\DeclareNewFootnote[style]{S}[fmt]`.
  RawTeX!(r"\newcommand\DeclareNewFootnote[2][plain]{\@ifnextchar[{\MFL@declare{#1}{#2}}{\MFL@declare{#1}{#2}[arabic]}}
\def\MFL@declare#1#2[#3]{%
\newfootnote[#1]{#2}%
\newcounter{footnote#2}%
\expandafter\edef\csname thefootnote#2\endcsname{\expandafter\noexpand\csname @#3\endcsname\expandafter\noexpand\csname c@footnote#2\endcsname}%
\expandafter\def\csname footnote#2\endcsname{\@ifnextchar[{\MFL@fn@opt{#2}}{\MFL@fn{#2}}}%
\expandafter\def\csname footnotemark#2\endcsname{\@ifnextchar[{\MFL@fnmark@opt{#2}}{\lx@notemark{footnote#2}}}%
\expandafter\def\csname footnotetext#2\endcsname{\@ifnextchar[{\MFL@fntext@opt{#2}}{\MFL@fntext{#2}}}%
\expandafter\def\csname Footnotemark#2\endcsname##1{\lx@MFL@notemark{footnote#2}{##1}}%
\expandafter\long\expandafter\def\csname Footnote#2\endcsname##1##2{\lx@MFL@note{footnote#2}{##1}{\csname MFL@hook@#2\endcsname##2}}}
\long\def\MFL@fn#1#2{\lx@note{footnote#1}{\csname MFL@hook@#1\endcsname#2}}
\long\def\MFL@fn@opt#1[#2]#3{\begingroup\csname c@footnote#1\endcsname=#2\relax\lx@note{footnote#1}[\csname thefootnote#1\endcsname]{\csname MFL@hook@#1\endcsname#3}\endgroup}
\def\MFL@fnmark@opt#1[#2]{\begingroup\csname c@footnote#1\endcsname=#2\relax\lx@notemark{footnote#1}[\csname thefootnote#1\endcsname]\endgroup}
\long\def\MFL@fntext#1#2{\lx@notetext{footnote#1}{\csname MFL@hook@#1\endcsname#2}}
\long\def\MFL@fntext@opt#1[#2]#3{\begingroup\csname c@footnote#1\endcsname=#2\relax\lx@notetext{footnote#1}[\csname thefootnote#1\endcsname]{\csname MFL@hook@#1\endcsname#3}\endgroup}
\@onlypreamble\DeclareNewFootnote");
});
