//! nccfoots.sty (ncctools v1.2, 2005/02/03) — footnotes with a given mark.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(m). `\Footnote{mark}{text}`,
//! `\Footnotemark{mark}` and `\Footnotetext{mark}{text}` (nccfoots.sty:36-38) set `\@thefnmark` to the
//! mark (an empty one keeps the current mark, `\NCC@makefnmark`, :28-34) and then make LaTeX's footnote,
//! mark or text without stepping the counter. Here each is the kernel's note (`\lx@note`,
//! `\lx@notemark`, `\lx@notetext`, latex_constructs/sect03.rs) of a note type with that mark; manyfoot's
//! series (manyfoot_sty.rs) use the same commands with their own type.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // The notes of a type `#1` with a given mark `#2` (empty: the type's current mark).
  RawTeX!(r"\long\def\lx@MFL@note#1#2#3{\if\relax\detokenize{#2}\relax\lx@note{#1}[\csname the#1\endcsname]{#3}\else\lx@note{#1}[{#2}]{#3}\fi}
\def\lx@MFL@notemark#1#2{\if\relax\detokenize{#2}\relax\lx@notemark{#1}[\csname the#1\endcsname]\else\lx@notemark{#1}[{#2}]\fi}
\long\def\lx@MFL@notetext#1#2#3{\if\relax\detokenize{#2}\relax\lx@notetext{#1}{#3}\else\lx@notetext{#1}[{#2}]{#3}\fi}");
  // nccfoots.sty:36-38
  RawTeX!(r"\newcommand*\Footnote[2]{\lx@MFL@note{footnote}{#1}{#2}}
\newcommand*\Footnotemark[1]{\lx@MFL@notemark{footnote}{#1}}
\newcommand*\Footnotetext[2]{\lx@MFL@notetext{footnote}{#1}{#2}}");
});
