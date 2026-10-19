//! centernot.sty (oberdiek, v1.4 2016/05/16) — a `\not` slash centred on the symbol it negates.
//!
//! No Perl binding exists; native binding, OXIDIZED_DESIGN #588(c). centernot.sty:42-58 defines
//! `\centernot` as `\mathpalette\@centernot`, which reads one argument `#2` and typesets
//! `\mathrel{\rlap{<kerns>$\not$}{#2}}`: a relation made of the symbol with `\not`'s slash kerned
//! over its middle. The kerns only move the slash, so the meaning is `\not` applied to the
//! argument: `\mathrel{\not #1}` gives the negated character (`\centernot\to` is U+219B, as
//! `\not\to` is) or the combining U+0338 overlay, through math_common.rs' `\not` rewrite.
//! Witnesses (errors in the default configuration, before → after): 2610.20583 (1 → 0).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // centernot.sty:42-44 (`\newcommand*{\centernot}{\mathpalette\@centernot}`): one argument,
  // read as `\mathpalette`'s `#2` is, and the result is a relation (`\mathrel`, :47).
  DefMacro!("\\centernot{}", "\\mathrel{\\not#1}");
});
